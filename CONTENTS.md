# Contents

Every document in this repository, the module it belongs to, and what it covers. Terms are defined in
[GLOSSARY.md](GLOSSARY.md), and the specifications behind them are in [STANDARDS.md](STANDARDS.md).

## Start here

| Document                        | Module | Kind               | Covers                                                                                                     |
|---------------------------------|--------|--------------------|------------------------------------------------------------------------------------------------------------|
| [rust-axum-app](README.md)      | —      | repository README  | What the repository is, its stack and where to start.                                                      |
| [Glossary](GLOSSARY.md)         | —      | reference          | Every term the docs use, and where it is explained.                                                        |
| [Standards](STANDARDS.md)       | —      | reference          | The RFCs and specifications the code implements or depends on.                                             |
| [Front-end](frontend/README.md) | —      | application README | The Topcoat pages: their layout, the proxy, text, running and checks.                                      |
| [Back-end](backend/README.md)   | —      | application README | The Axum API: its layout, modules, access, messages, database, mechanisms, running, checks and production. |

## Overview

| Document                                       | Module | Kind     | Covers                                                                      |
|------------------------------------------------|--------|----------|-----------------------------------------------------------------------------|
| [Development Setup](docs/development-setup.md) | —      | overview | Running the infrastructure, the application, migrations and checks.         |
| [Architecture](docs/architecture.md)           | —      | overview | Modules, ports and adapters, the composition root, routes and the database. |
| [Authentication](docs/authentication.md)       | —      | overview | Sign-in, token authorization and sign-out.                                  |
| [Authorization](docs/authorization.md)         | —      | overview | Permissions each module declares, roles, what they grant and the endpoints. |

## Mechanisms

| Document                                              | Module | Kind      | Covers                                                                                 |
|-------------------------------------------------------|--------|-----------|----------------------------------------------------------------------------------------|
| [Error responses](docs/mechanisms/error-responses.md) | —      | mechanism | What the API answers when it refuses a request, and how a client turns that into text. |

## Modules

| Document                                                                               | Module         | Kind          | Covers                                                                                                     |
|----------------------------------------------------------------------------------------|----------------|---------------|------------------------------------------------------------------------------------------------------------|
| [files](backend/src/files/README.md)                                                   | `files`        | module README | Stores uploaded files and their metadata: the bytes on a storage, the metadata in PostgreSQL.              |
| [File storage](backend/src/files/docs/mechanisms/file-storage.md)                      | `files`        | mechanism     | What happens to the bytes and the metadata row on upload, soft delete and delete.                          |
| [identity](backend/src/identity/README.md)                                             | `identity`     | module README | Signs people in through Keycloak, keeps their sessions, verifies their tokens and mirrors their accounts.  |
| [Cross-site requests](backend/src/identity/docs/mechanisms/cross-site-requests.md)     | `identity`     | mechanism     | Why a forged request from another site cannot act with the user's session.                                 |
| [Token refresh](backend/src/identity/docs/mechanisms/token-refresh.md)                 | `identity`     | mechanism     | How the session keeps a valid access token without signing the user out when requests race.                |
| [translations](backend/src/translations/README.md)                                     | `translations` | module README | Owns the text clients show: the catalog every error key and label resolves against, in Polish and English. |
| [Translation catalog](backend/src/translations/docs/mechanisms/translation-catalog.md) | `translations` | mechanism     | Where the text behind every message key comes from, and how an administrator's edits survive a deployment. |
