# Authentication

The application never handles a credential. Keycloak (realm `rust-axum-app`) owns every page that touches one: sign-up,
sign-in, email verification, password reset, two-factor authentication and acceptance of the terms of use. The
back-end is a BFF: it runs the sign-in, keeps the tokens on the server and gives the browser only a session cookie. The
browser reaches it through the front-end, which passes `/api/*` on unchanged:
[Architecture: Two applications](architecture.md#two-applications).

## Signing in

```mermaid
sequenceDiagram
    participant B as Browser
    participant F as Front-end
    participant A as Back-end
    participant K as Keycloak
    participant R as Redis

    B->>F: GET /api/auth/authorize
    F->>A: passed on
    A-->>B: 303 to Keycloak, with state and the PKCE challenge
    B->>K: sign in on Keycloak's page
    K-->>B: 302 to /api/auth/callback?code&state on the front-end's address
    B->>F: GET /api/auth/callback?code&state
    F->>A: passed on
    A->>K: code + client secret + PKCE verifier
    K-->>A: access, refresh and ID token
    A->>R: store the tokens in the session
    A-->>B: 303 to AUTH_POST_LOGIN_URL, the front-end's home page, Set-Cookie with the session id
    B->>F: page request + session cookie
    F->>A: API call + session cookie
    A->>R: read the session
    Note over A: the request is authorized by the session's access token
```

1. The browser opens `GET /api/auth/authorize`. The application stores a fresh `state` and PKCE verifier in the session
   and redirects to Keycloak with the challenge. Optional parameters pass through: `register=true` opens the sign-up
   page, and `kc_action` starts one of `CONFIGURE_TOTP`, `UPDATE_PASSWORD` or `delete_credential:<id>` with a UUID
   ([Account actions](#account-actions)); any other value is dropped. The `Accept-Language`
   header picks Keycloak's language when it is `pl` or `en`, as `ui_locales`. Keycloak shows a signed-in person the
   language stored on their account instead, so before a `kc_action` the front-end stores that language there, through
   `PUT /api/users/me/language`. `return_to` names the page to come back to: a path on the front-end's own site, kept
   in the session, and dropped when it is anything else, so the sign-in cannot send the browser elsewhere. The
   front-end passes the page the reader is on with every sign-in, and a page that needs a session answers a signed-out
   reader with `303` to the sign-in, so they come back to that page signed in.
2. Keycloak returns to `GET /api/auth/callback`. The application checks that the `state` is the one it issued to this
   browser, exchanges the code with the confidential client `rust-axum-app` and its secret, verifies the access token
   and mirrors the account into PostgreSQL.
3. It then issues a new session identifier, stores the tokens in the session and redirects to the `return_to` page, on
   the address of `AUTH_POST_LOGIN_URL`, or to `AUTH_POST_LOGIN_URL` itself when none was given.
   When the account cannot be written it ends the Keycloak session again; any failure redirects with
   `?error=sign_in_failed`, and a `state` this browser was not given with `?error=state_mismatch`.

The browser holds only the `RUST_AXUM_APP_SESSION` cookie: `HttpOnly`, `SameSite=Lax`, and `Secure` when
`SESSION_COOKIE_SECURE=true`.

## Account actions

The account page changes the password and turns the second factor on and off on Keycloak's own pages, so no password
and no TOTP secret passes through the application. `GET /api/users/me/second-factor`, of the `keycloak_account`
module, reads the caller's OTP credential
from Keycloak's account API (`/realms/<realm>/account/credentials`) with the caller's own access token, which carries
the `account` audience and the `manage-account` role every account gets, so the application needs no right to read
anyone else's credentials. The page then offers `CONFIGURE_TOTP` when there is none and `delete_credential:<id>` when
there is one: Keycloak asks the person to confirm, signs them in again when their sign-in is old, removes the
credential only when it is theirs, and returns through the usual sign-in.

## Every request is authorized by a token

`authenticate`, the middleware `bootstrap` layers onto the protected routers, takes the access token from
`Authorization: Bearer` or, for a browser, from its session. `JwksTokenVerifier` checks the signature against
Keycloak's published keys (cached by key id), the issuer, the expiry and the audience (`KEYCLOAK_AUDIENCE`), and the
roles come from `realm_access.roles`. The resulting `Caller`, with what its roles grant, is what `Auth` and
`Permitted` read: [Authorization](authorization.md).

Either way the decision rests on the token alone, so any instance can serve any request.

## Sessions and refreshing

The session lives in Redis, and its access token is refreshed once per refresh token, across instances: [Token
refresh](../backend/src/identity/docs/mechanisms/token-refresh.md).

## Signing out

`POST /api/auth/logout` revokes the refresh token at Keycloak, which ends the Keycloak session, and clears the local
one. The front-end's sign-out button posts to its own `/sign-out`, which calls it with the session cookie, passes the
cookie's removal on to the browser and returns home.

## Cross-site requests

An unsafe request from another site gets no token: [Cross-site
requests](../backend/src/identity/docs/mechanisms/cross-site-requests.md).

## The sign-in pages

Keycloak draws the sign-in, sign-up, password reset, terms and second-factor pages, and the mails it sends, with the
theme in [`keycloak/themes/rust-axum-app`](../keycloak/themes/rust-axum-app), so they look like the front-end:

| Part                      | Holds                                                                                                                                    |
|---------------------------|------------------------------------------------------------------------------------------------------------------------------------------|
| `login/theme.properties`  | extends Keycloak's `keycloak.v2`, whose templates draw every page                                                                        |
| `login/template.ftl`      | the page frame: the front-end's top bar, with the theme and language menus, and its footer                                               |
| `login/resources`         | `css/app.css`, the front-end's colors, font and card on PatternFly; `js/theme.js`, the theme; `js/language.js`, the language handed back |
| `login/messages`          | the texts of the bar and the footer, in Polish and English, as the front-end words them                                                  |
| `email/html/template.ftl` | the frame of every mail; the texts stay Keycloak's own, in Polish and English                                                            |

The theme menu offers System, Light and Dark, as the front-end's does. A realm that turns dark mode off gets the light
theme and no menu. The theme the reader last chose, in the front-end or in that menu, is kept in the cookie
`ui_theme_rust-axum-app`, which both read and write: the front-end writes it with its own when the reader picks a theme,
`theme.js` writes it on a choice in its menu, and each reads it on every page. It lives under the domain Keycloak shares
with the front-end, the parent of Keycloak's host (on localhost, every port), for a year, and is not `HttpOnly`, as
Keycloak's script reads it. Being a shared record rather than a message, it keeps the two in step however the reader
moves between them: through a sign-in, a cancelled account action, whose redirect drops the `ui_theme` the front-end
also sends, the browser's history or another tab.

The language goes both ways too. The language the reader last chose, in the front-end or in Keycloak's own menu, is kept
in the cookie `ui_locale_rust-axum-app`, which both write, under the same domain as the theme's and for a year: the
front-end writes it with its own `lang` cookie when the reader picks a language and reads it on every page, and
`language.js` writes it on a choice in Keycloak's menu. Keycloak shows the sign-in page in `ui_locales` and a signed-in
person their account's language, so the front-end stores a signed-in reader's choice there at once, through `PUT
/api/users/me/language`, and again before a `kc_action`.

`template.ftl` is a copy of `keycloak.v2`'s own, changed only where it says `rust-axum-app`. Keycloak builds its pages
from that template, so after an upgrade of Keycloak the copy is compared with the new release's and brought up to date.
The Keycloak theme workflow does the comparison: `keycloak/check-template.sh` fails when the template in the Keycloak
image of `docker-compose.local.yml` is not the one the copy was made from, recorded as `parentTemplateSha256` in
`theme.properties`, which changes with the copy. The colors are a copy of the front-end's tokens, and the front-end's
`keycloak_theme` test fails when the two differ. A mail's colors are those tokens as hex, since mail clients read
neither `oklch()` nor `light-dark()`. The links of the bar and the footer lead to the client's base URL, `baseUrl` in
the realm.

The realm names the theme in `loginTheme` and `emailTheme`. Locally, compose mounts the folder into Keycloak. A
Keycloak that serves several applications, each in its own realm, takes it as a JAR in `/opt/keycloak/providers`:

```bash
keycloak/build-theme.sh   # writes keycloak/target/rust-axum-app-keycloak-theme.jar
```

`keycloak/Dockerfile` builds an image holding that JAR in `/themes`, which the Keycloak theme workflow publishes on a
`keycloak-theme-v*` tag.

The realm file of that Keycloak names `rust-axum-app` in `loginTheme` and `emailTheme`, and another theme replaces
it by its own JAR and those two fields.

## Code

The paths are under `backend/src/`; the front-end's side is in its [README](../frontend/README.md), and what it shares
with Keycloak's pages in `frontend/src/keycloak` and `frontend/src/keycloak_cookies.rs`.

| Path                                                 | Role                                               |
|------------------------------------------------------|----------------------------------------------------|
| `identity/infrastructure/http/routes/auth.rs`        | authorize, callback, session, logout               |
| `identity/infrastructure/http/extract.rs`            | the `authenticate` middleware, `Auth`, `Permitted` |
| `identity/infrastructure/keycloak/client.rs`         | code exchange, refresh, revocation                 |
| `identity/infrastructure/keycloak/shared_refresh.rs` | one refresh per refresh token across instances     |
| `identity/infrastructure/keycloak/verifier.rs`       | token verification against Keycloak's keys         |
| `identity/infrastructure/session_store.rs`           | the Redis session store                            |
| `identity/application/service.rs`                    | sign-in, refresh, sign-out and the account mirror  |
| `keycloak_account/`                                  | the second factor and the language on the account  |
