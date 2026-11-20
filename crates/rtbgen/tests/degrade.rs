//! The public `degrade` contract: one deterministic violation per request.

#![expect(clippy::expect_used, reason = "in tests a panic = a failed test")]

use rtbgen::{BytesEntropy, DegradeRate, Mutation, Profile, Request, Seed, SpecVersion};
use serde_json::Value;

fn parse(request: &Request) -> Value {
    serde_json::from_str(&request.to_json_string()).expect("a degraded request is valid JSON")
}

#[test]
fn a_zero_rate_is_byte_identical_to_no_knob() {
    for profile in Profile::ALL {
        for spec in SpecVersion::ALL {
            let plain = Request::builder(profile).seed(Seed::new(42)).spec(spec).build();
            let zero = Request::builder(profile)
                .seed(Seed::new(42))
                .spec(spec)
                .degrade(DegradeRate::percent(0))
                .build();

            assert_eq!(plain.to_json_string(), zero.to_json_string(), "{profile} {spec}");
            assert!(zero.mutation().is_none(), "{profile} {spec} mutated at 0%");
        }
    }
}

#[test]
fn a_full_rate_always_mutates() {
    for profile in Profile::ALL {
        for spec in SpecVersion::ALL {
            for seed in [0_u64, 1, 42, u64::MAX] {
                let request = Request::builder(profile)
                    .seed(Seed::new(seed))
                    .spec(spec)
                    .degrade(DegradeRate::percent(100))
                    .build();

                assert!(
                    request.mutation().is_some(),
                    "{profile} {spec} seed {seed} did not mutate"
                );
                assert!(parse(&request).is_object(), "{profile} emitted invalid JSON");
            }
        }
    }
}

#[test]
fn an_unmutated_request_keeps_the_plain_bytes() {
    let mut seen_mutated = false;
    let mut seen_clean = false;

    for seed in 0_u64..64 {
        let plain = Request::builder(Profile::SiteDisplay).seed(Seed::new(seed)).build();
        let degraded = Request::builder(Profile::SiteDisplay)
            .seed(Seed::new(seed))
            .degrade(DegradeRate::percent(50))
            .build();

        if degraded.mutation().is_some() {
            seen_mutated = true;
        } else {
            seen_clean = true;
            assert_eq!(
                plain.to_json_string(),
                degraded.to_json_string(),
                "seed {seed} changed without a mutation"
            );
        }
    }

    assert!(seen_mutated, "no seed mutated at 50%");
    assert!(seen_clean, "every seed mutated at 50%");
}

#[test]
fn every_mutation_is_reachable_and_named() {
    let mut seen: Vec<Mutation> = Vec::new();

    for seed in 0_u64..2_000 {
        let request = Request::builder(Profile::Maximal)
            .seed(Seed::new(seed))
            .degrade(DegradeRate::percent(100))
            .build();

        if let Some(mutation) = request.mutation()
            && !seen.contains(&mutation)
        {
            seen.push(mutation);
        }
    }

    for mutation in Mutation::ALL {
        assert!(seen.contains(mutation), "{mutation} is unreachable");
        assert!(!mutation.name().is_empty(), "{mutation} has no name");
    }
}

#[test]
fn the_entropy_entry_point_takes_the_rate() {
    let mut plain_entropy = BytesEntropy::new(&[0x2A; 64]);
    let plain = Request::from_entropy(Profile::SiteDisplay, SpecVersion::V26, &mut plain_entropy);
    assert!(plain.mutation().is_none(), "from_entropy mutated");

    let mut degraded_entropy = BytesEntropy::new(&[0x2A; 64]);
    let degraded = Request::from_entropy_with(
        Profile::SiteDisplay,
        SpecVersion::V26,
        DegradeRate::percent(100),
        &mut degraded_entropy,
    );
    assert!(degraded.mutation().is_some(), "from_entropy_with did not mutate");
    let value: Value =
        serde_json::from_str(&degraded.to_json_string()).expect("the degraded request is JSON");
    assert!(value.is_object(), "the degraded request is not an object");
}

#[test]
fn the_same_seed_and_rate_reproduce_the_same_defect() {
    for seed in [0_u64, 7, 42] {
        let first = Request::builder(Profile::SiteVideo)
            .seed(Seed::new(seed))
            .degrade(DegradeRate::percent(100))
            .build();
        let second = Request::builder(Profile::SiteVideo)
            .seed(Seed::new(seed))
            .degrade(DegradeRate::percent(100))
            .build();

        assert_eq!(first.mutation(), second.mutation());
        assert_eq!(first.to_json_string(), second.to_json_string());
    }
}

#[test]
fn a_rate_above_one_hundred_saturates() {
    assert_eq!(DegradeRate::percent(200).basis_points(), 10_000);

    let request = Request::builder(Profile::Sparse)
        .seed(Seed::new(3))
        .degrade(DegradeRate::percent(200))
        .build();
    assert!(request.mutation().is_some());
}
