# Development Setup

## Prerequisites

- Docker or Podman, with Compose
- Rust 1.98 or newer (edition 2024)

## Start the infrastructure

```bash
docker compose -f docker-compose.local.yml up -d
```

Every `docker compose` command here works verbatim as `podman compose`.

| Service      | Address                                     | Purpose                                         |
|--------------|---------------------------------------------|-------------------------------------------------|
| PostgreSQL   | `localhost:5432` (`postgres` / `postgres`)  | the `rust_axum_app` database                    |
| Redis        | `localhost:6379`                            | sessions and the shared refresh lock            |
| Keycloak     | `http://localhost:9000` (`admin` / `admin`) | realm `rust-axum-app`, imported on start        |
| MailDev      | `http://localhost:1080`                     | catches Keycloak's verification and reset mails |
| RedisInsight | `http://localhost:5540`                     | browsing the sessions in Redis                  |

Keycloak imports `keycloak/realm-export.json` on its first start, with two accounts:

| Account                     | Password              | Roles           |
|-----------------------------|-----------------------|-----------------|
| `admin@rust-axum-app.local` | `rust-axum-app-local` | `USER`, `ADMIN` |
| `user@rust-axum-app.local`  | `rust-axum-app-local` | `USER`          |

> [!NOTE]
>
> The realm lives in the `keycloak-data` volume afterwards, so a change to the export file only takes effect after
> `docker compose -f docker-compose.local.yml down -v`.

## Run the application

```bash
cargo run
```

Configuration comes only from environment variables. Every one has a local default matching the containers above, so
nothing has to be set; `.env.example` lists them all, and a `.env` next to it is read when present.

On start, in the `local` environment, the application pushes the schema to the database, seeds the roles and fills the
translation catalog from `translations/`. It listens on `http://localhost:3000`.

Open `http://localhost:3000/api/auth/authorize` in a browser to sign in; the session cookie then authorizes every
call from that browser.

## Migrations

Migrations are Toasty's, in `toasty/migrations`, driven by the `cli` binary:

```bash
cargo run --bin cli -- migration generate --name <name>
cargo run --bin cli -- migration apply
```

Locally the schema is pushed on start, so `apply` is only needed when that push fails.

> [!WARNING]
>
> Outside `local` nothing is pushed, and the image carries neither the CLI nor the migrations: apply them against the
> target database before deploying.

## Checks

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

The Redis-backed tests (the session store and the shared refresh) start Redis with Testcontainers, so Docker or Podman
must be running. CI runs the same commands, then the Sonar analysis and the image build.

## Production

The image runs with `APP_ENVIRONMENT=production`. Outside `local` the application refuses to start while any of these is
unset, instead of falling back to a local default:

| Variable                                           | Purpose                                   |
|----------------------------------------------------|-------------------------------------------|
| `APP_URL`                                          | the public address                        |
| `DB_HOST`, `DB_USERNAME`, `DB_PASSWORD`, `DB_NAME` | PostgreSQL                                |
| `REDIS_URL`                                        | Redis                                     |
| `SESSION_COOKIE_SECURE`                            | `true` behind HTTPS                       |
| `KEYCLOAK_REALM_URL`, `KEYCLOAK_CLIENT_SECRET`     | the realm and the confidential client     |
| `AUTH_CALLBACK_URL`, `AUTH_POST_LOGIN_URL`         | where Keycloak returns and where it lands |
| `SMTP_HOST`, `EMAIL_FROM`                          | mail                                      |

Uploaded files are written to `FILES_UPLOAD_DIR` (`uploads` by default), which must be a persistent volume.
