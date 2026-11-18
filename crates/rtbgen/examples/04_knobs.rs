//! Knobs are typed and widen the profile; they never conflict with it.

use std::process::ExitCode;

use rtbgen::{Currency, Deals, Floor, Profile, Request, Seed, SpecVersion};

fn main() -> ExitCode {
    let json = Request::builder(Profile::PmpDeals)
        .seed(Seed::new(5))
        .spec(SpecVersion::V25)
        .deals(Deals::new(4))
        .floor(Floor::cpm(2.0, Currency::EUR))
        .build()
        .to_json_string();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) else {
        return ExitCode::FAILURE;
    };

    let deals = value
        .pointer("/imp/0/pmp/deals")
        .and_then(serde_json::Value::as_array);
    assert!(
        matches!(deals.map(Vec::len), Some(4)),
        "the deal knob did not add four deals"
    );
    assert_eq!(
        value
            .pointer("/imp/0/bidfloorcur")
            .and_then(serde_json::Value::as_str),
        Some("EUR"),
        "the floor knob did not set the currency"
    );
    assert!(
        value.pointer("/regs/gdpr").is_none(),
        "a 2.5 request must not carry the 2.6 gdpr field"
    );

    ExitCode::SUCCESS
}
