#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

extern crate std;

use std::string::String;

use serde_json::Value;

use super::RequestModel;
use crate::ids::RequestId;
use crate::version::SpecVersion;
use crate::write::request;

fn render(model: &RequestModel, spec: SpecVersion) -> String {
    let mut out = String::new();
    request(&mut out, model, spec).expect("a String sink cannot fail");
    out
}

fn maximal_value() -> Value {
    serde_json::from_str(&render(
        &RequestModel::maximal(RequestId::from_seed(crate::seed::Seed::new(42))),
        SpecVersion::V26,
    ))
    .expect("the writer must emit valid JSON")
}

#[test]
fn maximal_top_level_is_ordered_and_complete() {
    let json = render(
        &RequestModel::maximal(RequestId::from_seed(crate::seed::Seed::new(42))),
        SpecVersion::V26,
    );

    assert!(json.starts_with(r#"{"id":"req-000042","imp":["#));
    assert!(!json.contains('\n'));

    let value = maximal_value();
    assert_eq!(value.pointer("/id").and_then(Value::as_str), Some("req-000042"));
    assert_eq!(value.pointer("/at").and_then(Value::as_u64), Some(2));
    assert_eq!(value.pointer("/tmax").and_then(Value::as_u64), Some(1_000));
    assert_eq!(value.pointer("/cur/0").and_then(Value::as_str), Some("USD"));
    assert_eq!(value.pointer("/bcat/0").and_then(Value::as_str), Some("IAB1"));
    assert_eq!(value.pointer("/badv/0").and_then(Value::as_str), Some("bad.example"));
    assert_eq!(value.pointer("/bapp/0").and_then(Value::as_str), Some("bad.app"));
    assert_eq!(value.pointer("/ext/k").and_then(Value::as_u64), Some(1));
}

#[test]
fn maximal_imp_is_complete() {
    let value = maximal_value();

    assert_eq!(value.pointer("/imp/0/id").and_then(Value::as_str), Some("1"));
    assert_eq!(value.pointer("/imp/0/banner/w").and_then(Value::as_u64), Some(300));
    assert_eq!(value.pointer("/imp/0/banner/id").and_then(Value::as_str), Some("b7"));
    assert_eq!(
        value.pointer("/imp/0/video/companionad/0/mimes/0").and_then(Value::as_str),
        Some("image/jpeg")
    );
    assert_eq!(value.pointer("/imp/0/video/startdelay").and_then(Value::as_i64), Some(-1));
    assert_eq!(
        value.pointer("/imp/0/native/request").and_then(Value::as_str),
        Some(r#"{"native":{"ver":"1.2"}}"#)
    );
    assert_eq!(value.pointer("/imp/0/bidfloor"), Some(&serde_json::json!(1.5_f64)));
    assert_eq!(value.pointer("/imp/0/pmp/deals/0/id").and_then(Value::as_str), Some("deal-1"));
    assert_eq!(value.pointer("/imp/0/ext/k").and_then(Value::as_u64), Some(1));
}

#[test]
fn maximal_party_is_complete() {
    let value = maximal_value();

    assert_eq!(value.pointer("/site/id").and_then(Value::as_str), Some("site-1"));
    assert_eq!(value.pointer("/site/publisher/id").and_then(Value::as_str), Some("pub-2"));
    assert_eq!(value.pointer("/site/ref").and_then(Value::as_str), Some("https://search.example/"));
    assert_eq!(value.pointer("/app/bundle").and_then(Value::as_str), Some("com.example.app"));
    assert_eq!(value.pointer("/device/geo/country").and_then(Value::as_str), Some("RU"));
    assert_eq!(value.pointer("/device/geo/utcoffset").and_then(Value::as_i64), Some(-180));
    assert_eq!(value.pointer("/user/id").and_then(Value::as_str), Some("u4"));
    assert_eq!(value.pointer("/user/buyeruid").and_then(Value::as_str), Some("buyer-5"));
}

#[test]
fn maximal_context_is_complete() {
    let value = maximal_value();

    assert_eq!(value.pointer("/user/consent").and_then(Value::as_str), Some("CPX000"));
    assert_eq!(value.pointer("/source/schain/nodes/0/hp").and_then(Value::as_u64), Some(1));
    assert_eq!(value.pointer("/regs/gpp_sid/0").and_then(Value::as_u64), Some(2));
    assert_eq!(value.pointer("/regs/ext/k").and_then(Value::as_u64), Some(1));
}

#[test]
fn v25_is_v26_without_the_26_only_fields() {
    let v26_json = render(
        &RequestModel::maximal(RequestId::from_seed(crate::seed::Seed::new(42))),
        SpecVersion::V26,
    );
    let v25_json = render(
        &RequestModel::maximal(RequestId::from_seed(crate::seed::Seed::new(42))),
        SpecVersion::V25,
    );

    for field in ["rwdd", "ssai", "consent", "gdpr", "gpp", "gpp_sid"] {
        assert!(!v25_json.contains(&std::format!("\"{field}\"")), "{field} leaked into V25");
    }

    let v26: Value = serde_json::from_str(&v26_json).expect("V26 is valid JSON");
    let v25: Value = serde_json::from_str(&v25_json).expect("V25 is valid JSON");
    let mut expected = v26;

    for path in
        ["/imp/0/rwdd", "/imp/0/ssai", "/user/consent", "/regs/gdpr", "/regs/gpp", "/regs/gpp_sid"]
    {
        assert!(remove_path(&mut expected, path), "{path} was not present");
    }

    assert_eq!(expected, v25);
}

fn remove_path(value: &mut Value, path: &str) -> bool {
    let Some((parent, key)) = path.rsplit_once('/') else {
        return false;
    };
    value
        .pointer_mut(parent)
        .and_then(Value::as_object_mut)
        .and_then(|object| object.remove(key))
        .is_some()
}
