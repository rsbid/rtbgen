use crate::deals::Deals;
use crate::flag::Flag;
use crate::rng::{Entropy, Share};
use crate::wire::{Pmp, RequestModel};

use super::Config;
use super::knobs;
use super::parts::{self, Media};

pub(super) fn build(rng: &mut impl Entropy, config: &Config) -> RequestModel {
    let mut model = parts::build_site_request(rng, config, Media::Banner);

    let count = config.deals.map_or_else(
        || 2_u8.saturating_add(u8::from(rng.bit(Share::percent(50)))),
        Deals::count,
    );
    let deals = knobs::build_deals(rng, count);

    for imp in model.impressions_mut().iter_mut() {
        imp.set_pmp(Some(Pmp::new(Flag::Yes, &deals)));
    }

    model
}
