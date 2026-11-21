//! The distributions shared by every profile.
//!
//! A profile fixes which fields may appear and with what probability; the seed
//! chooses the instance. The helpers here draw in a fixed order so that the
//! profiles stay byte-stable when they are refactored.

use crate::code::{AuctionType, Linearity, LocationType, Placement, PlacementPosition};
use crate::currency::Currency;
use crate::flag::Flag;
use crate::ids::{AppId, BannerId, BuyerUserId, ImpId, PublisherId, SiteId, UserId};
use crate::money::Money;
use crate::raw::RawJson;
use crate::rng::{Entropy, Share, Span};
use crate::wire::{
    App, Banner, BannerList, Currencies, Device, Geo, Imp, ImpList, Native, Publisher, Regs,
    RequestModel, Site, User, Video, WireCurrency, WireId, WireNumber,
};

use super::Config;
use super::data::{
    APP_DEVICES, APP_EXT, APP_VERSIONS, APPS, AT, BADV, BANNER_API, BANNER_BATTR, BANNER_BTYPE,
    BANNER_EXPDIR, BANNER_MIMES, BANNER_POS, BANNER_TOPTRAME, BAPP, BCAT, COMPANION_TYPES, CUR_USD,
    DEFAULT_APP, DEFAULT_APP_DEVICE, DEFAULT_DEVICE, DEFAULT_GEO, DEFAULT_IFA,
    DEFAULT_NATIVE_REQUEST, DEFAULT_SITE, DEFAULT_VIDEO_FORMAT, DEVICES, DeviceProfile, FLOORS,
    GPP_SID, GeoProfile, IFAS, IPS, NATIVE_EXT, NATIVE_REQUESTS, NATIVE_VERSIONS, PUBLISHER_CAT,
    PUBLISHER_NAMES, REGS_EXT, SITE_CAT, SITE_EXT, SITES, SIZES, TMAX, USER_EXT, VIDEO_DELIVERY,
    VIDEO_DURATIONS, VIDEO_FORMATS, VIDEO_PLACEMENTS, VIDEO_SIZES,
};
use super::privacy::Scenario;

/// The media object a profile puts into an impression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Media {
    /// A display banner.
    Banner,
    /// A video ad.
    Video,
    /// A native ad.
    Native,
}

/// Picks one item, or the fallback for an empty slice.
pub(super) fn choose<T: Copy>(rng: &mut impl Entropy, values: &[T], fallback: T) -> T {
    rng.pick(values).copied().unwrap_or(fallback)
}

/// Returns the resolved impression count of a profile.
pub(super) fn impression_count(config: &Config, default: u8) -> u8 {
    config.impressions.map_or(default, crate::impressions::ImpressionCount::count)
}

/// Returns the constant site profile the request was generated for.
pub(super) fn site_profile(
    rng: &mut impl Entropy,
) -> (&'static str, &'static str, &'static str, &'static str) {
    choose(rng, SITES, DEFAULT_SITE)
}

/// Returns the publisher name the request was generated for.
pub(super) fn publisher_name(rng: &mut impl Entropy) -> &'static str {
    choose(rng, PUBLISHER_NAMES, "Example Media")
}

fn build_publisher(rng: &mut impl Entropy, name: &'static str, domain: &'static str) -> Publisher {
    let mut publisher = Publisher::default();
    publisher.set_id(Some(PublisherId::new(u64::from(rng.u32(Span::from_zero(999))))));
    publisher.set_name(Some(name));
    publisher.set_domain(Some(domain));
    publisher.set_cat(PUBLISHER_CAT);
    publisher
}

