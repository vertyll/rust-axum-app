# Architecture tests

The rules the back-end's and the front-end's modules keep, checked on their source on every change, as ArchUnit checks
a Java build. A rule broken by a change fails the [Architecture workflow](../.github/workflows/architecture.yml) and
names the line that broke it.

## What is checked

The rules live in [`tests/backend.rs`](tests/backend.rs) and [`tests/frontend.rs`](tests/frontend.rs), one test per
rule, each saying why it holds.

| Crate     | Rules                                                                                                     |
|-----------|-----------------------------------------------------------------------------------------------------------|
| back-end  | the modules form no cycle, and each is reached only through what its `mod.rs` exports, but by `bootstrap` |
| back-end  | the `domain` knows neither `application` nor an adapter, and the `application` knows no adapter           |
| back-end  | `shared_kernel` uses no other module, `shared_http` only the kernel, and no module depends on `bootstrap` |
| front-end | the modules and the pages form no cycle, and only the entry point and the router know the pages           |
| front-end | components know no page and no session, and the back-end client knows no user interface                   |
| front-end | `keycloak` is reached through its root                                                                    |

They describe the [back-end's architecture](../docs/architecture.md) and the [front-end's
layout](../frontend/README.md#layout).

## How it works

`Architecture::scan` reads a crate's module tree from `src/lib.rs` or `src/main.rs`, following the `mod` declarations as
the compiler does, inline ones and `#[path]` included. It records each path one module writes to another: every path of
a `use` declaration, groups and renames expanded, and every path in the code that starts with `crate`, `self`, `super`,
a child module or a module a `use` brought in. Each points at the deepest module on its path, so
`crate::identity::Permitted` is a use of `identity`'s root and `crate::identity::domain::user::UserId` one of its
domain.

A rule is one of three, each finished by `because`, which the failure shows:

- `Rule::forbid`: the modules `from` matches may not depend on the ones `to` matches, with exceptions on either side;
- `Rule::no_cycles`: the modules `between` matches depend on one another without a cycle;
- `Rule::through_root`: other modules reach these only through their root.

A pattern names modules by their path from the crate root: a name matches itself, `*` any one segment and `**` any
number of them, none included. A pattern that matches no module fails the check, since a rule about nothing is a typo.
Code compiled for tests only, under `#[cfg(test)]`, is left out unless a rule asks for it.

The scan reads source, not the compiler's name resolution. It misses a path written inside a macro invocation, and a
name a glob import brought in and used bare; neither crosses a module boundary without a `use` it does see.

## Why its own crate

The back-end's and the front-end's images are built from their own folders, so a path dependency on this crate would
not resolve there. The rules read their source from beside them instead, and run with the rest of CI whenever
`backend/src`, `frontend/src` or this folder changes. It depends on `syn` to parse Rust and on nothing that runs at
build time.

## Running it

From `architecture/`:

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```
