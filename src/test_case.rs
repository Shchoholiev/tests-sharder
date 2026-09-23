use std::num::NonZeroU32;

use serde::{Deserialize, Deserializer, Serialize, de::Error};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Test {
    pub id: String,
    #[serde(deserialize_with = "positive_duration")]
    pub duration_ms: NonZeroU32,
}

fn positive_duration<'de, D>(deserializer: D) -> Result<NonZeroU32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u32::deserialize(deserializer)?;
    NonZeroU32::new(value).ok_or_else(|| D::Error::custom("duration_ms must be positive"))
}

impl Test {
    pub fn new(id: impl Into<String>, duration_ms: u32) -> Self {
        Self {
            id: id.into(),
            duration_ms: NonZeroU32::new(duration_ms).expect("test duration must be non-zero"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "test duration must be non-zero")]
    fn zero_duration_is_rejected() {
        Test::new("zero", 0);
    }
}
