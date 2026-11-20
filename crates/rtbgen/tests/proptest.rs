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

#[cfg(feature = "serde")]
proptest! {
    #[test]
    fn the_writer_and_serde_agree_on_the_tree(request in rtbgen::proptest::request()) {
        let from_writer: serde_json::Value = serde_json::from_str(&request.to_json_string())
            .expect("a generated request is valid JSON");
        let from_serde: serde_json::Value =
            serde_json::to_value(request).expect("a request always serializes");

        prop_assert_eq!(from_writer, from_serde);
    }
}
