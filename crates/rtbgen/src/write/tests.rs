#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

extern crate std;

use std::string::String;

use serde_json::Value;

use super::{json_string, request};
use crate::version::SpecVersion;
use crate::wire::RequestModel;

fn render(model: &RequestModel, spec: SpecVersion) -> String {
    let mut out = String::new();
    request(&mut out, model, spec).expect("a String sink cannot fail");
    out
}

#[test]
fn minimal_model_keeps_only_the_required_fields() {
    assert_eq!(
        render(&RequestModel::default(), SpecVersion::V26),
        r#"{"id":"req-000000","imp":[]}"#
    );
}

#[test]
fn escaping_is_exact() {
    let input = "quote\" slash\\ back\u{08}\u{0C} newline\n return\r tab\t nul\u{0} one\u{1} unit\u{1F} / < > & é 😀";
    let mut out = String::new();

    json_string(&mut out, input).expect("a String sink cannot fail");

    assert_eq!(
        out,
        r#""quote\" slash\\ back\b\f newline\n return\r tab\t nul\u0000 one\u0001 unit\u001f / < > & é 😀""#
    );
    assert_eq!(
        serde_json::from_str::<Value>(&out).expect("the escaped string is valid JSON"),
        Value::String(String::from(input))
    );
}

#[test]
fn empty_arrays_are_omitted() {
    let json = render(&RequestModel::default(), SpecVersion::V26);

    assert!(!json.contains("\"cur\""), "cur must be omitted");
    assert!(!json.contains("\"bcat\""), "bcat must be omitted");
    assert!(!json.contains("\"badv\""), "badv must be omitted");
    assert!(!json.contains("\"bapp\""), "bapp must be omitted");
}
