# Standards

The specifications this repository implements or depends on, what for, and where the documentation covers it.

| Standard | Title | Used for | Explained in |
|---|---|---|---|
| [RFC 6749](https://www.rfc-editor.org/rfc/rfc6749) | OAuth 2.0 | the authorization code flow and refresh tokens | [Authentication: Signing in](docs/authentication.md#signing-in) |
| [RFC 7636](https://www.rfc-editor.org/rfc/rfc7636) | PKCE | binding the code to the browser | [Authentication: Signing in](docs/authentication.md#signing-in) |
| [OIDC Core](https://openid.net/specs/openid-connect-core-1_0.html) | OpenID Connect Core 1.0 | the ID token and the `openid` scope | [Authentication: Signing in](docs/authentication.md#signing-in) |
| [RFC 7519](https://www.rfc-editor.org/rfc/rfc7519) | JSON Web Token (JWT) | the access token and its claims | [Authentication: Every request is authorized by a token](docs/authentication.md#every-request-is-authorized-by-a-token) |
| [RFC 7517](https://www.rfc-editor.org/rfc/rfc7517) | JSON Web Key (JWK) | Keycloak's published signing keys | [Authentication: Every request is authorized by a token](docs/authentication.md#every-request-is-authorized-by-a-token) |
| [RFC 6750](https://www.rfc-editor.org/rfc/rfc6750) | OAuth 2.0 Bearer Token Usage | `Authorization: Bearer` on API calls | [Authentication: Every request is authorized by a token](docs/authentication.md#every-request-is-authorized-by-a-token) |
| [RFC 9457](https://www.rfc-editor.org/rfc/rfc9457) | Problem Details for HTTP APIs | the error body of every refusal | [Architecture: Errors are message keys](docs/architecture.md#errors-are-message-keys) |
| [RFC 6265bis](https://datatracker.ietf.org/doc/draft-ietf-httpbis-rfc6265bis/) | Cookies: HTTP State Management Mechanism (draft) | the `SameSite=Lax` session cookie | [Authentication: Cross-site requests](docs/authentication.md#cross-site-requests) |
| [Fetch Metadata](https://www.w3.org/TR/fetch-metadata/) | Fetch Metadata Request Headers (W3C) | refusing unsafe cross-site requests by `Sec-Fetch-Site` | [Authentication: Cross-site requests](docs/authentication.md#cross-site-requests) |
| [ICU MessageFormat](https://unicode-org.github.io/icu/userguide/format_parse/messages/) | ICU MessageFormat | the syntax of every message in the catalog | [translations: Defaults and overrides](src/translations/README.md#defaults-and-overrides) |
| [RFC 6238](https://www.rfc-editor.org/rfc/rfc6238) | TOTP: Time-Based One-Time Password | the second factor configured on Keycloak's pages | [Authentication: Signing in](docs/authentication.md#signing-in) |
| [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110) | HTTP Semantics | `Accept-Language`, which picks Keycloak's language | [Authentication: Signing in](docs/authentication.md#signing-in) |
