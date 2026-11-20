use crate::impressions::ImpressionCount;
use crate::rng::{Entropy, Share};
use crate::wire::RequestModel;

use super::Config;
use super::parts;

/// Builds a site request with two or three mixed-media impressions.
///
/// The `2..=3` count is the profile's default distribution.
/// [`impressions_exact`](crate::RequestBuilder::impressions_exact) overrides it
/// with an explicit count, including a single impression.
pub(super) fn build(rng: &mut impl Entropy, config: &Config) -> RequestModel {
    parts::build_site_request_custom(rng, config, |rng| {
        let default = 2_u8.saturating_add(u8::from(rng.bit(Share::percent(50))));
        let count = config.impressions().map_or(default, ImpressionCount::count);

        parts::build_mixed_imp_list(rng, count)
    })
}
