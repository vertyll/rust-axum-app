# Rust Axum App — front-end

A server-rendered web application on [Topcoat](https://github.com/tokio-rs/topcoat), in Rust. It holds no token and no
session of its own: the back-end signs people in and keeps their sessions ([Authentication](../docs/authentication.md)),
and every page calls the back-end's API with the browser's session cookie.

## Layout

The code is grouped by feature. A feature's module holds its pages, its form routes and, in `api.rs`, its calls to the
back-end; its URL is its module's path, so `app/files/file_id` serves `/files/{file_id}`.

| Path                         | Holds                                                                                                   |
|------------------------------|---------------------------------------------------------------------------------------------------------|
| `src/app`                    | the layout, the home page, the security headers and every feature below                                 |
| `src/app/navigation`         | the top bar with the modules, the account, theme and language menus, and the side bar with the sections |
| `src/app/files`              | the files: list, upload (`upload.js`), details, download, changes; `api.rs` calls `/api/files`          |
| `src/app/account`            | the reader's account, what their roles grant, password and second factor                                |
| `src/app/admin/users`        | the accounts and their roles; `api.rs` calls `/api/users` and `/api/admin/users`                        |
| `src/app/admin/roles`        | the roles and what each grants; `api.rs` calls `/api/admin/roles` and `/api/admin/permissions`          |
| `src/app/admin/translations` | the message catalog; `api.rs` calls `/api/admin/translations`                                           |
| `src/app/legal`              | the terms of use and the privacy policy, written in the catalog                                         |
| `src/auth`                   | who is signed in and what their roles grant; `api.rs` calls `/api/users/me` and `/api/auth/logout`      |
| `src/i18n`                   | the reader's language and the catalog; `api.rs` calls `/api/translations/{language}`                    |
| `src/theme.rs`               | the reader's color theme, from the cookie shared with Keycloak's pages, else the `theme` cookie         |
| `src/keycloak`               | Keycloak's pages: the sign-in in the reader's theme and language, account actions, the second factor    |
| `src/keycloak_cookies.rs`    | the theme and language shared with Keycloak's pages, which the theme and the language read              |
| `src/backend`                | what the features' calls share: the HTTP client, problem documents, and the proxy                       |
| `src/forms.rs`               | how a form answers: `303 See Other` when accepted, the same page with the refusal when not              |
| `src/components`             | the Topcoat UI components, installed with `topcoat ui add`, and the application's own                   |
| `styles.css`                 | the Tailwind input and the tokens, each a light and a dark color                                        |

`src/components` stays flat, as `topcoat ui` installs and updates its components there; a component one feature alone
uses lives in that feature's module.

## Navigation and tables

The top bar holds the modules (Start, Files, Administration), the account menu, the theme menu and the
language menu; the side bar on the left holds the sections of the open module, such as Translations, Roles and Users
under Administration, each shown only when the reader's roles grant it. On a phone the modules fold into a menu, and
the sections are listed under the bar. The menus are `<details>` elements, so they work without scripts.

A table (`data_table`) shows every column and scrolls sideways on a narrow screen, while its actions column stays at
the right edge (`row_actions`). Every list (files, users, roles, translations) is a shard paged by
`pagination`: the first, previous, next and last page, 5 to 100 items per page, the page and the total. Turning a
page, choosing a size or typing in the translations filter re-renders only the list, which asks the back-end for that
page ([Architecture: Collections are paged](../docs/architecture.md#collections-are-paged)); both numbers come from
the browser, so a size not offered falls back to the default, and a page past the end shows the last one.

The pages are built from the components in `src/components`: those `topcoat ui` installed, and the application's own,
such as `icon` (a Heroicon by name), `dropdown`, `pagination`, `data_table`, `row_actions`,
`back_link`, `page_header`, `text_field`, `confirm` and `feedback`. The icons are Heroicons 2.2.0 (MIT),
kept in `components/icons.rs`.

## Theme

The theme menu offers System, Light and Dark, kept in the `theme` cookie and in the cookie shared with Keycloak's pages,
which is read first ([Authentication](../docs/authentication.md)), and written on `<html data-theme>`; a reader who
chose nothing gets System. Each token of `styles.css` holds a light and a dark color in `light-dark()`, and
`data-theme` sets the `color-scheme` that picks between them, so System follows the reader's system without a script,
and the page never flashes the other theme while it loads. Keycloak's pages wear the same colors
([Authentication: The sign-in pages](../docs/authentication.md#the-sign-in-pages)).

## Pages

| URL                                   | Shows                                                   | Needs                                      |
|---------------------------------------|---------------------------------------------------------|--------------------------------------------|
| `/`                                   | sign-in and sign-up for a guest, the sections otherwise | —                                          |
| `/account`                            | the profile, what the roles grant, password and 2FA     | a session                                  |
| `/files`, `/files/{id}`               | the files, an upload, a file's details                  | `FILES_VIEW`; `FILES_MANAGE` to change     |
| `/admin/users`, `/admin/users/{id}`   | the accounts and the roles each holds                   | `USERS_VIEW`; `USERS_MANAGE` to give roles |
| `/admin/roles`, `/admin/roles/{name}` | the roles and what each grants                          | `ROLES_VIEW`; `ROLES_MANAGE` to change     |
| `/admin/translations`, `.../{key}`    | the message catalog, filtered and paged by the back-end | `TRANSLATIONS_VIEW`; `TRANSLATIONS_EDIT`   |
| `/legal/terms`, `/legal/privacy`      | the terms of use and the privacy policy                 | —                                          |

Each page calls `require_auth` or `require_permission` itself, and hides what the reader's roles do not grant; the back
end checks every call again, so a hidden link is not the protection. A page the reader may not open answers `401`,
`403` or `404` through the layout. Keycloak's terms page links to the two legal pages.

## The back-end behind it

The browser talks only to this application. `/api/*` and `/uploads/*` are passed on to the back-end as the browser sent
them, without the hop-by-hop headers and refusing any `.` or `..` segment, and redirects come back unfollowed. So the
session cookie is set on this address, Keycloak returns to `/api/auth/callback` here, and every call is same-origin.
A file is uploaded by `src/app/files/upload.js`, which asks `/api/files/upload-ticket` for a signed link, sends the
bytes there with a progress bar, and confirms the upload; `/files/{id}/download` asks for a fresh download link at each
click and redirects to it ([File storage](../backend/src/files/docs/mechanisms/file-storage.md)). A link to a bucket
leaves this address; a link to the local disk is under `/uploads`, and what it returns is always sent as a download
that cannot run as a page (`Content-Disposition: attachment`, `nosniff`, a sandboxing `Content-Security-Policy`), so
a script in it never acts with the reader's session.

When a page calls the back-end itself, it sends only the session cookie and passes every `Set-Cookie` of the answer on
to the browser, such as the cookie's renewed expiry. These calls are made before the page renders, as cookies cannot
be written once the response has started.

## Changes and cross-site requests

A change is a form posted to a route here, never to the API. Topcoat's origin policy refuses a change sent from
another site before the route runs; the back-end sees the call from this server as same-origin, so that policy is the
check. Every response forbids being framed by another site (`X-Frame-Options: DENY`, `frame-ancestors 'none'`), so a
page cannot be laid under another site's clicks.

## Text

Every label and every error is a key of the back-end's catalog: `texts.t(key)`, `texts.t_with(key, args)`, and
`texts.problem(&problem)` for a refusal. The language is the one chosen in the header or on Keycloak's pages, kept in a
cookie both share, then the `lang` cookie, then the browser's `Accept-Language`, then Polish; the sign-in hands it to
Keycloak. A key missing from the reader's language is taken
from Polish and reported. Dates are shown in UTC, as the server does not know the reader's time zone.
`tests/catalog.rs` fails when a page uses a key the catalog lacks in either language.

## Running it

Start the infrastructure and the back-end first ([Development Setup](../docs/development-setup.md)). The Topcoat CLI
is built into `.tools/` once:

```bash
cargo install topcoat-cli --version 0.10.0 --locked --root .tools
```

Then:

```bash
.tools/bin/topcoat dev
```

The application is at `http://localhost:3000`, and rebuilds when a source file changes. The first build downloads the
pinned Tailwind CLI and checks it against the release's SHA-256; `build.rs` holds the version and the checksums.

Configuration comes from environment variables; `.env.example` lists them, and a `.env` next to it is read when present.

| Variable                  | Default                 | Purpose                                                         |
|---------------------------|-------------------------|-----------------------------------------------------------------|
| `HOST`, `PORT`            | `127.0.0.1`, `3000`     | where it listens                                                |
| `APP_ENVIRONMENT`         | `local`                 | `production` also makes the `lang` cookie `Secure`              |
| `BACKEND_INTERNAL_URL`    | `http://localhost:8080` | the back-end, as this server reaches it                         |
| `BACKEND_TIMEOUT_SECONDS` | `30`                    | how long an API call may take, and a proxied transfer may stall |

## Checks

```bash
.tools/bin/topcoat fmt --rustfmt --check src tests build.rs
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

The formatter is given the sources by name: on its own it walks every folder, `target/` too, where the code generated
for the dependencies is not formatted.

The rules between its modules, such as components knowing no page, are checked by the [architecture
tests](../architecture/README.md).

## Production

The image builds the binary and its asset bundle together, as the bundle must come from the same build, and runs with
`APP_ENVIRONMENT=production`, which requires `BACKEND_INTERNAL_URL`. Its health check reads `/health`. The back-end's
`AUTH_CALLBACK_URL` and `AUTH_POST_LOGIN_URL` name this application's public address.
