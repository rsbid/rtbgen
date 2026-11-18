use crate::code::{
    ApiFramework, AuctionType, BannerType, CreativeAttribute, DeviceType, ExpandableDirection,
    GppSectionId, LocationType, PlacementPosition,
};
use crate::currency::Currency;
use crate::flag::Flag;
use crate::ids::{BannerId, BuyerUserId, ImpId, PublisherId, RequestId, SiteId, UserId};
use crate::money::Money;
use crate::raw::RawJson;
use crate::request::Request;
use crate::rng::{Entropy, Rng, Share};
use crate::seed::Seed;
use crate::version::SpecVersion;
use crate::wire::{Banner, Device, Geo, Imp, ImpList, Publisher, Regs, RequestModel, Site, User};

const DEFAULT_SITE: (&str, &str, &str, &str) = (
    "example.com",
    "Example News",
    "https://example.com/news/42",
    "news,sports",
);
const SITES: &[(&str, &str, &str, &str)] = &[
    DEFAULT_SITE,
    (
        "news.example",
        "Example Daily",
        "https://news.example/weather/today",
        "weather,local",
    ),
    (
        "shop.example",
        "Example Shop",
        "https://shop.example/item/7",
        "deals,shopping",
    ),
    (
        "publisher.test",
        "Example Sports",
        "https://publisher.test/scores/42",
        "sports,scores",
    ),
];
const PUBLISHER_NAMES: &[&str] = &["Example Media", "Example Publishing"];
const SITE_CAT: &[&str] = &["IAB1"];
const PUBLISHER_CAT: &[&str] = &["IAB1", "IAB2"];
const BCAT: &[&str] = &["IAB1", "IAB2"];
const BADV: &[&str] = &["bad.example", "spam.test"];

const DEFAULT_DEVICE: (&str, &str, &str, &str, &str, DeviceType) = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36",
    "Dell",
    "XPS 13",
    "Windows",
    "11",
    DeviceType::PERSONAL_COMPUTER,
);
const DEVICES: &[(&str, &str, &str, &str, &str, DeviceType)] = &[
    DEFAULT_DEVICE,
    (
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_6) AppleWebKit/605.1.15 Version/18.1 Safari/605.1.15",
        "Apple",
        "MacBookPro18,1",
        "macOS",
        "14.6",
        DeviceType::PERSONAL_COMPUTER,
    ),
    (
        "Mozilla/5.0 (Linux; Android 15; Pixel 8) AppleWebKit/537.36 Chrome/131.0.0.0 Mobile Safari/537.36",
        "Google",
        "Pixel 8",
        "Android",
        "15",
        DeviceType::PHONE,
    ),
];
const IPS: &[&str] = &["203.0.113.7", "198.51.100.23", "192.0.2.44"];
const DEFAULT_IDFA: &str = "A1B2C3D4-0000-4000-8000-000000000001";
const IDFAS: &[&str] = &[DEFAULT_IDFA, "A1B2C3D4-0000-4000-8000-000000000002"];
const GEOS: &[(&str, &str, &str, &str)] = &[
    ("RU", "MOW", "Moscow", "101000"),
    ("US", "NY", "New York", "10001"),
    ("DE", "BE", "Berlin", "10115"),
    ("GB", "ENG", "London", "EC1A"),
];
const LANGUAGES: &[&str] = &["en", "ru", "de"];
const DEFAULT_CONSENT: &str = "CPX0000000000000000000000000000000000000000000000000000000000000";
const CONSENTS: &[&str] = &[
    DEFAULT_CONSENT,
    "CPX1111111111111111111111111111111111111111111111111111111111111",
];
const GPP: &str = "DBABMA~CPX0000000000000000000000000000000000000000000000000000000000000";
const GPP_SID: &[GppSectionId] = &[GppSectionId::TCF_EU_V2];

