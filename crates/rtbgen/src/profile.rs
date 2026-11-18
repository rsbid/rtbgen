use core::fmt;

/// A request shape that `rtbgen` can generate.
///
/// A profile is a distribution, not a template: it fixes which fields may appear
/// and with what probabilities. The seed chooses the concrete request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// A website with a display banner.
    SiteDisplay,
    /// A website with a video impression.
    SiteVideo,
    /// A website with a native impression.
    SiteNative,
    /// An application with a rewarded video impression.
    AppRewarded,
    /// An application with a display banner.
    AppBanner,
    /// A website with a private marketplace and deals.
    PmpDeals,
    /// A website with several mixed-media impressions.
    MultiImp,
    /// The minimum request: an id and an impression.
    Sparse,
    /// Every field the model can emit.
    Maximal,
}

impl Profile {
    /// Every profile, in the order used by `--list-profiles` and the corpus.
    pub const ALL: [Self; 9] = [
        Self::SiteDisplay,
        Self::SiteVideo,
        Self::SiteNative,
        Self::AppRewarded,
        Self::AppBanner,
        Self::PmpDeals,
        Self::MultiImp,
        Self::Sparse,
        Self::Maximal,
    ];

    /// The `snake_case` name accepted by [`Profile::from_name`].
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SiteDisplay => "site_display",
            Self::SiteVideo => "site_video",
            Self::SiteNative => "site_native",
            Self::AppRewarded => "app_rewarded",
            Self::AppBanner => "app_banner",
            Self::PmpDeals => "pmp_deals",
            Self::MultiImp => "multi_imp",
            Self::Sparse => "sparse",
            Self::Maximal => "maximal",
        }
    }

    /// A short description of the shape, used by `--list-profiles`.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::SiteDisplay => "website with a display banner",
            Self::SiteVideo => "website with an in-stream or outstream video",
            Self::SiteNative => "website with a native impression",
            Self::AppRewarded => "application with a rewarded video",
            Self::AppBanner => "application with a display banner",
            Self::PmpDeals => "website with a private marketplace and deals",
            Self::MultiImp => "website with mixed-media impressions",
            Self::Sparse => "the minimum: an id and one banner impression",
            Self::Maximal => "every field the model can emit in one valid request",
        }
    }

    /// Parses the exact `snake_case` name of a profile.
    ///
    /// The match is case-sensitive; there are no aliases. The names come from
    /// [`Profile::ALL`], so a new profile is parsed as soon as it is listed.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownProfile`] when `name` is not one of [`Profile::ALL`]'s names.
    pub fn from_name(name: &str) -> Result<Self, UnknownProfile> {
        Self::ALL
            .into_iter()
            .find(|profile| profile.name() == name)
            .ok_or(UnknownProfile)
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The error returned by [`Profile::from_name`] for an unknown name.
#[derive(Debug, Clone, Copy)]
pub struct UnknownProfile;

impl fmt::Display for UnknownProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown profile; expected one of:")?;

        let mut names = Profile::ALL.iter();
        if let Some(first) = names.next() {
            write!(f, " {first}")?;
        }
        for profile in names {
            write!(f, ", {profile}")?;
        }

        Ok(())
    }
}

impl core::error::Error for UnknownProfile {}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::{Profile, UnknownProfile};

    #[test]
    fn every_profile_round_trips_through_its_name() {
        for profile in Profile::ALL {
            assert!(matches!(
                Profile::from_name(profile.name()),
                Ok(found) if found == profile
            ));
        }
    }

    #[test]
    fn names_are_unique_snake_case() {
        let mut names = std::vec::Vec::new();
        for profile in Profile::ALL {
            let name = profile.name();
            assert!(
                name.bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte == b'_'),
                "{name} is not snake_case"
            );
            assert!(
                names.iter().all(|seen| *seen != name),
                "{name} is a duplicate"
            );
            names.push(name);
        }
    }

    #[test]
    fn unknown_and_malformed_names_are_rejected() {
        assert!(matches!(
            Profile::from_name("AppRewarded"),
            Err(UnknownProfile)
        ));
        assert!(matches!(
            Profile::from_name("app-rewarded"),
            Err(UnknownProfile)
        ));
        assert!(matches!(Profile::from_name(""), Err(UnknownProfile)));
    }

    #[test]
    fn unknown_profile_lists_every_name() {
        let error = UnknownProfile;
        let message = std::format!("{error}");

        for profile in Profile::ALL {
            assert!(
                message.contains(profile.name()),
                "{message} does not list {}",
                profile.name()
            );
        }
    }
}
