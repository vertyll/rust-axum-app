//! Reading a crate: its module tree from the root file, then every path one module writes to another.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{Attribute, Expr, Item, Lit, Meta, UseTree};

/// Why a crate could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("reading {}: {source}", path.display())]
    Read { path: PathBuf, source: std::io::Error },
    #[error("parsing {}:{line}: {message}", path.display())]
    Parse {
        path: PathBuf,
        line: usize,
        message: String,
    },
    #[error("module crate::{module} is declared, but neither {} nor {} exists", tried.0.display(), tried.1.display())]
    MissingModule { module: String, tried: (PathBuf, PathBuf) },
}

/// A path one module writes to another, resolved to where it points from the crate root.
pub(crate) struct Written {
    pub(crate) from: Vec<String>,
    pub(crate) target: Vec<String>,
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) test: bool,
}

pub(crate) struct Scanned {
    pub(crate) modules: BTreeSet<Vec<String>>,
    pub(crate) written: Vec<Written>,
}

struct SourceFile {
    path: PathBuf,
    module: Vec<String>,
    test: bool,
    syntax: syn::File,
}

pub(crate) fn scan(root: &Path) -> Result<Scanned, ScanError> {
    let mut modules = BTreeSet::new();
    let mut files = Vec::new();
    let children = root.parent().unwrap_or(Path::new(".")).to_path_buf();
    load(root, Vec::new(), &children, false, &mut modules, &mut files)?;

    let mut aliases = BTreeMap::new();
    for file in &files {
        let mut collector = Collector::new(file, &modules, &mut aliases, Mode::Aliases);
        collector.visit_file(&file.syntax);
    }
    // Paths are shown from the crate's own folder, the one holding `src`, as `src/files/mod.rs`.
    let base = root.parent().and_then(Path::parent).unwrap_or(Path::new(""));
    for file in &mut files {
        if let Ok(relative) = file.path.strip_prefix(base) {
            file.path = relative.to_path_buf();
        }
    }
    let mut written = Vec::new();
    for file in &files {
        let mut collector = Collector::new(file, &modules, &mut aliases, Mode::Paths(&mut written));
        collector.visit_file(&file.syntax);
    }
    Ok(Scanned { modules, written })
}

