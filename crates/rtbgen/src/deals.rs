/// The number of private-marketplace deals a request carries.
///
/// The value is clamped to `1..=8`. [`crate::RequestBuilder::deals`] widens any
/// profile with a `pmp` object: the profile keeps its shape, and the knob adds
/// the deals section instead of conflicting with it. `Deals::new(0)` therefore
/// means one deal; removing a marketplace is not part of the knob contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deals(u8);

impl Deals {
    /// The largest number of deals a single `pmp` object can carry.
    pub const MAX: Self = Self(8);

    /// Builds a deal count, clamping it into `1..=8`.
    #[must_use]
    pub const fn new(count: u8) -> Self {
        let maximum = Self::MAX.0;
        Self(if count < 1 {
            1
        } else if count > maximum {
            maximum
        } else {
            count
        })
    }

    /// Returns the clamped deal count.
    #[must_use]
    pub const fn count(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Deals;

    #[test]
    fn counts_inside_the_range_are_kept() {
        for count in 1..=Deals::MAX.count() {
            assert_eq!(Deals::new(count).count(), count);
        }
    }

    #[test]
    fn counts_are_clamped_instead_of_rejected() {
        assert_eq!(Deals::new(0).count(), 1);
        assert_eq!(Deals::new(u8::MAX).count(), 8);
    }

    #[test]
    fn the_cap_is_the_wire_cap() {
        assert_eq!(usize::from(Deals::MAX.count()), crate::wire::MAX_DEALS);
    }
}
