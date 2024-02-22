#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub span: (usize, usize),
}

impl Location {
    pub fn new(start: usize, end: usize) -> Self {
        Self { span: (start, end) }
    }
}
