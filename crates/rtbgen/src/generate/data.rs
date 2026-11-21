//! The constant tables of the generation distributions.
//!
//! Everything here is in-memory wire data: no allocation, no environment, no
//! clock. A new value in a table changes the bytes of the profiles that pick it,
//! so a table edit is a release decision, not formatting.

use crate::code::{
    ApiFramework, AuctionType, BlockedBannerType, BlockedCreativeAttribute, CompanionType,
    DealAuctionType, DeliveryMethod, DeviceType, ExpandableDirection, GppSectionId, Placement,
    PlacementPosition, Protocol,
};
use crate::currency::Currency;
use crate::flag::Flag;
use crate::raw::RawJson;

pub(super) const DEFAULT_SITE: (&str, &str, &str, &str) =
    ("example.com", "Example News", "https://example.com/news/42", "news,sports");
pub(super) const SITES: &[(&str, &str, &str, &str)] = &[
    DEFAULT_SITE,
    ("news.example", "Example Daily", "https://news.example/weather/today", "weather,local"),
    ("shop.example", "Example Shop", "https://shop.example/item/7", "deals,shopping"),
    ("publisher.test", "Example Sports", "https://publisher.test/scores/42", "sports,scores"),
];
pub(super) const PUBLISHER_NAMES: &[&str] = &["Example Media", "Example Publishing"];
pub(super) const SITE_CAT: &[&str] = &["IAB1"];
pub(super) const PUBLISHER_CAT: &[&str] = &["IAB1", "IAB2"];
pub(super) const BCAT: &[&str] = &["IAB1", "IAB2"];
pub(super) const BADV: &[&str] = &["bad.example", "spam.test"];
pub(super) const BAPP: &[&str] = &["bad.app", "spam.example"];

pub(super) const DEFAULT_APP: (&str, &str, &str, &str, &str) =
    ("com.example.app", "Example App", "app.example", "https://apps.example/app", "games");
pub(super) const APPS: &[(&str, &str, &str, &str, &str)] = &[
    DEFAULT_APP,
    (
        "com.example.game",
        "Example Game",
        "game.example",
        "https://apps.example/game",
        "games,puzzle",
    ),
    (
        "com.example.shop",
        "Example Shop",
        "shop.example",
        "https://apps.example/shop",
        "shopping,deals",
    ),
];
pub(super) const APP_VERSIONS: &[&str] = &["1.2.3", "2.0.0", "3.4.1"];

