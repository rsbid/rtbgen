use core::fmt;

/// A non-negative money amount with a fixed scale of 10⁻⁴ units.
///
/// The wire model never holds an `f64`: a float is parsed once, at the builder
/// boundary, and converted into this integer representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Money(i64);

impl Money {
    const SCALE: u64 = 10_000;

    /// Creates an amount from its scaled integer representation.
    #[must_use]
    pub(crate) const fn from_scaled(value: i64) -> Self {
        Self(value)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let magnitude = self.0.unsigned_abs();
        let units = magnitude.checked_div(Self::SCALE).unwrap_or(0);
        let fraction = magnitude % Self::SCALE;
        let sign = if self.0 < 0 { "-" } else { "" };

        write!(f, "{sign}{units}.")?;

        if fraction == 0 {
            return f.write_str("0");
        }

        // Trim trailing zeros, but keep at least one fractional digit.
        let (divisor, digits) = if fraction % 1_000 == 0 {
            (1_000_u64, 1_u8)
        } else if fraction % 100 == 0 {
            (100_u64, 2_u8)
        } else if fraction % 10 == 0 {
            (10_u64, 3_u8)
        } else {
            (1_u64, 4_u8)
        };

        let scaled = fraction.checked_div(divisor).unwrap_or(0);

        match digits {
            1 => write!(f, "{scaled}"),
            2 => write!(f, "{scaled:02}"),
            3 => write!(f, "{scaled:03}"),
            _ => write!(f, "{fraction:04}"),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::ToString;

    use super::Money;

    #[test]
    fn formats_with_at_least_one_fractional_digit() {
        let cases = [
            (0_i64, "0.0"),
            (1, "0.0001"),
            (10, "0.001"),
            (100, "0.01"),
            (1_000, "0.1"),
            (15_000, "1.5"),
            (20_000, "2.0"),
            (11_500, "1.15"),
            (10_500, "1.05"),
            (12_345, "1.2345"),
        ];

        for (scaled, expected) in cases {
            assert_eq!(Money::from_scaled(scaled).to_string(), expected);
        }
    }

    #[test]
    fn formats_negative_amounts_without_negative_zero() {
        assert_eq!(Money::from_scaled(-15_000).to_string(), "-1.5");
        assert_eq!(Money::from_scaled(-1).to_string(), "-0.0001");
        assert_eq!(Money::from_scaled(-10_000).to_string(), "-1.0");
    }
}
