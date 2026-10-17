# Authorization

Answers: who decides what a person may do, and where that decision is made.

## The split

| Concern                         | Owner                        | Where it lives                                           |
|---------------------------------|------------------------------|----------------------------------------------------------|
| Who the person is               | Keycloak                     | the access token, `realm_access.roles`                   |
| Which permissions exist         | the module that checks one   | its `CATALOGUE`, in `application/permissions.rs`         |
| What a role grants              | the `identity` module        | the `roles`, `permissions` and `role_permissions` tables |
| Whether this request is allowed | the handler's `Permitted<P>` | the `Caller`, built once per request                     |

Keycloak is the identity provider, not the authorization system: it says who is calling and which realm roles they
hold, never what those roles may do. What a role grants stays out of the token, so a permission taken away stops
working with the next request rather than when the token expires.

## A permission is declared where it is checked

A permission is real only where some code checks it, so it is a type in the module that checks it:

```rust
pub struct FilesManage;

impl Permission for FilesManage {
    const NAME: &'static str = "FILES_MANAGE";
    const DESCRIPTION: &'static str = "Change any file's details, and delete files";
}

pub const CATALOGUE: Catalogue = Catalogue {
    module: "files",
    permissions: &[Declared::of::<FilesManage>()],
    stock_roles: &[],
};
```

A handler names the type in its extractor, so a misspelt permission does not compile:

```rust
async fn delete_file(Permitted(caller, _): Permitted<FilesManage>, /* … */) { /* … */ }
```

`bootstrap::CATALOGUES` lists every module's catalog, and the application registers them on every start, before it
listens:

| Rule                                                | Why                                                                     |
|-----------------------------------------------------|-------------------------------------------------------------------------|
| the module that checks a permission declares it     | a name nobody checks would be a lie in the list an administrator uses   |
| a permission no module declares any more is removed | and withdrawn from every role, so nothing grants a dead name            |
| two modules may not declare the same name           | the start fails, as does a stock role granting an undeclared permission |

## Permissions

| Module         | Permission          | Allows                                                 |
|----------------|---------------------|--------------------------------------------------------|
| `identity`     | `USERS_VIEW`        | seeing every account and the roles it holds            |
| `identity`     | `USERS_MANAGE`      | giving accounts roles and taking them away             |
| `identity`     | `ROLES_VIEW`        | seeing the roles, the permissions and what each grants |
| `identity`     | `ROLES_MANAGE`      | making, changing and removing roles                    |
| `files`        | `FILES_VIEW`        | seeing and downloading every file                      |
| `files`        | `FILES_MANAGE`      | uploading files, changing their details, deleting them |
| `translations` | `TRANSLATIONS_VIEW` | seeing every message with its default                  |
| `translations` | `TRANSLATIONS_EDIT` | changing messages and resetting them                   |

What needs only a signed-in caller, such as reading one's own account, takes `Auth` and no permission.

## Stock roles

A module may name the roles it ships with and what they start out granting; `identity` ships `admin` and `user`. A
stock role is made only where no role of its name exists, and an existing one gains only permissions never registered
before. So a module added later reaches the roles it belongs to, while an administrator's changes survive every start.
A stock role other than the system ones that an administrator removes comes back, empty of their changes, on the next
start.

## `unrestricted`

`admin` holds every permission, also one a module declares tomorrow. It is a flag, not a list: deriving it from
"currently holds all" would silently widen a role the day a new permission appeared.

## What cannot be undone by accident

| Refused                                                             | Answer                                         |
|---------------------------------------------------------------------|------------------------------------------------|
| removing `admin` or `user`, the system roles                        | `409`, `roles.errors.system_role`              |
| removing an unrestricted role; it is made restricted first          | `409`, `roles.errors.unrestricted_role`        |
| making the last unrestricted role restricted                        | `409`, `roles.errors.last_unrestricted_role`   |
| removing a role some account holds                                  | `409`, `roles.errors.still_assigned`           |
| taking the last unrestricted role from the last account holding one | `409`, `roles.errors.last_unrestricted_holder` |

So somebody can always manage the roles. These rules span several roles and accounts, so no single role can check
them: they are a domain service, `identity::domain::role_rules`, which the use cases hand the roles and accounts it
needs, and which does no I/O. Who holds a role is read from the accounts mirrored from the tokens.

## Roles live in Keycloak too

A role a person can hold is a realm role, since the token carries realm roles. Making or removing a role here does the
same in Keycloak, and giving a person a role is done in Keycloak, through its admin API, as the application's service
account. The service account of the `rust-axum-app` client holds `manage-realm` and `manage-users` of
`realm-management` for that.

| Here            | In Keycloak                                  |
|-----------------|----------------------------------------------|
| role `editor`   | realm role `EDITOR`                          |
| `admin`, `user` | `ADMIN`, `USER`, already in the realm export |

A role given or taken away reaches the person's token, and so this application, when the token is next refreshed: within
the access token's lifetime, five minutes. A realm role with no role here, such as Keycloak's own `offline_access`,
grants nothing.

## Endpoints

| Method and path                             | Needs          | Does                                                    |
|---------------------------------------------|----------------|---------------------------------------------------------|
| `GET /api/users/me/permissions`             | a session      | what the caller's roles grant: `unrestricted` and names |
| `GET /api/users/me/second-factor`           | a session      | whether the caller set up a second factor, and its id   |
| `GET /api/admin/permissions`                | `ROLES_VIEW`   | the permissions, grouped by the module declaring them   |
| `GET /api/admin/roles`                      | `ROLES_VIEW`   | every role and what it grants                           |
| `GET /api/admin/roles/{name}`               | `ROLES_VIEW`   | one role                                                |
| `POST /api/admin/roles`                     | `ROLES_MANAGE` | makes a role, here and in Keycloak                      |
| `PUT /api/admin/roles/{name}`               | `ROLES_MANAGE` | sets its description, `unrestricted` and permissions    |
| `DELETE /api/admin/roles/{name}`            | `ROLES_MANAGE` | removes it, here and in Keycloak                        |
| `PUT /api/admin/users/{id}/roles/{name}`    | `USERS_MANAGE` | gives the account the role in Keycloak                  |
| `DELETE /api/admin/users/{id}/roles/{name}` | `USERS_MANAGE` | takes it away in Keycloak                               |

A role is written as:

```json
{
  "name": "editor",
  "description": "Edits the texts",
  "unrestricted": false,
  "permissions": ["TRANSLATIONS_VIEW", "TRANSLATIONS_EDIT"]
}
```

A caller without the permission gets a `403` problem whose `code` is `auth.errors.permission_denied`, with the
permission's name as the `permission` argument.

## Adding a permission

1. Declare it as a type implementing `Permission` in the module's `application/permissions.rs`, and add it to the
   module's `CATALOGUE`.
2. A new module adds its `CATALOGUE` to `bootstrap::CATALOGUES`.
3. Guard the handler with `Permitted<ThePermission>`.

Nothing else changes: the next start registers it, and `admin` holds it at once.
