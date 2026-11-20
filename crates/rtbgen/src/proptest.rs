//! Strategies for property tests.
//!
//! The strategy varies the typed knobs and rebuilds through
//! [`Request::builder`](crate::Request::builder), so `proptest` shrinks toward a
//! smaller seed and absent or smaller knobs instead of raw bytes.

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
/// The seed and the revision are arbitrary; the impression count stays inside
/// `1..=3` and the deal count inside `1..=8`, and both are optional, so the
/// strategy also produces requests with the profile's own defaults and without
/// a `pmp`. The floor is optional too. Every generated request is built by the
/// same core as a hand-written builder, so a failing case shrinks to a small
/// seed or a smaller knob rather than to an opaque byte string.
pub fn request() -> impl Strategy<Value = Request> {
    (
        select(PROFILES),
        any::<u64>(),
        select(SPECS),
        option::of(1_u8..=ImpressionCount::MAX.count()),
        option::of(1_u8..=Deals::MAX.count()),
        option::of((0_i64..=1_000_000_i64, select(CURRENCIES))),
    )
        .prop_map(|(profile, seed, spec, impressions, deals, floor)| {
            let builder = Request::builder(profile).seed(Seed::new(seed)).spec(spec);
            let builder = impressions
                .map_or(builder, |count| builder.impressions_exact(ImpressionCount::new(count)));
            let builder = deals.map_or(builder, |count| builder.deals(Deals::new(count)));
            let builder = floor.map_or(builder, |(scaled, currency)| {
                builder.floor(Floor::from_scaled(scaled, currency))
            });

            builder.build()
        })
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "in tests a panic = a failed test")]
mod tests {
    use ::proptest::strategy::{Strategy as _, ValueTree as _};
    use ::proptest::test_runner::{Config, TestRunner};

    use super::request;

    #[test]
    fn the_strategy_covers_knob_absence_and_presence() {
        let mut runner = TestRunner::new(Config::default());
        let mut without_pmp = false;
        let mut with_pmp = false;

        for _ in 0_u16..256 {
            let tree = request().new_tree(&mut runner).expect("the strategy always builds");
            let request = tree.current();
            let json = request.to_json_string();
            let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
            let has_pmp = value.pointer("/imp/0/pmp").is_some();

            with_pmp |= has_pmp;
            without_pmp |= !has_pmp;
        }

        assert!(with_pmp, "the strategy never generates a pmp");
        assert!(without_pmp, "the strategy never omits the pmp");
    }
}
