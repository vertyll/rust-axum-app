<p align="center">
    <img alt="" src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/Tokio-463F5F?style=for-the-badge&logo=tokio&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/Redis-DC382D?style=for-the-badge&logo=redis&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/Keycloak-00b8e3?style=for-the-badge&logo=keycloak&logoColor=4D4D4D">
</p>

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

- **Identity provider**: Keycloak (realm `rust-axum-app`) owns every page that touches a credential: sign-up, sign-in,
  email verification, password reset, two-factor authentication and acceptance of the terms of use. The application
  never sees a password.
- **Pattern**: BFF. A browser signs in at `GET /api/auth/authorize` with the authorization code flow and PKCE; the
  application keeps the tokens and the browser holds only the `RUST_AXUM_APP_SESSION` cookie (`HttpOnly`,
  `SameSite=Lax`, `Secure` in production).
- **Session store**: Redis (tower-sessions), so the application holds no state of its own.
- **JWT**: the auth guard takes the access token from `Authorization: Bearer` or from the session and verifies its
  signature against Keycloak's published keys, the issuer, the expiry and the audience (`rust-axum-app`).
- **Token lifecycle**: access tokens live five minutes; every refresh returns a new refresh token and invalidates the
  old one, and concurrent requests of one session share a single refresh. Signing out revokes the refresh token at
  Keycloak.
- **Cross-site requests**: `SameSite=Lax` plus `Sec-Fetch-Site`, so a write or a logout sent from another site is
  refused.
- **Accounts**: mirrored into PostgreSQL with their realm roles (`USER`, `ADMIN`) whenever they change.

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
