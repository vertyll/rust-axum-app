//! The rules a crate's modules keep, and how each finds what breaks it.

use std::collections::{BTreeMap, BTreeSet};

use crate::architecture::{Architecture, Dependency, ModulePath};
use crate::pattern::Pattern;
use crate::violation::Violation;

/// A rule about the dependencies between a crate's modules. One starts with [`Rule::forbid`], [`Rule::no_cycles`] or
/// [`Rule::through_root`], and is finished by `because`, which says why it holds.
///
/// Patterns name modules by their path from the crate root: a name matches itself, `*` any one segment and `**` any
/// number of them, none included, so `*::domain::**` is every module's domain and everything in it. A pattern that
/// matches no module is reported as a violation, since a rule about nothing is a typo.
#[derive(Debug, Clone)]
pub struct Rule {
    name: String,
    because: String,
    kind: Kind,
}

#[derive(Debug, Clone)]
enum Kind {
    Forbid(Forbid),
    NoCycles(NoCycles),
    ThroughRoot(ThroughRoot),
}

impl Rule {
    /// Forbids the modules matched by `from` to depend on those matched by `to`.
    pub fn forbid(name: impl Into<String>) -> Forbid {
        Forbid {
            name: name.into(),
            from: Vec::new(),
            except_from: Vec::new(),
            to: Vec::new(),
            except_to: Vec::new(),
            tests: false,
        }
    }

    /// Forbids a cycle between the modules matched by `between`, each standing for itself and everything inside it.
    pub fn no_cycles(name: impl Into<String>) -> NoCycles {
        NoCycles {
            name: name.into(),
            between: None,
        }
    }

