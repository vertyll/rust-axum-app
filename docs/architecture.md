# Architecture

## A modular monolith with hexagonal modules

The application is one binary made of three modules, each split into the same three layers:

| Module                                          | Holds                                                                         |
|-------------------------------------------------|-------------------------------------------------------------------------------|
| [`identity`](../src/identity/README.md)         | sign-in, sessions, token verification and the accounts mirrored from Keycloak |
| [`files`](../src/files/README.md)               | uploads stored on the local disk, with their metadata in PostgreSQL           |
| [`translations`](../src/translations/README.md) | the message catalog, its defaults and the admin overrides                     |

| Layer            | Holds                                                                           |
|------------------|---------------------------------------------------------------------------------|
| `domain`         | entities, value types, errors and the repository traits; no I/O                 |
| `application`    | the service and its use cases, and the ports: traits for everything outside     |
| `infrastructure` | the adapters: HTTP routes, Toasty persistence, Keycloak, Redis, the file system |

Dependencies point inwards: `infrastructure` implements the ports of `application`, which uses `domain`. Each
module's `application/testing` holds in-memory fakes of its ports, so a use case is tested without a database or
Keycloak.

## The composition root

`src/bootstrap` is the only place that names a concrete adapter. A service is generic over its ports, and `bootstrap`
fixes them with type aliases (`AppIdentityService`, `AppFilesService`, `AppTranslationsService`), so wiring is resolved
at compile time with no container and no trait objects. Adding a port to `identity` is one associated type in
`IdentityPorts`.

`bootstrap` also decides which routers sit behind the auth guard; a module only provides its routes.

## Routes

The routes are declared in each module's `infrastructure/http/routes`, and `bootstrap::router` nests each module's
router under its path and decides which sit behind the auth guard. There is no OpenAPI description; the routes and
their extractors are the reference.

A handler states what it needs through its extractor: `Auth` for a signed-in caller, `RequireAdmin` for the `ADMIN`
role.

## Errors are message keys

Every refusal is an RFC 9457 problem document (`application/problem+json`, built in
`shared_infrastructure/problem.rs`). The server never sends prose a person reads:

| Field                     | Holds                                                                 |
|---------------------------|-----------------------------------------------------------------------|
| `type`, `title`, `status` | `about:blank`, the status's reason phrase, the HTTP status            |
| `code`                    | a key of the translation catalog, e.g. `errors.not_found`             |
| `detail`                  | the same key as `code`                                                |
| `args`                    | the ICU arguments for that key; in a validation error, keyed by field |
| `errors`                  | in a validation error, the message keys of each invalid field         |

The client translates: it loads the catalog from `GET /api/translations/{language}` and formats `code` with `args` as
an ICU MessageFormat message, in its reader's language. A new error is therefore a new key in `translations/en.json`
and `pl.json`, never a sentence in the code. Each module maps its domain errors to keys in
`infrastructure/http/error.rs`.

## Database

PostgreSQL through the Toasty ORM. Each module keeps its persistence records in `infrastructure/persistence/records`,
separate from the domain types, and converts between the two in its repository.
