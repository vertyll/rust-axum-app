use super::super::kernel::{self, Money};

#[derive(Default)]
pub struct Invoice {
    pub total: kernel::Money,
}

pub fn total(money: Money) -> Money {
    money
}
