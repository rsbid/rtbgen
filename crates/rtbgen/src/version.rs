use core::fmt;

/// The `OpenRTB` specification revision a request is generated for.
///
/// The 2.x revisions share one wire model: 2.5 is the 2.6 model without the
/// fields the 2.6 revision introduced, so a profile keeps its shape under both
/// and only loses the newer fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpecVersion {
    /// `OpenRTB` 2.5.
    V25,
    /// `OpenRTB` 2.6, the default.
    #[default]
    V26,
}

impl SpecVersion {
    /// Every supported revision, oldest first.
    pub const ALL: [Self; 2] = [Self::V25, Self::V26];

    /// The dotted name accepted by [`SpecVersion::from_name`].
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::V25 => "2.5",
            Self::V26 => "2.6",
        }
    }

    /// Parses the exact dotted name of a supported revision.
    ///
    /// The match is case-sensitive; there are no aliases such as `"v26"` or
    /// `"2"`. The names come from [`SpecVersion::ALL`], so a new revision is
    /// parsed as soon as it is listed.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownSpecVersion`] when `name` is not one of
    /// [`SpecVersion::ALL`]'s names.
    pub fn from_name(name: &str) -> Result<Self, UnknownSpecVersion> {
        Self::ALL
            .into_iter()
            .find(|version| version.name() == name)
            .ok_or(UnknownSpecVersion)
    }
}

impl fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The error returned by [`SpecVersion::from_name`] for an unsupported revision.
#[derive(Debug, Clone, Copy)]
pub struct UnknownSpecVersion;

impl fmt::Display for UnknownSpecVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unsupported OpenRTB revision; expected one of:")?;

        let mut names = SpecVersion::ALL.iter();
        if let Some(first) = names.next() {
            write!(f, " {first}")?;
        }
        for version in names {
            write!(f, ", {version}")?;
        }

        Ok(())
    }
}

impl core::error::Error for UnknownSpecVersion {}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::{SpecVersion, UnknownSpecVersion};

    #[test]
    fn default_is_2_6() {
        assert_eq!(SpecVersion::default(), SpecVersion::V26);
    }

    #[test]
    fn every_version_round_trips_through_its_name() {
        for version in SpecVersion::ALL {
            assert!(matches!(
                SpecVersion::from_name(version.name()),
                Ok(found) if found == version
            ));
        }
    }

    #[test]
    fn names_are_unique_dotted_revisions() {
        let mut names = std::vec::Vec::new();
        for version in SpecVersion::ALL {
            let name = version.name();
            assert!(
                name.bytes()
                    .all(|byte| byte.is_ascii_digit() || byte == b'.'),
                "{name} is not a dotted revision"
            );
            assert!(
                names.iter().all(|seen| *seen != name),
                "{name} is a duplicate"
            );
            names.push(name);
        }
    }

    #[test]
    fn unsupported_revisions_are_rejected() {
        assert!(matches!(
            SpecVersion::from_name("2.7"),
            Err(UnknownSpecVersion)
        ));
        assert!(matches!(
            SpecVersion::from_name("v26"),
            Err(UnknownSpecVersion)
        ));
        assert!(matches!(
            SpecVersion::from_name(""),
            Err(UnknownSpecVersion)
        ));
    }

    #[test]
    fn error_lists_every_version() {
        let error = UnknownSpecVersion;
        let message = std::format!("{error}");

        for version in SpecVersion::ALL {
            assert!(
                message.contains(version.name()),
                "{message} does not list {}",
                version.name()
            );
        }
    }
}