/// Builds a website with its publisher.
pub(super) fn build_site(
    rng: &mut impl Entropy,
    site_profile: (&'static str, &'static str, &'static str, &'static str),
    publisher_name: &'static str,
) -> Site {
    let (domain, site_name, page, keywords) = site_profile;
    let publisher = build_publisher(rng, publisher_name, domain);

    let mut site = Site::default();
    site.set_id(Some(SiteId::new(u64::from(rng.u32(Span::from_zero(999))))));
    site.set_name(Some(site_name));
    site.set_domain(Some(domain));
    site.set_cat(SITE_CAT);
    site.set_page(Some(page));
    site.set_ref_page(rng.bit(Share::percent(30)).then_some("https://search.example/"));
    site.set_mobile(Some(choose(rng, &[Flag::No, Flag::Yes], Flag::No)));
    site.set_privacypolicy(Some(Flag::Yes));
    site.set_publisher(Some(publisher));
    site.set_keywords(Some(keywords));
    site.set_ext(rng.bit(Share::percent(30)).then_some(SITE_EXT));
    site
}

/// Builds an application with its publisher.
pub(super) fn build_app(rng: &mut impl Entropy) -> App {
    let (bundle, name, domain, storeurl, keywords) = choose(rng, APPS, DEFAULT_APP);
    let publisher_name = choose(rng, PUBLISHER_NAMES, "Example Media");
    let publisher = build_publisher(rng, publisher_name, domain);

    let mut app = App::default();
    app.set_id(Some(AppId::new(u64::from(rng.u32(Span::from_zero(999))))));
    app.set_name(Some(name));
    app.set_bundle(Some(bundle));
    app.set_storeurl(Some(storeurl));
    app.set_domain(Some(domain));
    app.set_cat(SITE_CAT);
    app.set_ver(Some(choose(rng, APP_VERSIONS, "1.2.3")));
    app.set_publisher(Some(publisher));
    app.set_keywords(Some(keywords));
    app.set_ext(rng.bit(Share::percent(30)).then_some(APP_EXT));
    app
}

pub(super) fn build_geo(location: &GeoProfile, precise: bool) -> Geo {
    let mut geo = Geo::default();
    geo.set_country(Some(location.country));

    if precise {
        geo.set_region(Some(location.region));
        geo.set_city(Some(location.city));
        geo.set_zip(Some(location.zip));
        geo.set_type_code(Some(LocationType::IP_ADDRESS));
    }

    geo.set_utcoffset(Some(location.utcoffset));
    geo
}

/// The parts of the device policy a scenario controls.
#[derive(Clone, Copy, Debug)]
struct DevicePolicy {
    always_ifa: bool,
    privacy: Scenario,
    location: GeoProfile,
}

fn build_device_from(
    rng: &mut impl Entropy,
    profiles: &[DeviceProfile],
    fallback: &DeviceProfile,
    policy: &DevicePolicy,
) -> Device {
    let (ua, make, model, os, osv, device_type) = choose(rng, profiles, *fallback);

    let mut device = Device::default();
    device.set_ua(Some(ua));

    if policy.privacy.allows_ip() {
        device.set_ip(Some(choose(rng, IPS, "203.0.113.7")));
    }

    device.set_geo(Some(build_geo(&policy.location, policy.privacy.precise_geo())));

    let has_ifa =
        policy.privacy.allows_identifiers() && (policy.always_ifa || rng.bit(Share::percent(60)));
    device.set_ifa(has_ifa.then(|| choose(rng, IFAS, DEFAULT_IFA)));

    device.set_dnt(rng.bit(Share::percent(30)).then_some(Flag::No));
    device.set_lmt(rng.bit(Share::percent(30)).then_some(Flag::No));
    device.set_devicetype(Some(device_type));
    device.set_make(Some(make));
    device.set_model(Some(model));
    device.set_os(Some(os));
    device.set_osv(Some(osv));

    let language = if policy.privacy.allows_identifiers() && !rng.bit(Share::percent(30)) {
        policy.location.language
    } else {
        "en"
    };
    device.set_language(Some(language));

    device
}

/// Builds a desktop or laptop device.
pub(super) fn build_device(
    rng: &mut impl Entropy,
    privacy: Scenario,
    location: &GeoProfile,
) -> Device {
    let policy = DevicePolicy { always_ifa: false, privacy, location: *location };
    build_device_from(rng, DEVICES, &DEFAULT_DEVICE, &policy)
}

