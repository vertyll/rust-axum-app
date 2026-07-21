//! Modular monolith with hexagonal architecture (ports & adapters).
//!
//! # Modules (bounded contexts)
//!
//! * [`identity`] — users, credentials, sessions, roles, e-mail confirmation
//!   and password flows.
//! * [`files`] — file upload, storage and metadata.
//!
//! Each module is layered the same way:
//!
//! ```text
//! module/
//! ├── domain/          pure business model: entities, value objects,
//! │                    domain rules, repository ports. No framework/IO types.
//! ├── application/     use cases orchestrating the domain. Defines the
//! │                    driven ports (hasher, token service, mailer, storage).
//! └── infrastructure/  adapters: Toasty persistence, Argon2, JWT, SMTP,
//!                      local FS storage, Axum HTTP (routes, DTOs, guards).
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
//! * [`shared_infrastructure`] holds shared technical infrastructure (i18n only) that any layer may use.

pub mod bootstrap;
pub mod files;
pub mod identity;
pub mod shared_infrastructure;

rust_i18n::i18n!("translations", fallback = "en");
