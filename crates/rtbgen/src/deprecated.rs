use core::fmt;

/// Whether the generator emits fields that the target revision deprecated.
///
/// The policy is applied after the profile body and its knobs, and before
/// `degrade`. `OpenRTB` 2.6 deprecated `imp.video.placement` in favour of
/// `imp.video.plcmt`; 2.5 defines `placement` and no `plcmt`. The default
/// therefore writes `plcmt` under 2.6 and `placement` under 2.5, while
/// [`Self::Include`] keeps the deprecated field next to its replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DeprecatedFields {
    /// Omit the fields the target revision deprecated; this is the default.
    #[default]
    Omit,
    /// Emit the deprecated fields as well as their replacements.
    Include,
}

impl DeprecatedFields {
    /// Every policy, in the order used by the corpus manifest.
    pub const ALL: [Self; 2] = [Self::Omit, Self::Include];

    /// The stable `snake_case` name accepted by [`DeprecatedFields::from_name`].
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Omit => "omit",
            Self::Include => "include",
        }
    }

    /// Parses the exact `snake_case` name of a policy.
    ///
    /// The match is case-sensitive; there are no aliases such as `"Omit"` or
    /// `"omit_fields"`. The names come from [`DeprecatedFields::ALL`].
    ///
    /// # Errors
    ///
    /// Returns [`UnknownDeprecatedFields`] when `name` is not one of
    /// [`DeprecatedFields::ALL`]'s names.
    pub fn from_name(name: &str) -> Result<Self, UnknownDeprecatedFields> {
        Self::ALL.into_iter().find(|policy| policy.name() == name).ok_or(UnknownDeprecatedFields)
    }
}

impl fmt::Display for DeprecatedFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The error returned by [`DeprecatedFields::from_name`] for an unknown name.
#[derive(Debug, Clone, Copy)]
pub struct UnknownDeprecatedFields;

impl fmt::Display for UnknownDeprecatedFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown deprecated-field policy; expected one of:")?;

        let mut names = DeprecatedFields::ALL.iter();
        if let Some(first) = names.next() {
            write!(f, " {first}")?;
        }
        for policy in names {
            write!(f, ", {policy}")?;
        }

        Ok(())
    }
}

impl core::error::Error for UnknownDeprecatedFields {}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::{DeprecatedFields, UnknownDeprecatedFields};

    #[test]
    fn the_default_is_omit() {
        assert_eq!(DeprecatedFields::default(), DeprecatedFields::Omit);
    }

    #[test]
    fn every_policy_round_trips_through_its_name() {
        for policy in DeprecatedFields::ALL {
            assert!(matches!(
                DeprecatedFields::from_name(policy.name()),
                Ok(found) if found == policy
            ));
        }
    }

    #[test]
    fn unknown_and_malformed_names_are_rejected() {
        for name in ["Include", "include_fields", "", "permissive"] {
            assert!(matches!(DeprecatedFields::from_name(name), Err(UnknownDeprecatedFields)));
        }
    }

    #[test]
    fn the_error_lists_every_policy() {
        let message = std::format!("{UnknownDeprecatedFields}");

        for policy in DeprecatedFields::ALL {
            assert!(message.contains(policy.name()), "{message} does not list {}", policy.name());
        }
    }
}
