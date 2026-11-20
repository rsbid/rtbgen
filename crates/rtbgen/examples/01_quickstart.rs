//! The canonical quick start: one deterministic request on stdout.

use std::io::{self, Write};
use std::process::ExitCode;

use rtbgen::{Currency, Deals, Floor, ImpressionCount, Profile, Request, Seed};

#[expect(clippy::redundant_type_annotations, reason = "the canonical snippet spells the type out")]
fn main() -> ExitCode {
    let req = Request::builder(Profile::AppRewarded)
        .seed(Seed::new(42))
        .impressions_exact(ImpressionCount::new(2))
        .deals(Deals::new(3))
        .floor(Floor::cpm(1.50, Currency::USD))
        .build();

    let json: String = req.to_json_string();
    assert!(json.starts_with('{'), "a request is a JSON object");

    let mut out = io::stdout().lock();
    if out.write_all(json.as_bytes()).is_err() {
        return ExitCode::FAILURE;
    }
    if out.write_all(b"\n").is_err() {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