/// Builds a phone device, the realistic `device` of an in-app request.
pub(super) fn build_app_device(
    rng: &mut impl Entropy,
    privacy: Scenario,
    location: &GeoProfile,
) -> Device {
    let policy = DevicePolicy { always_ifa: true, privacy, location: *location };
    build_device_from(rng, APP_DEVICES, &DEFAULT_APP_DEVICE, &policy)
}

/// Builds the user, or nothing for the requests that carry no user.
pub(super) fn build_user(rng: &mut impl Entropy, privacy: Scenario) -> Option<User> {
    let consent = privacy.consent();
    let identifiers = privacy.allows_identifiers();

    if !identifiers && consent.is_none() {
        return None;
    }
    if identifiers && consent.is_none() && !rng.bit(Share::percent(70)) {
        return None;
    }

    let mut user = User::default();

    if identifiers {
        user.set_id(Some(UserId::new(u64::from(rng.u32(Span::from_zero(999_999))))));
        user.set_buyeruid(Some(BuyerUserId::new(u64::from(rng.u32(Span::from_zero(999_999))))));
        user.set_ext(rng.bit(Share::percent(30)).then_some(USER_EXT));
    }

    user.set_consent(consent);
    user.set_legacy_consent(consent);
    Some(user)
}

/// Builds the privacy regulations, or nothing for the requests without them.
pub(super) fn build_regs(privacy: Scenario, country: &str) -> Option<Regs> {
    if !privacy.has_regs() {
        return None;
    }

    let mut regs = Regs::default();
    regs.set_coppa(Some(privacy.coppa()));

    if let Some(gdpr) = privacy.gdpr() {
        regs.set_gdpr(Some(gdpr));
        regs.set_legacy_gdpr(Some(gdpr));
    }

    if privacy.consent().is_some() {
        let gpp = privacy.gpp();
        regs.set_gpp(gpp);
        regs.set_gpp_sid(if gpp.is_some() { GPP_SID } else { &[] });
    }

    if privacy.has_us_privacy(country) {
        regs.set_ext(Some(REGS_EXT));
        regs.set_legacy_us_privacy(Some("1YNN"));
    }

    Some(regs)
}

fn build_banner(rng: &mut impl Entropy) -> Banner {
    let (width, height) = choose(rng, SIZES, (300_i32, 250_i32));

    let mut banner = Banner::default();
    banner.set_w(Some(width));
    banner.set_h(Some(height));
    banner.set_btype(if rng.bit(Share::percent(40)) { BANNER_BTYPE } else { &[] });
    banner.set_battr(if rng.bit(Share::percent(30)) { BANNER_BATTR } else { &[] });
    banner.set_pos(Some(choose(rng, BANNER_POS, PlacementPosition::ABOVE_THE_FOLD)));
    banner.set_mimes(BANNER_MIMES);
    banner.set_topframe(Some(choose(rng, BANNER_TOPTRAME, Flag::Yes)));
    banner.set_expdir(if rng.bit(Share::percent(20)) { BANNER_EXPDIR } else { &[] });
    banner.set_api(if rng.bit(Share::percent(50)) { BANNER_API } else { &[] });
    banner.set_id(
        rng.bit(Share::percent(50))
            .then(|| BannerId::new(u64::from(rng.u32(Span::from_zero(999))))),
    );
    banner
}

