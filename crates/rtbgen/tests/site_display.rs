//! Golden bytes for the `SiteDisplay` fixture.

#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use rtbgen::{Profile, Request, Seed};
use serde_json::Value;

const GOLDEN: &[(u64, &str)] = &[
    (
        0,
        r#"{"id":"req-000000","imp":[{"id":"1","banner":{"w":300,"h":250,"pos":3,"mimes":["image/jpeg","image/png"],"topframe":0},"bidfloor":0.5,"bidfloorcur":"USD","secure":1}],"site":{"id":"site-496","name":"Example Shop","domain":"shop.example","cat":["IAB1"],"page":"https://shop.example/item/7","ref":"https://search.example/","mobile":1,"privacypolicy":1,"publisher":{"id":"pub-654","name":"Example Publishing","domain":"shop.example","cat":["IAB1","IAB2"]},"keywords":"deals,shopping","ext":{"exchange":"rtbgen"}},"device":{"ua":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36","ip":"203.0.113.7","geo":{"country":"DE","region":"BE","city":"Berlin","zip":"10115","type":2,"utcoffset":180},"idfa":"A1B2C3D4-0000-4000-8000-000000000001","devicetype":2,"make":"Dell","model":"XPS 13","os":"Windows","osv":"11","language":"en"},"user":{"id":"u913513","buyeruid":"buyer-987883","ext":{"eids":[]}},"at":2,"cur":["USD"],"bcat":["IAB1","IAB2"],"regs":{"coppa":0,"gdpr":1,"gpp_sid":[2]}}"#,
    ),
    (
        42,
        r#"{"id":"req-000042","imp":[{"id":"1","banner":{"w":300,"h":600,"pos":3,"mimes":["image/jpeg","image/png"],"topframe":1},"secure":1}],"site":{"id":"site-321","name":"Example News","domain":"example.com","cat":["IAB1"],"page":"https://example.com/news/42","mobile":1,"privacypolicy":1,"publisher":{"id":"pub-801","name":"Example Media","domain":"example.com","cat":["IAB1","IAB2"]},"keywords":"news,sports"},"device":{"ua":"Mozilla/5.0 (Linux; Android 15; Pixel 8) AppleWebKit/537.36 Chrome/131.0.0.0 Mobile Safari/537.36","ip":"192.0.2.44","geo":{"country":"GB","region":"ENG","city":"London","zip":"EC1A","type":2,"utcoffset":180},"idfa":"A1B2C3D4-0000-4000-8000-000000000001","devicetype":4,"make":"Google","model":"Pixel 8","os":"Android","osv":"15","language":"en"},"user":{"id":"u398991","buyeruid":"buyer-789121","ext":{"eids":[]}},"at":2,"bcat":["IAB1","IAB2"],"regs":{"coppa":0,"gdpr":1}}"#,
    ),
    (
        u64::MAX,
        r#"{"id":"req-18446744073709551615","imp":[{"id":"1","banner":{"w":728,"h":90,"pos":3,"mimes":["image/jpeg","image/png"],"topframe":0},"bidfloor":0.5,"bidfloorcur":"USD","secure":1}],"site":{"id":"site-191","name":"Example Shop","domain":"shop.example","cat":["IAB1"],"page":"https://shop.example/item/7","mobile":0,"privacypolicy":1,"publisher":{"id":"pub-481","name":"Example Publishing","domain":"shop.example","cat":["IAB1","IAB2"]},"keywords":"deals,shopping"},"device":{"ua":"Mozilla/5.0 (Macintosh; Intel Mac OS X 14_6) AppleWebKit/605.1.15 Version/18.1 Safari/605.1.15","ip":"203.0.113.7","geo":{"country":"US","region":"NY","city":"New York","zip":"10001","type":2,"utcoffset":180},"idfa":"A1B2C3D4-0000-4000-8000-000000000002","devicetype":2,"make":"Apple","model":"MacBookPro18,1","os":"macOS","osv":"14.6","language":"en"},"user":{"id":"u462553","buyeruid":"buyer-596495","consent":"CPX0000000000000000000000000000000000000000000000000000000000000"},"at":2,"cur":["USD"],"regs":{"coppa":0,"gdpr":1,"gpp":"DBABMA~CPX0000000000000000000000000000000000000000000000000000000000000","gpp_sid":[2]}}"#,
    ),
];

#[test]
fn site_display_bytes_are_frozen() {
    for (seed, expected) in GOLDEN {
        let actual =
            Request::builder(Profile::SiteDisplay).seed(Seed::new(*seed)).build().to_json_string();

        assert_eq!(actual.as_str(), *expected, "seed {seed}");

        let value: Value =
            serde_json::from_str(&actual).expect("the generated request is valid JSON");
        assert!(value.pointer("/site/domain").is_some(), "seed {seed} has no site domain");
        assert!(value.pointer("/imp/0/banner").is_some(), "seed {seed} has no banner");
        assert!(value.pointer("/app").is_none(), "SiteDisplay must not emit an app object");
    }
}

#[test]
fn the_2_6_consent_field_is_emitted_when_present() {
    let value: Value = serde_json::from_str(
        &Request::builder(Profile::SiteDisplay).seed(Seed::new(u64::MAX)).build().to_json_string(),
    )
    .expect("the generated request is valid JSON");

    assert_eq!(
        value.pointer("/user/consent").and_then(Value::as_str),
        Some("CPX0000000000000000000000000000000000000000000000000000000000000")
    );
}

#[test]
fn the_request_auction_type_is_not_a_deal_only_code() {
    for seed in [0_u64, 42, u64::MAX] {
        let value: Value = serde_json::from_str(
            &Request::builder(Profile::SiteDisplay).seed(Seed::new(seed)).build().to_json_string(),
        )
        .expect("the generated request is valid JSON");

        assert!(
            matches!(value.pointer("/at").and_then(Value::as_u64), Some(1 | 2)),
            "seed {seed} must carry a request-level auction type"
        );
    }
}

#[test]
fn the_same_seed_is_byte_identical() {
    let first = Request::builder(Profile::SiteDisplay).seed(Seed::new(42)).build().to_json_string();
    let second =
        Request::builder(Profile::SiteDisplay).seed(Seed::new(42)).build().to_json_string();

    assert_eq!(first, second);
}
