use core::fmt::{self, Write};

/// A three-letter ISO-4217 currency code.
///
/// The type carries only well-formed codes: three uppercase ASCII letters.
/// Lowercase, symbols and wrong lengths are rejected at construction, so the
/// wire model never has to escape a currency and the writer never emits an
/// invalid code. The deliberately invalid values (`"usd"`, `"RUR"`, `""`) are a
/// `degrade` concern and cannot be built here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Currency([u8; 3]);

impl Currency {
    /// `USD`: the currency of every default profile.
    pub const USD: Self = Self(*b"USD");

    /// `EUR`.
    pub const EUR: Self = Self(*b"EUR");

    /// `GBP`.
    pub const GBP: Self = Self(*b"GBP");

    /// Parses a three-letter uppercase ASCII currency code.
    ///
    /// The match requires exactly three bytes and no lowercase letters: the
    /// decoder of a bid request is expected to accept the abstract ISO code,
    /// not a spelling variant.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownCurrency`] for any length other than three and for any
    /// byte outside `A..=Z`.
    pub fn from_code(code: &str) -> Result<Self, UnknownCurrency> {
        let mut bytes = [0_u8; 3];

        if code.len() != bytes.len() {
            return Err(UnknownCurrency);
        }

        for (slot, byte) in bytes.iter_mut().zip(code.bytes()) {
            if !byte.is_ascii_uppercase() {
                return Err(UnknownCurrency);
            }
            *slot = byte;
        }

        Ok(Self(bytes))
    }

    /// Writes the code into a JSON string literal.
    ///
    /// Every code is validated ASCII, so no escaping is needed.
    pub(crate) fn write_literal(self, out: &mut impl Write) -> fmt::Result {
        out.write_char('"')?;
        for byte in self.0 {
            out.write_char(char::from(byte))?;
        }
        out.write_char('"')
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            f.write_char(char::from(byte))?;
        }
        Ok(())
    }
}

/// The error returned by [`Currency::from_code`] for a malformed code.
#[derive(Debug, Clone, Copy)]
pub struct UnknownCurrency;

impl fmt::Display for UnknownCurrency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "invalid currency code; expected three uppercase ASCII letters, such as \"USD\"",
        )
    }
}

impl core::error::Error for UnknownCurrency {}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::ToString;

    use super::{Currency, UnknownCurrency};

    #[test]
    fn known_codes_display_the_iso_code() {
        assert_eq!(Currency::USD.to_string(), "USD");
        assert_eq!(Currency::EUR.to_string(), "EUR");
        assert_eq!(Currency::GBP.to_string(), "GBP");
    }

    #[test]
    fn every_code_round_trips_through_from_code() {
        for currency in [Currency::USD, Currency::EUR, Currency::GBP] {
            let code = currency.to_string();
            assert!(matches!(
                Currency::from_code(&code),
                Ok(found) if found == currency
            ));
        }
    }

    #[test]
    fn lowercase_short_and_malformed_codes_are_rejected() {
        for code in ["usd", "US", "USDD", "", "U5D", "U S"] {
            assert!(
                matches!(Currency::from_code(code), Err(UnknownCurrency)),
                "{code:?} was accepted"
            );
        }
    }

    #[test]
    fn unknown_codes_are_valid_iso_shaped_values() {
        let custom = Currency::from_code("CHF");

        assert!(matches!(custom, Ok(code) if code.to_string() == "CHF"));
    }
}
