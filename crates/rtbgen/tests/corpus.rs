//! Corpus verification: the committed bytes must equal the generator output.

#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use std::fs;
use std::path::Path;

use rtbgen::corpus::{self, CorpusEntry};
use rtbgen::{Profile, Seed, SpecVersion};
use serde_json::Value;

#[test]
fn corpus_matches_generator() {
    for entry in corpus::ALL {
        let path = path_of(*entry);
        let expected = fs::read_to_string(&path).expect("the corpus file exists");
        let actual = entry.request().to_json_string();

        assert_eq!(actual, expected, "{} differs from the generator", entry.name());

        let value: Value = serde_json::from_str(&actual).expect("a corpus file is valid JSON");
        assert!(value.pointer("/id").is_some(), "{} has no id", entry.name());
    }
}

#[test]
fn the_manifest_lists_every_entry_and_nothing_else() {
    let manifest = parse_manifest();
    assert_eq!(manifest.len(), corpus::ALL.len(), "the manifest size differs");

    for entry in corpus::ALL {
        let listed = manifest
            .iter()
            .find(|listed| listed.name == entry.name())
            .expect("every corpus entry is listed in the manifest");

        assert_eq!(listed.spec, entry.spec().name(), "{}", entry.name());
        assert_eq!(listed.profile, entry.profile().name(), "{}", entry.name());
        assert_eq!(listed.seed, entry.seed().into_inner(), "{}", entry.name());
        assert_eq!(listed.knobs & 1 != 0, entry.impressions().is_some(), "{}", entry.name());
        assert_eq!(listed.knobs & 2 != 0, entry.deals().is_some(), "{}", entry.name());
        assert_eq!(listed.knobs & 4 != 0, entry.floor().is_some(), "{}", entry.name());
    }
}

#[test]
fn the_corpus_directory_has_no_orphans() {
    let directory = Path::new(corpus::DIR);
    let mut files = Vec::new();

    for entry in fs::read_dir(directory).expect("the corpus directory exists") {
        let entry = entry.expect("a readable directory entry");
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name.ends_with(".json") {
            files.push(name.trim_end_matches(".json").to_owned());
        }
    }

    assert_eq!(files.len(), corpus::ALL.len(), "an orphaned fixture exists");

    for name in &files {
        assert!(
            corpus::ALL.iter().any(|entry| entry.name() == name),
            "{name}.json is not listed in corpus::ALL"
        );
    }
}

fn path_of(entry: CorpusEntry) -> std::path::PathBuf {
    Path::new(corpus::DIR).join(std::format!("{}.json", entry.name()))
}

struct Listed {
    name: String,
    spec: String,
    profile: String,
    seed: u64,
    /// Bit set: 1 = impressions, 2 = deals, 4 = floor.
    knobs: u8,
}

fn parse_manifest() -> Vec<Listed> {
    let text = fs::read_to_string(Path::new(corpus::DIR).join("manifest.toml"))
        .expect("the corpus manifest exists");
    let mut listed = Vec::new();
    let mut current: Option<Listed> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("version") {
            continue;
        }
        if line == "[[request]]" {
            if let Some(entry) = current.take() {
                listed.push(entry);
            }
            current = Some(Listed {
                name: String::new(),
                spec: String::new(),
                profile: String::new(),
                seed: 0,
                knobs: 0,
            });
            continue;
        }

        let Some((key, value)) = line.split_once(" = ") else {
            continue;
        };
        let value = value.trim_matches('"');
        let entry = current.as_mut().expect("a key belongs to a request");

        match key {
            "name" => value.clone_into(&mut entry.name),
            "spec" => value.clone_into(&mut entry.spec),
            "profile" => value.clone_into(&mut entry.profile),
            "seed" => entry.seed = value.parse().expect("the manifest seed is a u64"),
            "impressions" => entry.knobs |= 1,
            "deals" => entry.knobs |= 2,
            "floor_cpm" => entry.knobs |= 4,
            _ => {}
        }
    }

    if let Some(entry) = current.take() {
        listed.push(entry);
    }

    listed
}

#[test]
fn every_spec_version_and_profile_has_a_golden() {
    for profile in Profile::ALL {
        for spec in SpecVersion::ALL {
            assert!(
                corpus::ALL.iter().any(|entry| entry.profile() == profile && entry.spec() == spec),
                "{profile} has no {spec} golden"
            );
        }
    }
}

#[test]
fn the_seed_alone_reproduces_a_corpus_entry() {
    let entry = corpus::ALL
        .iter()
        .find(|entry| entry.name() == "site_display-v26-42")
        .expect("the canonical site_display entry exists");

    let rebuilt =
        rtbgen::Request::builder(Profile::SiteDisplay).seed(Seed::new(42)).build().to_json_string();

    assert_eq!(rebuilt, entry.request().to_json_string());
}
