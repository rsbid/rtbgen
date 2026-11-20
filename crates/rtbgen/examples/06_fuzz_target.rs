//! A fuzz target: every byte slice becomes one valid request.
//!
//! Drop the body of [`main`] into a `cargo-fuzz` target as
//! `fuzz_target!(|data: &[u8]| { let _ = rtbgen::Request::arbitrary(&mut Unstructured::new(data)); });`
//! The adapter never panics, so the target fails only on an actual invariant
//! break, not on a short or empty input.

use std::process::ExitCode;

use arbitrary::{Arbitrary, Unstructured};
use rtbgen::Request;

fn main() -> ExitCode {
    let inputs: [&[u8]; 4] = [&[], &[0x2A], &[0x5A; 64], &[0xFF; 256]];

    for bytes in inputs {
        let mut unstructured = Unstructured::new(bytes);
        let Ok(request) = Request::arbitrary(&mut unstructured) else {
            return ExitCode::FAILURE;
        };
        let json = request.to_json_string();
        if !json.starts_with('{') {
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
