use rust_axum_app_architecture::{Architecture, ScanError};

const SHOP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/shop/src/lib.rs");

fn shop() -> Architecture {
    Architecture::scan(SHOP).expect("the shop fixture is a valid crate")
}

/// Every dependency as `from -> path`, those in test code marked.
fn written(architecture: &Architecture) -> Vec<String> {
    let mut written: Vec<String> = architecture
        .dependencies()
        .iter()
        .map(|dependency| {
            let test = if dependency.test { " (test)" } else { "" };
            format!("{} -> {}{test}", dependency.from, dependency.path)
        })
        .collect();
    written.sort();
    written.dedup();
    written
}

#[test]
fn the_module_tree_follows_mod_declarations_inline_and_by_path() {
    let modules: Vec<String> = shop().modules().map(ToString::to_string).collect();

    assert_eq!(
        modules,
        [
            "crate",
            "crate::billing",
            "crate::billing::internal",
            "crate::bootstrap",
            "crate::kernel",
            "crate::orders",
            "crate::orders::domain",
            "crate::orders::inline",
            "crate::orders::tests",
            "crate::static",
        ]
    );
}

#[test]
fn every_way_of_naming_a_module_is_followed() {
    assert_eq!(
        written(&shop()),
        [
            "crate::billing -> crate::billing::internal::Invoice",
            "crate::billing -> crate::billing::internal::Invoice::default",
            "crate::billing::internal -> crate::kernel",
            "crate::billing::internal -> crate::kernel::Money",
            "crate::bootstrap -> crate::billing::Invoice",
            "crate::bootstrap -> crate::billing::bill",
            "crate::bootstrap -> crate::orders",
            "crate::bootstrap -> crate::orders::domain::Order",
            "crate::bootstrap -> crate::orders::place",
            "crate::orders -> crate::billing",
            "crate::orders -> crate::billing::internal::total",
            "crate::orders -> crate::kernel::Money::default",
            "crate::orders::domain -> crate::billing::Invoice",
            "crate::orders::domain -> crate::bootstrap::wire (test)",
            "crate::orders::domain -> crate::static::Asset",
            "crate::orders::inline -> crate::orders::domain",
            "crate::orders::tests -> crate::bootstrap::wire (test)",
        ]
    );
}

#[test]
fn a_dependency_points_at_the_deepest_module_on_its_path_and_says_where_it_is_written() {
    let architecture = shop();
    let through_alias = architecture
        .dependencies()
        .iter()
        .find(|dependency| dependency.path == "crate::billing::internal::total")
        .expect("the path written through the `billing` alias is followed");

    assert_eq!(through_alias.to.to_string(), "crate::billing::internal");
    assert!(through_alias.file.ends_with("orders/mod.rs"));
    assert_eq!(through_alias.line, 7);
}

#[test]
fn a_declared_module_without_a_file_is_an_error() {
    let broken = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/broken/src/lib.rs");

    let error = Architecture::scan(broken).expect_err("the module `gone` has no file");

    assert!(matches!(error, ScanError::MissingModule { ref module, .. } if module == "gone"));
}
