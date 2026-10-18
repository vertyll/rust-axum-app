use rust_axum_app_architecture::{Architecture, Rule, Violations};

fn scan(fixture: &str) -> Architecture {
    let root = format!("{}/tests/fixtures/{fixture}/src/lib.rs", env!("CARGO_MANIFEST_DIR"));
    Architecture::scan(root).expect("the fixture is a valid crate")
}

fn details(result: Result<(), Violations>) -> Vec<String> {
    result.err().map_or_else(Vec::new, |violations| {
        violations.iter().map(|violation| violation.detail.clone()).collect()
    })
}

#[test]
fn a_forbidden_dependency_is_reported_with_where_it_is_written() {
    let rule = Rule::forbid("orders keep out of billing's inside")
        .from("orders::**")
        .to("billing::*")
        .because("billing is used through its root");

    let found = details(scan("shop").check(&[rule]));

    assert_eq!(found.len(), 1);
    assert!(
        found[0].starts_with("crate::orders uses crate::billing::internal::total ("),
        "{found:?}"
    );
    assert!(found[0].ends_with("orders/mod.rs:7)"), "{found:?}");
}

#[test]
fn exceptions_leave_modules_out_on_either_side() {
    let rule = Rule::forbid("nothing reaches billing")
        .from("**")
        .except_from("billing::**")
        .except_from("bootstrap")
        .to("billing::**")
        .except_to("billing")
        .because("only to show the exceptions");

    let found = details(scan("shop").check(&[rule]));

    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("crate::orders uses crate::billing::internal::total"));
}

#[test]
fn test_code_is_left_out_unless_the_rule_includes_it() {
    let shipped = Rule::forbid("orders know no bootstrap")
        .from("orders::**")
        .to("bootstrap")
        .because("the composition root depends on modules, not the reverse");
    let everything = Rule::forbid("orders know no bootstrap, even in tests")
        .from("orders::**")
        .to("bootstrap")
        .including_tests()
        .because("the composition root depends on modules, not the reverse");

    assert_eq!(details(scan("shop").check(&[shipped])), Vec::<String>::new());
    assert_eq!(details(scan("shop").check(&[everything])).len(), 2);
}

#[test]
fn a_module_is_reached_only_through_its_root() {
    let rule = Rule::through_root("billing is reached through its root")
        .module("billing")
        .except_from("bootstrap")
        .because("its inside is its own");

    let found = details(scan("shop").check(&[rule]));

    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("crate::orders uses crate::billing::internal::total"));
    assert!(found[0].ends_with("inside crate::billing rather than through its root"));
}

#[test]
fn a_cycle_between_modules_is_reported_with_each_step() {
    let rule = Rule::no_cycles("top-level modules form no cycle")
        .between("*")
        .because("a cycle makes two modules one");

    let found = details(scan("cycle").check(&[rule]));

    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("crate::first, crate::second depend on one another: "),
        "{found:?}"
    );
    assert!(found[0].contains("crate::first uses crate::second::middle"));
    assert!(found[0].contains("crate::second uses crate::first::start"));
}

#[test]
fn the_shop_has_no_cycle_in_what_it_ships() {
    let rule = Rule::no_cycles("top-level modules form no cycle")
        .between("*")
        .because("a cycle makes two modules one");

    assert_eq!(details(scan("shop").check(&[rule])), Vec::<String>::new());
}

#[test]
fn a_pattern_that_matches_no_module_is_a_violation() {
    let rule = Rule::forbid("a typo")
        .from("ordrs::**")
        .to("billing")
        .because("it checks nothing");

    assert_eq!(
        details(scan("shop").check(&[rule])),
        ["the pattern `ordrs::**` matches no module"]
    );
}

#[test]
fn a_cycle_rule_needs_a_pattern_of_fixed_length() {
    let rule = Rule::no_cycles("nothing fixed")
        .between("**")
        .because("it names no set of modules");

    assert_eq!(
        details(scan("shop").check(&[rule])),
        ["the pattern `**` has a `**`, so it names no fixed set of modules"]
    );
}

#[test]
fn violations_are_shown_grouped_under_their_rule() {
    let rule = Rule::through_root("billing is reached through its root")
        .module("billing")
        .because("its inside is its own");

    let shown = scan("shop")
        .check(&[rule])
        .expect_err("orders reaches inside billing")
        .to_string();

    assert!(
        shown.contains("billing is reached through its root — its inside is its own\n  crate::"),
        "{shown}"
    );
}
