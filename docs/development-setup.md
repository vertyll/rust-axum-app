# Development Setup

## Prerequisites

- Docker or Podman, with Compose
- Rust 1.99 (edition 2024), the version `backend/rust-toolchain.toml` and `frontend/rust-toolchain.toml` pin;
  `rustup install` fetches it

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
| Garage       | `http://localhost:3900`                     | the S3 bucket `rust-axum-app-files`             |
| Garage UI    | `http://localhost:3909`                     | browsing the bucket                             |

Keycloak imports `keycloak/realm-export.json` on its first start, with two accounts; its terms of use link to the
front-end's `/legal/terms` and `/legal/privacy`:

| Account                     | Password              | Roles           |
|-----------------------------|-----------------------|-----------------|
| `admin@rust-axum-app.local` | `rust-axum-app-local` | `USER`, `ADMIN` |
| `user@rust-axum-app.local`  | `rust-axum-app-local` | `USER`          |

> [!NOTE]
>
> The realm lives in the `keycloak-data` volume afterwards, so a change to the export file only takes effect after
> `docker compose -f docker-compose.local.yml down -v`.

Compose mounts the sign-in theme ([Authentication: The sign-in pages](authentication.md#the-sign-in-pages)) into
Keycloak, which in development reads it afresh on every page, so a change shows on a reload. A realm imported before the
export named the theme gets it, and the base URL its links lead to, without a reset:

```bash
podman exec rust-axum-app-keycloak-1 sh -c 'cd /opt/keycloak/bin \
  && ./kcadm.sh config credentials --server http://localhost:8080 --realm master --user admin --password admin \
  && ./kcadm.sh update realms/rust-axum-app -s loginTheme=rust-axum-app -s emailTheme=rust-axum-app \
  && id=$(./kcadm.sh get clients -r rust-axum-app -q clientId=rust-axum-app --fields id --format csv --noquotes) \
  && ./kcadm.sh update clients/$id -r rust-axum-app -s baseUrl=http://localhost:3000/'
```

## Run the back-end

From `backend/`:

```bash
cargo run
```

Configuration comes only from environment variables. Every one has a local default matching the containers above, so
nothing has to be set; `.env.example` lists them all, and a `.env` next to it is read when present.

On start, in the `local` environment, the application pushes the schema to the database, registers every module's
permissions with the stock roles `admin` and `user`, and fills the translation catalog from the modules' `messages/` and
the front-end's `backend/translations/`. It listens on `http://localhost:8080`, which only the front-end calls.

## Run the front-end

From `frontend/`, after building the Topcoat CLI into `.tools/` once:

```bash
cargo install topcoat-cli --version 0.11.0 --locked --root .tools
.tools/bin/topcoat dev
```

Open `http://localhost:3000` and sign in with one of the accounts above. The front-end passes `/api` on to the back-end,
so Keycloak returns to `http://localhost:3000/api/auth/callback`, and the sign-in lands on the front-end's home page.
More in the [front-end's README](../frontend/README.md).

## Migrations

Migrations are Toasty's, in `backend/toasty/migrations`, driven by the `cli` binary, run from `backend/`:

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

The front-end's checks are in [its README](../frontend/README.md#checks). From `backend/`:

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

The Redis-backed tests (the session store and the shared refresh) start Redis with Testcontainers, so Docker or Podman
must be running. CI runs the same commands, then the Sonar analysis and the image build.

The rules both applications' modules keep are checked from `architecture/` with the same three commands; see [its
README](../architecture/README.md).

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
| `FILES_SIGNING_KEY`                                | signs the local disk's links              |
| `S3_ENDPOINT`, `S3_BUCKET`                         | the bucket, with `FILES_STORAGE=s3`       |
| `S3_ACCESS_KEY`, `S3_SECRET_KEY`                   | its credentials, with `FILES_STORAGE=s3`  |

Uploads go to the bucket (`FILES_STORAGE=s3`, the default) or to `FILES_UPLOAD_DIR` (`FILES_STORAGE=local`), which
must then be a persistent volume. `S3_PUBLIC_ENDPOINT` is the bucket's address as the browser reaches it, when it
differs from `S3_ENDPOINT`, and the bucket allows `PUT`, `GET` and `HEAD` from the application's origin (CORS), as
`garage-init` sets it locally. `FILES_MAX_SIZE` limits an upload, in bytes (25 MiB by default).

## Documentation checks

The Markdown is formatted and linted with [markright](https://github.com/vertyll/markright), at the version the
[Docs workflow](../.github/workflows/docs.yml) pins. Install that version, from an archive of its
[release](https://github.com/vertyll/markright/releases) or with `cargo install --locked markright@VERSION`, then run:

```bash
markright fmt
markright check
```

`fmt` rewrites what it can; `check` reports the rest and fails on any issue, as the workflow does on every push and
pull request, and `check --fix` does both in one run. [`.markright.toml`](../.markright.toml) chooses the formatters
and linters and what each one skips.
