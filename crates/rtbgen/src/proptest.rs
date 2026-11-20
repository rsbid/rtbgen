//! Strategies for property tests.
//!
//! The strategy varies the typed knobs and rebuilds through
//! [`Request::builder`](crate::Request::builder), so `proptest` shrinks toward a
//! smaller seed, fewer impressions and fewer deals instead of raw bytes.

use ::proptest::option;
use ::proptest::prelude::{Strategy, any};
use ::proptest::sample::select;

use crate::currency::Currency;
use crate::deals::Deals;
use crate::floor::Floor;
use crate::impressions::ImpressionCount;
use crate::profile::Profile;
use crate::request::Request;
use crate::seed::Seed;
use crate::version::SpecVersion;

const PROFILES: &[Profile] = &Profile::ALL;
const SPECS: &[SpecVersion] = &SpecVersion::ALL;
const CURRENCIES: &[Currency] = &[Currency::USD, Currency::EUR, Currency::GBP];

/// A strategy over every profile, revision and typed knob.
///
/// The seed is arbitrary, the impression count stays inside `1..=3` and the deal
/// count inside `1..=8`; the floor is optional. Every generated request is
/// built by the same core as a hand-written builder, so a failing case shrinks
/// to a small seed or a smaller knob rather than to an opaque byte string.
pub fn request() -> impl Strategy<Value = Request> {
    (
        select(PROFILES),
        any::<u64>(),
        select(SPECS),
        1_u8..=ImpressionCount::MAX.count(),
        1_u8..=Deals::MAX.count(),
        option::of((0_i64..=1_000_000_i64, select(CURRENCIES))),
    )
        .prop_map(|(profile, seed, spec, impressions, deals, floor)| {
            let builder = Request::builder(profile)
                .seed(Seed::new(seed))
                .spec(spec)
                .impressions(ImpressionCount::new(impressions))
                .deals(Deals::new(deals));
            let builder = floor.map_or(builder, |(scaled, currency)| {
                builder.floor(Floor::from_scaled(scaled, currency))
            });

            builder.build()
        })
}
