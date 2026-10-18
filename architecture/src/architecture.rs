//! A scanned crate: its modules and the dependencies between them.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::rule::Rule;
use crate::scan::{self, ScanError};
use crate::violation::Violations;

/// A module's path from the crate root, shown as `crate::identity::domain`; the root itself is `crate`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath(Vec<String>);

impl ModulePath {
    pub(crate) fn new(segments: Vec<String>) -> Self {
        Self(segments)
    }

    #[must_use]
    pub fn segments(&self) -> &[String] {
        &self.0
    }

    #[must_use]
    pub fn starts_with(&self, other: &Self) -> bool {
        self.0.starts_with(&other.0)
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("crate")?;
        self.0.iter().try_for_each(|segment| write!(f, "::{segment}"))
    }
}

/// One module's use of another: `from` writes a path that leads into `to`, the deepest module on that path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub from: ModulePath,
    pub to: ModulePath,
    /// The path as written, resolved from the crate root, such as `crate::identity::domain::user::UserId`.
    pub path: String,
    pub file: PathBuf,
    pub line: usize,
    /// Whether the path sits in code compiled for tests only.
    pub test: bool,
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} uses {} ({}:{})",
            self.from,
            self.path,
            self.file.display(),
            self.line
        )
    }
}

/// A crate's modules and the dependencies between them, as its source writes them.
#[derive(Debug, Clone)]
pub struct Architecture {
    modules: BTreeSet<ModulePath>,
    dependencies: Vec<Dependency>,
}

impl Architecture {
    /// Reads the crate whose root file is `root`, such as `src/lib.rs` or `src/main.rs`, and every module it declares.
    ///
    /// # Errors
    ///
    /// When a file cannot be read or parsed, or a declared module has no file.
    pub fn scan(root: impl AsRef<Path>) -> Result<Self, ScanError> {
        let scanned = scan::scan(root.as_ref())?;
        let modules: BTreeSet<ModulePath> = scanned.modules.iter().cloned().map(ModulePath::new).collect();
        let dependencies = scanned
            .written
            .into_iter()
            .filter_map(|written| {
                let to = deepest_module(&scanned.modules, &written.target);
                let from = ModulePath::new(written.from);
                (from != to).then(|| Dependency {
                    from,
                    to,
                    path: ModulePath::new(written.target).to_string(),
                    file: written.file,
                    line: written.line,
                    test: written.test,
                })
            })
            .collect();
        Ok(Self { modules, dependencies })
    }

    pub fn modules(&self) -> impl Iterator<Item = &ModulePath> {
        self.modules.iter()
    }

    #[must_use]
    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    /// Checks every rule, and lists what breaks them.
    ///
    /// # Errors
    ///
    /// The violations, when there are any; a rule whose pattern matches no module is one, as it would check nothing.
    pub fn check(&self, rules: &[Rule]) -> Result<(), Violations> {
        let violations: Vec<_> = rules.iter().flat_map(|rule| rule.violations(self)).collect();
        if violations.is_empty() {
            Ok(())
        } else {
            Err(Violations::new(violations))
        }
    }
}

fn deepest_module(modules: &BTreeSet<Vec<String>>, target: &[String]) -> ModulePath {
    let depth = (0..=target.len())
        .rev()
        .find(|&depth| modules.contains(&target[..depth]))
        .unwrap_or(0);
    ModulePath::new(target[..depth].to_vec())
}
