//! Shape invariants of every profile.

#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use rtbgen::{Profile, Request, Seed, SpecVersion};
use serde_json::Value;

const SPEC_FIELDS: [&str; 6] = ["rwdd", "ssai", "consent", "gdpr", "gpp", "gpp_sid"];

fn render(profile: Profile, spec: SpecVersion, seed: u64) -> Value {
    let text = Request::builder(profile).seed(Seed::new(seed)).spec(spec).build().to_json_string();

    serde_json::from_str(&text).expect("a generated request is valid JSON")
}

fn keys(value: &Value) -> Vec<String> {
    value.as_object().expect("the request is an object").keys().cloned().collect()
}

#[test]
fn every_profile_and_spec_generates_valid_json() {
    for profile in Profile::ALL {
        for spec in SpecVersion::ALL {
            let value = render(profile, spec, 42);

            assert!(value.pointer("/id").is_some(), "{profile} has no id");
            let imps = value
                .pointer("/imp")
                .and_then(Value::as_array)
                .expect("the request has impressions");
            assert!(!imps.is_empty(), "{profile} generated no impression");
        }
    }
}

#[test]
fn the_same_configuration_is_byte_identical() {
    for profile in Profile::ALL {
        let first = Request::builder(profile).seed(Seed::new(9)).build();
        let second = Request::builder(profile).seed(Seed::new(9)).build();

        assert_eq!(first.to_json_string(), second.to_json_string());
    }
}

#[test]
fn site_profiles_have_a_site_and_no_app() {
    for profile in [
        Profile::SiteDisplay,
        Profile::SiteVideo,
        Profile::SiteNative,
        Profile::PmpDeals,
        Profile::MultiImp,
    ] {
        let value = render(profile, SpecVersion::V26, 1);

        assert!(value.pointer("/site").is_some(), "{profile} has no site");
        assert!(value.pointer("/app").is_none(), "{profile} has an app");
    }
}

#[test]
fn app_profiles_have_an_app_and_no_site() {
    for profile in [Profile::AppRewarded, Profile::AppBanner] {
        let value = render(profile, SpecVersion::V26, 1);

        assert!(value.pointer("/app").is_some(), "{profile} has no app");
        assert!(value.pointer("/site").is_none(), "{profile} has a site");
        assert_eq!(
            value.pointer("/device/devicetype").and_then(Value::as_u64),
            Some(4),
            "{profile} must use a phone device"
        );
    }
}

#[test]
fn the_media_object_matches_the_profile() {
    let cases = [
        (Profile::SiteDisplay, "/imp/0/banner"),
        (Profile::SiteVideo, "/imp/0/video"),
        (Profile::SiteNative, "/imp/0/native"),
        (Profile::AppRewarded, "/imp/0/video"),
        (Profile::AppBanner, "/imp/0/banner"),
    ];

    for (profile, pointer) in cases {
        let value = render(profile, SpecVersion::V26, 21);

        assert!(value.pointer(pointer).is_some(), "{profile} has no {pointer}");
    }
}

#[test]
fn rewarded_video_is_secure() {
    for seed in [0_u64, 42, u64::MAX] {
        let value = render(Profile::AppRewarded, SpecVersion::V26, seed);
        let imps =
            value.pointer("/imp").and_then(Value::as_array).expect("the request has impressions");

        for imp in imps {
            assert_eq!(
                imp.pointer("/secure").and_then(Value::as_u64),
                Some(1),
                "rewarded video must be secure"
            );
        }
    }
}

#[test]
fn pmp_deals_carries_a_private_auction() {
    let value = render(Profile::PmpDeals, SpecVersion::V26, 5);

    assert_eq!(value.pointer("/imp/0/pmp/private_auction").and_then(Value::as_u64), Some(1));
    let deals = value
        .pointer("/imp/0/pmp/deals")
        .and_then(Value::as_array)
        .expect("the marketplace has deals");
    assert!(!deals.is_empty(), "the marketplace has no deals");
}

#[test]
fn multi_imp_mixes_media_types() {
    let value = render(Profile::MultiImp, SpecVersion::V26, 9);
    let imps =
        value.pointer("/imp").and_then(Value::as_array).expect("the request has impressions");

    assert!((2..=3).contains(&imps.len()), "multi_imp has one impression");

    let media = ["banner", "video", "native"];
    let mut seen = Vec::new();
    for imp in imps {
        for name in media {
            if imp.pointer(&std::format!("/{name}")).is_some() {
                seen.push(name);
            }
        }
    }

    assert_eq!(seen.len(), imps.len(), "an impression has two media objects");
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), imps.len(), "multi_imp repeated a media type");
}

#[test]
fn sparse_is_the_minimum() {
    let value = render(Profile::Sparse, SpecVersion::V26, 0);

    assert_eq!(keys(&value), ["id", "imp"]);
}

#[test]
fn maximal_covers_every_writable_field() {
    let mut top_level = Vec::new();
    let mut impressions = Vec::new();
    let mut has_site = false;
    let mut has_app = false;

    for seed in 0_u64..16 {
        let value = render(Profile::Maximal, SpecVersion::V26, seed);
        has_site |= value.pointer("/site").is_some();
        has_app |= value.pointer("/app").is_some();

        for key in keys(&value) {
            if !top_level.contains(&key) {
                top_level.push(key);
            }
        }

        let imp = value.pointer("/imp/0").expect("maximal has an impression");
        for key in keys(imp) {
            if !impressions.contains(&key) {
                impressions.push(key);
            }
        }
    }

    assert!(has_site, "no seed produced the site variant");
    assert!(has_app, "no seed produced the app variant");

    for key in [
        "id", "imp", "site", "app", "device", "user", "at", "tmax", "cur", "bcat", "badv", "bapp",
        "source", "regs", "ext",
    ] {
        assert!(top_level.iter().any(|seen| seen == key), "no {key} field");
    }

    for key in [
        "id",
        "banner",
        "video",
        "audio",
        "native",
        "pmp",
        "bidfloor",
        "bidfloorcur",
        "secure",
        "rwdd",
        "ssai",
        "ext",
    ] {
        assert!(impressions.iter().any(|seen| seen == key), "no imp.{key} field");
    }
}

#[test]
fn v25_drops_exactly_the_26_fields() {
    for profile in Profile::ALL {
        let v26 = Request::builder(profile).seed(Seed::new(42)).build().to_json_string();
        let v25 = Request::builder(profile)
            .seed(Seed::new(42))
            .spec(SpecVersion::V25)
            .build()
            .to_json_string();

        for field in SPEC_FIELDS {
            let quoted = std::format!("\"{field}\"");
            assert!(!v25.contains(&quoted), "{profile}: {field} leaked into 2.5");
        }

        let mut expected: Value = serde_json::from_str(&v26).expect("2.6 is valid JSON");
        for path in [
            "/imp/0/rwdd",
            "/imp/0/ssai",
            "/user/consent",
            "/regs/gdpr",
            "/regs/gpp",
            "/regs/gpp_sid",
        ] {
            remove_path(&mut expected, path);
        }

        let actual: Value = serde_json::from_str(&v25).expect("2.5 is valid JSON");
        assert_eq!(actual, expected, "{profile}");
    }
}

fn remove_path(value: &mut Value, path: &str) {
    let Some((parent, key)) = path.rsplit_once('/') else {
        return;
    };

    if let Some(object) = value.pointer_mut(parent).and_then(Value::as_object_mut) {
        drop(object.remove(key));
    }
}
