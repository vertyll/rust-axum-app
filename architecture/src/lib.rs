//! Architecture tests for the application's Rust crates: the rules their modules keep, checked on their source, as
//! `ArchUnit` checks a Java build.
//!
//! [`Architecture::scan`] reads a crate's module tree from its root file, following `mod` declarations as the compiler
//! does, and records every dependency of one module on another: each path of a `use` declaration, and each path in the
//! code that starts with `crate`, `self`, `super`, a child module or a module a `use` brought in. A [`Rule`] says which
//! of them are allowed, and [`Architecture::check`] lists every [`Violation`].
//!
//! ```no_run
//! use rust_axum_app_architecture::{Architecture, Rule};
//!
//! let backend = Architecture::scan("../backend/src/lib.rs").unwrap();
//! backend
//!     .check(&[Rule::forbid("the domain knows no adapter")
//!         .from("*::domain::**")
//!         .to("*::infrastructure::**")
//!         .because("adapters implement the ports of the application, never the other way round")])
//!     .unwrap();
//! ```
//!
//! A module is named by its path from the crate root, without `crate`, and a [pattern](Rule) matches those paths
//! segment by segment: `*` stands for one segment and `**` for any number of them, none included.
//!
//! The scan reads source, not the compiler's name resolution, so a path is missed where it never appears as one:
//! inside a macro invocation, or a name brought in by a glob import and used bare. Those stay inside the module that
//! wrote them, so a rule about crossing modules still sees the `use` that crossed.

mod architecture;
mod pattern;
mod rule;
mod scan;
mod violation;

pub use architecture::{Architecture, Dependency, ModulePath};
pub use rule::{Forbid, NoCycles, Rule, ThroughRoot};
pub use scan::ScanError;
pub use violation::{Violation, Violations};
