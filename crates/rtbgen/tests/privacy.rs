//! The privacy contract: one scenario owns `regs`, consent, `GPP`, location
//! and identifiers, so no generated request contradicts itself.

#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use rtbgen::{Consent, Privacy, Profile, Request, Seed, SpecVersion};
use serde_json::Value;

const EU: [&str; 6] = ["DEU", "GBR", "FRA", "ESP", "ITA", "NLD"];
const NON_EU: [&str; 2] = ["RUS", "USA"];
const ALL: [&str; 8] = ["DEU", "GBR", "FRA", "ESP", "ITA", "NLD", "RUS", "USA"];

fn generated(profile: Profile, seed: u64, spec: SpecVersion) -> Value {
    let json = Request::builder(profile).seed(Seed::new(seed)).spec(spec).build().to_json_string();
    serde_json::from_str(&json).expect("a generated request is valid JSON")
}

fn with_privacy(privacy: Privacy, profile: Profile, seed: u64, spec: SpecVersion) -> Value {
    let json = Request::builder(profile)
        .seed(Seed::new(seed))
        .spec(spec)
        .privacy(privacy)
        .build()
        .to_json_string();
    serde_json::from_str(&json).expect("a generated request is valid JSON")
}

fn text<'value>(value: &'value Value, path: &str) -> Option<&'value str> {
    value.pointer(path).and_then(Value::as_str)
}

fn number(value: &Value, path: &str) -> Option<u64> {
    value.pointer(path).and_then(Value::as_u64)
}

fn utcoffset(country: &str) -> i64 {
    match country {
        "RUS" => 180,
        "USA" => -300,
        "GBR" => 0,
        _ => 60,
    }
}

fn language(country: &str) -> &'static str {
    match country {
        "RUS" => "ru",
        "DEU" => "de",
        "FRA" => "fr",
        "ESP" => "es",
        "ITA" => "it",
        "NLD" => "nl",
        _ => "en",
    }
}

/// Checks the invariants that hold for every request, however it was built.
fn check(value: &Value, spec: SpecVersion) {
    check_location(value);
    check_consent_transport(value);
    check_gdpr_location(value);
    check_coppa(value);
    check_regs(value, spec);
    check_us_privacy(value);
}

fn check_location(value: &Value) {
    let Some(country) = text(value, "/device/geo/country") else {
        return;
    };

    assert!(ALL.contains(&country), "{country} is not a location the generator knows");
    assert_eq!(
        value.pointer("/device/geo/utcoffset").and_then(Value::as_i64),
        Some(utcoffset(country)),
        "{country} has the wrong UTC offset"
    );
    let tag = text(value, "/device/language");
    assert!(tag == Some("en") || tag == Some(language(country)), "{country} speaks {tag:?}");
}

fn check_consent_transport(value: &Value) {
    let consent = text(value, "/user/consent");

    if consent.is_some() {
        assert_eq!(number(value, "/regs/gdpr"), Some(1), "consent without `regs.gdpr=1`");
    }

    let Some(section) = text(value, "/regs/gpp") else {
        return;
    };

    assert_eq!(number(value, "/regs/gdpr"), Some(1), "GPP without `regs.gdpr=1`");
    assert_eq!(number(value, "/regs/gpp_sid/0"), Some(2), "missing TCF section id");
    assert_eq!(
        section.split_once('~').map(|(_, payload)| payload),
        consent,
        "GPP and `user.consent` disagree"
    );
}

fn check_gdpr_location(value: &Value) {
    let Some(country) = text(value, "/device/geo/country") else {
        return;
    };
    let Some(gdpr) = number(value, "/regs/gdpr") else {
        return;
    };

    if gdpr == 1 {
        assert!(EU.contains(&country), "GDPR request from {country}");
    } else {
        assert_eq!(gdpr, 0, "`regs.gdpr` is a two-valued flag");
        assert!(NON_EU.contains(&country), "non-GDPR request from {country}");
    }
}

fn check_coppa(value: &Value) {
    if number(value, "/regs/coppa") != Some(1) {
        return;
    }

    let country = text(value, "/device/geo/country");
    assert!(
        country.is_none() || country == Some("USA"),
        "COPPA is a US regulation, not {country:?}"
    );

    for path in [
        "/user",
        "/device/ifa",
        "/device/ip",
        "/device/geo/region",
        "/device/geo/city",
        "/device/geo/zip",
        "/device/geo/type",
    ] {
        assert!(value.pointer(path).is_none(), "COPPA kept {path}");
    }

    assert_eq!(number(value, "/regs/gdpr"), None, "COPPA and GDPR are mutually exclusive");
    assert!(text(value, "/user/consent").is_none(), "COPPA request carried consent");
    assert!(text(value, "/regs/gpp").is_none(), "COPPA request carried GPP");
}

fn check_regs(value: &Value, spec: SpecVersion) {
    if value.get("regs").is_none() {
        assert!(text(value, "/user/consent").is_none(), "consent without `regs`");
        assert!(text(value, "/regs/gpp").is_none(), "GPP without `regs`");
        return;
    }

    if spec != SpecVersion::V26 {
        return;
    }

    let coppa = number(value, "/regs/coppa");
    assert!(coppa.is_some(), "2.6 `regs` must carry `coppa`");

    if coppa != Some(1) {
        assert!(number(value, "/regs/gdpr").is_some(), "2.6 `regs` must carry `gdpr`");
    }
}