fn build_video(rng: &mut impl Entropy) -> Video {
    let (mimes, protocols) = choose(rng, VIDEO_FORMATS, DEFAULT_VIDEO_FORMAT);
    let (width, height) = choose(rng, VIDEO_SIZES, (640, 480));
    let (minimum, maximum) = choose(rng, VIDEO_DURATIONS, (5, 15));

    let mut video = Video::default();
    video.set_mimes(mimes);
    video.set_minduration(Some(minimum));
    video.set_maxduration(Some(maximum));
    video.set_protocols(protocols);
    video.set_w(Some(width));
    video.set_h(Some(height));
    video.set_startdelay(Some(-1_i32));
    video.set_placement(Some(choose(rng, VIDEO_PLACEMENTS, Placement::IN_STREAM)));
    video.set_linearity(Some(Linearity::LINEAR));
    let skip = rng.bit(Share::percent(50)).then_some(Flag::Yes);
    video.set_skip(skip);
    video.set_skipmin(skip.map(|_| 5));
    video.set_skipafter(skip.map(|_| 5));
    video.set_api(if rng.bit(Share::percent(50)) { BANNER_API } else { &[] });
    video.set_battr(if rng.bit(Share::percent(30)) { BANNER_BATTR } else { &[] });
    video.set_delivery(VIDEO_DELIVERY);
    video.set_pos(Some(choose(rng, BANNER_POS, PlacementPosition::ABOVE_THE_FOLD)));
    if rng.bit(Share::percent(30)) {
        let mut companion = BannerList::default();
        companion.push(&build_banner(rng));
        video.set_companionad(companion);
        video.set_companiontype(COMPANION_TYPES);
    }
    video
}

fn build_native(rng: &mut impl Entropy) -> Native {
    let mut native = Native::default();
    native.set_request(choose(rng, NATIVE_REQUESTS, DEFAULT_NATIVE_REQUEST));
    native.set_ver(choose(rng, NATIVE_VERSIONS, "1.2"));
    native.set_api(if rng.bit(Share::percent(50)) { BANNER_API } else { &[] });
    native.set_battr(if rng.bit(Share::percent(30)) { BANNER_BATTR } else { &[] });
    native.set_ext(rng.bit(Share::percent(20)).then_some(NATIVE_EXT));
    native
}

fn build_imp(rng: &mut impl Entropy, media: Media, id: u32) -> Imp {
    let mut imp = Imp::default();
    imp.set_id(WireId::id(ImpId::new(id)));

    match media {
        Media::Banner => imp.set_banner(Some(build_banner(rng))),
        Media::Video => imp.set_video(Some(build_video(rng))),
        Media::Native => imp.set_native(Some(build_native(rng))),
    }

    let floor =
        rng.bit(Share::percent(60)).then(|| Money::from_scaled(choose(rng, FLOORS, 10_000)));
    imp.set_bidfloor(floor.map(WireNumber::number));
    imp.set_bidfloorcur(floor.map(|_| WireCurrency::code(Currency::USD)));
    imp.set_secure(rng.bit(Share::percent(85)).then_some(Flag::Yes));
    imp
}

/// Builds `count` impressions of one media type, ids starting at one.
pub(super) fn build_imp_list(rng: &mut impl Entropy, media: Media, count: u8) -> ImpList {
    let mut imps = ImpList::default();

    for index in 1..=count {
        imps.push(&build_imp(rng, media, u32::from(index)));
    }

    imps
}

/// Builds `count` impressions cycling through banner, video and native.
pub(super) fn build_mixed_imp_list(rng: &mut impl Entropy, count: u8) -> ImpList {
    let mut imps = ImpList::default();

    for index in 1..=count {
        let media = match index.checked_rem(3) {
            Some(1) => Media::Banner,
            Some(2) => Media::Video,
            _ => Media::Native,
        };
        imps.push(&build_imp(rng, media, u32::from(index)));
    }

    imps
}

fn build_at(rng: &mut impl Entropy) -> AuctionType {
    choose(rng, AT, AuctionType::SECOND_PRICE_PLUS)
}

fn build_tmax(rng: &mut impl Entropy) -> Option<u32> {
    rng.bit(Share::percent(70)).then(|| choose(rng, TMAX, 1_000))
}

fn build_cur(rng: &mut impl Entropy) -> &'static [Currency] {
    if rng.bit(Share::percent(70)) { CUR_USD } else { &[] }
}

