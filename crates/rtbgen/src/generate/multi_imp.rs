use crate::impressions::ImpressionCount;
use crate::rng::{Entropy, Share};
use crate::wire::RequestModel;

use super::Config;
use super::parts;

pub(super) fn build(rng: &mut impl Entropy, config: &Config) -> RequestModel {
    parts::build_site_request_custom(rng, config, |rng| {
        let default = 2_u8.saturating_add(u8::from(rng.bit(Share::percent(50))));
        let count = config.impressions().map_or(default, ImpressionCount::count);

        parts::build_mixed_imp_list(rng, count)
    })
}