    /// Lets other modules reach the modules matched by `modules` only through what each one's root declares or
    /// re-exports, never through the modules inside it.
    pub fn through_root(name: impl Into<String>) -> ThroughRoot {
        ThroughRoot {
            name: name.into(),
            modules: Vec::new(),
            except_from: Vec::new(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn violations(&self, architecture: &Architecture) -> Vec<Violation> {
        let unmatched = self
            .patterns()
            .into_iter()
            .filter(|pattern| !architecture.modules().any(|module| pattern.matches(module.segments())))
            .map(|pattern| self.violation(format!("the pattern `{pattern}` matches no module")));
        let broken = match &self.kind {
            Kind::Forbid(forbid) => forbid.violations(architecture),
            Kind::NoCycles(no_cycles) => no_cycles.violations(architecture),
            Kind::ThroughRoot(through_root) => through_root.violations(architecture),
        };
        unmatched
            .chain(broken.into_iter().map(|detail| self.violation(detail)))
            .collect()
    }

    fn patterns(&self) -> Vec<&Pattern> {
        match &self.kind {
            Kind::Forbid(forbid) => forbid
                .from
                .iter()
                .chain(&forbid.except_from)
                .chain(&forbid.to)
                .chain(&forbid.except_to)
                .collect(),
            Kind::NoCycles(no_cycles) => no_cycles.between.iter().collect(),
            Kind::ThroughRoot(through_root) => through_root.modules.iter().chain(&through_root.except_from).collect(),
        }
    }

    fn violation(&self, detail: String) -> Violation {
        Violation {
            rule: self.name.clone(),
            because: self.because.clone(),
            detail,
        }
    }
}

/// A rule forbidding some modules to depend on others; see [`Rule::forbid`].
#[derive(Debug, Clone)]
pub struct Forbid {
    name: String,
    from: Vec<Pattern>,
    except_from: Vec<Pattern>,
    to: Vec<Pattern>,
    except_to: Vec<Pattern>,
    tests: bool,
}

impl Forbid {
    /// Modules the rule applies to; several calls add to one another.
    #[must_use]
    pub fn from(mut self, pattern: &str) -> Self {
        self.from.push(Pattern::parse(pattern));
        self
    }

    /// Modules `from` matched that the rule leaves out.
    #[must_use]
    pub fn except_from(mut self, pattern: &str) -> Self {
        self.except_from.push(Pattern::parse(pattern));
        self
    }

    /// Modules that may not be depended on; several calls add to one another.
    #[must_use]
    pub fn to(mut self, pattern: &str) -> Self {
        self.to.push(Pattern::parse(pattern));
        self
    }

    /// Modules `to` matched that may still be depended on.
    #[must_use]
    pub fn except_to(mut self, pattern: &str) -> Self {
        self.except_to.push(Pattern::parse(pattern));
        self
    }

    /// Applies the rule to code compiled for tests too; by default it covers what ships.
    #[must_use]
    pub fn including_tests(mut self) -> Self {
        self.tests = true;
        self
    }

    #[must_use]
    pub fn because(self, reason: &str) -> Rule {
        Rule {
            name: self.name.clone(),
            because: reason.to_owned(),
            kind: Kind::Forbid(self),
        }
    }

    fn violations(&self, architecture: &Architecture) -> Vec<String> {
        if self.from.is_empty() || self.to.is_empty() {
            return vec!["the rule names no module to forbid from or to".to_owned()];
        }
        architecture
            .dependencies()
            .iter()
            .filter(|dependency| self.tests || !dependency.test)
            .filter(|dependency| {
                any_matches(&self.from, &dependency.from)
                    && !any_matches(&self.except_from, &dependency.from)
                    && any_matches(&self.to, &dependency.to)
                    && !any_matches(&self.except_to, &dependency.to)
            })
            .map(Dependency::to_string)
            .collect()
    }
}

/// A rule forbidding cycles between modules; see [`Rule::no_cycles`].
#[derive(Debug, Clone)]
pub struct NoCycles {
    name: String,
    between: Option<Pattern>,
}

impl NoCycles {
    /// The modules that may not depend on one another in a circle, such as `*` for the crate's top-level modules. The
    /// pattern has no `**`, so each module it matches stands for a whole subtree.
    #[must_use]
    pub fn between(mut self, pattern: &str) -> Self {
        self.between = Some(Pattern::parse(pattern));
        self
    }

    #[must_use]
    pub fn because(self, reason: &str) -> Rule {
        Rule {
            name: self.name.clone(),
            because: reason.to_owned(),
            kind: Kind::NoCycles(self),
        }
    }

    fn violations(&self, architecture: &Architecture) -> Vec<String> {
        let Some(between) = &self.between else {
            return vec!["the rule names no modules to keep free of cycles".to_owned()];
        };
        let Some(depth) = between.fixed_len() else {
            return vec![format!(
                "the pattern `{between}` has a `**`, so it names no fixed set of modules"
            )];
        };
        let group = |module: &ModulePath| {
            let segments = module.segments();
            (segments.len() >= depth && between.matches(&segments[..depth]))
                .then(|| ModulePath::new(segments[..depth].to_vec()))
        };
        let mut edges: BTreeMap<ModulePath, BTreeMap<ModulePath, &Dependency>> = BTreeMap::new();
        for dependency in architecture.dependencies().iter().filter(|dependency| !dependency.test) {
            if let (Some(from), Some(to)) = (group(&dependency.from), group(&dependency.to))
                && from != to
            {
                edges.entry(from).or_default().entry(to).or_insert(dependency);
            }
        }
        strongly_connected(&edges)
            .into_iter()
            .map(|component| {
                let steps: Vec<String> = component
                    .iter()
                    .flat_map(|from| {
                        edges[from]
                            .iter()
                            .filter(|(to, _)| component.contains(*to))
                            .map(|(_, dependency)| dependency.to_string())
                    })
                    .collect();
                let members: Vec<String> = component.iter().map(ToString::to_string).collect();
                format!("{} depend on one another: {}", members.join(", "), steps.join("; "))
            })
            .collect()
    }
}

/// A rule letting modules be reached only through their root; see [`Rule::through_root`].
#[derive(Debug, Clone)]
pub struct ThroughRoot {
    name: String,
    modules: Vec<Pattern>,
    except_from: Vec<Pattern>,
}

impl ThroughRoot {
    /// Modules reached only through their root, such as `identity`; a pattern has no `**`. Several calls add to one
    /// another.
    #[must_use]
    pub fn module(mut self, pattern: &str) -> Self {
        self.modules.push(Pattern::parse(pattern));
        self
    }

    /// Modules allowed inside the others anyway, such as the composition root that wires their adapters.
    #[must_use]
    pub fn except_from(mut self, pattern: &str) -> Self {
        self.except_from.push(Pattern::parse(pattern));
        self
    }

    #[must_use]
    pub fn because(self, reason: &str) -> Rule {
        Rule {
            name: self.name.clone(),
            because: reason.to_owned(),
            kind: Kind::ThroughRoot(self),
        }
    }

    fn violations(&self, architecture: &Architecture) -> Vec<String> {
        if self.modules.is_empty() {
            return vec!["the rule names no module to reach through its root".to_owned()];
        }
        let mut found = Vec::new();
        for pattern in &self.modules {
            let Some(depth) = pattern.fixed_len() else {
                found.push(format!(
                    "the pattern `{pattern}` has a `**`, so it names no module's root"
                ));
                continue;
            };
            for dependency in architecture.dependencies().iter().filter(|dependency| !dependency.test) {
                let to = dependency.to.segments();
                if to.len() <= depth || !pattern.matches(&to[..depth]) {
                    continue;
                }
                let root = ModulePath::new(to[..depth].to_vec());
                if !dependency.from.starts_with(&root) && !any_matches(&self.except_from, &dependency.from) {
                    found.push(format!("{dependency}, inside {root} rather than through its root"));
                }
            }
        }
        found
    }
}

fn any_matches(patterns: &[Pattern], module: &ModulePath) -> bool {
    patterns.iter().any(|pattern| pattern.matches(module.segments()))
}

/// Tarjan's algorithm: the groups of modules that reach one another, each of more than one module.
fn strongly_connected<V>(edges: &BTreeMap<ModulePath, BTreeMap<ModulePath, V>>) -> Vec<BTreeSet<ModulePath>> {
    struct State<'g, V> {
        edges: &'g BTreeMap<ModulePath, BTreeMap<ModulePath, V>>,
        index: BTreeMap<&'g ModulePath, usize>,
        low: BTreeMap<&'g ModulePath, usize>,
        stack: Vec<&'g ModulePath>,
        on_stack: BTreeSet<&'g ModulePath>,
        components: Vec<BTreeSet<ModulePath>>,
    }

    fn visit<'g, V>(state: &mut State<'g, V>, node: &'g ModulePath) {
        let index = state.index.len();
        state.index.insert(node, index);
        state.low.insert(node, index);
        state.stack.push(node);
        state.on_stack.insert(node);
        for next in state.edges.get(node).into_iter().flat_map(BTreeMap::keys) {
            if !state.index.contains_key(next) {
                visit(state, next);
                let low = state.low[node].min(state.low[next]);
                state.low.insert(node, low);
            } else if state.on_stack.contains(next) {
                let low = state.low[node].min(state.index[next]);
                state.low.insert(node, low);
            }
        }
        if state.low[node] == state.index[node] {
            let mut component = BTreeSet::new();
            while let Some(member) = state.stack.pop() {
                state.on_stack.remove(member);
                component.insert(member.clone());
                if member == node {
                    break;
                }
            }
            if component.len() > 1 {
                state.components.push(component);
            }
        }
    }

    let mut state = State {
        edges,
        index: BTreeMap::new(),
        low: BTreeMap::new(),
        stack: Vec::new(),
        on_stack: BTreeSet::new(),
        components: Vec::new(),
    };
    for node in edges.keys() {
        if !state.index.contains_key(node) {
            visit(&mut state, node);
        }
    }
    state.components
}
