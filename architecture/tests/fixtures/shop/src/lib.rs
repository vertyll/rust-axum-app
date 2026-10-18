// A small crate whose modules exercise every way of naming another module that the scanner follows.
mod billing;
mod bootstrap;
mod orders;
#[path = "shared/kernel.rs"]
mod kernel;
mod r#static;
