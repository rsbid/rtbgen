use crate::currency::Currency;
use crate::money::Money;

/// A `bidfloor` with its currency.
///
/// The floor is a knob: [`crate::RequestBuilder::floor`] writes it onto every
/// impression of the request, whatever the profile chose on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Floor {
    money: Money,
    currency: Currency,
}

impl Floor {
    /// Builds a floor from a CPM amount and its currency.
    ///
    /// This is the only `f64` in the public API, and it is a parse point: the
    /// amount is converted to the fixed 10⁻⁴ scale immediately and never
    /// reaches the wire model as a float. `NaN`, negative and non-finite values
    /// become `0`, and amounts above one million CPM clamp to the maximum; the
    /// result is rounded half up.
    #[must_use]
    pub fn cpm(cpm: f64, currency: Currency) -> Self {
        Self {
            money: Money::from_cpm(cpm),
            currency,
        }
    }

    /// Builds a floor from the fixed-scale representation, for the corpus.
    #[must_use]
    pub(crate) const fn from_scaled(scaled: i64, currency: Currency) -> Self {
        Self {
            money: Money::from_scaled(scaled),
            currency,
        }
    }

    /// Returns the fixed-scale amount.
    #[must_use]
    pub(crate) const fn money(self) -> Money {
        self.money
    }

    /// Returns the currency.
    #[must_use]
    pub(crate) const fn currency(self) -> Currency {
        self.currency
    }
}

#[cfg(test)]
mod tests {
    use super::Floor;
    use crate::currency::Currency;

    #[test]
    fn cpm_is_converted_once_at_the_boundary() {
        let floor = Floor::cpm(1.5, Currency::USD);

        assert_eq!(floor.money().scaled(), 15_000);
        assert_eq!(floor.currency(), Currency::USD);
    }

    #[test]
    fn invalid_cpm_does_not_panic() {
        assert_eq!(Floor::cpm(f64::NAN, Currency::EUR).money().scaled(), 0);
        assert_eq!(Floor::cpm(-2.0, Currency::EUR).money().scaled(), 0);
    }

    #[test]
    fn zero_is_a_real_floor() {
        let floor = Floor::cpm(0.0, Currency::GBP);

        assert_eq!(floor.money().scaled(), 0);
        assert_eq!(floor.currency(), Currency::GBP);
    }
}
