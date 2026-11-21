//! Deliberately malformed requests.
//!
//! Valid `OpenRTB` fixtures are the default of this crate; `degrade` is the
//! opposite feature: a request carries **one** spec violation, so a parser test
//! sees a realistic document with exactly one defect. The mutation is drawn
//! after the profile body has been generated, so a request without a mutation is
//! byte-identical to the same request without the knob.
//!
//! The output is always valid JSON. Only the `OpenRTB` contract is broken; a
//! mutation that broke JSON syntax would test the wrong layer.

use core::fmt;

use crate::code::AuctionType;
use crate::raw::RawJson;
use crate::rng::{Entropy, Share, Span};
use crate::version::SpecVersion;
use crate::wire::{App, Currencies, Imp, RequestModel, Site, WireCurrency, WireId, WireNumber};

/// The share of requests that carry one deliberate violation.
///
/// `DegradeRate::percent(5)` mutates about one request in twenty;
/// `DegradeRate::percent(0)` is byte-identical to not calling the knob at all,
/// and `DegradeRate::percent(100)` always mutates. The mutation is chosen
/// deterministically from the same entropy as the request, so a seed plus a
/// rate reproduces the exact same defect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DegradeRate {
    share: Share,
}

impl Default for DegradeRate {
    fn default() -> Self {
        Self::OFF
    }
}

impl DegradeRate {
    /// No mutation: the default.
    pub const OFF: Self = Self { share: Share::ZERO };

    /// Every request mutates.
    pub const MAX: Self = Self { share: Share::MAX };

    /// Builds a rate from a whole percentage, saturating at 100%.
    #[must_use]
    pub const fn percent(percent: u16) -> Self {
        Self { share: Share::percent(percent) }
    }

    /// Builds a rate from basis points, or `None` above 100%.
    #[must_use]
    pub const fn from_basis_points(basis_points: u16) -> Option<Self> {
        match Share::from_basis_points(basis_points) {
            Some(share) => Some(Self { share }),
            None => None,
        }
    }

    /// Returns the rate in basis points.
    #[must_use]
    pub const fn basis_points(self) -> u16 {
        self.share.basis_points()
    }

    pub(crate) const fn share(self) -> Share {
        self.share
    }
}

/// One deliberate violation of the `OpenRTB` contract.
///
/// The list is closed: the crate never corrupts arbitrary JSON, it picks one of
/// the defects a real exchange has to survive. A mutation changes exactly the
/// field it names, so the rest of the request keeps the profile's shape.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutation {
    /// `at: 0`, outside the two standard codes.
    AuctionTypeZero,
    /// `at: 5`, a reserved standard code.
    AuctionTypeReserved,
    /// `at: 499`, just below the exchange-specific range.
    AuctionTypeExchangeSpecific,
    /// `bidfloor` as the JSON string `"1.50"`.
    BidFloorAsString,
    /// `bidfloorcur: "RUR"`, a retired code.
    BidFloorCurrencyUnknown,
    /// `bidfloorcur: "usd"`, the wrong case.
    BidFloorCurrencyLowercase,
    /// `bidfloorcur: ""`, an empty code.
    BidFloorCurrencyEmpty,
    /// `device.ip: "2a00:1450::1"`, an IPv6 address in an IPv4 field.
    DeviceIpv6,
    /// `id: ""`, an empty request id.
    RequestIdEmpty,
    /// `id: " "`, a blank request id.
    RequestIdBlank,
    /// `id` with a non-ASCII character, which changes bytes but not chars.
    RequestIdUnicode,
    /// Both `site` and `app`, mutually exclusive supply sources.
    SiteAndApp,
    /// Four to ten impressions, above the exchange cap.
    ImpressionsOverCap,
    /// Ten thousand impressions, an oversized body.
    HugeImpressions,
    /// The same `imp[].id` twice.
    DuplicateImpIds,
    /// `tmax: 120000`, above any sane deadline.
    TmaxHigh,
    /// `tmax: 0`, which some readers treat as "no deadline".
    TmaxZero,
    /// `tmax` absent while the rest of the request expects one.
    TmaxMissing,
    /// A `user` object without `consent`.
    ConsentMissing,
    /// `consent` with a dot, which a TCF reader must reject.
    ConsentDotted,
    /// A negative banner width.
    BannerNegativeSize,
    /// A banner without `w` and `h`.
    BannerMissingSize,
    /// An `ext` object nested far deeper than a profile ever nests it.
    DeepExt,
    /// `cur` as the string `"USD"` instead of an array.
    CurrenciesAsString,
    /// `cur: ["US1"]`, a code that is not a currency.
    CurrenciesInvalidCode,
    /// `device.geo.country: "RU"`, an alpha-2 code instead of ISO-3166-1 alpha-3.
    GeoCountryAlpha2,
    /// `device.geo.city: "Moscow"`, a city name instead of a UN/LOCODE.
    GeoCityName,
    /// `device.geo.region: "MOW"`, an ISO-3166-2 subdivision without its country prefix.
    GeoRegionWithoutCountry,
}

