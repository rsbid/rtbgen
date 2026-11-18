use core::fmt;

/// An ISO-4217 alpha currency code.
///
/// The model carries only valid codes; invalid values are a `degrade` concern.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Currency(&'static str);

impl Currency {
    pub(crate) const USD: Self = Self("USD");

    /// Returns the three-letter code.
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::ToString;

    use super::Currency;

    #[test]
    fn displays_the_iso_code() {
        assert_eq!(Currency::USD.to_string(), "USD");
    }

    #[test]
    fn codes_are_three_uppercase_ascii_letters() {
        let code = Currency::USD.as_str();
        assert_eq!(code.len(), 3);
        assert!(code.bytes().all(|byte| byte.is_ascii_uppercase()));
    }
}
