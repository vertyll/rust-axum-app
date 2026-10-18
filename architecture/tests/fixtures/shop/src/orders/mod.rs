pub mod domain;

use crate::billing;

pub fn place() {
    // Through the name a `use` brought in: crate::billing::internal.
    let _ = billing::internal::total(crate::kernel::Money::default());
    // A path inside a macro is not read.
    println!("{}", stringify!(crate::bootstrap::wire));
}

mod inline {
    use super::domain::*;

    pub fn first() -> Order {
        Order
    }
}

#[cfg(test)]
mod tests {
    use crate::bootstrap::wire;
}
