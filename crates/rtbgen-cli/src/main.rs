//! Command-line interface for `rtbgen`.
//!
//! Use this binary to generate fixtures for a load test or to verify the
//! versioned corpus. Library users do not need it: the CLI is a thin wrapper
//! over the same generator.

pub mod args;
pub mod run;

use std::io::Write;
use std::process::ExitCode;

use args::{Command, Error};

fn main() -> ExitCode {
    match collect() {
        Ok(command) => match run::dispatch(command) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => fail(&error),
        },
        Err(error) => fail(&error),
    }
}

fn collect() -> Result<Command, Error> {
    let mut arguments = Vec::new();

    for argument in std::env::args_os().skip(1) {
        let Ok(value) = argument.into_string() else {
            return Err(Error::NonUnicode);
        };
        arguments.push(value);
    }

    Command::parse(arguments.into_iter())
}

fn fail(error: &Error) -> ExitCode {
    let mut stderr = std::io::stderr().lock();
    if writeln!(stderr, "rtbgen: {error}").is_err() {
        return ExitCode::FAILURE;
    }
    if writeln!(stderr, "try `rtbgen --help`").is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::FAILURE
}
