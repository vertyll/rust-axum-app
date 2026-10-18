use crate::orders::{self, domain::Order};

pub fn wire() -> crate::billing::Invoice {
    let _: Option<Order> = None;
    orders::place();
    crate::billing::bill()
}
