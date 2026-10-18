// Two modules that depend on each other, and a third outside the circle.
mod first {
    pub fn start() {
        crate::second::middle();
    }
}

mod second {
    pub fn middle() {}

    pub fn back() {
        super::first::start();
    }
}

mod third {
    use crate::first;
}
