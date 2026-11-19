use crate::ids::ImpId;
use crate::rng::Entropy;
use crate::wire::{Banner, Imp, ImpList, RequestModel};

use super::Config;
use super::parts;

/// Builds the smallest valid request: an id and one impression with a banner.
///
/// The profile deliberately omits `site`, `app`, `device`, `user`, `at`, `tmax`
/// and `cur`, which is what makes it useful for presence rules: a parser must
/// treat every one of those objects as optional.
pub(super) fn build(_rng: &mut impl Entropy, config: &Config) -> RequestModel {
    let count = parts::impression_count(config, 1);
    let mut imps = ImpList::default();

    for index in 1..=count {
        let mut banner = Banner::default();
        banner.set_w(Some(300));
        banner.set_h(Some(250));

        let mut imp = Imp::default();
        imp.set_id(ImpId::new(u32::from(index)));
        imp.set_banner(Some(banner));
        imps.push(&imp);
    }

    let mut request = RequestModel::default();
    request.set_id(config.id());
    request.set_imp(imps);
    request
}