pub(super) type DeviceProfile =
    (&'static str, &'static str, &'static str, &'static str, &'static str, DeviceType);
pub(super) const DEFAULT_DEVICE: DeviceProfile = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36",
    "Dell",
    "XPS 13",
    "Windows",
    "11",
    DeviceType::PERSONAL_COMPUTER,
);
pub(super) const DEVICES: &[DeviceProfile] = &[
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
pub(super) const DEFAULT_APP_DEVICE: DeviceProfile = (
    "ExampleApp/1.2.3 (iPhone15,2; iOS 18.1)",
    "Apple",
    "iPhone15,2",
    "iOS",
    "18.1",
    DeviceType::PHONE,
);
pub(super) const APP_DEVICES: &[DeviceProfile] = &[
    DEFAULT_APP_DEVICE,
    (
        "ExampleApp/2.0.0 (Pixel 8; Android 15)",
        "Google",
        "Pixel 8",
        "Android",
        "15",
        DeviceType::PHONE,
    ),
];
pub(super) const IPS: &[&str] = &["203.0.113.7", "198.51.100.23", "192.0.2.44"];
pub(super) const DEFAULT_IFA: &str = "A1B2C3D4-0000-4000-8000-000000000001";
pub(super) const IFAS: &[&str] = &[DEFAULT_IFA, "A1B2C3D4-0000-4000-8000-000000000002"];
pub(super) const GEOS: &[(&str, &str, &str, &str)] = &[
    ("RUS", "RU-MOW", "RUMOW", "101000"),
    ("USA", "US-NY", "USNYC", "10001"),
    ("DEU", "DE-BE", "DEBER", "10115"),
    ("GBR", "GB-ENG", "GBLON", "EC1A"),
];
pub(super) const LANGUAGES: &[&str] = &["en", "ru", "de"];
pub(super) const DEFAULT_CONSENT: &str =
    "CPX0000000000000000000000000000000000000000000000000000000000000";
pub(super) const CONSENTS: &[&str] =
    &[DEFAULT_CONSENT, "CPX1111111111111111111111111111111111111111111111111111111111111"];
pub(super) const GPP: &str =
    "DBABMA~CPX0000000000000000000000000000000000000000000000000000000000000";
pub(super) const GPP_SID: &[GppSectionId] = &[GppSectionId::TCF_EU_V2];

pub(super) const BANNER_MIMES: &[&str] = &["image/jpeg", "image/png"];
pub(super) const BANNER_BTYPE: &[BlockedBannerType] = &[BlockedBannerType::IFRAME];
pub(super) const BANNER_BATTR: &[BlockedCreativeAttribute] =
    &[BlockedCreativeAttribute::AUDIO_AUTOPLAY, BlockedCreativeAttribute::AUDIO_USER_INITIATED];
pub(super) const BANNER_EXPDIR: &[ExpandableDirection] =
    &[ExpandableDirection::LEFT, ExpandableDirection::UP];
pub(super) const BANNER_API: &[ApiFramework] = &[ApiFramework::MRAID_1, ApiFramework::MRAID_2];
pub(super) const BANNER_POS: &[PlacementPosition] = &[
    PlacementPosition::ABOVE_THE_FOLD,
    PlacementPosition::LOCKED,
    PlacementPosition::BELOW_THE_FOLD,
];
pub(super) const BANNER_TOPTRAME: &[Flag] = &[Flag::No, Flag::Yes];
pub(super) const SIZES: &[(i32, i32)] = &[(300, 250), (320, 50), (728, 90), (300, 600), (970, 250)];
pub(super) const FLOORS: &[i64] = &[5_000, 10_000, 15_000, 20_000];
pub(super) const AT: &[AuctionType] = &[AuctionType::FIRST_PRICE, AuctionType::SECOND_PRICE_PLUS];
pub(super) const TMAX: &[u32] = &[800, 1_000, 1_200, 1_500];
pub(super) const CUR_USD: &[Currency] = &[Currency::USD];
pub(super) const SITE_EXT: RawJson = RawJson::from_static(r#"{"exchange":"rtbgen"}"#);
pub(super) const USER_EXT: RawJson = RawJson::from_static(r#"{"eids":[]}"#);
pub(super) const REGS_EXT: RawJson = RawJson::from_static(r#"{"us_privacy":"1YNN"}"#);
pub(super) const APP_EXT: RawJson = RawJson::from_static(r#"{"store":"generic"}"#);
pub(super) const NATIVE_EXT: RawJson = RawJson::from_static(r#"{"renderer":"generic"}"#);

pub(super) type VideoFormat = (&'static [&'static str], &'static [Protocol]);
pub(super) const DEFAULT_VIDEO_FORMAT: VideoFormat =
    (&["video/mp4"], &[Protocol::VAST_2_0, Protocol::VAST_3_0]);
pub(super) const VIDEO_FORMATS: &[VideoFormat] = &[
    DEFAULT_VIDEO_FORMAT,
    (&["video/mp4", "video/webm"], &[Protocol::VAST_3_0, Protocol::VAST_4_0]),
];
pub(super) const VIDEO_SIZES: &[(u32, u32)] = &[(640, 480), (1280, 720), (1920, 1080)];
pub(super) const VIDEO_DURATIONS: &[(u32, u32)] = &[(5, 15), (15, 30), (30, 60)];
pub(super) const VIDEO_PLACEMENTS: &[Placement] =
    &[Placement::IN_STREAM, Placement::IN_ARTICLE, Placement::IN_FEED, Placement::INTERSTITIAL];
pub(super) const VIDEO_DELIVERY: &[DeliveryMethod] = &[DeliveryMethod::PROGRESSIVE];
pub(super) const COMPANION_TYPES: &[CompanionType] = &[CompanionType::STATIC];

/// The native request is a JSON document inside a JSON string.
pub(super) const DEFAULT_NATIVE_REQUEST: &str = r#"{"ver":"1.2","assets":[{"id":1,"required":1,"title":{"len":80}},{"id":2,"required":1,"img":{"type":3,"w":300,"h":250}}]}"#;
pub(super) const NATIVE_REQUESTS: &[&str] = &[
    DEFAULT_NATIVE_REQUEST,
    r#"{"ver":"1.2","assets":[{"id":1,"required":1,"title":{"len":90}},{"id":2,"required":1,"data":{"type":2,"len":200}}]}"#,
];
pub(super) const NATIVE_VERSIONS: &[&str] = &["1.2"];

pub(super) const DEAL_FLOORS: &[i64] =
    &[2_000, 5_000, 10_000, 15_000, 20_000, 25_000, 30_000, 40_000];
pub(super) const DEAL_AT: &[DealAuctionType] = &[
    DealAuctionType::FIRST_PRICE,
    DealAuctionType::SECOND_PRICE_PLUS,
    DealAuctionType::AGREED_DEAL_PRICE,
];
pub(super) const ADV_DOMAINS: &[&[&str]] =
    &[&["adv.example"], &["brand.example"], &["shop.example"]];
