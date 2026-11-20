//! The `arbitrary` adapter: one byte slice in, one valid request out.
//!
//! The adapter draws the typed knobs first, then hands every remaining byte to
//! [`BytesEntropy`](crate::BytesEntropy). Generation therefore runs through the
//! same core as the seeded builder, and an input that ends in the middle of a
//! request still produces a request instead of a panic.

use ::arbitrary::{Arbitrary, Unstructured};

use crate::currency::Currency;
use crate::deals::Deals;
use crate::floor::Floor;
use crate::generate::{self, Config};
use crate::impressions::ImpressionCount;
use crate::profile::Profile;
use crate::request::Request;
use crate::rng::BytesEntropy;
use crate::version::SpecVersion;

/// The currencies the adapter may attach to an explicit floor.
const CURRENCIES: &[Currency] = &[Currency::USD, Currency::EUR, Currency::GBP];

impl<'bytes> Arbitrary<'bytes> for Request {
    fn arbitrary(u: &mut Unstructured<'bytes>) -> ::arbitrary::Result<Self> {
        Ok(Self::from_unstructured(u))
    }

    fn arbitrary_take_rest(u: Unstructured<'bytes>) -> ::arbitrary::Result<Self> {
        let mut u = u;

        Ok(Self::from_unstructured(&mut u))
    }
}

impl Request {
    /// Draws the knobs, then generates from every remaining byte.
    ///
    /// Every draw has a fallback, so an empty or truncated input is a valid
    /// request rather than an error: a fuzzer may hand over any byte slice.
    fn from_unstructured(u: &mut Unstructured<'_>) -> Self {
        let profile = u.choose(&Profile::ALL).copied().unwrap_or(Profile::SiteDisplay);
        let spec = u.choose(&SpecVersion::ALL).copied().unwrap_or(SpecVersion::V26);

        let impressions = u
            .arbitrary::<bool>()
            .unwrap_or(false)
            .then(|| ImpressionCount::new(u.arbitrary::<u8>().unwrap_or(0)));
        let deals = u
            .arbitrary::<bool>()
            .unwrap_or(false)
            .then(|| Deals::new(u.arbitrary::<u8>().unwrap_or(0)));
        let floor = u.arbitrary::<bool>().unwrap_or(false).then(|| {
            let currency = u.choose(CURRENCIES).copied().unwrap_or(Currency::USD);
            let scaled = i64::from(u.arbitrary::<u32>().unwrap_or(0));

            Floor::from_scaled(scaled, currency)
        });

        let remaining = u.len();
        let bytes = u.bytes(remaining).unwrap_or(&[]);
        let mut entropy = BytesEntropy::new(bytes);
        let config = Config::from_entropy(&mut entropy, impressions, deals, floor);
        let model = generate::build_with(profile, &config, &mut entropy);

        Self::from_model(profile, None, spec, &model)
    }
}
