//! The versioned golden corpus.
//!
//! Every entry is one `(spec, profile, seed, knobs)` tuple whose bytes are
//! checked into `corpus/v1/`. The corpus is the executable form of the
//! determinism contract: regenerate it with `rtbgen corpus --write`, never by
//! hand, and treat a changed file as a byte change of the public API.

use crate::deals::Deals;
use crate::floor::Floor;
use crate::impressions::ImpressionCount;
use crate::profile::Profile;
use crate::request::Request;
use crate::seed::Seed;
use crate::version::SpecVersion;

/// The version of the corpus contract.
pub const VERSION: &str = "v1";

/// The directory, relative to the crate root, that holds the corpus.
pub const DIR: &str = "corpus/v1";

/// One golden fixture.
#[derive(Clone, Copy, Debug)]
pub struct CorpusEntry {
    name: &'static str,
    spec: SpecVersion,
    profile: Profile,
    seed: Seed,
    impressions: Option<ImpressionCount>,
    deals: Option<Deals>,
    floor: Option<Floor>,
}

impl CorpusEntry {
    /// The file stem of the entry inside [`DIR`].
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// The `OpenRTB` revision the entry is generated for.
    #[must_use]
    pub const fn spec(self) -> SpecVersion {
        self.spec
    }

    /// The profile of the entry.
    #[must_use]
    pub const fn profile(self) -> Profile {
        self.profile
    }

    /// The seed of the entry.
    #[must_use]
    pub const fn seed(self) -> Seed {
        self.seed
    }

    /// The impression override, when the entry exercises that knob.
    #[must_use]
    pub const fn impressions(self) -> Option<ImpressionCount> {
        self.impressions
    }

    /// The deal override, when the entry exercises that knob.
    #[must_use]
    pub const fn deals(self) -> Option<Deals> {
        self.deals
    }

    /// The floor override, when the entry exercises that knob.
    #[must_use]
    pub const fn floor(self) -> Option<Floor> {
        self.floor
    }

    /// Regenerates the fixture.
    #[must_use]
    pub fn request(self) -> Request {
        let mut builder = Request::builder(self.profile)
            .seed(self.seed)
            .spec(self.spec);

        if let Some(count) = self.impressions {
            builder = builder.impressions(count);
        }
        if let Some(count) = self.deals {
            builder = builder.deals(count);
        }
        if let Some(floor) = self.floor {
            builder = builder.floor(floor);
        }

        builder.build()
    }
}

const fn entry(name: &'static str, spec: SpecVersion, profile: Profile, seed: u64) -> CorpusEntry {
    CorpusEntry {
        name,
        spec,
        profile,
        seed: Seed::new(seed),
        impressions: None,
        deals: None,
        floor: None,
    }
}

const fn with_impressions(mut entry: CorpusEntry, count: u8) -> CorpusEntry {
    entry.impressions = Some(ImpressionCount::new(count));
    entry
}

const fn with_deals(mut entry: CorpusEntry, count: u8) -> CorpusEntry {
    entry.deals = Some(Deals::new(count));
    entry
}

const fn with_floor(mut entry: CorpusEntry, scaled: i64) -> CorpusEntry {
    entry.floor = Some(Floor::from_scaled(scaled, crate::Currency::USD));
    entry
}

const fn with_floor_in(
    mut entry: CorpusEntry,
    scaled: i64,
    currency: crate::Currency,
) -> CorpusEntry {
    entry.floor = Some(Floor::from_scaled(scaled, currency));
    entry
}

