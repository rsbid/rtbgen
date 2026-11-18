//! A parser test: choose the profile that matches the feature under test.

use std::process::ExitCode;

use rtbgen::{Profile, Request, Seed};

fn main() -> ExitCode {
    let json = Request::builder(Profile::SiteVideo)
        .seed(Seed::new(7))
        .build()
        .to_json_string();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) else {
        return ExitCode::FAILURE;
    };

    assert!(
        value.pointer("/imp/0/video/mimes").is_some(),
        "the video impression has no mimes"
    );
    assert!(value.pointer("/site").is_some(), "the request has no site");
    assert!(
        value.pointer("/app").is_none(),
        "a website request must not carry an app"
    );
    assert!(
        value.pointer("/imp/0/id").is_some(),
        "the impression has no id"
    );

    ExitCode::SUCCESS
}
