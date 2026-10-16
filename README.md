<p align="center">
    <img alt="" src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/Tokio-463F5F?style=for-the-badge&logo=tokio&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/Redis-DC382D?style=for-the-badge&logo=redis&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/Keycloak-00b8e3?style=for-the-badge&logo=keycloak&logoColor=4D4D4D">
    <img alt="" src="https://img.shields.io/badge/Tailwind_CSS-06B6D4?style=for-the-badge&logo=tailwindcss&logoColor=white">
</p>

# Rust Axum App

## Project Assumptions

Two applications in Rust. The back-end (`backend/`) is a modular monolith with hexagonal architecture and Domain-Driven
Design layering: three bounded contexts (`identity`, `files`, `translations`), each with its own domain, application
and infrastructure layers. The front-end (`frontend/`) renders its pages on the server with Topcoat and calls the back
end's API: [Architecture](docs/architecture.md#two-applications).

## Technology Stack

### Back-end

- Rust.
- Axum.
- Tokio.
- Toasty ORM.
- PostgreSQL.
- Redis (tower-sessions).
- reqwest.
- jsonwebtoken (JWKS).
- ICU MessageFormat.

### Front-end

- Rust.
- Topcoat (server-rendered pages, browser reactivity).
- Tailwind CSS, with the Topcoat UI components.
- ICU MessageFormat (FormatJS), from the back-end's catalog.
- Details: [front-end](frontend/README.md).

### Authentication

- **Identity provider**: Keycloak (realm `rust-axum-app`); the application never sees a password.
- **Pattern**: BFF; the application keeps the tokens, the browser holds only a session cookie.
- **Session store**: Redis (tower-sessions).
- **JWT**: verified by the auth guard, from a Bearer header or from the session.
- **Details**: [Authentication](docs/authentication.md).

### Core back-end

- Hexagonal architecture: ports are traits wired with generics, adapters are chosen only in `bootstrap`.
- The application has an exception handling mechanism (RFC 9457 problem details with message codes).
- The application has a logging mechanism (tracing).
- The application has configuration from environment variables only.
- The application has RBAC (Role Based Access Control).
- The application has versioned database migrations (Toasty CLI).
- The application has translations (ICU MessageFormat) editable by an admin.
- And many other features that can be found in the application code.

### Other

- Docker for development environment.
- Clippy for static code analysis.
- rustfmt for code formatting.

## Documentation

- [Contents](CONTENTS.md) – every document in the repository, the module it belongs to, and what it covers.
- [Glossary](GLOSSARY.md) – every term the docs use, and where it is explained.
- [Standards](STANDARDS.md) – the RFCs and specifications the code implements or depends on.
