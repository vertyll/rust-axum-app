# Authentication

- **Identity provider**: Keycloak (realm `rust-axum-app`) owns every page that touches a credential: sign-up, sign-in,
  email verification, password reset, two-factor authentication and acceptance of the terms of use. The application
  never sees a password.
- **Pattern**: BFF. A browser signs in at `GET /api/auth/authorize` with the authorization code flow and PKCE; the
  application keeps the tokens and the browser holds only the `RUST_AXUM_APP_SESSION` cookie (`HttpOnly`,
  `SameSite=Lax`, `Secure` in production).
- **Session store**: Redis (tower-sessions, `rust-axum-app:session` namespace).
- **JWT**: the auth guard takes the access token from `Authorization: Bearer` or from the session and verifies its
  signature against Keycloak's published keys, the issuer, the expiry and the audience (`rust-axum-app`).
- **State**: the back-end is stateless: every request is authorized by the JWT alone, so any instance can serve it. The
  only state is the browser session, and it lives in Redis, outside the application.
- **Token lifecycle**: access tokens live five minutes; every refresh returns a new refresh token and invalidates the
  old one, and concurrent requests of one session share a single refresh, across replicas too (a lock in Redis). Signing
  out revokes the refresh token at Keycloak.
- **Cross-site requests**: `SameSite=Lax` plus `Sec-Fetch-Site`, so a write or a logout sent from another site is
  refused.
- **Accounts**: mirrored into PostgreSQL with their realm roles (`USER`, `ADMIN`) whenever they change.
