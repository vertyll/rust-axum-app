//! Modular monolith with hexagonal architecture (ports & adapters).
//!
//! # Modules (bounded contexts)
//!
//! * [`identity`] — accounts mirrored from Keycloak, browser sessions on
//!   Keycloak's hosted pages, roles and the auth guards.
//! * [`files`] — file upload, storage and metadata.
//! * [`translations`] — the ICU `MessageFormat` catalogue clients render
//!   message keys with; each module ships its defaults in `messages/`.
//!
//! Each module is layered the same way:
//!
//! ```text
//! module/
//! ├── domain/          pure business model: entities, value objects,
//! │                    domain rules, repository ports. No framework/IO types.
//! ├── application/     use cases orchestrating the domain. Defines the
//! │                    driven ports (Keycloak client, token verifier, storage).
//! └── infrastructure/  adapters: Toasty persistence, Keycloak, JWKS, local FS
//!                      storage, Axum HTTP (routes, DTOs, guards).
//! ```
//!
//! # Dependency rules
//!
//! * Within a module dependencies point **inward only**:
//!   `infrastructure → application → domain`. The domain imports nothing
//!   from the outer layers.
//! * Across modules: a module may use another module's **public API**
//!   (its `mod.rs` re-exports) in one direction only. Here `files` uses
//!   `identity`'s auth guards; `identity` never imports `files`.
//! * [`bootstrap`] is the composition root: it may see every module, wires
//!   concrete adapters into the application services (static-dispatch DI)
//!   and assembles the Axum router. Nothing imports `bootstrap`.
//! * [`shared_kernel`] holds plain value types any module's domain may use,
//!   such as a page of a collection; no I/O.
//! * [`shared_http`] holds what every module's HTTP adapter shares: RFC 9457
//!   problem documents and the paging parameters and responses.

pub mod bootstrap;
pub mod files;
pub mod identity;
pub mod shared_http;
pub mod shared_kernel;
pub mod translations;
