//! The back-end's architecture: a modular monolith whose modules are hexagons, wired by `bootstrap`
//! (`backend/docs/architecture.md`).

use std::sync::LazyLock;

use rust_axum_app_architecture::{Architecture, Rule};

static BACKEND: LazyLock<Architecture> = LazyLock::new(|| {
    Architecture::scan(concat!(env!("CARGO_MANIFEST_DIR"), "/../backend/src/lib.rs"))
        .expect("the back-end's source is a valid crate")
});

/// The modules that each own a capability, and what each publishes to the others: `contracts`, which any layer may use,
/// and `guards`, which only adapters take.
const MODULES: [(&str, &[&str]); 4] = [
    ("identity", &["contracts", "guards"]),
    ("translations", &["contracts"]),
    ("files", &[]),
    ("keycloak_account", &[]),
];

fn check(rules: &[Rule]) {
    if let Err(violations) = BACKEND.check(rules) {
        panic!("{violations}");
    }
}

#[test]
fn modules_depend_on_one_another_without_cycles() {
    check(&[Rule::no_cycles("the top-level modules form no cycle")
        .between("*")
        .because("a cycle makes two modules one, and neither can change alone")]);
}

#[test]
fn a_module_is_reached_only_through_what_it_publishes() {
    let rules: Vec<Rule> = MODULES
        .iter()
        .map(|(module, published)| {
            published
                .iter()
                .fold(
                    Rule::forbid(format!("{module} is reached through what it publishes"))
                        .from("**")
                        .except_from(&format!("{module}::**"))
                        .except_from("bootstrap::**")
                        .except_from("crate")
                        .to(&format!("{module}::**")),
                    |rule, part| rule.except_to(&format!("{module}::{part}::**")),
                )
                .because("its domain, application and adapters are its own; only the composition root wires them")
        })
        .collect();
    check(&rules);
}

#[test]
fn only_adapters_take_identitys_guards() {
    check(&[Rule::forbid("identity's guards serve HTTP adapters")
        .from("*::domain::**")
        .from("*::application::**")
        .to("identity::guards::**")
        .because("a guard is an HTTP extractor; a domain or an application knows no transport")]);
}

#[test]
fn what_a_module_publishes_carries_no_adapter() {
    check(&[Rule::forbid("contracts are the published language")
        .from("*::contracts::**")
        .to("*::infrastructure::**")
        .to("shared_http::**")
        .because("every layer of another module may use them, so they hold the domain's words and nothing else")]);
}

#[test]
fn the_domain_knows_neither_the_application_nor_any_adapter() {
    check(&[Rule::forbid("the domain stays pure")
        .from("*::domain::**")
        .to("*::application::**")
        .to("*::infrastructure::**")
        .to("shared_http::**")
        .to("bootstrap::**")
        .because("dependencies point inwards: the domain is the centre of the hexagon")]);
}

#[test]
fn the_application_knows_no_adapter() {
    check(&[Rule::forbid("the application knows only its ports")
        .from("*::application::**")
        .to("*::infrastructure::**")
        .to("shared_http::**")
        .to("bootstrap::**")
        .because("adapters implement the application's ports, never the other way round")]);
}

#[test]
fn the_shared_modules_know_no_capability() {
    check(&[
        Rule::forbid("the shared kernel stands alone")
            .from("shared_kernel::**")
            .to("**")
            .except_to("shared_kernel::**")
            .except_to("crate")
            .because("every module may use the kernel, so it may use none of them"),
        Rule::forbid("the shared HTTP code knows only the kernel")
            .from("shared_http::**")
            .to("**")
            .except_to("shared_http::**")
            .except_to("shared_kernel::**")
            .except_to("crate")
            .because("every module's adapters use it, so it may use none of them"),
    ]);
}

#[test]
fn only_the_composition_root_knows_it() {
    check(&[Rule::forbid("nothing depends on bootstrap")
        .from("**")
        .except_from("bootstrap::**")
        .except_from("crate")
        .to("bootstrap::**")
        .because("bootstrap wires the modules together, so it depends on them and they never on it")]);
}
