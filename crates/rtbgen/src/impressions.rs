/// The number of impressions a generated request carries.
///
/// The value is clamped to `1..=3`: a request always has at least one
/// impression, and `MAX_IMPRESSIONS` is the exchange cap `MultiImp` and the
/// future `degrade` mutation share.
/// [`RequestBuilder::impressions_exact`](crate::RequestBuilder::impressions_exact)
/// overrides a profile's default count, so `ImpressionCount::new(0)` means one
/// impression rather than none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImpressionCount(u8);

impl ImpressionCount {
    /// The largest impression count a request can carry.
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "the wire cap is 3, which fits u8"
    )]
    pub const MAX: Self = Self(crate::wire::MAX_IMPRESSIONS as u8);

    /// Builds a count, clamping it into `1..=3`.
    ///
    /// Clamping keeps `build()` infallible: the caller cannot express an
    /// impression count the exchange would reject, and the type still records
    /// exactly how many impressions the request has.
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

    /// Returns the clamped count.
    #[must_use]
    pub const fn count(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::ImpressionCount;

    #[test]
    fn counts_inside_the_range_are_kept() {
        for count in 1..=ImpressionCount::MAX.count() {
            assert_eq!(ImpressionCount::new(count).count(), count);
        }
    }

    #[test]
    fn counts_are_clamped_instead_of_rejected() {
        assert_eq!(ImpressionCount::new(0).count(), 1);
        assert_eq!(ImpressionCount::new(u8::MAX).count(), 3);
    }
}