fn load(
    path: &Path,
    module: Vec<String>,
    children: &Path,
    test: bool,
    modules: &mut BTreeSet<Vec<String>>,
    files: &mut Vec<SourceFile>,
) -> Result<(), ScanError> {
    let source = std::fs::read_to_string(path).map_err(|source| ScanError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let syntax = syn::parse_file(&source).map_err(|error| ScanError::Parse {
        path: path.to_path_buf(),
        line: error.span().start().line,
        message: error.to_string(),
    })?;
    modules.insert(module.clone());
    declared_modules(&syntax.items, &module, children, test, modules, files)?;
    files.push(SourceFile {
        path: path.to_path_buf(),
        module,
        test,
        syntax,
    });
    Ok(())
}

/// Follows the `mod` declarations among the items, as the compiler does: an inline module's items are read where they
/// are, and a declared one from `name.rs` or `name/mod.rs` beside its parent, or from its `#[path]`.
fn declared_modules(
    items: &[Item],
    module: &[String],
    children: &Path,
    test: bool,
    modules: &mut BTreeSet<Vec<String>>,
    files: &mut Vec<SourceFile>,
) -> Result<(), ScanError> {
    for item in items {
        let Item::Mod(declared) = item else { continue };
        let name = ident_name(&declared.ident);
        let child = [module, std::slice::from_ref(&name)].concat();
        let child_test = test || is_test_only(&declared.attrs);
        if let Some((_, inline)) = &declared.content {
            modules.insert(child.clone());
            declared_modules(inline, &child, &children.join(&name), child_test, modules, files)?;
            continue;
        }
        let file = if let Some(relative) = path_attribute(&declared.attrs) {
            children.join(relative)
        } else {
            let beside = children.join(format!("{name}.rs"));
            let own_folder = children.join(&name).join("mod.rs");
            if beside.is_file() {
                beside
            } else if own_folder.is_file() {
                own_folder
            } else {
                return Err(ScanError::MissingModule {
                    module: child.join("::"),
                    tried: (beside, own_folder),
                });
            }
        };
        let grandchildren = if file.file_name().is_some_and(|name| name == "mod.rs") {
            file.parent().unwrap_or(children).to_path_buf()
        } else {
            children.join(&name)
        };
        load(&file, child, &grandchildren, child_test, modules, files)?;
    }
    Ok(())
}

enum Mode<'w> {
    /// Learns which module each `use` names, so a path written through that name is followed later.
    Aliases,
    /// Records every path that leaves the module it is written in.
    Paths(&'w mut Vec<Written>),
}

type Aliases = BTreeMap<Vec<String>, BTreeMap<String, Vec<String>>>;

struct Collector<'a, 'w> {
    file: &'a Path,
    modules: &'a BTreeSet<Vec<String>>,
    aliases: &'a mut Aliases,
    mode: Mode<'w>,
    module: Vec<String>,
    test: bool,
}

impl<'a, 'w> Collector<'a, 'w> {
    fn new(file: &'a SourceFile, modules: &'a BTreeSet<Vec<String>>, aliases: &'a mut Aliases, mode: Mode<'w>) -> Self {
        Self {
            file: &file.path,
            modules,
            aliases,
            mode,
            module: file.module.clone(),
            test: file.test,
        }
    }

    /// Where a path written in the current module points, from the crate root, or `None` for a path into another crate
    /// or to a local name.
    fn resolve(&self, segments: &[String]) -> Option<Vec<String>> {
        let (first, rest) = segments.split_first()?;
        match first.as_str() {
            "crate" => Some(rest.to_vec()),
            "self" => Some([self.module.as_slice(), rest].concat()),
            "super" => {
                let ups = segments.iter().take_while(|segment| *segment == "super").count();
                let base = self.module.len().checked_sub(ups)?;
                Some([&self.module[..base], &segments[ups..]].concat())
            }
            name => {
                let child = [self.module.as_slice(), &[name.to_owned()]].concat();
                if self.modules.contains(&child) {
                    return Some([child.as_slice(), rest].concat());
                }
                let alias = self.aliases.get(&self.module)?.get(name)?;
                Some([alias.as_slice(), rest].concat())
            }
        }
    }

    fn written(&mut self, segments: &[String], bound: Option<&str>, span: Span) {
        let Some(target) = self.resolve(segments) else { return };
        match &mut self.mode {
            Mode::Aliases => {
                if let Some(name) = bound
                    && self.modules.contains(&target)
                {
                    self.aliases
                        .entry(self.module.clone())
                        .or_default()
                        .insert(name.to_owned(), target);
                }
            }
            Mode::Paths(written) => written.push(Written {
                from: self.module.clone(),
                target,
                file: self.file.to_path_buf(),
                line: span.start().line,
                test: self.test,
            }),
        }
    }

    fn use_tree(&mut self, tree: &UseTree, prefix: &mut Vec<String>) {
        match tree {
            UseTree::Path(path) => {
                prefix.push(ident_name(&path.ident));
                self.use_tree(&path.tree, prefix);
                prefix.pop();
            }
            UseTree::Name(name) => {
                let ident = ident_name(&name.ident);
                if ident == "self" {
                    let bound = prefix.last().cloned();
                    self.written(&prefix.clone(), bound.as_deref(), name.ident.span());
                } else {
                    let path = [prefix.as_slice(), std::slice::from_ref(&ident)].concat();
                    self.written(&path, Some(&ident), name.ident.span());
                }
            }
            UseTree::Rename(rename) => {
                let path = [prefix.as_slice(), &[ident_name(&rename.ident)]].concat();
                self.written(&path, Some(&ident_name(&rename.rename)), rename.ident.span());
            }
            UseTree::Glob(glob) => self.written(&prefix.clone(), None, glob.star_token.spans[0]),
            UseTree::Group(group) => {
                for item in &group.items {
                    self.use_tree(item, prefix);
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for Collector<'_, '_> {
    fn visit_item(&mut self, item: &'ast Item) {
        let outer = self.test;
        self.test |= item_attributes(item).is_some_and(is_test_only);
        visit::visit_item(self, item);
        self.test = outer;
    }

    fn visit_item_mod(&mut self, declared: &'ast syn::ItemMod) {
        // A module in a file of its own is read with that file.
        if declared.content.is_some() {
            self.module.push(ident_name(&declared.ident));
            visit::visit_item_mod(self, declared);
            self.module.pop();
        }
    }

    fn visit_item_use(&mut self, declaration: &'ast syn::ItemUse) {
        self.use_tree(&declaration.tree, &mut Vec::new());
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.leading_colon.is_none() && path.segments.len() > 1 {
            let segments: Vec<String> = path.segments.iter().map(|segment| ident_name(&segment.ident)).collect();
            if matches!(self.mode, Mode::Paths(_)) {
                self.written(&segments, None, path.segments[0].ident.span());
            }
        }
        visit::visit_path(self, path);
    }
}

fn ident_name(ident: &syn::Ident) -> String {
    let name = ident.to_string();
    name.strip_prefix("r#").map(str::to_owned).unwrap_or(name)
}

fn item_attributes(item: &Item) -> Option<&[Attribute]> {
    Some(match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        _ => return None,
    })
}

/// Whether the attributes compile the item for tests only: `#[cfg(test)]`, or `#[cfg(all(test, …))]`.
fn is_test_only(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .meta
                .require_list()
                .is_ok_and(|list| requires_test(list.tokens.clone()))
    })
}

fn requires_test(tokens: TokenStream) -> bool {
    let tokens: Vec<TokenTree> = tokens.into_iter().collect();
    match tokens.as_slice() {
        [TokenTree::Ident(ident)] => ident == "test",
        [TokenTree::Ident(all), TokenTree::Group(group)] if all == "all" => group
            .stream()
            .into_iter()
            .any(|token| matches!(token, TokenTree::Ident(ident) if ident == "test")),
        _ => false,
    }
}

fn path_attribute(attributes: &[Attribute]) -> Option<String> {
    attributes.iter().find_map(|attribute| match &attribute.meta {
        Meta::NameValue(pair) if pair.path.is_ident("path") => match &pair.value {
            Expr::Lit(literal) => match &literal.lit {
                Lit::Str(text) => Some(text.value()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    })
}
