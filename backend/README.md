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

## Running it

Start the infrastructure first ([Development Setup](../docs/development-setup.md)), then:

```bash
cargo run
```

## Checks

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

The Redis-backed tests start Redis with Testcontainers, so Docker or Podman must be running.
