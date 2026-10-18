//! The front-end's architecture: pages in `app` on top of components, forms, the back-end client and the modules the
//! pages share (`frontend/README.md`).

use std::sync::LazyLock;

use rust_axum_app_architecture::{Architecture, Rule};

static FRONTEND: LazyLock<Architecture> = LazyLock::new(|| {
    Architecture::scan(concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/src/main.rs"))
        .expect("the front-end's source is a valid crate")
});

fn check(rules: &[Rule]) {
    if let Err(violations) = FRONTEND.check(rules) {
        panic!("{violations}");
    }
}

#[test]
fn modules_depend_on_one_another_without_cycles() {
    check(&[
        Rule::no_cycles("the top-level modules form no cycle")
            .between("*")
            .because("a cycle makes two modules one, and neither can change alone"),
        Rule::no_cycles("the pages form no cycle")
            .between("app::*")
            .because("a page that needs another's code takes it from a shared module"),
    ]);
}

#[test]
fn only_the_entry_point_knows_the_pages() {
    check(&[Rule::forbid("nothing below the pages knows them")
        .from("**")
        .except_from("app::**")
        .except_from("crate")
        .to("app::**")
        .because("the pages sit on top: they use the rest, and the rest serves every page")]);
}

#[test]
fn a_page_keeps_its_inside_to_itself() {
    check(&[Rule::through_root("a page is reached through its root")
        .module("app::*")
        .except_from("app")
        .because(
            "what one page needs from another belongs in a shared module; only the router in `app` lists each page's \
             routes",
        )]);
}

#[test]
fn components_know_no_page_and_no_session() {
    check(&[Rule::forbid("components are the user interface's kit")
        .from("components::**")
        .to("app::**")
        .to("auth::**")
        .to("keycloak::**")
        .because("a component shows what it is given, whoever is signed in and wherever it is used")]);
}

#[test]
fn the_back_end_client_knows_no_user_interface() {
    check(&[Rule::forbid("the back-end client speaks HTTP only")
        .from("backend::**")
        .to("app::**")
        .to("components::**")
        .to("forms::**")
        .to("keycloak::**")
        .to("theme::**")
        .to("i18n::**")
        .to("auth::**")
        .because("the pages and the session use it to reach the back end, so it uses none of them")]);
}

#[test]
fn keycloak_is_reached_through_its_root() {
    check(&[Rule::through_root("keycloak is reached through its root")
        .module("keycloak")
        .because("the way into Keycloak's pages is one module, and the rest uses only what it re-exports")]);
}
