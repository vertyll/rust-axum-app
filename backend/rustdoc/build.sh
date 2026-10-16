#!/usr/bin/env bash
# Builds the API documentation of the shared modules with rustdoc, under a
# landing page that groups them by layer (target/doc/index.html).
set -euo pipefail
cd "$(dirname "$0")/.."

RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --lib --locked

crate=rust_axum_app_backend
card() {
  printf '      <li><a href="%s/%s/index.html"><code>%s</code></a><span>%s</span></li>' "$crate" "$1" "$1" "$2"
}

page=$(<rustdoc/landing.html)
page=${page//@@PROJECT@@/rust-axum-app}
page=${page//@@SUBJECT@@/shared module}
page=${page//@@HEADING@@/shared modules}
page=${page//@@LEDE@@/API documentation generated from the doc comments with rustdoc.}
page=${page//@@PURE_SCOPE@@/safe for a domain or application layer}
page=${page//@@UNIT@@/module}
page=${page//@@PURE@@/$(card shared_kernel "Paging and the messages a module ships: plain values, no I/O")}
page=${page//@@INFRASTRUCTURE@@/$(card shared_http "Problem details, the paging query and response, and the extractors that answer with a problem")}
printf '%s\n' "$page" > target/doc/index.html

echo "rustdoc: file://$PWD/target/doc/index.html"
