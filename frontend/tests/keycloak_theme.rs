//! The sign-in pages Keycloak draws (`keycloak/themes/rust-axum-app`) carry their own copy of the colors, as Keycloak
//! serves its theme apart from this application.

use std::fs;
use std::path::Path;

#[test]
fn the_keycloak_theme_has_the_colors_of_the_app() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let app = fs::read_to_string(root.join("styles.css")).expect("the stylesheet is readable");
    let keycloak = fs::read_to_string(root.join("../keycloak/themes/rust-axum-app/login/resources/css/app.css"))
        .expect("the Keycloak theme is next to the front-end");

    assert_eq!(tokens(&keycloak), tokens(&app));
}

/// The first `:root` block, where both stylesheets keep their tokens.
fn tokens(css: &str) -> &str {
    let start = css.find(":root {").expect("the stylesheet defines its tokens on :root");
    let end = start + css[start..].find("\n}").expect("the :root block is closed");
    &css[start..end]
}
