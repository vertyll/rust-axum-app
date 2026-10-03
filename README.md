# Rust Axum App

A modular monolith in Rust – Axum, hexagonal architecture with DDD layering.

## Architecture

Three bounded contexts, each an independent module with the same three layers:

```
src/
├── identity/                    # accounts mirrored from Keycloak, browser sessions, roles, auth guards
│   ├── domain/                  # User aggregate, KeycloakIdentity, roles, UserRepository port
│   ├── application/             # IdentityService; ports: KeycloakClient, TokenVerifier
│   └── infrastructure/          # adapters: Toasty persistence, Keycloak token endpoint (single-flight refresh),
│                                # JWKS token verifier, PKCE, axum HTTP (sign-in routes, auth guards)
├── files/                       # file upload & metadata
│   ├── domain/                  # StoredFile, StorageKind, FileRepository port
│   ├── application/             # FilesService; FileStorage port
│   └── infrastructure/          # Toasty persistence, local-FS storage, axum HTTP
├── translations/                # ICU MessageFormat catalogue (defaults in translations/*.json)
│   ├── domain/                  # Translation, Language, ICU syntax checks, TranslationRepository port
│   ├── application/             # TranslationsService: catalogue, admin edits, startup synchronization
│   └── infrastructure/          # shipped defaults, Toasty persistence, axum HTTP
├── shared_infrastructure/       # shared technical infrastructure: RFC 9457 problem documents only
├── bootstrap/                   # composition root: config, seeding, wiring, router
├── main.rs                      # server binary
└── bin/cli.rs                   # migration CLI (toasty-cli)
```

### Dependency rules

1. Within a module, dependencies point inward only: `infrastructure → application → domain`.
   The domain imports no framework, ORM or IO types.
2. Across modules, dependencies are one-directional and go through the module's public
   API (`mod.rs` re-exports): `files` and `translations` use identity's `Auth`/`RequireAdmin`
   guards and the `Caller` they carry; `identity` knows nothing about either. There are no cross-module ORM
   relations — foreign contexts are referenced by plain ids.
3. Composition root: `bootstrap` sees everything and is imported by nothing. It is the only place naming
   concrete adapters: dependency injection is constructor calls plus three type aliases
   (`AppIdentityService`, `AppFilesService`, `AppTranslationsService`), all resolved at compile time.

> [!NOTE]
>
> Ports are traits with native `async fn` and are wired through **generics (static
> dispatch)** — no `async_trait`, no `Arc<dyn …>`, no per-call future boxing. Swapping an
> adapter (e.g. local storage → S3) is a one-line change in `bootstrap`.
>
> Transactions are an adapter detail scoped to a single aggregate: multi-row writes
> (user + role links) are one repository-port method whose
> Toasty adapter runs one transaction. Nothing above the adapter sees a transaction.

## Running locally

