/// The seed that makes a generated request reproducible.
///
/// A seed plus a profile is the whole fixture: keep the seed in the test, not the JSON.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

    /// Returns the seed `steps` values later, wrapping at the end of the range.
    ///
    /// The command-line interface uses this to turn one `--seed` and one
    /// `--count` into a deterministic sequence of independent fixtures.
    #[must_use]
    pub fn advance(self, steps: u32) -> Self {
        Self(self.0.wrapping_add(u64::from(steps)))
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

    #[test]
    fn the_default_seed_is_zero() {
        assert_eq!(Seed::default(), Seed::new(0));
    }

    #[test]
    fn advance_is_a_wrapping_counter() {
        assert_eq!(Seed::new(41).advance(1), Seed::new(42));
        assert_eq!(Seed::new(u64::MAX).advance(1), Seed::new(0));
        assert_eq!(Seed::new(0).advance(0), Seed::new(0));
    }
}
