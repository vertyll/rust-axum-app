## Project Assumptions

Modular monolith in Rust with hexagonal architecture and Domain-Driven Design layering: three bounded contexts
(`identity`, `files`, `translations`), each with its own domain, application and infrastructure layers.

## Technology Stack

### Back-end:

- Rust.
- Axum.
- Tokio.
- Toasty ORM.
- PostgreSQL.
- Redis (tower-sessions).
- reqwest.
- jsonwebtoken (JWKS).
- ICU MessageFormat.

### Authentication:

- Keycloak (realm `rust-axum-app`) handles sign-up, sign-in, email verification, password reset, two-factor
  authentication and acceptance of the terms of use.
- A browser signs in at `GET /api/auth/authorize` with the authorization code flow and PKCE, and then holds only the
  `RUST_AXUM_APP_SESSION` cookie (`HttpOnly`, `SameSite=Lax`, `Secure` in production). The tokens stay in the session,
  stored in Redis.
- Requests are authenticated with a bearer token or the session; the token's signature, issuer, expiry and audience
  (`rust-axum-app`) are verified against Keycloak's published keys. Refresh tokens rotate on every use.
- Locally, `docker-compose.local.yml` runs PostgreSQL, Redis, RedisInsight (`:5540`, connected to Redis), Keycloak on
  `:9000` (admin/admin) and maildev. The realm from `keycloak/realm-export.json` has `admin@rust-axum-app.local`
  (`ADMIN`) and `user@rust-axum-app.local`, both with the password `rust-axum-app-local`.
- Copy `.env.example` to `.env` and run `cargo run` (`:3000`).

### Core back-end:

- Hexagonal architecture: ports are traits wired with generics, adapters are chosen only in `bootstrap`.
- The application has an exception handling mechanism (RFC 9457 problem details with message codes).
- The application has a logging mechanism (tracing).
- The application has configuration from environment variables only.
- The application has RBAC (Role Based Access Control).
- The application has versioned database migrations (Toasty CLI).
- The application has translations (ICU MessageFormat) editable by an admin.
- And many other features that can be found in the application code.

### Other:

- Docker for development environment.
- Clippy for static code analysis.
- rustfmt for code formatting.