> [!IMPORTANT]
>
> Requires: Rust 1.95+ (Toasty's MSRV), Docker.

```bash
# 1. Infrastructure: PostgreSQL + maildev (SMTP sandbox at http://localhost:1080)
docker compose -f docker-compose.local.yml up -d   # PostgreSQL :5432, Redis :6379, Keycloak :9000, maildev :1025/:1080

# 2. Configuration
cp .env.example .env

# 3. Run — locally the schema is pushed straight from the models
cargo run
```

The API listens on `http://127.0.0.1:3000`.

### Migrations

Command `cargo run` uses `push_schema` locally (plain CREATE TABLEs from the models).
For managed, versioned migrations use the bundled CLI (files land in `toasty/`, configured by `Toasty.toml`):
```bash
cargo run --bin cli -- migration generate --name <name>
cargo run --bin cli -- migration apply
cargo run --bin cli -- snapshot        # capture current DB state
```

## Authentication

Keycloak (realm `rust-axum-app`, imported from `keycloak/realm-export.json`) handles sign-up, sign-in, e-mail
verification, password reset, two-factor authentication and acceptance of the terms of use on its own pages. The local
realm has `admin@rust-axum-app.local` (`ADMIN`) and `user@rust-axum-app.local`, both with the password
`rust-axum-app-local`; e-mails land in maildev at <http://localhost:1080>.

- A browser signs in at `GET /api/auth/authorize` (`?register=true` opens sign-up): the authorization code flow with
  PKCE, after which it holds only the `RUST_AXUM_APP_SESSION` cookie (`HttpOnly`, `SameSite=Lax`, `Secure` when
  `SESSION_COOKIE_SECURE=true`). The tokens stay in the session, which `tower-sessions` keeps in Redis, so the
  application holds no state of its own.
- The auth guard takes the access token from `Authorization: Bearer` or from the session — refreshing it when it is about
  to expire; refresh tokens rotate and concurrent requests share one refresh; a cross-site write gets no token — and
  verifies its signature against Keycloak's published keys, the issuer, the expiry and the audience (`rust-axum-app`).
- The account is mirrored into `users` (with its realm roles) on every authenticated request that changes it.
- The terms and the privacy policy are served at `/legal/terms.html` and `/legal/privacy.html`.

## API overview

| Method                               | Path                                          | Auth                    |
|--------------------------------------|-----------------------------------------------|-------------------------|
| GET                                  | `/api/auth/authorize`, `/api/auth/callback`   | —                       |
| GET                                  | `/api/auth/session`                           | session (204 without)   |
| POST                                 | `/api/auth/logout`                            | session                 |
| GET                                  | `/api/users/me`                               | bearer or session       |
| GET                                  | `/api/users`, `/api/users/{id}`               | bearer or session + admin |
| GET/POST                             | `/api/files`, `/api/files/{id}`               | bearer or session       |
| PUT/DELETE, POST `/{id}/soft-delete` | `/api/files…`                                 | bearer or session + admin |
| GET                                  | `/uploads/{file}`, `/legal/{page}`            | — (static)              |
| GET                                  | `/api/translations/{pl\|en}`                  | —                       |
| GET, PUT `/{key}`                    | `/api/admin/translations`                     | bearer or session + admin |
| DELETE                               | `/api/admin/translations/{key}/customization` | bearer or session + admin |

Responses carry plain data. Every error — including a malformed body, a bad path parameter and an unknown route — is an
RFC 9457 document (`application/problem+json`) holding a message key rather than prose:

```json
{
  "type": "about:blank",
  "title": "Bad Request",
  "status": 400,
  "detail": "errors.validation",
  "code": "errors.validation",
  "errors": { "message": ["translations.validators.message.length"] },
  "args": { "message": { "min": 1, "max": 2000 } }
}
```

A client renders `code` and each key in `errors` with the ICU MessageFormat catalogue from `/api/translations/{language}`,
passing `args` (keyed by field in a validation problem). The defaults ship in `translations/{pl,en}.json` and are
synchronized into the `translations` table at startup: new keys are added, changed defaults adopted and obsolete keys
dropped, while an admin's override survives until it is reset. An edited message must parse as ICU MessageFormat
(plural and select need an `other` branch) and may use only the arguments of its default.

## Configuration

Environment variables are the single source of truth (12-factor): `dotenvy` loads `.env` locally, the platform injects 
them in production. There is deliberately no TOML layer — a second configuration source adds precedence rules, and 
config files in a repository are the classic way secrets leak. Defaults exist only for local development: outside local
the app refuses to boot while the public URL, database connection, Keycloak, Redis, SMTP host or sender address are
unset, and on startup it logs one structured line of the effective configuration with secrets redacted.

## Tests

Command `cargo test` runs the unit suite. The domain layer is tested directly — the aggregates are pure functions of `now`, so 
no mocking is needed. Every application use case runs against in-memory fake adapters plugged into the same ports as 
production, so the full flows (sign-in with account provisioning, revocation when it fails, role mirroring, upload compensation)
execute without Keycloak or a database. Adapter-level tests cover the Keycloak client against a local HTTP server
(single-flight refresh, rejected refreshes, outages), the claims the verifier accepts, PKCE and the local file storage.

For a coverage report:
```sh
cargo install cargo-llvm-cov
cargo llvm-cov --html
```