fn check_us_privacy(value: &Value) {
    if let Some(us) = text(value, "/regs/ext/us_privacy") {
        assert_eq!(us, "1YNN", "the US Privacy string is frozen");
        assert_eq!(text(value, "/device/geo/country"), Some("USA"), "US Privacy outside the USA");
    }
}

#[test]
fn every_generated_request_is_coherent() {
    for profile in Profile::ALL {
        for spec in SpecVersion::ALL {
            for seed in 0..120 {
                let value = generated(profile, seed, spec);
                check(&value, spec);
            }
        }
    }
}

#[test]
fn coppa_and_both_gdpr_consent_values_are_reachable() {
    let mut coppa = false;
    let mut granted = false;
    let mut denied = false;
    let mut no_consent = false;
    let mut disabled = false;
    let mut us_privacy = false;

    for seed in 0..2_000 {
        let value = generated(Profile::SiteDisplay, seed, SpecVersion::V26);

        coppa |= number(&value, "/regs/coppa") == Some(1);
        granted |= text(&value, "/user/consent") == Some(consent_string(true));
        denied |= text(&value, "/user/consent") == Some(consent_string(false));
        no_consent |=
            number(&value, "/regs/gdpr") == Some(1) && value.pointer("/user/consent").is_none();
        disabled |= number(&value, "/regs/gdpr") == Some(0);
        us_privacy |= value.pointer("/regs/ext/us_privacy").is_some();
    }

    assert!(coppa, "coppa=1 is never drawn");
    assert!(granted, "a granted TC string is never drawn");
    assert!(denied, "a denied TC string is never drawn");
    assert!(no_consent, "`gdpr=1` without consent is never drawn");
    assert!(disabled, "`gdpr=0` is never drawn");
    assert!(us_privacy, "the US Privacy string is never drawn");
}

/// The exact strings the crate ships; the integration test only compares them.
const fn consent_string(granted: bool) -> &'static str {
    if granted {
        "CQdWToAQdWToAAAABAENEsEsAP_gAAAAAAYgAAoAAgAA"
    } else {
        "CQdWToAQdWToAAAABAENEsEgAAAAAAAAAAYgAAgAAgAA"
    }
}

fn check_knob(value: &Value, spec: SpecVersion, privacy: Privacy) {
    check(value, spec);

    match privacy {
        Privacy::Coppa => {
            assert!(value.get("user").is_none(), "COPPA kept the user");
            assert!(value.pointer("/device/ifa").is_none(), "COPPA kept the advertising id");
            assert!(value.pointer("/device/ip").is_none(), "COPPA kept the address");
        }
        Privacy::Gdpr { consent } if spec == SpecVersion::V26 => {
            let expected = consent_string(consent == Consent::Granted);
            assert_eq!(text(value, "/user/consent"), Some(expected), "the TC string differs");
            assert_eq!(
                text(value, "/regs/gpp").and_then(|gpp| gpp.split_once('~')),
                Some(("DBABMA", expected)),
                "the GPP section differs from the TC string"
            );
        }
        Privacy::Gdpr { .. } => {}
        Privacy::GdprWithoutConsent | Privacy::GdprDisabled | Privacy::None => {
            assert!(text(value, "/user/consent").is_none(), "{privacy:?} carried consent");
            assert!(text(value, "/regs/gpp").is_none(), "{privacy:?} carried GPP");
        }
    }
}

const fn expected_location(privacy: Privacy) -> &'static str {
    match privacy {
        Privacy::Coppa => "USA",
        Privacy::Gdpr { .. } | Privacy::GdprWithoutConsent => "DEU",
        Privacy::GdprDisabled | Privacy::None => "RUS",
    }
}

const fn expected_gdpr(privacy: Privacy) -> Option<u64> {
    match privacy {
        Privacy::Gdpr { .. } | Privacy::GdprWithoutConsent => Some(1),
        Privacy::GdprDisabled => Some(0),
        Privacy::Coppa | Privacy::None => None,
    }
}

fn check_case(value: &Value, profile: Profile, spec: SpecVersion, privacy: Privacy) {
    check_knob(value, spec, privacy);

    if profile != Profile::Sparse {
        assert_eq!(
            text(value, "/device/geo/country"),
            Some(expected_location(privacy)),
            "{profile} {privacy:?} {spec}"
        );
    }
    if spec == SpecVersion::V26 {
        assert_eq!(
            number(value, "/regs/gdpr"),
            expected_gdpr(privacy),
            "{profile} {privacy:?} {spec}"
        );
    }
}

#[test]
fn the_privacy_knob_is_coherent_on_every_profile() {
    let cases = [
        Privacy::Coppa,
        Privacy::Gdpr { consent: Consent::Granted },
        Privacy::Gdpr { consent: Consent::Denied },
        Privacy::GdprWithoutConsent,
        Privacy::GdprDisabled,
        Privacy::None,
    ];

    for profile in Profile::ALL {
        for spec in SpecVersion::ALL {
            for privacy in cases {
                let value = with_privacy(privacy, profile, 42, spec);
                check_case(&value, profile, spec, privacy);
            }
        }
    }
}
