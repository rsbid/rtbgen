//! A malformed request: exactly one deliberate violation, still valid JSON.

use std::io::{self, Write};
use std::process::ExitCode;

use rtbgen::{DegradeRate, Profile, Request, Seed};

fn main() -> ExitCode {
    let seed = Seed::new(7);
    let clean = Request::builder(Profile::SiteVideo).seed(seed).build().to_json_string();
    let malformed =
        Request::builder(Profile::SiteVideo).seed(seed).degrade(DegradeRate::percent(100)).build();

    let Some(mutation) = malformed.mutation() else {
        return ExitCode::FAILURE;
    };
    let json = malformed.to_json_string();

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) else {
        return ExitCode::FAILURE;
    };
    if !value.is_object() || clean == json {
        return ExitCode::FAILURE;
    }

    let mut out = io::stdout().lock();
    if writeln!(out, "{}", mutation.name()).is_err() {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
