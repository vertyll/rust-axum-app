# Cross-site requests

Why a forged request from another site cannot act with the user's session. Three layers stand in its way: the
`SameSite=Lax` session cookie, the `Sec-Fetch-Site` check and a CSRF token bound to the session.

## The session cookie

The session cookie is `SameSite=Lax`, so the browser leaves it off a request another site sends with an unsafe method.
`Lax` alone is not enough: it still sends the cookie with a request from a sibling subdomain, which counts as the same
site, and with a top-level `GET`.

## Where the request came from

`authenticate` gives the session's token only to a request from this origin: an unsafe request whose `Sec-Fetch-Site`
is neither `same-origin` nor `none` is answered `401`, and a sign-out from another site `403` with the problem
`auth.errors.cross_site_request`. A browser too old to send the header passes this check, and the CSRF token below
still applies.

## The CSRF token

The contract is the one a single-page client of a Spring Security API expects:

- At sign-in the back-end draws a new token, 32 bytes from a cryptographically secure generator in base64url, keeps it
  in the session in Redis, and sets it in the cookie `XSRF-TOKEN`: `Path=/`, `SameSite=Lax`, not `HttpOnly`, so the
  application's scripts can read it, and `Secure` exactly where the session cookie is; a local machine without TLS
  drops it, as the session cookie does. The token known before a sign-in is worthless after it.
- Every unsafe request the session authenticates, any method but `GET`, `HEAD`, `OPTIONS` and `TRACE`, sends the
  token back in the header `X-XSRF-TOKEN`. The back-end compares it in constant time with the session's token, not
  with the cookie, so a cookie a sibling subdomain planted does not match. A missing or different token is answered
  `403` with the problem `auth.errors.csrf_token_invalid`; a request without a session is answered `401` first, so an
  ended session still sends the reader to sign in.
- A request authenticated by `Authorization: Bearer` needs no token: another site cannot make a browser send that
  header. A request with any other scheme, such as `Basic`, is authenticated by its session and needs the token.
- The cookie ends with the browser. Each request the session authenticates writes it again when the browser's cookie
  holds another value or none, and gives a session from before the token existed its first one.
- `POST /api/auth/logout` needs the token as well while someone is signed in, and removes the cookie.

The check lives in the `identity` module's HTTP adapter, `infrastructure/http/csrf.rs`, which `authenticate` and the
sign-out call. It is written for this application rather than taken from a crate: the crates on offer keep the token in
a cookie of their own instead of the session in Redis, or are built on an old axum, while the code here needs only
`subtle`, for the constant-time comparison, and `tower-cookies`, the cookie jar the session layer already runs.

## How the front-end sends it

- A form is posted to a route of the front-end, never to the API, and Topcoat's origin policy refuses one sent from
  another site before the route runs. When the route calls the back-end it passes on the browser's session cookie and
  its `XSRF-TOKEN` cookie, and a change sends the token in `X-XSRF-TOKEN`, as a script of the browser would. The
  back-end's `Set-Cookie`, the token's included, goes back to the browser. The forms carry no hidden field, so their
  markup and the `Content-Security-Policy` stay as they were.
- The upload script, `app/files/upload.js`, calls the API through the proxy itself. It reads `XSRF-TOKEN` from
  `document.cookie` and sends it in `X-XSRF-TOKEN`. The proxy passes the browser's headers on as they came and never
  adds the token, so a request another site makes through it is held to the same check.