fn build_bcat(rng: &mut impl Entropy) -> &'static [&'static str] {
    if rng.bit(Share::percent(40)) { BCAT } else { &[] }
}

fn build_badv(rng: &mut impl Entropy) -> &'static [&'static str] {
    if rng.bit(Share::percent(20)) { BADV } else { &[] }
}

fn build_bapp(rng: &mut impl Entropy) -> &'static [&'static str] {
    if rng.bit(Share::percent(20)) { BAPP } else { &[] }
}

fn build_request_ext(rng: &mut impl Entropy) -> Option<RawJson> {
    rng.bit(Share::percent(30)).then_some(SITE_EXT)
}

type SiteChoice = ((&'static str, &'static str, &'static str, &'static str), &'static str);

fn choose_site(rng: &mut impl Entropy) -> SiteChoice {
    (site_profile(rng), publisher_name(rng))
}

/// Builds the complete website request of a site profile with one media type.
pub(super) fn build_site_request(
    rng: &mut impl Entropy,
    config: &Config,
    media: Media,
) -> RequestModel {
    build_site_request_custom(rng, config, |rng| {
        build_imp_list(rng, media, impression_count(config, 1))
    })
}

/// Builds a website request with caller-chosen impressions.
///
/// The site profile and the publisher are drawn before the impressions, which
/// is the historical order of `SiteDisplay`; the callback receives the entropy
/// after those two draws.
pub(super) fn build_site_request_custom<E, F>(
    rng: &mut E,
    config: &Config,
    make_imps: F,
) -> RequestModel
where
    E: Entropy,
    F: FnOnce(&mut E) -> ImpList,
{
    let choice = choose_site(rng);
    let imps = make_imps(rng);
    build_site_tail(rng, config, &imps, &choice)
}

fn build_site_tail(
    rng: &mut impl Entropy,
    config: &Config,
    imps: &ImpList,
    choice: &SiteChoice,
) -> RequestModel {
    let (site_profile, publisher) = *choice;
    let site = build_site(rng, site_profile, publisher);
    let privacy = Scenario::draw(rng);
    let location = choose(rng, privacy.geos(), DEFAULT_GEO);
    let device = build_device(rng, privacy, &location);
    let user = build_user(rng, privacy);
    let regs = build_regs(privacy, location.country);
    let at = build_at(rng);
    let tmax = build_tmax(rng);
    let cur = build_cur(rng);
    let bcat = build_bcat(rng);
    let badv = build_badv(rng);
    let ext = build_request_ext(rng);

    let mut request = RequestModel::default();
    request.set_id(WireId::id(config.id()));
    request.set_imp(*imps);
    request.set_site(Some(site));
    request.set_device(Some(device));
    request.set_user(user);
    request.set_at(Some(at));
    request.set_tmax(tmax);
    request.set_cur(Currencies::codes(cur));
    request.set_bcat(bcat);
    request.set_badv(badv);
    request.set_regs(regs);
    request.set_ext(ext);
    request
}

/// Builds the complete in-app request of an app profile.
pub(super) fn build_app_request(
    rng: &mut impl Entropy,
    config: &Config,
    media: Media,
    rewarded: bool,
) -> RequestModel {
    let mut imps = build_imp_list(rng, media, impression_count(config, 1));
    if rewarded {
        for imp in imps.iter_mut() {
            imp.set_secure(Some(Flag::Yes));
        }
    }
    let app = build_app(rng);
    let privacy = Scenario::draw(rng);
    let location = choose(rng, privacy.geos(), DEFAULT_GEO);
    let device = build_app_device(rng, privacy, &location);
    let user = build_user(rng, privacy);
    let regs = build_regs(privacy, location.country);
    let at = build_at(rng);
    let tmax = build_tmax(rng);
    let cur = build_cur(rng);
    let bcat = build_bcat(rng);
    let badv = build_badv(rng);
    let bapp = build_bapp(rng);
    let ext = build_request_ext(rng);

    let mut request = RequestModel::default();
    request.set_id(WireId::id(config.id()));
    request.set_imp(imps);
    request.set_app(Some(app));
    request.set_device(Some(device));
    request.set_user(user);
    request.set_at(Some(at));
    request.set_tmax(tmax);
    request.set_cur(Currencies::codes(cur));
    request.set_bcat(bcat);
    request.set_badv(badv);
    request.set_bapp(bapp);
    request.set_regs(regs);
    request.set_ext(ext);
    request
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::{GeoProfile, Media, build_imp_list, build_mixed_imp_list, choose};
    use crate::generate::data::{COPPA_GEOS, EU_GEOS, NON_EU_GEOS};
    use crate::rng::RngEntropy;
    use crate::seed::Seed;
    use crate::wire::WireId;

    #[test]
    fn an_imp_list_has_consecutive_ids() {
        let mut rng = RngEntropy::from_seed(Seed::new(42));
        let imps = build_imp_list(&mut rng, Media::Banner, 3);
        let ids: std::vec::Vec<u64> = imps
            .iter()
            .filter_map(|imp| match imp.id() {
                WireId::Id(id) => Some(id.into_inner()),
                WireId::Literal(_) => None,
            })
            .collect();

        assert_eq!(ids, [1, 2, 3]);
    }

    #[test]
    fn a_mixed_list_cycles_through_three_media_types() {
        let mut rng = RngEntropy::from_seed(Seed::new(7));
        let imps = build_mixed_imp_list(&mut rng, 3);

        assert_eq!(imps.count(), 3);
    }

    #[test]
    fn choose_falls_back_on_an_empty_slice() {
        let mut rng = RngEntropy::from_seed(Seed::new(0));
        let picked: u8 = choose(&mut rng, &[], 9);

        assert_eq!(picked, 9);
    }

    #[test]
    fn the_geo_tables_follow_the_spec_code_lists() {
        let mut locations = std::vec::Vec::new();
        locations.extend_from_slice(EU_GEOS);
        locations.extend_from_slice(NON_EU_GEOS);
        locations.extend_from_slice(COPPA_GEOS);

        for location in locations {
            check_geo(location);
        }
    }

    fn check_geo(GeoProfile { country, region, city, zip, utcoffset, language }: GeoProfile) {
        assert_eq!(country.len(), 3, "{country} is not an ISO-3166-1 alpha-3 code");
        assert!(
            country.bytes().all(|byte| byte.is_ascii_uppercase()),
            "{country} is not an ISO-3166-1 alpha-3 code"
        );

        assert_eq!(city.len(), 5, "{city} is not a UN/LOCODE");
        assert!(
            city.bytes().skip(2).all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit()),
            "{city} is not a UN/LOCODE"
        );

        if country == "USA" {
            // OpenRTB asks for the 2-letter state code for the USA, not the full ISO-3166-2 code.
            assert_eq!(region.len(), 2, "{region} is not a 2-letter US state code");
            assert!(
                region.bytes().all(|byte| byte.is_ascii_uppercase()),
                "{region} is not a 2-letter US state code"
            );
            assert_eq!(city.get(..2), Some("US"), "{city} is not a US UN/LOCODE");
        } else {
            let mut parts = region.split('-');
            let prefix = parts.next();
            let subdivision = parts.next();
            assert!(parts.next().is_none(), "{region} has more than one hyphen");
            assert_eq!(prefix, city.get(..2), "{region} and {city} disagree on the country prefix");
            assert!(
                subdivision.is_some_and(|code| code
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())),
                "{region} is not an ISO-3166-2 code"
            );
        }

        assert!(!zip.is_empty(), "{country} has no postal code");
        assert!(
            language.len() == 2 && language.bytes().all(|byte| byte.is_ascii_lowercase()),
            "{country} has an invalid language"
        );
        assert!((-720..=840).contains(&utcoffset), "{country} has an implausible UTC offset");
    }
}
