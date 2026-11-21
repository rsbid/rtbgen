//! The privacy contract: one scenario owns `regs`, consent, `GPP`, location
//! and identifiers, so no generated request contradicts itself.

#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use rtbgen::{Consent, DegradeRate, Privacy, Profile, Request, Seed, SpecVersion};
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

/// Returns the GDPR flag of the revision: first-class in 2.6, `ext` in 2.5.
fn gdpr(value: &Value, spec: SpecVersion) -> Option<u64> {
    match spec {
        SpecVersion::V25 => number(value, "/regs/ext/gdpr"),
        SpecVersion::V26 => number(value, "/regs/gdpr"),
    }
}

/// Returns the TC string of the revision: first-class in 2.6, `ext` in 2.5.
fn consent(value: &Value, spec: SpecVersion) -> Option<&str> {
    match spec {
        SpecVersion::V25 => text(value, "/user/ext/consent"),
        SpecVersion::V26 => text(value, "/user/consent"),
    }
}

/// Returns whether the request carries a storage identifier.
fn has_storage_identifiers(value: &Value) -> bool {
    value.pointer("/device/ifa").is_some()
        || text(value, "/user/id").is_some()
        || text(value, "/user/buyeruid").is_some()
        || value
            .pointer("/user/ext/eids")
            .and_then(Value::as_array)
            .is_some_and(|eids| !eids.is_empty())
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
    check_consent_transport(value, spec);
    check_gdpr_location(value, spec);
    check_identifiers(value, spec);
    check_coppa(value, spec);
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

fn check_consent_transport(value: &Value, spec: SpecVersion) {
    let consent = consent(value, spec);

    if consent.is_some() {
        assert_eq!(gdpr(value, spec), Some(1), "consent without `gdpr=1`");
    }

    if spec != SpecVersion::V26 {
        assert!(value.pointer("/regs/gpp").is_none(), "GPP leaked into OpenRTB 2.5");
        return;
    }

    let Some(section) = text(value, "/regs/gpp") else {
        return;
    };

    assert_eq!(gdpr(value, spec), Some(1), "GPP without `gdpr=1`");
    assert_eq!(number(value, "/regs/gpp_sid/0"), Some(2), "missing TCF section id");
    assert_eq!(
        section.split_once('~').map(|(_, payload)| payload),
        consent,
        "GPP and `user.consent` disagree"
    );
}

fn check_identifiers(value: &Value, spec: SpecVersion) {
    let coppa = number(value, "/regs/coppa");
    let gdpr = gdpr(value, spec);
    let consent = consent(value, spec);
    let positive_consent = consent == Some(consent_string(true));
    let identifiers_allowed = coppa != Some(1) && (gdpr != Some(1) || positive_consent);

    if !identifiers_allowed {
        assert!(
            !has_storage_identifiers(value),
            "storage identifiers without a positive consent signal"
        );
    }
}

fn check_gdpr_location(value: &Value, spec: SpecVersion) {
    let Some(country) = text(value, "/device/geo/country") else {
        return;
    };
    let Some(gdpr) = gdpr(value, spec) else {
        return;
    };

    if gdpr == 1 {
        assert!(EU.contains(&country), "GDPR request from {country}");
    } else {
        assert_eq!(gdpr, 0, "`regs.gdpr` is a two-valued flag");
        assert!(NON_EU.contains(&country), "non-GDPR request from {country}");
    }
}

fn check_coppa(value: &Value, spec: SpecVersion) {
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

    assert_eq!(gdpr(value, spec), None, "COPPA and GDPR are mutually exclusive");
    assert!(consent(value, spec).is_none(), "COPPA request carried consent");
    assert!(text(value, "/regs/gpp").is_none(), "COPPA request carried GPP");
}

fn check_regs(value: &Value, spec: SpecVersion) {
    if value.get("regs").is_none() {
        assert!(consent(value, spec).is_none(), "consent without `regs`");
        assert!(text(value, "/regs/gpp").is_none(), "GPP without `regs`");
        return;
    }

    if number(value, "/regs/coppa") == Some(1) {
        return;
    }

    assert!(gdpr(value, spec).is_some(), "`regs` must carry the GDPR flag");
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
        Privacy::Gdpr { consent: expected } => {
            let expected = consent_string(expected == Consent::Granted);
            assert_eq!(consent(value, spec), Some(expected), "the TC string differs");

            if spec == SpecVersion::V26 {
                assert_eq!(
                    text(value, "/regs/gpp").and_then(|gpp| gpp.split_once('~')),
                    Some(("DBABM", expected)),
                    "the GPP section differs from the TC string"
                );
            }
        }
        Privacy::GdprWithoutConsent | Privacy::GdprDisabled | Privacy::None => {
            assert!(consent(value, spec).is_none(), "{privacy:?} carried consent");
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
    assert_eq!(gdpr(value, spec), expected_gdpr(privacy), "{profile} {privacy:?} {spec}");
}

#[test]
fn the_privacy_knob_survives_a_mutation() {
    for privacy in Privacy::ALL {
        let request = Request::builder(Profile::SiteDisplay)
            .seed(Seed::new(35))
            .privacy(privacy)
            .degrade(DegradeRate::percent(100))
            .build();

        assert!(request.mutation().is_some(), "{privacy:?} did not mutate");
        assert!(
            serde_json::from_str::<Value>(&request.to_json_string()).is_ok(),
            "{privacy:?} broke JSON"
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

#[test]
fn a_denied_or_absent_consent_carries_no_storage_identifiers() {
    for spec in SpecVersion::ALL {
        for privacy in [Privacy::Gdpr { consent: Consent::Denied }, Privacy::GdprWithoutConsent] {
            for profile in Profile::ALL {
                let value = with_privacy(privacy, profile, 42, spec);

                assert!(
                    !has_storage_identifiers(&value),
                    "{profile} {privacy:?} {spec} carried a storage identifier"
                );
            }
        }
    }
}

#[test]
fn openrtb_25_carries_the_gdpr_signals_in_ext() {
    let granted = with_privacy(
        Privacy::Gdpr { consent: Consent::Granted },
        Profile::AppRewarded,
        42,
        SpecVersion::V25,
    );
    assert_eq!(consent(&granted, SpecVersion::V25), Some(consent_string(true)));
    assert_eq!(gdpr(&granted, SpecVersion::V25), Some(1));
    assert!(granted.pointer("/user/consent").is_none(), "2.5 has no first-class consent");

    let denied = with_privacy(
        Privacy::Gdpr { consent: Consent::Denied },
        Profile::AppRewarded,
        42,
        SpecVersion::V25,
    );
    assert_eq!(consent(&denied, SpecVersion::V25), Some(consent_string(false)));
    assert_eq!(gdpr(&denied, SpecVersion::V25), Some(1));
    assert!(!has_storage_identifiers(&denied), "a denied request kept identifiers");
}
