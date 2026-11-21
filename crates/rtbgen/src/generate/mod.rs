//! The generation core: one dispatch, one `Entropy`, one model.
//!
//! Every profile is a distribution over the same wire model. A profile module
//! receives the entropy of the seed and the configuration knobs; the knobs are
//! applied afterwards, so a knob that is not set cannot change the bytes of the
//! profile.

mod app_banner;
mod app_rewarded;
mod data;
mod knobs;
mod maximal;
mod multi_imp;
mod parts;
mod pmp_deals;
mod privacy;
mod site_display;
mod site_native;
mod site_video;
mod sparse;

use crate::deals::Deals;
use crate::degrade::{self, DegradeRate, Mutation};
use crate::deprecated::DeprecatedFields;
use crate::floor::Floor;
use crate::ids::RequestId;
use crate::impressions::ImpressionCount;
use crate::privacy::Privacy;
use crate::profile::Profile;
use crate::rng::{Entropy, RngEntropy};
use crate::seed::Seed;
use crate::version::SpecVersion;
use crate::wire::RequestModel;

/// The resolved builder configuration.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Config {
    id: RequestId,
    impressions: Option<ImpressionCount>,
    deals: Option<Deals>,
    floor: Option<Floor>,
    privacy: Option<Privacy>,
    degrade: DegradeRate,
    spec: SpecVersion,
    deprecated: DeprecatedFields,
}

impl Config {
    pub(crate) const fn new(
        id: RequestId,
        impressions: Option<ImpressionCount>,
        deals: Option<Deals>,
        floor: Option<Floor>,
    ) -> Self {
        Self {
            id,
            impressions,
            deals,
            floor,
            privacy: None,
            degrade: DegradeRate::OFF,
            spec: SpecVersion::V26,
            deprecated: DeprecatedFields::Omit,
        }
    }

    /// Sets the share of requests that carry one deliberate violation.
    pub(crate) const fn degrade(mut self, degrade: DegradeRate) -> Self {
        self.degrade = degrade;
        self
    }

    /// Sets the revision the mutation selector is filtered by.
    pub(crate) const fn spec(mut self, spec: SpecVersion) -> Self {
        self.spec = spec;
        self
    }

    /// Sets the deprecated-field policy.
    pub(crate) const fn deprecated(mut self, deprecated: DeprecatedFields) -> Self {
        self.deprecated = deprecated;
        self
    }

    /// Draws the request id, then builds the configuration.
    ///
    /// The id is consumed before the profile distribution, so an entropy
    /// stream maps to exactly one request. Every entropy entry point goes
    /// through this helper, so the order cannot drift between them.
    pub(crate) fn from_entropy(
        entropy: &mut impl Entropy,
        impressions: Option<ImpressionCount>,
        deals: Option<Deals>,
        floor: Option<Floor>,
    ) -> Self {
        Self::new(RequestId::from_entropy(entropy), impressions, deals, floor)
    }

    pub(crate) const fn id(self) -> RequestId {
        self.id
    }

    pub(crate) const fn impressions(self) -> Option<ImpressionCount> {
        self.impressions
    }

    pub(crate) const fn deals(self) -> Option<Deals> {
        self.deals
    }

    pub(crate) const fn floor(self) -> Option<Floor> {
        self.floor
    }

    /// Returns the privacy scenario override, when the builder set one.
    pub(crate) const fn privacy(self) -> Option<Privacy> {
        self.privacy
    }

    /// Sets the privacy scenario override.
    pub(crate) const fn privacy_override(mut self, privacy: Privacy) -> Self {
        self.privacy = Some(privacy);
        self
    }

    pub(crate) const fn degrade_rate(self) -> DegradeRate {
        self.degrade
    }

    pub(crate) const fn spec_version(self) -> SpecVersion {
        self.spec
    }

    /// Returns the deprecated-field policy.
    pub(crate) const fn deprecated_fields(self) -> DeprecatedFields {
        self.deprecated
    }
}

