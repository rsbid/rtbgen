//! The `arbitrary` adapter: any byte slice is a valid request.
#![cfg(feature = "arbitrary")]
#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use arbitrary::{Arbitrary, Unstructured};
use rtbgen::{Profile, Request};
use serde_json::Value;

fn generate(bytes: &[u8]) -> Request {
    Request::arbitrary(&mut Unstructured::new(bytes))
        .expect("the adapter is total over every byte slice")
}

fn parse(request: &Request) -> Value {
    serde_json::from_str(&request.to_json_string()).expect("a generated request is valid JSON")
}

#[test]
fn empty_input_still_produces_a_valid_request() {
    let value = parse(&generate(&[]));

    assert!(value.pointer("/id").is_some(), "the request has an id");
    assert!(value.pointer("/imp").is_some(), "the request has impressions");
}

#[test]
fn a_short_tail_is_zero_padded() {
    let value = parse(&generate(&[0x2A]));

    assert!(value.pointer("/id").is_some(), "the request has an id");
    assert!(value.pointer("/imp").is_some(), "the request has impressions");
}

#[test]
fn the_same_bytes_produce_the_same_request() {
    let bytes = [0x2A; 128];

    assert_eq!(generate(&bytes).to_json_string(), generate(&bytes).to_json_string());
}

#[test]
fn arbitrary_and_arbitrary_take_rest_agree() {
    let bytes = [0x5A; 96];
    let by_reference = generate(&bytes);
    let by_value = Request::arbitrary_take_rest(Unstructured::new(&bytes))
        .expect("take_rest is total over every byte slice");

    assert_eq!(by_reference.to_json_string(), by_value.to_json_string());
}

#[test]
fn a_byte_generated_request_has_no_seed() {
    assert_eq!(generate(&[1, 2, 3, 4]).seed(), None);
}

#[test]
fn some_byte_inputs_are_degraded() {
    let degraded = (0_u8..=u8::MAX).any(|byte| generate(&[byte; 64]).mutation().is_some());

    assert!(degraded, "the adapter never selects a mutation");
}

#[test]
fn every_profile_is_reachable() {
    let mut seen = Vec::new();

    for first in 0_u8..=u8::MAX {
        let profile = generate(&[first; 64]).profile();
        if !seen.contains(&profile) {
            seen.push(profile);
        }
    }

    assert_eq!(seen.len(), Profile::ALL.len(), "some profile is unreachable: {seen:?}");
}

#[test]
fn the_adapter_never_reads_outside_the_input() {
    let inputs: [&[u8]; 5] = [&[], &[0], &[0xFF; 1], &[0xAA; 8], &[0x55; 32]];

    for bytes in inputs {
        let mut unstructured = Unstructured::new(bytes);
        let request = Request::arbitrary(&mut unstructured)
            .expect("a truncated input is still a valid request");
        let value = parse(&request);
        assert!(value.is_object(), "a request is a JSON object");
    }
}
