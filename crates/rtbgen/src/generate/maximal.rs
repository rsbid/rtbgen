use crate::ids::ImpId;
use crate::rng::{Entropy, Share};
use crate::wire::{ImpList, RequestModel, WireId};

use super::Config;

/// Builds the largest valid request the model can emit.
///
/// `site` and `app` are mutually exclusive in `OpenRTB`, so the profile keeps
/// exactly one of them: the seed chooses which. That is why the profile is a
/// pair of shapes and why the inventory test walks several seeds instead of one.
pub(super) fn build(rng: &mut impl Entropy, config: &Config) -> RequestModel {
    let keep_site = rng.bit(Share::percent(50));
    let mut model = RequestModel::maximal(config.id());

    if keep_site {
        model.set_app(None);
    } else {
        model.set_site(None);
    }

    if let Some(count) = config.impressions() {
        model.set_imp(maximal_impressions(&model, count.count()));
    }

    model
}

fn maximal_impressions(model: &RequestModel, count: u8) -> ImpList {
    let mut imps = ImpList::default();
    let Some(base) = model.impressions().iter().next().copied() else {
        return imps;
    };

    for index in 1..=count {
        let mut imp = base;
        imp.set_id(WireId::id(ImpId::new(u32::from(index))));
        imps.push(&imp);
    }

    imps
}
