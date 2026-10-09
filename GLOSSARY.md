# Glossary

Every term the documentation uses without defining it on the spot, and where it is explained. The specifications behind
them are in [STANDARDS.md](STANDARDS.md).

| Term | Meaning | Explained in |
|---|---|---|
| Access token | Short-lived JWT (five minutes) that authorizes one request. | [Authentication: Every request is authorized by a token](docs/authentication.md#every-request-is-authorized-by-a-token) |
| Adapter | An `infrastructure` implementation of a port: HTTP, Toasty, Keycloak, Redis, the file system. | [Architecture: A modular monolith with hexagonal modules](docs/architecture.md#a-modular-monolith-with-hexagonal-modules) |
| Audience | The `aud` claim naming whom a token is for; a token for anyone else is refused. | [Authentication: Every request is authorized by a token](docs/authentication.md#every-request-is-authorized-by-a-token) |
| `Auth`, `RequireAdmin` | The extractors a handler takes to require a signed-in caller or the `ADMIN` role. | [Architecture: Routes](docs/architecture.md#routes) |
| Authorization code flow | Sign-in by redirecting to Keycloak and exchanging the code it returns, on the server. | [Authentication: Signing in](docs/authentication.md#signing-in) |
| BFF (backend for frontend) | The application signs the user in and keeps the tokens; the browser holds only a session cookie. | [Authentication](docs/authentication.md) |
| Catalog | Every message the application can send, keyed, in Polish and English. | [translations](src/translations/README.md) |
| Composition root | `src/bootstrap`, the only place that names a concrete adapter. | [Architecture: The composition root](docs/architecture.md#the-composition-root) |
| Default | A message as shipped in `translations/`; refreshed at every start. | [translations: Defaults and overrides](src/translations/README.md#defaults-and-overrides) |
| Keycloak realm | The Keycloak tenant holding this application's users, roles and clients. | [Authentication](docs/authentication.md) |
| Layer | `domain`, `application` or `infrastructure`; dependencies point inward. | [Architecture: A modular monolith with hexagonal modules](docs/architecture.md#a-modular-monolith-with-hexagonal-modules) |
| Message key | A key of the catalog sent instead of a sentence; the client renders it. | [Architecture: Errors are message keys](docs/architecture.md#errors-are-message-keys) |
| Migration | A Toasty schema change in `toasty/migrations`, applied with the `cli` binary. | [Development Setup: Migrations](docs/development-setup.md#migrations) |
| Mirrored account | The local copy of a Keycloak user, updated whenever the token differs from it. | [identity: Accounts mirror Keycloak](src/identity/README.md#accounts-mirror-keycloak) |
| Module | `identity`, `files` or `translations`: one hexagon inside the binary. | [Architecture: A modular monolith with hexagonal modules](docs/architecture.md#a-modular-monolith-with-hexagonal-modules) |
| Override | An administrator's text for a key; it survives new defaults until it is reset. | [translations: Defaults and overrides](src/translations/README.md#defaults-and-overrides) |
| PKCE | A one-time secret binding the returned code to the browser that started the sign-in. | [Authentication: Signing in](docs/authentication.md#signing-in) |
| Port | A trait a module's `application` layer owns for anything outside it. | [Architecture: A modular monolith with hexagonal modules](docs/architecture.md#a-modular-monolith-with-hexagonal-modules) |
| Problem document | The JSON body of every refusal, carrying a message key and its arguments. | [Architecture: Errors are message keys](docs/architecture.md#errors-are-message-keys) |
| Refresh token | Long-lived token traded for a new access token; Keycloak rotates it on every use. | [Authentication: Sessions and refreshing](docs/authentication.md#sessions-and-refreshing) |
| Refresh token rotation | Every refresh invalidates the refresh token it used; replaying a spent one ends the session. | [Authentication: Sessions and refreshing](docs/authentication.md#sessions-and-refreshing) |
| Role | `USER` or `ADMIN`, a Keycloak realm role read from the access token. | [identity: Accounts mirror Keycloak](src/identity/README.md#accounts-mirror-keycloak) |
| `Sec-Fetch-Site` | Header saying where a request came from; an unsafe cross-site request gets no token. | [Authentication: Cross-site requests](docs/authentication.md#cross-site-requests) |
| Session | `RUST_AXUM_APP_SESSION`, kept in Redis, ending after ten hours without a request. | [Authentication: Sessions and refreshing](docs/authentication.md#sessions-and-refreshing) |
| Sign-out | Ends the local session and the Keycloak session. | [Authentication: Signing out](docs/authentication.md#signing-out) |
| Single-flight refresh | One refresh per refresh token, shared by every request of the session that needs it at once. | [Authentication: Sessions and refreshing](docs/authentication.md#sessions-and-refreshing) |
| Soft delete | Marking a file deleted, recording who and when, while keeping its row and bytes. | [files: Behavior](src/files/README.md#behavior) |
| Storage kind | Where a file's bytes live; only the local disk today. | [files: Ports](src/files/README.md#ports) |