/// Every golden fixture, in the order the manifest lists them.
pub const ALL: &[CorpusEntry] = &[
    entry(
        "site_display-v26-0",
        SpecVersion::V26,
        Profile::SiteDisplay,
        0,
    ),
    entry(
        "site_display-v26-42",
        SpecVersion::V26,
        Profile::SiteDisplay,
        42,
    ),
    entry(
        "site_display-v26-max",
        SpecVersion::V26,
        Profile::SiteDisplay,
        u64::MAX,
    ),
    entry(
        "site_display-v25-42",
        SpecVersion::V25,
        Profile::SiteDisplay,
        42,
    ),
    entry("site_video-v26-7", SpecVersion::V26, Profile::SiteVideo, 7),
    entry("site_video-v25-7", SpecVersion::V25, Profile::SiteVideo, 7),
    entry(
        "site_native-v26-3",
        SpecVersion::V26,
        Profile::SiteNative,
        3,
    ),
    entry(
        "site_native-v25-3",
        SpecVersion::V25,
        Profile::SiteNative,
        3,
    ),
    entry(
        "app_rewarded-v26-42",
        SpecVersion::V26,
        Profile::AppRewarded,
        42,
    ),
    entry(
        "app_rewarded-v25-42",
        SpecVersion::V25,
        Profile::AppRewarded,
        42,
    ),
    entry(
        "app_banner-v26-11",
        SpecVersion::V26,
        Profile::AppBanner,
        11,
    ),
    entry(
        "app_banner-v25-11",
        SpecVersion::V25,
        Profile::AppBanner,
        11,
    ),
    entry("pmp_deals-v26-5", SpecVersion::V26, Profile::PmpDeals, 5),
    entry("pmp_deals-v25-5", SpecVersion::V25, Profile::PmpDeals, 5),
    entry("multi_imp-v26-9", SpecVersion::V26, Profile::MultiImp, 9),
    entry("multi_imp-v25-9", SpecVersion::V25, Profile::MultiImp, 9),
    entry("sparse-v26-0", SpecVersion::V26, Profile::Sparse, 0),
    entry("sparse-v25-0", SpecVersion::V25, Profile::Sparse, 0),
    entry("maximal-v26-2", SpecVersion::V26, Profile::Maximal, 2),
    entry("maximal-v26-0", SpecVersion::V26, Profile::Maximal, 0),
    entry("maximal-v25-2", SpecVersion::V25, Profile::Maximal, 2),
    entry("maximal-v25-0", SpecVersion::V25, Profile::Maximal, 0),
    with_floor(
        with_deals(
            with_impressions(
                entry(
                    "knobs-app-rewarded-v26-42",
                    SpecVersion::V26,
                    Profile::AppRewarded,
                    42,
                ),
                2,
            ),
            3,
        ),
        15_000,
    ),
    with_floor_in(
        with_deals(
            with_impressions(
                entry(
                    "knobs-site-display-v25-8",
                    SpecVersion::V25,
                    Profile::SiteDisplay,
                    8,
                ),
                3,
            ),
            1,
        ),
        2_500,
        crate::Currency::EUR,
    ),
    with_deals(
        with_impressions(
            entry(
                "knobs-multi-imp-v26-13",
                SpecVersion::V26,
                Profile::MultiImp,
                13,
            ),
            1,
        ),
        8,
    ),
    with_floor(
        with_deals(
            entry(
                "knobs-pmp-deals-v26-12",
                SpecVersion::V26,
                Profile::PmpDeals,
                12,
            ),
            8,
        ),
        30_000,
    ),
    with_floor_in(
        with_impressions(
            entry(
                "knobs-site-video-v26-6",
                SpecVersion::V26,
                Profile::SiteVideo,
                6,
            ),
            2,
        ),
        5_000,
        crate::Currency::GBP,
    ),
    with_deals(
        with_impressions(
            entry(
                "knobs-app-banner-v25-4",
                SpecVersion::V25,
                Profile::AppBanner,
                4,
            ),
            2,
        ),
        2,
    ),
];

#[cfg(test)]
mod tests {
    extern crate std;

    use super::ALL;
    use crate::profile::Profile;
    use crate::version::SpecVersion;

    #[test]
    fn names_are_unique_and_filesystem_safe() {
        let mut names = std::vec::Vec::new();

        for entry in ALL {
            assert!(
                entry.name().bytes().all(|byte| byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || byte == b'-'
                    || byte == b'_'),
                "{} is not a safe file stem",
                entry.name()
            );
            assert!(
                !names.contains(&entry.name()),
                "{} is listed twice",
                entry.name()
            );
            names.push(entry.name());
        }
    }

    #[test]
    fn every_profile_and_spec_has_a_golden() {
        for profile in Profile::ALL {
            for spec in SpecVersion::ALL {
                assert!(
                    ALL.iter()
                        .any(|entry| entry.profile() == profile && entry.spec() == spec),
                    "{profile} has no {spec} golden"
                );
            }
        }
    }

    #[test]
    fn the_knobs_are_covered() {
        assert!(ALL.iter().any(|entry| entry.impressions().is_some()));
        assert!(ALL.iter().any(|entry| entry.deals().is_some()));
        assert!(ALL.iter().any(|entry| entry.floor().is_some()));
    }

    #[test]
    fn entries_rebuild_the_same_bytes() {
        for entry in ALL {
            let first = entry.request().to_json_string();
            let second = entry.request().to_json_string();

            assert_eq!(first, second, "{} is not reproducible", entry.name());
        }
    }
}
