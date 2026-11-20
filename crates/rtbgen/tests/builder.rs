//! The builder API: every knob is accepted by every profile.

#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use rtbgen::{
    BytesEntropy, Currency, Deals, Floor, ImpressionCount, Profile, Request, Seed, SpecVersion,
};
use serde_json::Value;

fn render(builder: rtbgen::RequestBuilder) -> Value {
    let text = builder.build().to_json_string();

    serde_json::from_str(&text).expect("a generated request is valid JSON")
}

fn impressions(value: &Value) -> usize {
    value
        .pointer("/imp")
        .and_then(Value::as_array)
        .map(Vec::len)
        .expect("the request has impressions")
}

#[test]
fn the_impressions_knob_works_on_every_profile() {
    // The knob is an explicit override: even `MultiImp`, whose default is
    // 2..=3, carries exactly the requested count.
    for profile in Profile::ALL {
        for count in 1_u8..=3 {
            let value = render(
                Request::builder(profile)
                    .seed(Seed::new(5))
                    .impressions_exact(ImpressionCount::new(count)),
            );

            assert_eq!(
                impressions(&value),
                usize::from(count),
                "{profile} ignored impressions_exact({count})"
            );
        }
    }
}

#[test]
fn the_impressions_knob_clamps() {
    for profile in [Profile::SiteDisplay, Profile::MultiImp, Profile::Sparse] {
        let low = render(
            Request::builder(profile).seed(Seed::new(1)).impressions_exact(ImpressionCount::new(0)),
        );
        let high = render(
            Request::builder(profile)
                .seed(Seed::new(1))
                .impressions_exact(ImpressionCount::new(u8::MAX)),
        );

        assert_eq!(impressions(&low), 1);
        assert_eq!(impressions(&high), 3);
    }
}

#[test]
fn the_deals_knob_adds_a_marketplace_to_every_profile() {
    for profile in Profile::ALL {
        for count in [1_u8, 3, 8] {
            let value =
                render(Request::builder(profile).seed(Seed::new(7)).deals(Deals::new(count)));

            let imps = value
                .pointer("/imp")
                .and_then(Value::as_array)
                .expect("the request has impressions");

            for imp in imps {
                assert_eq!(
                    imp.pointer("/pmp/private_auction").and_then(Value::as_u64),
                    Some(1),
                    "{profile} has no private auction"
                );
                let deals = imp
                    .pointer("/pmp/deals")
                    .and_then(Value::as_array)
                    .expect("the marketplace has deals");
                assert_eq!(deals.len(), usize::from(count), "{profile} has the wrong deal count");
            }
        }
    }
}

#[test]
fn the_deals_knob_clamps_to_one() {
    let value =
        render(Request::builder(Profile::AppBanner).seed(Seed::new(0)).deals(Deals::new(0)));
    let deals = value
        .pointer("/imp/0/pmp/deals")
        .and_then(Value::as_array)
        .expect("the marketplace has deals");

    assert_eq!(deals.len(), 1);
}

#[test]
fn the_floor_knob_overrides_every_impression() {
    for profile in Profile::ALL {
        let value = render(
            Request::builder(profile).seed(Seed::new(3)).floor(Floor::cpm(1.5, Currency::EUR)),
        );

        let imps =
            value.pointer("/imp").and_then(Value::as_array).expect("the request has impressions");

        for imp in imps {
            assert_eq!(
                imp.pointer("/bidfloor"),
                Some(&serde_json::json!(1.5_f64)),
                "{profile} has the wrong floor"
            );
            assert_eq!(
                imp.pointer("/bidfloorcur").and_then(Value::as_str),
                Some("EUR"),
                "{profile} has the wrong currency"
            );
        }
    }
}

#[test]
fn the_spec_knob_keeps_the_profile_shape() {
    for profile in Profile::ALL {
        let v25 = render(Request::builder(profile).seed(Seed::new(4)).spec(SpecVersion::V25));

        assert!(v25.pointer("/id").is_some());
        assert!(v25.pointer("/imp").is_some());
    }
}

#[test]
fn the_canonical_configuration_builds() {
    let request = Request::builder(Profile::AppRewarded)
        .seed(Seed::new(42))
        .impressions_exact(ImpressionCount::new(2))
        .deals(Deals::new(3))
        .floor(Floor::cpm(1.50, Currency::USD))
        .build();

    assert_eq!(request.profile(), Profile::AppRewarded);
    assert_eq!(request.seed(), Some(Seed::new(42)));
    assert_eq!(request.spec(), SpecVersion::V26);

    let value: Value = serde_json::from_str(&request.to_json_string())
        .expect("the canonical request is valid JSON");
    assert_eq!(impressions(&value), 2);
    assert!(value.pointer("/app").is_some());

    for imp in value.pointer("/imp").and_then(Value::as_array).expect("the request has impressions")
    {
        assert_eq!(imp.pointer("/pmp/deals").and_then(Value::as_array).map(Vec::len), Some(3));
        assert_eq!(imp.pointer("/bidfloor"), Some(&serde_json::json!(1.5_f64)));
    }
}

#[test]
fn the_default_seed_is_zero_and_reproducible() {
    let first = Request::builder(Profile::SiteVideo).build();
    let second = Request::builder(Profile::SiteVideo).seed(Seed::new(0)).build();

    assert_eq!(first.seed(), Some(Seed::new(0)));
    assert_eq!(first.to_json_string(), second.to_json_string());
}

#[test]
fn entropy_generation_is_deterministic_and_seedless() {
    let bytes = [0x2A; 64];
    let first = Request::from_entropy(
        Profile::SiteDisplay,
        SpecVersion::V26,
        &mut BytesEntropy::new(&bytes),
    );
    let second = Request::from_entropy(
        Profile::SiteDisplay,
        SpecVersion::V26,
        &mut BytesEntropy::new(&bytes),
    );

    assert_eq!(first.seed(), None, "an entropy request has no seed");
    assert_eq!(first.profile(), Profile::SiteDisplay);
    assert_eq!(first.spec(), SpecVersion::V26);
    assert_eq!(first.to_json_string(), second.to_json_string());
}

#[test]
fn knobs_do_not_change_the_unrelated_fields() {
    let plain = render(Request::builder(Profile::SiteDisplay).seed(Seed::new(42)));
    let with_deals =
        render(Request::builder(Profile::SiteDisplay).seed(Seed::new(42)).deals(Deals::new(2)));

    assert_eq!(plain.pointer("/site"), with_deals.pointer("/site"));
    assert_eq!(plain.pointer("/device"), with_deals.pointer("/device"));
    assert_eq!(plain.pointer("/id"), with_deals.pointer("/id"));
}
