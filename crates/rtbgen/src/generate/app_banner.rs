use crate::rng::Entropy;
use crate::wire::RequestModel;

use super::Config;
use super::parts::{self, Media};

pub(super) fn build(rng: &mut impl Entropy, config: &Config) -> RequestModel {
    parts::build_app_request(rng, config, Media::Banner, false)
}
