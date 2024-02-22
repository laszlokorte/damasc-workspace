#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub start: usize,
    pub end: usize,
}

impl Location {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
