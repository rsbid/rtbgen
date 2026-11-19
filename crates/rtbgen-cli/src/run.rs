//! Command dispatch: generation, profile listing and corpus verification.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use rtbgen::{Profile, Request, Seed, corpus};

use crate::args::{Command, Corpus, Error, Generate};

const HELP: &str = "\
rtbgen — deterministic OpenRTB 2.5/2.6 bid request generator

USAGE:
    rtbgen [OPTIONS]
    rtbgen corpus --verify|--write [--dir <PATH>]

OPTIONS:
    --profile <NAME>      profile to generate (default site_display)
    --seed <U64>          seed of the first request (default 0)
    --spec <2.5|2.6>      OpenRTB revision (default 2.6)
    --count <N>           number of requests (default 1)
    --impressions <N>     impression count, clamped to 1..=3
    --deals <N>           add a private marketplace with N deals, clamped to 1..=8
    --floor <CPM>         bid floor of every impression
    --currency <CODE>     currency of --floor (default USD)
    --out <DIR>           write request-NNNNNN.json files instead of stdout
    --list-profiles       print the profiles and their shapes
    -h, --help            print this help
    -V, --version         print the version
";

/// Runs one parsed command.
pub(crate) fn dispatch(command: Command) -> Result<(), Error> {
    match command {
        Command::Generate(generate) => generate_requests(&generate),
        Command::ListProfiles => list_profiles(),
        Command::Corpus(corpus) => run_corpus(&corpus),
        Command::Help => write_stdout(HELP),
        Command::Version => write_stdout(&std::format!("rtbgen {}\n", env!("CARGO_PKG_VERSION"))),
    }
}

fn write_stdout(text: &str) -> Result<(), Error> {
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes())?;
    Ok(())
}

fn build(generate: &Generate, seed: Seed) -> Request {
    let mut builder = Request::builder(generate.profile).seed(seed).spec(generate.spec);

    if let Some(count) = generate.impressions {
        builder = builder.impressions(count);
    }
    if let Some(count) = generate.deals {
        builder = builder.deals(count);
    }
    if let Some(floor) = generate.floor {
        builder = builder.floor(floor);
    }

    builder.build()
}

fn generate_requests(generate: &Generate) -> Result<(), Error> {
    if let Some(directory) = &generate.out {
        fs::create_dir_all(directory)?;
        for index in 0..generate.count {
            let request = build(generate, generate.seed.advance(index));
            let path = directory.join(std::format!("request-{index:06}.json"));
            fs::write(path, request.to_json_string())?;
        }
        let mut out = io::stdout().lock();
        writeln!(out, "wrote {} requests to {}", generate.count, directory.display())?;
        return Ok(());
    }

    let mut out = io::stdout().lock();
    for index in 0..generate.count {
        let request = build(generate, generate.seed.advance(index));
        writeln!(out, "{}", request.to_json_string())?;
    }
    Ok(())
}

fn list_profiles() -> Result<(), Error> {
    let mut out = io::stdout().lock();

    for profile in Profile::ALL {
        writeln!(out, "{}\t{}", profile.name(), profile.description())?;
    }

    Ok(())
}

fn run_corpus(corpus: &Corpus) -> Result<(), Error> {
    if corpus.write {
        fs::create_dir_all(&corpus.dir)?;
        for entry in corpus::ALL {
            let path = corpus.dir.join(std::format!("{}.json", entry.name()));
            fs::write(&path, entry.request().to_json_string())?;
        }
        let mut out = io::stdout().lock();
        writeln!(out, "wrote {} fixtures to {}", corpus::ALL.len(), corpus.dir.display())?;
        return Ok(());
    }

    verify_corpus(&corpus.dir)
}

fn verify_corpus(directory: &Path) -> Result<(), Error> {
    let mut failures = 0_u32;
    let mut stderr = io::stderr().lock();

    for entry in corpus::ALL {
        let path = directory.join(std::format!("{}.json", entry.name()));
        let actual = entry.request().to_json_string();

        match fs::read_to_string(&path) {
            Ok(expected) if expected == actual => {}
            Ok(_) => {
                writeln!(stderr, "{}: differs from the generator", path.display())?;
                failures = failures.saturating_add(1);
            }
            Err(error) => {
                writeln!(stderr, "{}: {error}", path.display())?;
                failures = failures.saturating_add(1);
            }
        }
    }

    if failures > 0 {
        return Err(Error::CorpusMismatch(failures));
    }

    let mut out = io::stdout().lock();
    writeln!(out, "{} fixtures verified", corpus::ALL.len())?;
    Ok(())
}
