/// A trusted JSON literal for an `ext` object.
///
/// The core has no JSON parser. Construction is crate-private and reserved for
/// static literals that are valid JSON by inspection; the writer splices the
/// literal verbatim, so it must never come from untrusted input.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawJson(&'static str);

impl RawJson {
    /// Wraps a static literal that is already valid JSON.
    #[must_use]
    pub(crate) const fn from_static(raw: &'static str) -> Self {
        Self(raw)
    }

    /// Returns the literal.
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        self.0
    }
}
