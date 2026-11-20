//! The `proptest` adapter: every generated request is valid and byte-stable.
#![cfg(feature = "proptest")]
#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use proptest::prelude::*;
use rtbgen::Profile;

proptest! {
    #[test]
    fn every_generated_request_is_valid_json(request in rtbgen::proptest::request()) {
        let json = request.to_json_string();
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("a generated request is valid JSON");

        prop_assert!(value.pointer("/id").is_some(), "the request has an id");
        prop_assert!(value.pointer("/imp").is_some(), "the request has impressions");
        prop_assert!(Profile::ALL.contains(&request.profile()));
        prop_assert!(request.seed().is_some());
    }

    #[test]
    fn writing_the_same_request_is_stable(request in rtbgen::proptest::request()) {
        prop_assert_eq!(request.to_json_string(), request.to_json_string());
    }

    #[test]
    fn the_spec_survives_the_strategy(request in rtbgen::proptest::request()) {
        let json = request.to_json_string();
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("a generated request is valid JSON");

        if request.spec() == rtbgen::SpecVersion::V25 {
            prop_assert!(value.pointer("/regs/gdpr").is_none(), "2.5 has no gdpr field");
        }
    }
}
