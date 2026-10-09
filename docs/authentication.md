# Authentication

The application never handles a credential. Keycloak (realm `rust-axum-app`) owns every page that touches one: sign-up,
sign-in, email verification, password reset, two-factor authentication and acceptance of the terms of use. The
application is a BFF: it runs the sign-in, keeps the tokens on the server and gives the browser only a session cookie.

## Signing in

1. The browser opens `GET /api/auth/authorize`. The application stores a fresh `state` and PKCE verifier in the session
   and redirects to Keycloak with the challenge. Optional parameters pass through: `register=true` opens the sign-up
   page, and `kc_action` starts one of `CONFIGURE_TOTP`, `UPDATE_PASSWORD` or `delete_credential`. The
   `Accept-Language` header picks Keycloak's language when it is `pl` or `en`.
2. Keycloak returns to `GET /api/auth/callback`. The application checks that the `state` is the one it issued to this
   browser, exchanges the code with the confidential client `rust-axum-app` and its secret, verifies the access token
   and mirrors the account into PostgreSQL.
3. It then issues a new session identifier, stores the tokens in the session and redirects to `AUTH_POST_LOGIN_URL`.
   When the account cannot be written it ends the Keycloak session again; any failure redirects with
   `?error=sign_in_failed`, and a `state` this browser was not given with `?error=state_mismatch`.

The browser holds only the `RUST_AXUM_APP_SESSION` cookie: `HttpOnly`, `SameSite=Lax`, and `Secure` when
`SESSION_COOKIE_SECURE=true`.

## Every request is authorized by a token

`authenticate`, the middleware `bootstrap` layers onto the protected routers, takes the access token from
`Authorization: Bearer` or, for a browser, from its session. `JwksTokenVerifier` checks the signature against
Keycloak's published keys (cached by key id), the issuer, the expiry and the audience (`KEYCLOAK_AUDIENCE`), and the
roles come from `realm_access.roles`. The resulting `Caller` is what `Auth` and `RequireAdmin` read.

Either way the decision rests on the token alone, so any instance can serve any request.

## Sessions and refreshing

The session lives in Redis (tower-sessions, keys under `rust-axum-app`) and expires after ten hours without a request
(`SESSION_INACTIVITY_TIMEOUT`). Access tokens live five minutes, and the session's token is refreshed when less than 30
seconds of it is left.

Keycloak rotates refresh tokens: every refresh returns a new one and invalidates the old one, and replaying a spent one
ends the session. Two requests of one session refreshing at once would therefore sign the user out, so a refresh runs
once per refresh token:

- within one instance, `HttpKeycloakClient` lets the first request refresh and hands its result to the others;
- across instances, `SharedRefreshes` takes a lock in Redis; the instance holding it refreshes and leaves the new tokens
  in Redis for 30 seconds, where the others pick them up. When Redis is unreachable an instance refreshes on its own.

When Keycloak refuses a refresh, the session is cleared and the request is answered `401`: a blocked account or a
revoked session stops working within five minutes.

## Signing out

`POST /api/auth/logout` revokes the refresh token at Keycloak, which ends the Keycloak session, and clears the local
one.

## Cross-site requests

The cookie is `SameSite=Lax`, which keeps it off cross-site writes. The middleware adds a second check: an unsafe request
whose `Sec-Fetch-Site` is neither `same-origin` nor `none` is not given the session's token, and a logout from another
site is refused with `403`. CSRF tokens are therefore not used.

## Code

| Path                                                 | Role                                                  |
|------------------------------------------------------|-------------------------------------------------------|
| `identity/infrastructure/http/routes/auth.rs`        | authorize, callback, session, logout                  |
| `identity/infrastructure/http/extract.rs`            | the `authenticate` middleware, `Auth`, `RequireAdmin` |
| `identity/infrastructure/keycloak/client.rs`         | code exchange, refresh, revocation                    |
| `identity/infrastructure/keycloak/shared_refresh.rs` | one refresh per refresh token across instances        |
| `identity/infrastructure/keycloak/verifier.rs`       | token verification against Keycloak's keys            |
| `identity/infrastructure/session_store.rs`           | the Redis session store                               |
| `identity/application/service.rs`                    | sign-in, refresh, sign-out and the account mirror     |
