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
mod site_display;
mod site_native;
mod site_video;
mod sparse;

use crate::deals::Deals;
use crate::floor::Floor;
use crate::impressions::ImpressionCount;
use crate::profile::Profile;
use crate::rng::Rng;
use crate::seed::Seed;
use crate::wire::RequestModel;

/// The resolved builder configuration.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Config {
    seed: Seed,
    impressions: Option<ImpressionCount>,
    deals: Option<Deals>,
    floor: Option<Floor>,
}

impl Config {
    pub(crate) const fn new(
        seed: Seed,
        impressions: Option<ImpressionCount>,
        deals: Option<Deals>,
        floor: Option<Floor>,
    ) -> Self {
        Self {
            seed,
            impressions,
            deals,
            floor,
        }
    }

    pub(crate) const fn seed(self) -> Seed {
        self.seed
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
}

/// Generates the request for a profile and a configuration.
pub(crate) fn build(profile: Profile, config: &Config) -> RequestModel {
    let mut rng = Rng::from_seed(config.seed());

    let mut model = match profile {
        Profile::SiteDisplay => site_display::build(&mut rng, config),
        Profile::SiteVideo => site_video::build(&mut rng, config),
        Profile::SiteNative => site_native::build(&mut rng, config),
        Profile::AppRewarded => app_rewarded::build(&mut rng, config),
        Profile::AppBanner => app_banner::build(&mut rng, config),
        Profile::PmpDeals => pmp_deals::build(&mut rng, config),
        Profile::MultiImp => multi_imp::build(&mut rng, config),
        Profile::Sparse => sparse::build(&mut rng, config),
        Profile::Maximal => maximal::build(&mut rng, config),
    };

    knobs::apply_knobs(&mut model, config);
    model
}

#[cfg(test)]
mod tests {
    use super::{Config, build};
    use crate::profile::Profile;
    use crate::seed::Seed;

    fn config(seed: u64) -> Config {
        Config::new(Seed::new(seed), None, None, None)
    }

    #[test]
    fn every_profile_generates() {
        for profile in Profile::ALL {
            let model = build(profile, &config(42));

            assert!(
                model.impressions().iter().next().is_some(),
                "{profile} generated no impression"
            );
        }
    }
}
