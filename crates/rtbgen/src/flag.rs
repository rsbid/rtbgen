//! A wire-level yes/no flag.

use core::fmt;

/// A two-valued wire flag, written as `0` or `1`.
///
/// `OpenRTB` distinguishes "absent" from "no": the third state lives in
/// `Option<Flag>` at the field, so the flag itself has exactly two values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Flag {
    /// The `0` value.
    No,
    /// The `1` value.
    Yes,
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::No => "0",
            Self::Yes => "1",
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::ToString;

    use super::Flag;

    #[test]
    fn displays_zero_and_one() {
        assert_eq!(Flag::No.to_string(), "0");
        assert_eq!(Flag::Yes.to_string(), "1");
    }
}
