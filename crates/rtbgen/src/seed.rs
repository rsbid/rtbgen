/// The seed that makes a generated request reproducible.
///
/// A seed plus a profile is the whole fixture: keep the seed in the test, not the JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seed(u64);

impl Seed {
    /// Creates a seed from its raw value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw seed value.
    #[must_use]
    pub const fn into_inner(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Seed;

    #[test]
    fn seed_round_trips() {
        let seed = Seed::new(42);

        assert_eq!(seed.into_inner(), 42);
        assert_eq!(seed, Seed::new(42));
    }
}
