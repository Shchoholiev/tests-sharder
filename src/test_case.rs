#[derive(PartialEq, Eq, Clone)]
pub struct Test {
    pub id: String,
    pub duration_ms: u32,
}

impl Test {
    pub fn new(id: impl Into<String>, duration_ms: u32) -> Self {
        Self {
            id: id.into(),
            duration_ms,
        }
    }
}

impl Ord for Test {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.duration_ms
            .cmp(&other.duration_ms)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for Test {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
