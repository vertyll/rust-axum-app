# Rust Axum App

A modular monolith in Rust – Axum, hexagonal architecture with DDD layering.

## Architecture

Two bounded contexts, each an independent module with the same three layers:

```
src/
├── identity/                    # accounts, auth, sessions, roles, e-mail flows
│   ├── domain/                  # User aggregate + value objects, RefreshSession, domain rules, repository ports
│   ├── application/             # IdentityService use cases; ports: PasswordHasher, TokenService, IdentityMailer
│   └── infrastructure/          # adapters: Toasty persistence, Argon2, JWT,
│                                # SMTP+Tera, axum HTTP (routes, DTOs, auth guards)
├── files/                       # file upload & metadata
│   ├── domain/                  # StoredFile, StorageKind, FileRepository port
│   ├── application/             # FilesService; FileStorage port
│   └── infrastructure/          # Toasty persistence, local-FS storage, axum HTTP
├── shared_infrastructure/       # shared technical infrastructure: i18n only
├── bootstrap/                   # composition root: config, seeding, wiring, router
├── main.rs                      # server binary
└── bin/cli.rs                   # migration CLI (toasty-cli)
```

### Dependency rules

1. Within a module, dependencies point inward only: `infrastructure → application → domain`.
   The domain imports no framework, ORM or IO types.
2. Across modules, dependencies are one-directional and go through the module's public
   API (`mod.rs` re-exports): `files` uses identity's `Auth`/`RequireAdmin` guards and
   `AccessClaims`; `identity` knows nothing about `files`. There are no cross-module ORM
   relations — foreign contexts are referenced by plain ids.
3. Composition root: `bootstrap` sees everything and is imported by nothing. It is the only place naming
   concrete adapters: dependency injection is constructor calls plus two type aliases
   (`AppIdentityService`, `AppFilesService`), all resolved at compile time.

> [!NOTE]
>
> Ports are traits with native `async fn` and are wired through **generics (static
> dispatch)** — no `async_trait`, no `Arc<dyn …>`, no per-call future boxing. Swapping an
> adapter (e.g. local storage → S3) is a one-line change in `bootstrap`.
>
> Transactions are an adapter detail scoped to a single aggregate: multi-row writes
> (user + role links, e-mail change + audit entry) are one repository-port method whose
> Toasty adapter runs one transaction. Nothing above the adapter sees a transaction.

## Running locally

> [!IMPORTANT]
>
> Requires: Rust 1.95+ (Toasty's MSRV), Docker.

```bash
# 1. Infrastructure: PostgreSQL + maildev (SMTP sandbox at http://localhost:1080)
docker compose -f docker-compose.dev.yml up -d db mail-dev

# 2. Configuration
cp .env.example .env

# 3. Run — in development the schema is pushed straight from the models
cargo run
```

The API listens on `http://127.0.0.1:3000`.

### Migrations

Command `cargo run` uses `push_schema` in development (plain CREATE TABLEs from the models).
For managed, versioned migrations use the bundled CLI (files land in `toasty/`, configured by `Toasty.toml`):
```bash
cargo run --bin cli -- migration generate --name init
cargo run --bin cli -- migration apply
cargo run --bin cli -- snapshot        # capture current DB state
```

## API overview

| Method                               | Path                                         | Auth           |
|--------------------------------------|----------------------------------------------|----------------|
| POST                                 | `/api/auth/register`                         | —              |
| POST                                 | `/api/auth/login`                            | —              |
| POST                                 | `/api/auth/refresh-token`                    | refresh cookie |
| GET                                  | `/api/auth/confirm-email?token=…`            | —              |
| POST                                 | `/api/auth/password/reset`                   | —              |
| POST                                 | `/api/auth/confirm-password-reset`           | —              |
| GET                                  | `/api/auth/confirm-email-change?token=…`     | —              |
| POST                                 | `/api/auth/logout`, `/logout-all`            | bearer         |
| POST                                 | `/api/auth/password/change`, `/email/change` | bearer         |
| GET                                  | `/api/users`, `/api/users/{id}`              | bearer         |
| POST/PUT/DELETE                      | `/api/users…`                                | bearer + admin |
| GET/POST                             | `/api/files`, `/api/files/{id}`              | bearer         |
| PUT/DELETE, POST `/{id}/soft-delete` | `/api/files…`                                | bearer + admin |
| GET                                  | `/uploads/{file}`                            | — (static)     |

Locale is negotiated per request from `Accept-Language`.

## Configuration

Environment variables are the single source of truth (12-factor): `dotenvy` loads `.env` locally, the platform injects 
them in production. There is deliberately no TOML layer — a second configuration source adds precedence rules, and 
config files in a repository are the classic way secrets leak. Outside development the app refuses to boot when the 
token secrets are unset, and on startup it logs one structured line of the effective configuration with secrets redacted.

## Tests

Command `cargo test` runs the unit suite. The domain layer is tested directly — the aggregates are pure functions of `now`, so 
no mocking is needed. Every application use case runs against in-memory fake adapters plugged into the same ports as 
production, so the full flows (registration, login, token refresh, e-mail change, upload compensation) execute without 
a database, an SMTP server or real cryptography. Adapter-level tests cover JWT round-trips, Argon2 hashing and the 
local file storage.

For a coverage report:
```sh
cargo install cargo-llvm-cov
cargo llvm-cov --html
```
