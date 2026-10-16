# Rust Axum App — back-end

An Axum API over PostgreSQL and Redis, built as a modular monolith with hexagonal modules
([Architecture](../docs/architecture.md)). It signs users in and keeps their tokens
([Authentication](../docs/authentication.md)), decides what each role may do
([Authorization](../docs/authorization.md)) and answers errors as message keys
([Error responses](../docs/mechanisms/error-responses.md)).

## Layout

| Path                  | Holds                                                                                |
|-----------------------|--------------------------------------------------------------------------------------|
| `src/identity`        | sign-in, sessions, token verification, roles and the mirrored accounts               |
| `src/files`           | uploads stored on the local disk, with their metadata in PostgreSQL                  |
| `src/translations`    | the message catalog, its defaults and the admin overrides                            |
| `src/shared_kernel`   | the value types every module's domain may use, such as a page request                |
| `src/shared_http`     | RFC 9457 problem documents, paging parameters and responses                          |
| `src/bootstrap`       | the composition root: configuration, adapters and the router                         |
| `src/bin/cli.rs`      | the `cli` binary, which drives the migrations                                        |
| `src/*/messages`      | the messages each module answers with, in Polish and English                         |
| `translations/<area>` | the front-end's texts, one folder per area of its pages, such as `layout` or `files` |
| `toasty`              | the migrations and the schema snapshots                                              |

## Modules

Each module is a hexagon of its own, with a README on its ports and adapters:

- [identity](src/identity/README.md) – Sign-in through Keycloak, sessions, token verification, roles and the mirrored
  accounts.
- [files](src/files/README.md) – Uploaded files: the bytes on a storage, the metadata in PostgreSQL.
- [translations](src/translations/README.md) – The message catalog every error key and label resolves against.

## Access

The composition root (`src/bootstrap`) puts every router but sign-in, the public catalog and `/health` behind the auth
layer, which reads the access token from the session or from `Authorization: Bearer`. A handler then names the
permission it needs with the `Permitted<P>` extractor, and `/uploads`, which has no handler of its own, with the
`require::<P>` middleware; a caller whose roles do not grant it gets `403`. Each module declares its permissions and the
stock roles that hold them ([Authorization](../docs/authorization.md)).

## Messages

Every error is a message key with its arguments, never a sentence. Each module ships its messages in
`src/<module>/messages`, and the front-end its texts in `translations/<area>`; at start-up they are merged into the
catalog, where an administrator may change any of them
([Translation catalog](src/translations/docs/mechanisms/translation-catalog.md)).

## Database

PostgreSQL through Toasty. In the `local` environment the schema is pushed at start-up; elsewhere the migrations in
`toasty/` are applied with `cargo run --bin cli -- migration apply`. Collections that grow, files and accounts, are
paged by the database ([Collections are paged](../docs/architecture.md#collections-are-paged)).

## Mechanisms

- [Token refresh](src/identity/docs/mechanisms/token-refresh.md) – How the session keeps a valid access token without
  signing the user out when requests race.
- [Cross-site requests](src/identity/docs/mechanisms/cross-site-requests.md) – Why a forged request from another site
  cannot act with the user's session.
- [File storage](src/files/docs/mechanisms/file-storage.md) – What happens to the bytes and the metadata row on upload,
  soft delete and delete.
- [Translation catalog](src/translations/docs/mechanisms/translation-catalog.md) – Where the text behind every message
  key comes from, and how an administrator's edits survive a deployment.
- [Error responses](../docs/mechanisms/error-responses.md) – The RFC 9457 problem document every refusal answers with.

## Running it

Start the infrastructure first ([Development Setup](../docs/development-setup.md)), then:

```bash
cargo run
```

The `local` environment is the default and points at the local containers; it listens on `http://localhost:8080`, which
only the front-end calls. Uploaded files go to `uploads/`.

## Checks

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

The Redis-backed tests start Redis with Testcontainers, so Docker or Podman must be running.

## Production

`APP_ENVIRONMENT=production` takes its settings from the environment and refuses to start while one of these is unset,
rather than fall back to a local value:

| Variable                                           | Purpose                                               |
|----------------------------------------------------|-------------------------------------------------------|
| `APP_URL`                                          | the back-end's own address                            |
| `DB_HOST`, `DB_USERNAME`, `DB_PASSWORD`, `DB_NAME` | PostgreSQL                                            |
| `REDIS_URL`                                        | the sessions and the shared token refreshes           |
| `KEYCLOAK_REALM_URL`, `KEYCLOAK_CLIENT_SECRET`     | the realm and the confidential client                 |
| `AUTH_CALLBACK_URL`, `AUTH_POST_LOGIN_URL`         | the front-end's public address, where sign-in returns |
| `SESSION_COOKIE_SECURE`                            | `true` behind TLS                                     |
| `SMTP_HOST`, `EMAIL_FROM`                          | mail                                                  |

`.env.example` lists every variable with its local default. Passwords and the client secret are held as secrets, which
are never printed, also not in a debug dump of the configuration. The image runs the binary on port `8080`, and its
health check reads `/health`.
