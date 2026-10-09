# Token refresh

How the session keeps a valid access token without signing the user out when requests race.

The session lives in Redis (tower-sessions, keys under `rust-axum-app`) and expires after ten hours without a request
(`SESSION_INACTIVITY_TIMEOUT`). Access tokens live five minutes, and the session's token is refreshed when less than 30
seconds of it is left.

> [!IMPORTANT]
>
> Keycloak rotates refresh tokens: every refresh returns a new one and invalidates the old one, and replaying a spent
> one ends the session. Two requests of one session refreshing at once would therefore sign the user out.

A refresh therefore runs once per refresh token:

- within one instance, `HttpKeycloakClient` lets the first request refresh and hands its result to the others;
- across instances, `SharedRefreshes` takes a lock in Redis; the instance holding it refreshes and leaves the new tokens
  in Redis for 30 seconds, where the others pick them up. When Redis is unreachable an instance refreshes on its own.

When Keycloak refuses a refresh, the session is cleared and the request is answered `401`: a blocked account or a
revoked session stops working within five minutes.