impl Mutation {
    /// Every mutation, in the order the selector walks them.
    ///
    /// The order is part of the determinism contract: adding a variant changes
    /// which defect a given seed selects, which is a byte change for a degraded
    /// request and therefore a version event.
    pub const ALL: &'static [Self] = &[
        Self::AuctionTypeZero,
        Self::AuctionTypeReserved,
        Self::AuctionTypeExchangeSpecific,
        Self::BidFloorAsString,
        Self::BidFloorCurrencyUnknown,
        Self::BidFloorCurrencyLowercase,
        Self::BidFloorCurrencyEmpty,
        Self::DeviceIpv6,
        Self::RequestIdEmpty,
        Self::RequestIdBlank,
        Self::RequestIdUnicode,
        Self::SiteAndApp,
        Self::ImpressionsOverCap,
        Self::HugeImpressions,
        Self::DuplicateImpIds,
        Self::TmaxHigh,
        Self::TmaxZero,
        Self::TmaxMissing,
        Self::ConsentMissing,
        Self::ConsentDotted,
        Self::BannerNegativeSize,
        Self::BannerMissingSize,
        Self::DeepExt,
        Self::CurrenciesAsString,
        Self::CurrenciesInvalidCode,
        Self::GeoCountryAlpha2,
        Self::GeoCityName,
        Self::GeoRegionWithoutCountry,
    ];

    /// The stable name used by the corpus manifest and the command line.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AuctionTypeZero => "auction_type_zero",
            Self::AuctionTypeReserved => "auction_type_reserved",
            Self::AuctionTypeExchangeSpecific => "auction_type_exchange_specific",
            Self::BidFloorAsString => "bid_floor_as_string",
            Self::BidFloorCurrencyUnknown => "bid_floor_currency_unknown",
            Self::BidFloorCurrencyLowercase => "bid_floor_currency_lowercase",
            Self::BidFloorCurrencyEmpty => "bid_floor_currency_empty",
            Self::DeviceIpv6 => "device_ipv6",
            Self::RequestIdEmpty => "request_id_empty",
            Self::RequestIdBlank => "request_id_blank",
            Self::RequestIdUnicode => "request_id_unicode",
            Self::SiteAndApp => "site_and_app",
            Self::ImpressionsOverCap => "impressions_over_cap",
            Self::HugeImpressions => "huge_impressions",
            Self::DuplicateImpIds => "duplicate_imp_ids",
            Self::TmaxHigh => "tmax_high",
            Self::TmaxZero => "tmax_zero",
            Self::TmaxMissing => "tmax_missing",
            Self::ConsentMissing => "consent_missing",
            Self::ConsentDotted => "consent_dotted",
            Self::BannerNegativeSize => "banner_negative_size",
            Self::BannerMissingSize => "banner_missing_size",
            Self::DeepExt => "deep_ext",
            Self::CurrenciesAsString => "currencies_as_string",
            Self::CurrenciesInvalidCode => "currencies_invalid_code",
            Self::GeoCountryAlpha2 => "geo_country_alpha2",
            Self::GeoCityName => "geo_city_name",
            Self::GeoRegionWithoutCountry => "geo_region_without_country",
        }
    }

    /// Returns whether this mutation has a field to break in the model.
    ///
    /// The selector never picks a mutation the request cannot carry: a request
    /// without a device gets no IPv6 address, and 2.5 has no `consent` field.
    fn applies(self, model: &RequestModel, spec: SpecVersion) -> bool {
        match self {
            Self::DeviceIpv6 => model.device().is_some(),
            Self::GeoCountryAlpha2 | Self::GeoCityName | Self::GeoRegionWithoutCountry => {
                model.device().is_some_and(|device| device.geo().is_some())
            }
            Self::ConsentMissing => {
                spec.is_v26() && model.user().is_some_and(|user| user.consent().is_some())
            }
            Self::ConsentDotted => spec.is_v26() && model.user().is_some(),
            Self::TmaxMissing => model.tmax().is_some(),
            Self::BannerNegativeSize | Self::BannerMissingSize => has_banner(model),
            Self::AuctionTypeZero
            | Self::AuctionTypeReserved
            | Self::AuctionTypeExchangeSpecific
            | Self::BidFloorAsString
            | Self::BidFloorCurrencyUnknown
            | Self::BidFloorCurrencyLowercase
            | Self::BidFloorCurrencyEmpty
            | Self::RequestIdEmpty
            | Self::RequestIdBlank
            | Self::RequestIdUnicode
            | Self::SiteAndApp
            | Self::ImpressionsOverCap
            | Self::HugeImpressions
            | Self::DuplicateImpIds
            | Self::TmaxHigh
            | Self::TmaxZero
            | Self::DeepExt
            | Self::CurrenciesAsString
            | Self::CurrenciesInvalidCode => true,
        }
    }

    /// Applies the mutation to a model.
    fn apply_to(self, model: &mut RequestModel, entropy: &mut impl Entropy) {
        match self {
            Self::AuctionTypeZero => model.set_at(Some(AuctionType::from_raw(0))),
            Self::AuctionTypeReserved => model.set_at(Some(AuctionType::from_raw(5))),
            Self::AuctionTypeExchangeSpecific => model.set_at(Some(AuctionType::from_raw(499))),
            Self::BidFloorAsString => {
                if let Some(imp) = model.impressions_mut().first_mut() {
                    imp.set_bidfloor(Some(WireNumber::Text("1.50")));
                }
            }
            Self::BidFloorCurrencyUnknown => set_floor_currency(model, "RUR"),
            Self::BidFloorCurrencyLowercase => set_floor_currency(model, "usd"),
            Self::BidFloorCurrencyEmpty => set_floor_currency(model, ""),
            Self::DeviceIpv6 => {
                if let Some(device) = model.device_mut() {
                    device.set_ip(Some("2a00:1450::1"));
                }
            }
            Self::RequestIdEmpty => model.set_id(WireId::Literal("")),
            Self::RequestIdBlank => model.set_id(WireId::Literal(" ")),
            Self::RequestIdUnicode => model.set_id(WireId::Literal("req-\u{1f3af}")),
            Self::SiteAndApp => add_both_supply_sources(model),
            Self::ImpressionsOverCap => {
                let span = Span::new(4, 10).unwrap_or(Span::from_zero(4));
                let total = u16::try_from(entropy.u32(span)).unwrap_or(4);
                let extra = total.saturating_sub(base_count(model));
                model.impressions_mut().set_extra(extra);
            }
            Self::HugeImpressions => {
                let extra = 10_000_u16.saturating_sub(base_count(model));
                model.impressions_mut().set_extra(extra);
            }
            Self::DuplicateImpIds => duplicate_imp_ids(model),
            Self::TmaxHigh => model.set_tmax(Some(120_000)),
            Self::TmaxZero => model.set_tmax(Some(0)),
            Self::TmaxMissing => model.set_tmax(None),
            Self::ConsentMissing => {
                if let Some(user) = model.user_mut() {
                    user.set_consent(None);
                }
            }
            Self::ConsentDotted => {
                if let Some(user) = model.user_mut() {
                    user.set_consent(Some("CPX.000"));
                }
            }
            Self::BannerNegativeSize => {
                if let Some(banner) = first_banner(model) {
                    banner.set_w(Some(-1_i32));
                }
            }
            Self::BannerMissingSize => {
                if let Some(banner) = first_banner(model) {
                    banner.set_w(None);
                    banner.set_h(None);
                }
            }
            Self::DeepExt => {
                model.set_ext(Some(RawJson::from_static(r#"{"a":{"b":{"c":{"d":{"e":1}}}}}"#)));
            }
            Self::CurrenciesAsString => model.set_cur(Currencies::Text("USD")),
            Self::CurrenciesInvalidCode => {
                model.set_cur(Currencies::Raw(RawJson::from_static(r#"["US1"]"#)));
            }
            Self::GeoCountryAlpha2 => {
                if let Some(geo) = first_geo(model) {
                    geo.set_country(Some("RU"));
                }
            }
            Self::GeoCityName => {
                if let Some(geo) = first_geo(model) {
                    geo.set_city(Some("Moscow"));
                }
            }
            Self::GeoRegionWithoutCountry => {
                if let Some(geo) = first_geo(model) {
                    geo.set_region(Some("MOW"));
                }
            }
        }
    }
}

impl fmt::Display for Mutation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Draws at most one mutation and applies it.
///
/// The decision is drawn after the body, so a request without a mutation is
/// byte-identical to the same request generated with [`DegradeRate::OFF`].
#[expect(
    clippy::redundant_pub_crate,
    reason = "the module is private; the callers are inside the crate"
)]
pub(crate) fn apply(
    model: &mut RequestModel,
    rate: DegradeRate,
    spec: SpecVersion,
    entropy: &mut impl Entropy,
) -> Option<Mutation> {
    if !entropy.bit(rate.share()) {
        return None;
    }

    let total = Mutation::ALL.iter().filter(|mutation| mutation.applies(model, spec)).count();
    let total = u32::try_from(total).ok()?;
    if total == 0 {
        return None;
    }

    let index = entropy.u32(Span::from_zero(total.saturating_sub(1)));
    let index = usize::try_from(index).ok()?;
    let mutation = Mutation::ALL
        .iter()
        .copied()
        .filter(|mutation| mutation.applies(model, spec))
        .nth(index)?;
    mutation.apply_to(model, entropy);

    Some(mutation)
}

/// Returns whether the request has a banner to break.
fn has_banner(model: &RequestModel) -> bool {
    model.impressions().iter().any(|imp| imp.banner().is_some())
}

/// Returns the banner of the first impression that has one.
fn first_banner(model: &mut RequestModel) -> Option<&mut crate::wire::Banner> {
    model.impressions_mut().iter_mut().find_map(Imp::banner_mut)
}

/// Returns the device geo location, when the request has one.
fn first_geo(model: &mut RequestModel) -> Option<&mut crate::wire::Geo> {
    model.device_mut()?.geo_mut()
}

/// Returns how many base impressions the request carries.
fn base_count(model: &RequestModel) -> u16 {
    u16::from(model.impressions().count())
}

/// Puts an invalid currency code on the first impression.
const fn set_floor_currency(model: &mut RequestModel, code: &'static str) {
    if let Some(imp) = model.impressions_mut().first_mut() {
        imp.set_bidfloorcur(Some(WireCurrency::Text(code)));
    }
}

/// Adds whichever supply source the request does not have.
const fn add_both_supply_sources(model: &mut RequestModel) {
    let has_site = model.site().is_some();
    let has_app = model.app().is_some();

    if has_site && has_app {
        return;
    }
    if !has_site {
        model.set_site(Some(Site::minimal()));
    }
    if !has_app {
        model.set_app(Some(App::minimal()));
    }
}

/// Makes every `imp[].id` the same.
fn duplicate_imp_ids(model: &mut RequestModel) {
    let Some(first) = model.impressions().iter().next().map(Imp::id) else {
        return;
    };

    if model.impressions().count() < 2 {
        model.impressions_mut().set_extra(1);
        return;
    }

    for imp in model.impressions_mut().iter_mut() {
        imp.set_id(first);
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "in tests a panic = a failed test")]
mod tests {
    extern crate std;

    use std::string::String;

    use super::{DegradeRate, Mutation, apply};
    use crate::generate::{self, Config};
    use crate::ids::RequestId;
    use crate::profile::Profile;
    use crate::rng::RngEntropy;
    use crate::seed::Seed;
    use crate::version::SpecVersion;
    use crate::wire::RequestModel;
    use crate::write;

    fn render(model: &RequestModel, spec: SpecVersion) -> String {
        let mut out = String::new();
        write::request(&mut out, model, spec).expect("a String sink cannot fail");
        out
    }

    fn maximal() -> RequestModel {
        RequestModel::maximal(RequestId::from_seed(Seed::new(42)))
    }

    fn value(model: &RequestModel, spec: SpecVersion) -> serde_json::Value {
        serde_json::from_str(&render(model, spec)).expect("a request is valid JSON")
    }

    #[test]
    fn rates_are_basis_points_and_saturate() {
        assert_eq!(DegradeRate::percent(0).basis_points(), 0);
        assert_eq!(DegradeRate::percent(5).basis_points(), 500);
        assert_eq!(DegradeRate::percent(100).basis_points(), 10_000);
        assert_eq!(DegradeRate::percent(200).basis_points(), 10_000);
        assert!(DegradeRate::from_basis_points(10_001).is_none());
    }

    #[test]
    fn every_mutation_applies_to_the_maximal_model() {
        let mut entropy = RngEntropy::from_seed(Seed::new(7));

        for mutation in Mutation::ALL {
            let mut model = maximal();
            mutation.apply_to(&mut model, &mut entropy);
            let value = value(&model, SpecVersion::V26);

            assert!(value.is_object(), "{mutation} broke JSON");
        }
    }

    #[test]
    fn an_oversized_body_repeats_the_last_impression() {
        let mut entropy = RngEntropy::from_seed(Seed::new(11));
        let mut model = maximal();
        Mutation::HugeImpressions.apply_to(&mut model, &mut entropy);
        let value = value(&model, SpecVersion::V26);

        let impressions = value.pointer("/imp").and_then(serde_json::Value::as_array);
        assert_eq!(impressions.map(std::vec::Vec::len), Some(10_000));
    }

    #[test]
    fn a_mutation_changes_exactly_its_own_field() {
        let cases: &[(Mutation, &str)] = &[
            (Mutation::AuctionTypeZero, "/at"),
            (Mutation::BidFloorAsString, "/imp/0/bidfloor"),
            (Mutation::BidFloorCurrencyLowercase, "/imp/0/bidfloorcur"),
            (Mutation::DeviceIpv6, "/device/ip"),
            (Mutation::RequestIdEmpty, "/id"),
            (Mutation::TmaxHigh, "/tmax"),
            (Mutation::ConsentDotted, "/user/consent"),
            (Mutation::BannerNegativeSize, "/imp/0/banner/w"),
            (Mutation::DeepExt, "/ext"),
            (Mutation::CurrenciesAsString, "/cur"),
            (Mutation::GeoCountryAlpha2, "/device/geo/country"),
            (Mutation::GeoCityName, "/device/geo/city"),
            (Mutation::GeoRegionWithoutCountry, "/device/geo/region"),
        ];

        let mut entropy = RngEntropy::from_seed(Seed::new(3));
        for (mutation, path) in cases {
            let before = value(&maximal(), SpecVersion::V26);
            let mut model = maximal();
            mutation.apply_to(&mut model, &mut entropy);
            let after = value(&model, SpecVersion::V26);

            assert_ne!(
                before.pointer(path),
                after.pointer(path),
                "{mutation} did not change {path}"
            );
        }
    }

    #[test]
    fn geo_mutations_emit_the_invalid_legacy_formats() {
        let mut entropy = RngEntropy::from_seed(Seed::new(5));
        let cases = [
            (Mutation::GeoCountryAlpha2, "/device/geo/country", "RU"),
            (Mutation::GeoCityName, "/device/geo/city", "Moscow"),
            (Mutation::GeoRegionWithoutCountry, "/device/geo/region", "MOW"),
        ];

        for (mutation, path, expected) in cases {
            let mut model = maximal();
            mutation.apply_to(&mut model, &mut entropy);
            let value = value(&model, SpecVersion::V26);

            assert_eq!(
                value.pointer(path).and_then(serde_json::Value::as_str),
                Some(expected),
                "{mutation} did not write the legacy format at {path}"
            );
        }
    }

    #[test]
    fn a_zero_rate_keeps_the_bytes() {
        for profile in Profile::ALL {
            let plain = generate::build(
                profile,
                Seed::new(42),
                &Config::new(RequestId::from_seed(Seed::new(42)), None, None, None),
            );
            let degraded = generate::build(
                profile,
                Seed::new(42),
                &Config::new(RequestId::from_seed(Seed::new(42)), None, None, None)
                    .degrade(DegradeRate::percent(0)),
            );

            assert_eq!(
                render(plain.model(), SpecVersion::V26),
                render(degraded.model(), SpecVersion::V26),
                "{profile} changed with a zero rate"
            );
            assert!(degraded.mutation().is_none(), "{profile} mutated at a zero rate");
        }
    }

    fn assert_full_rate_mutates(profile: Profile, spec: SpecVersion) {
        for seed in [0_u64, 1, 42, u64::MAX] {
            let config = Config::new(RequestId::from_seed(Seed::new(seed)), None, None, None)
                .degrade(DegradeRate::MAX)
                .spec(spec);
            let generated = generate::build(profile, Seed::new(seed), &config);

            assert!(generated.mutation().is_some(), "{profile} {spec} seed {seed} did not mutate");
            assert!(value(generated.model(), spec).is_object(), "{profile} emitted invalid JSON");
        }
    }

    #[test]
    fn a_full_rate_always_mutates() {
        for profile in Profile::ALL {
            for spec in SpecVersion::ALL {
                assert_full_rate_mutates(profile, spec);
            }
        }
    }

    fn assert_full_rate_changes(profile: Profile, spec: SpecVersion) {
        for seed in 0_u64..64 {
            let plain =
                Config::new(RequestId::from_seed(Seed::new(seed)), None, None, None).spec(spec);
            let degraded = plain.degrade(DegradeRate::MAX);
            let before = generate::build(profile, Seed::new(seed), &plain);
            let after = generate::build(profile, Seed::new(seed), &degraded);

            assert_ne!(
                render(before.model(), spec),
                render(after.model(), spec),
                "{profile} {spec} seed {seed} selected a no-op mutation"
            );
        }
    }

    #[test]
    fn a_full_rate_changes_the_bytes() {
        for profile in Profile::ALL {
            for spec in SpecVersion::ALL {
                assert_full_rate_changes(profile, spec);
            }
        }
    }

    #[test]
    fn mutations_that_need_a_field_are_not_selected_without_it() {
        let mut model = maximal();

        assert!(Mutation::TmaxMissing.applies(&model, SpecVersion::V26));
        model.set_tmax(None);
        assert!(!Mutation::TmaxMissing.applies(&model, SpecVersion::V26));

        assert!(Mutation::ConsentMissing.applies(&model, SpecVersion::V26));
        if let Some(user) = model.user_mut() {
            user.set_consent(None);
        }
        assert!(!Mutation::ConsentMissing.applies(&model, SpecVersion::V26));
        assert!(Mutation::ConsentDotted.applies(&model, SpecVersion::V26));
    }

    #[test]
    fn every_mutation_is_reachable() {
        let mut seen = std::vec::Vec::new();

        for seed in 0_u64..2_000 {
            let config = Config::new(RequestId::from_seed(Seed::new(seed)), None, None, None)
                .degrade(DegradeRate::MAX)
                .spec(SpecVersion::V26);
            let generated = generate::build(Profile::Maximal, Seed::new(seed), &config);

            if let Some(mutation) = generated.mutation()
                && !seen.contains(&mutation)
            {
                seen.push(mutation);
            }
        }

        for mutation in Mutation::ALL {
            assert!(seen.contains(mutation), "{mutation} is unreachable");
        }
    }

    #[test]
    fn a_selection_is_deterministic() {
        let config = Config::new(RequestId::from_seed(Seed::new(9)), None, None, None)
            .degrade(DegradeRate::MAX);
        let first = generate::build(Profile::SiteDisplay, Seed::new(9), &config);
        let second = generate::build(Profile::SiteDisplay, Seed::new(9), &config);

        assert_eq!(first.mutation(), second.mutation());
        assert_eq!(
            render(first.model(), SpecVersion::V26),
            render(second.model(), SpecVersion::V26)
        );
    }

    #[test]
    fn apply_is_none_before_the_draw() {
        let mut model = maximal();
        let mut entropy = RngEntropy::from_seed(Seed::new(1));
        let result = apply(&mut model, DegradeRate::OFF, SpecVersion::V26, &mut entropy);

        assert!(result.is_none());
    }
}
