# identity

Signs people in through Keycloak, keeps their sessions, verifies their tokens and mirrors their accounts. Every other
module that needs to know who is calling uses `Auth` or `RequireAdmin` from here and nothing else.

The sign-in flow, sessions and refreshing are described in [Authentication](../../docs/authentication.md).

## Ports

| Port             | Adapter                | What it does                                      |
|------------------|------------------------|---------------------------------------------------|
| `UserRepository` | `ToastyUserRepository` | the mirrored accounts in PostgreSQL               |
| `KeycloakClient` | `HttpKeycloakClient`   | code exchange, refresh and revocation             |
| `TokenVerifier`  | `JwksTokenVerifier`    | signature, issuer, expiry and audience of a token |

`IdentityPorts` bundles the three as associated types, so the service has one type parameter and `bootstrap` names the
production adapters in one place (`ProductionPorts`). `application/testing` holds in-memory fakes of all three.

## Accounts mirror Keycloak

Keycloak owns the person: credentials, email, name and realm roles. The `users` table holds a copy keyed by the
Keycloak identifier. It is created at the first sign-in and updated on any authenticated request whose token differs
from it, so a role granted or taken away in Keycloak reaches the database with the next request. Only the roles the
application knows (`USER`, `ADMIN`) are kept; they are seeded at startup.

When the copy cannot be written at sign-in, the new Keycloak session is revoked, so no one is signed in without an
account.
