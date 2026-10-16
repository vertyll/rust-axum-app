//! The back-end: a modular monolith of hexagonal modules ([`identity`],
//! [`files`], [`translations`]), wired together in [`bootstrap`]. The layers,
//! the dependency rules and the shared modules are described in
//! `docs/architecture.md`.

pub mod bootstrap;
pub mod files;
pub mod identity;
pub mod shared_http;
pub mod shared_kernel;
pub mod translations;
