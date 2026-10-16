# identity

Signs people in through Keycloak, keeps their sessions, verifies their tokens and mirrors their accounts, and holds
the roles and what they grant. Every other module that needs to know who is calling uses `Auth` or `Permitted` from
here and nothing else, and declares its own permissions with `Permission` and `Catalogue`.

The sign-in flow, sessions and refreshing are described in [Authentication](../../../docs/authentication.md); roles and
permissions in [Authorization](../../../docs/authorization.md).

## Ports

| Port             | Adapter                | What it does                                                     |
|------------------|------------------------|------------------------------------------------------------------|
| `UserRepository` | `ToastyUserRepository` | the mirrored accounts in PostgreSQL                              |
| `RoleRepository` | `ToastyRoleRepository` | the roles, what they grant and the registry of permissions       |
| `KeycloakClient` | `HttpKeycloakClient`   | code exchange, refresh and revocation                            |
| `RealmAdmin`     | `HttpRealmAdmin`       | realm roles made, removed, given and taken through the admin API |
| `TokenVerifier`  | `JwksTokenVerifier`    | signature, issuer, expiry and audience of a token                |

`IdentityPorts` bundles the five as associated types, so the service has one type parameter and `bootstrap` names the
production adapters in one place (`ProductionPorts`). `application/testing` holds in-memory fakes of all five.

## Accounts mirror Keycloak

Keycloak owns the person: credentials, email, name and realm roles. The `users` table holds a copy keyed by the
Keycloak identifier. It is created at the first sign-in and updated on any authenticated request whose token differs
from it, so a role granted or taken away in Keycloak reaches the database with the next request. Only the realm roles
with a role here are kept; Keycloak's own, such as `offline_access`, are left out.

When the copy cannot be written at sign-in, the new Keycloak session is revoked, so no one is signed in without an
account.

## Mechanisms

- [Cross-site requests](docs/mechanisms/cross-site-requests.md) – Why a forged request from another site cannot act with
  the user's session. CSRF tokens are not needed.
- [Token refresh](docs/mechanisms/token-refresh.md) – How the session keeps a valid access token without signing the
  user out when requests race.
