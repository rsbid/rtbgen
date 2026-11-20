//! The widening knobs, applied after the profile has chosen its shape.
//!
//! A knob never consumes the seed and never changes a request when it is not
//! set, so the default bytes of a profile are independent of the knob API.

use crate::code::DealAuctionType;
use crate::currency::Currency;
use crate::flag::Flag;
use crate::ids::DealId;
use crate::money::Money;
use crate::rng::{Entropy, Share};
use crate::wire::{Deal, DealList, Pmp, RequestModel, WireCurrency, WireNumber};

use super::Config;
use super::data::{ADV_DOMAINS, DEAL_AT, DEAL_FLOORS, FLOORS};
use super::parts::choose;

/// Builds the deals of a profile from the seed.
pub(super) fn build_deals(rng: &mut impl Entropy, count: u8) -> DealList {
    let mut deals = DealList::default();

    for index in 1..=count {
        let mut deal = Deal::new(DealId::new(u64::from(index)));
        deal.set_bidfloor(Some(Money::from_scaled(choose(rng, FLOORS, 15_000))));
        deal.set_bidfloorcur(Some(Currency::USD));
        deal.set_at(Some(choose(rng, DEAL_AT, DealAuctionType::SECOND_PRICE_PLUS)));
        if rng.bit(Share::percent(40)) {
            deal.set_wadomain(choose(rng, ADV_DOMAINS, &["adv.example"]));
        }
        deals.push(&deal);
    }

    deals
}

/// Builds the deals of a knob, independent of the seed.
///
/// A knob is a widening, not a distribution: the same count produces the same
/// deal section for every seed, so the caller can reason about the output.
pub(super) fn build_fixed_deals(count: u8) -> DealList {
    let mut deals = DealList::default();

    for index in 1..=count {
        let ordinal = usize::from(index);
        let offset = ordinal.checked_sub(1);

        let mut deal = Deal::new(DealId::new(u64::from(index)));
        let floor = offset.and_then(|value| DEAL_FLOORS.get(value)).copied();
        deal.set_bidfloor(Some(Money::from_scaled(floor.unwrap_or(15_000))));
        deal.set_bidfloorcur(Some(Currency::USD));
        let slot = offset.and_then(|value| value.checked_rem(DEAL_AT.len())).unwrap_or(0);
        let at = DEAL_AT.get(slot).copied();
        deal.set_at(Some(at.unwrap_or(DealAuctionType::SECOND_PRICE_PLUS)));
        deals.push(&deal);
    }

    deals
}

/// Applies the widening knobs to a generated model.
pub(super) fn apply_knobs(model: &mut RequestModel, config: &Config) {
    if let Some(floor) = config.floor() {
        for imp in model.impressions_mut().iter_mut() {
            imp.set_bidfloor(Some(WireNumber::number(floor.money())));
            imp.set_bidfloorcur(Some(WireCurrency::code(floor.currency())));
        }
    }

    if let Some(deals) = config.deals() {
        let list = build_fixed_deals(deals.count());
        for imp in model.impressions_mut().iter_mut() {
            imp.set_pmp(Some(Pmp::new(Flag::Yes, &list)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_knobs, build_deals, build_fixed_deals};
    use crate::deals::Deals;
    use crate::generate::Config;
    use crate::generate::parts::{Media, build_imp_list};
    use crate::ids::RequestId;
    use crate::rng::RngEntropy;
    use crate::seed::Seed;
    use crate::wire::RequestModel;

    #[test]
    fn fixed_deals_are_counted() {
        assert_eq!(build_fixed_deals(1).count(), 1);
        assert_eq!(build_fixed_deals(4).count(), 4);
        assert_eq!(build_fixed_deals(8).count(), 8);
    }

    #[test]
    fn seeded_deals_fit_the_cap() {
        let mut rng = RngEntropy::from_seed(Seed::new(3));
        let deals = build_deals(&mut rng, Deals::MAX.count());

        assert_eq!(deals.count(), 8);
    }

    #[test]
    fn knobs_apply_to_every_impression() {
        let mut model = RequestModel::default();
        let mut rng = RngEntropy::from_seed(Seed::new(1));
        model.set_imp(build_imp_list(&mut rng, Media::Banner, 2));

        apply_knobs(
            &mut model,
            &Config::new(RequestId::from_seed(Seed::new(1)), None, Some(Deals::new(2)), None),
        );

        for imp in model.impressions().iter() {
            assert!(imp.pmp_for_tests().is_some(), "an imp lost its pmp");
        }
    }
}
