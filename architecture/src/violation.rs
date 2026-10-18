//! What a check found: each broken rule, with where it breaks.

use std::fmt;

/// One place a rule is broken.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation {
    pub rule: String,
    pub because: String,
    /// What breaks the rule and where, such as `crate::files uses crate::identity::domain (src/files/mod.rs:3)`.
    pub detail: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.rule, self.detail)
    }
}

/// Every place the rules of a check are broken, grouped by rule when shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violations(Vec<Violation>);

impl Violations {
    pub(crate) fn new(mut violations: Vec<Violation>) -> Self {
        violations.sort();
        violations.dedup();
        Self(violations)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Violation> {
        self.0.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Violations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rule = None;
        for violation in &self.0 {
            if rule != Some(&violation.rule) {
                writeln!(f, "\n{} — {}", violation.rule, violation.because)?;
                rule = Some(&violation.rule);
            }
            writeln!(f, "  {}", violation.detail)?;
        }
        Ok(())
    }
}

impl std::error::Error for Violations {}
