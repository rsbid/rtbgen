use core::fmt;

/// Returns the integer part of a finite, non-negative `f64`.
///
/// `f64 as u64` is forbidden by the lint profile, and the standard library's
/// rounding helpers live outside `core`. The function instead decodes the IEEE
/// 754 representation, which is exact and independent of the Rust version: the
/// value is `mantissa × 2^exponent`, so the integer part is a shift.
fn truncated_to_u64(value: f64) -> u64 {
    if value <= 0.0_f64 {
        return 0;
    }

    let bits = value.to_bits();
    let exponent_bits = (bits >> 52_u32) & 0x7ff;
    if exponent_bits == 0 {
        // Zero or subnormal: smaller than one.
        return 0;
    }

    let mantissa = (bits & 0x000f_ffff_ffff_ffff) | 0x0010_0000_0000_0000;
    let exponent = i32::try_from(exponent_bits)
        .ok()
        .and_then(|exponent| exponent.checked_sub(1_075))
        .unwrap_or(-1_i32);

    if exponent >= 0 {
        let shift = u32::try_from(exponent).unwrap_or(63);
        mantissa.checked_shl(shift).unwrap_or(u64::MAX)
    } else {
        mantissa.checked_shr(exponent.unsigned_abs()).unwrap_or(0)
    }
}

/// A non-negative money amount with a fixed scale of 10⁻⁴ units.
///
/// The wire model never holds an `f64`: a float is parsed once, at the builder
/// boundary, and converted into this integer representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Money(i64);

impl Money {
    const SCALE: u64 = 10_000;

    /// The largest amount the builder accepts: one million currency units.
    const MAX_SCALED: f64 = 10_000_000_000.0;

    /// Creates an amount from its scaled integer representation.
    #[must_use]
    pub(crate) const fn from_scaled(value: i64) -> Self {
        Self(value)
    }

    /// Converts a CPM value into the fixed-scale representation.
    ///
    /// This is the only place where a float becomes money. `NaN`, negative and
    /// non-finite values collapse to zero, values above [`Self::MAX_SCALED`]
    /// clamp to the maximum, and the amount is rounded half up to the 10⁻⁴
    /// scale, so the writer always sees an integer.
    #[must_use]
    pub(crate) fn from_cpm(cpm: f64) -> Self {
        if cpm.is_nan() || cpm <= 0.0_f64 {
            return Self(0);
        }

        let scaled = cpm * 10_000.0_f64 + 0.5_f64;
        let capped = if scaled > Self::MAX_SCALED {
            Self::MAX_SCALED
        } else {
            scaled
        };

        // The value is finite, non-negative and capped; `truncated_to_u64`
        // recovers the integer part without an `as` cast.
        Self(i64::try_from(truncated_to_u64(capped)).unwrap_or(i64::MAX))
    }

    /// Returns the scaled integer representation; used by the money tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn scaled(self) -> i64 {
        self.0
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

    #[test]
    fn cpm_rounds_half_up_to_the_wire_scale() {
        let cases: [(f64, &str); 8] = [
            (0.0, "0.0"),
            (1.5, "1.5"),
            (2.0, "2.0"),
            (0.0001, "0.0001"),
            (1.15, "1.15"),
            (1.005, "1.005"),
            (1.234_56, "1.2346"),
            (9_999.999_95, "10000.0"),
        ];

        for (cpm, expected) in cases {
            assert_eq!(
                Money::from_cpm(cpm).to_string(),
                expected,
                "cpm {cpm} formatted wrong"
            );
        }
    }

    #[test]
    fn invalid_cpm_clamps_instead_of_panicking() {
        assert_eq!(Money::from_cpm(f64::NAN).to_string(), "0.0");
        assert_eq!(Money::from_cpm(f64::INFINITY).to_string(), "1000000.0");
        assert_eq!(Money::from_cpm(f64::NEG_INFINITY).to_string(), "0.0");
        assert_eq!(Money::from_cpm(-1.0).to_string(), "0.0");
        assert_eq!(Money::from_cpm(2_000_000.0).to_string(), "1000000.0");
    }
}
