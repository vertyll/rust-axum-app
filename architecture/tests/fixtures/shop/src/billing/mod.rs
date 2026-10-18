mod internal;

pub use internal::Invoice;

pub fn bill() -> Invoice {
    // Through a child module, written in the code.
    internal::Invoice::default()
}