const BANNER_MIMES: &[&str] = &["image/jpeg", "image/png"];
const BANNER_BTYPE: &[BannerType] = &[BannerType::IFRAME];
const BANNER_BATTR: &[CreativeAttribute] = &[
    CreativeAttribute::AUDIO_AUTOPLAY,
    CreativeAttribute::AUDIO_USER_INITIATED,
];
const BANNER_EXPDIR: &[ExpandableDirection] = &[ExpandableDirection::LEFT, ExpandableDirection::UP];
const BANNER_API: &[ApiFramework] = &[ApiFramework::MRAID_1, ApiFramework::MRAID_2];
const BANNER_POS: &[PlacementPosition] = &[
    PlacementPosition::ABOVE_THE_FOLD,
    PlacementPosition::LOCKED,
    PlacementPosition::BELOW_THE_FOLD,
];
const BANNER_TOPTRAME: &[Flag] = &[Flag::No, Flag::Yes];
const SIZES: &[(u32, u32)] = &[(300, 250), (320, 50), (728, 90), (300, 600), (970, 250)];
const FLOORS: &[i64] = &[5_000, 10_000, 15_000, 20_000];
const AT: &[AuctionType] = &[AuctionType::FIRST_PRICE, AuctionType::SECOND_PRICE_PLUS];
const TMAX: &[u32] = &[800, 1_000, 1_200, 1_500];
const CUR_USD: &[Currency] = &[Currency::USD];
const SITE_EXT: RawJson = RawJson::from_static(r#"{"exchange":"rtbgen"}"#);
const USER_EXT: RawJson = RawJson::from_static(r#"{"eids":[]}"#);
const REGS_EXT: RawJson = RawJson::from_static(r#"{"us_privacy":"1YNN"}"#);

fn choose<T: Copy>(rng: &mut impl Entropy, values: &[T], fallback: T) -> T {
    rng.pick(values).copied().unwrap_or(fallback)
}

pub(crate) fn site_display(seed: Seed) -> Request {
    let mut rng = Rng::from_seed(seed);
    let model = build_request(&mut rng, seed);
    Request::from_model(SpecVersion::V26, &model)
}

fn build_request(rng: &mut impl Entropy, seed: Seed) -> RequestModel {
    let (domain, site_name, page, keywords) = choose(rng, SITES, DEFAULT_SITE);
    let publisher_name = choose(rng, PUBLISHER_NAMES, "Example Media");
    let imp = build_imp(rng);
    let site = build_site(rng, (domain, site_name, page, keywords), publisher_name);
    let device = build_device(rng);
    let user = build_user(rng);
    let regs = build_regs(rng);

    let mut request = RequestModel::default();
    request.set_id(RequestId::from_seed(seed));
    request.set_imp(imp);
    request.set_site(Some(site));
    request.set_device(Some(device));
    request.set_user(user);
    request.set_at(Some(choose(rng, AT, AuctionType::SECOND_PRICE_PLUS)));
    request.set_tmax(
        rng.bit(Share::percent(70))
            .then(|| choose(rng, TMAX, 1_000)),
    );
    request.set_cur(if rng.bit(Share::percent(70)) {
        CUR_USD
    } else {
        &[]
    });
    request.set_bcat(if rng.bit(Share::percent(40)) {
        BCAT
    } else {
        &[]
    });
    request.set_badv(if rng.bit(Share::percent(20)) {
        BADV
    } else {
        &[]
    });
    request.set_regs(regs);
    request.set_ext(rng.bit(Share::percent(30)).then_some(SITE_EXT));
    request
}

fn build_imp(rng: &mut impl Entropy) -> ImpList {
    let banner = build_banner(rng);
    let floor = rng
        .bit(Share::percent(60))
        .then(|| Money::from_scaled(choose(rng, FLOORS, 10_000)));

    let mut imp = Imp::default();
    imp.set_id(ImpId::new(1));
    imp.set_banner(Some(banner));
    imp.set_bidfloor(floor);
    imp.set_bidfloorcur(floor.map(|_| Currency::USD));
    imp.set_secure(rng.bit(Share::percent(85)).then_some(Flag::Yes));

    let mut imps = ImpList::default();
    imps.push(&imp);
    imps
}

fn build_banner(rng: &mut impl Entropy) -> Banner {
    let (width, height) = choose(rng, SIZES, (300, 250));

    let mut banner = Banner::default();
    banner.set_w(Some(width));
    banner.set_h(Some(height));
    banner.set_btype(if rng.bit(Share::percent(40)) {
        BANNER_BTYPE
    } else {
        &[]
    });
    banner.set_battr(if rng.bit(Share::percent(30)) {
        BANNER_BATTR
    } else {
        &[]
    });
    banner.set_pos(Some(choose(
        rng,
        BANNER_POS,
        PlacementPosition::ABOVE_THE_FOLD,
    )));
    banner.set_mimes(BANNER_MIMES);
    banner.set_topframe(Some(choose(rng, BANNER_TOPTRAME, Flag::Yes)));
    banner.set_expdir(if rng.bit(Share::percent(20)) {
        BANNER_EXPDIR
    } else {
        &[]
    });
    banner.set_api(if rng.bit(Share::percent(50)) {
        BANNER_API
    } else {
        &[]
    });
    banner.set_id(
        rng.bit(Share::percent(50))
            .then(|| BannerId::new(u64::from(rng.below(1_000)))),
    );
    banner
}

fn build_site(
    rng: &mut impl Entropy,
    site_profile: (&'static str, &'static str, &'static str, &'static str),
    publisher_name: &'static str,
) -> Site {
    let (domain, site_name, page, keywords) = site_profile;

    let mut publisher = Publisher::default();
    publisher.set_id(Some(PublisherId::new(u64::from(rng.below(1_000)))));
    publisher.set_name(Some(publisher_name));
    publisher.set_domain(Some(domain));
    publisher.set_cat(PUBLISHER_CAT);

    let mut site = Site::default();
    site.set_id(Some(SiteId::new(u64::from(rng.below(1_000)))));
    site.set_name(Some(site_name));
    site.set_domain(Some(domain));
    site.set_cat(SITE_CAT);
    site.set_page(Some(page));
    site.set_ref_page(
        rng.bit(Share::percent(30))
            .then_some("https://search.example/"),
    );
    site.set_mobile(Some(choose(rng, &[Flag::No, Flag::Yes], Flag::No)));
    site.set_privacypolicy(Some(Flag::Yes));
    site.set_publisher(Some(publisher));
    site.set_keywords(Some(keywords));
    site.set_ext(rng.bit(Share::percent(30)).then_some(SITE_EXT));
    site
}

fn build_device(rng: &mut impl Entropy) -> Device {
    let (country, region, city, zip) = choose(rng, GEOS, ("RU", "MOW", "Moscow", "101000"));

    let mut geo = Geo::default();
    geo.set_country(Some(country));
    geo.set_region(Some(region));
    geo.set_city(Some(city));
    geo.set_zip(Some(zip));
    geo.set_type_code(Some(LocationType::IP_ADDRESS));
    geo.set_utcoffset(Some(180));

    let (ua, make, model, os, osv, devicetype) = choose(rng, DEVICES, DEFAULT_DEVICE);

    let mut device = Device::default();
    device.set_ua(Some(ua));
    device.set_ip(Some(choose(rng, IPS, "203.0.113.7")));
    device.set_geo(Some(geo));
    device.set_idfa(
        rng.bit(Share::percent(60))
            .then(|| choose(rng, IDFAS, DEFAULT_IDFA)),
    );
    device.set_dnt(rng.bit(Share::percent(30)).then_some(Flag::No));
    device.set_lmt(rng.bit(Share::percent(30)).then_some(Flag::No));
    device.set_devicetype(Some(devicetype));
    device.set_make(Some(make));
    device.set_model(Some(model));
    device.set_os(Some(os));
    device.set_osv(Some(osv));
    device.set_language(Some(choose(rng, LANGUAGES, "en")));
    device
}

fn build_user(rng: &mut impl Entropy) -> Option<User> {
    if !rng.bit(Share::percent(70)) {
        return None;
    }

    let mut user = User::default();
    user.set_id(Some(UserId::new(u64::from(rng.below(1_000_000)))));
    user.set_buyeruid(Some(BuyerUserId::new(u64::from(rng.below(1_000_000)))));
    user.set_consent(
        rng.bit(Share::percent(60))
            .then(|| choose(rng, CONSENTS, DEFAULT_CONSENT)),
    );
    user.set_ext(rng.bit(Share::percent(30)).then_some(USER_EXT));
    Some(user)
}

fn build_regs(rng: &mut impl Entropy) -> Option<Regs> {
    if !rng.bit(Share::percent(50)) {
        return None;
    }

    let mut regs = Regs::default();
    regs.set_coppa(Some(Flag::No));
    regs.set_gdpr(Some(Flag::Yes));
    regs.set_gpp(rng.bit(Share::percent(40)).then_some(GPP));
    regs.set_gpp_sid(if rng.bit(Share::percent(40)) {
        GPP_SID
    } else {
        &[]
    });
    regs.set_ext(rng.bit(Share::percent(20)).then_some(REGS_EXT));
    Some(regs)
}
