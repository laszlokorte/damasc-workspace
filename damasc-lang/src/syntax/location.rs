#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Location {
    pub span: std::ops::Range<usize>,
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.span.start.partial_cmp(&other.span.start)
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.span.start.cmp(&other.span.start)
    }
}