/// A generated body and the mutation it carries, if any.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Generated {
    model: RequestModel,
    mutation: Option<Mutation>,
}

impl Generated {
    /// Returns the wire model to write.
    pub(crate) const fn model(&self) -> &RequestModel {
        &self.model
    }

    /// Returns the mutation the request carries, if any.
    pub(crate) const fn mutation(&self) -> Option<Mutation> {
        self.mutation
    }
}

/// Generates the request for a profile and a seed.
pub(crate) fn build(profile: Profile, seed: Seed, config: &Config) -> Generated {
    let mut entropy = RngEntropy::from_seed(seed);

    build_with(profile, config, &mut entropy)
}

/// Generates the request from any entropy source.
///
/// This is the single generation path: the seeded builder and the statistical
/// adapters both land here, so a profile cannot drift between sources.
pub(crate) fn build_with(
    profile: Profile,
    config: &Config,
    entropy: &mut impl Entropy,
) -> Generated {
    let mut model = match profile {
        Profile::SiteDisplay => site_display::build(entropy, config),
        Profile::SiteVideo => site_video::build(entropy, config),
        Profile::SiteNative => site_native::build(entropy, config),
        Profile::AppRewarded => app_rewarded::build(entropy, config),
        Profile::AppBanner => app_banner::build(entropy, config),
        Profile::PmpDeals => pmp_deals::build(entropy, config),
        Profile::MultiImp => multi_imp::build(entropy, config),
        Profile::Sparse => sparse::build(entropy, config),
        Profile::Maximal => maximal::build(entropy, config),
    };

    knobs::apply_knobs(&mut model, config);
    knobs::apply_deprecated_fields(&mut model, config);

    let mutation =
        degrade::apply(&mut model, config.degrade_rate(), config.spec_version(), entropy);

    Generated { model, mutation }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::String;

    use serde_json::Value;

    use super::{Config, build, build_with};
    use crate::ids::RequestId;
    use crate::profile::Profile;
    use crate::rng::{BytesEntropy, RngEntropy};
    use crate::seed::Seed;
    use crate::version::SpecVersion;
    use crate::wire::RequestModel;
    use crate::write;

    fn config(seed: u64) -> Config {
        Config::new(RequestId::from_seed(Seed::new(seed)), None, None, None)
    }

    fn json(model: &RequestModel) -> String {
        let mut out = String::new();
        if write::request(&mut out, model, SpecVersion::V26).is_err() {
            out.clear();
        }
        out
    }

    fn from_bytes(profile: Profile, bytes: &[u8]) -> String {
        let mut entropy = BytesEntropy::new(bytes);
        let config = Config::from_entropy(&mut entropy, None, None, None);

        json(build_with(profile, &config, &mut entropy).model())
    }

    #[test]
    fn every_profile_generates() {
        for profile in Profile::ALL {
            let generated = build(profile, Seed::new(42), &config(42));

            assert!(
                generated.model().impressions().iter().next().is_some(),
                "{profile} generated no impression"
            );
        }
    }

    #[test]
    fn the_seeded_path_is_the_shared_core() {
        for profile in Profile::ALL {
            let seeded = build(profile, Seed::new(42), &config(42));
            let mut entropy = RngEntropy::from_seed(Seed::new(42));
            let shared = build_with(profile, &config(42), &mut entropy);

            assert_eq!(json(seeded.model()), json(shared.model()), "{profile} drifted");
        }
    }

    #[test]
    fn empty_byte_entropy_still_produces_valid_json() {
        for profile in Profile::ALL {
            let output = from_bytes(profile, &[]);

            assert!(
                serde_json::from_str::<Value>(&output).is_ok(),
                "{profile} produced invalid JSON from empty entropy"
            );
        }
    }

    #[test]
    fn the_same_bytes_produce_the_same_request() {
        let bytes = [0x2A; 64];

        for profile in Profile::ALL {
            assert_eq!(
                from_bytes(profile, &bytes),
                from_bytes(profile, &bytes),
                "{profile} is not deterministic over bytes"
            );
        }
    }
}
