# keycloak_account

Holds what a signed-in person keeps on their Keycloak account and changes on Keycloak's own pages: the second factor,
and the language those pages show them in. It reads and writes them through Keycloak's account API with the person's
own access token, which carries the `account` audience and the `manage-account` role every account gets, so it needs
no right over anyone else's account. It uses `CallerToken` from `identity`, behind the same auth guard as the other
modules, and `identity` never imports it.

What the account page and the language menu do with it is described in
[Authentication](../../../docs/authentication.md).

## Ports

| Port         | Adapter          | What it does                                                                       |
|--------------|------------------|------------------------------------------------------------------------------------|
| `AccountApi` | `HttpAccountApi` | the caller's OTP credential, and the `locale` attribute written back with the rest |

`application/testing` holds an in-memory fake of it.

## Routes

`bootstrap` nests them under `/api/users`, beside the routes of `identity`, behind the auth guard:

| Route                             | What it does                                                                 |
|-----------------------------------|------------------------------------------------------------------------------|
| `GET /api/users/me/second-factor` | whether the caller set up a second factor, and the id Keycloak removes it by |
| `PUT /api/users/me/language`      | stores the caller's language on their account, for Keycloak's pages          |

A language Keycloak does not store leaves the account as it was and is only logged, as Keycloak's pages then open in
the language the account already has. An unreachable Keycloak answers `503` with `keycloak_account.errors.unavailable`.
