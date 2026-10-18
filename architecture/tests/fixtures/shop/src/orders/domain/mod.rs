use crate::{billing::Invoice as Bill, r#static::Asset};

pub struct Order;

#[cfg(all(test, unix))]
fn only_in_tests() -> Bill {
    crate::bootstrap::wire()
}

pub fn priced(_asset: Asset) {}
