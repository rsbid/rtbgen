# rtbgen

[![crates.io](https://img.shields.io/crates/v/rtbgen.svg)](https://crates.io/crates/rtbgen)
[![docs.rs](https://img.shields.io/docsrs/rtbgen)](https://docs.rs/rtbgen)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Deterministic OpenRTB 2.5/2.6 bid request generator for tests, fuzzing and load runs.
Generates wire JSON, not types, so it works with every OpenRTB crate.

```rust
use rtbgen::{Currency, Deals, Floor, ImpressionCount, Profile, Request, Seed};

let req = Request::builder(Profile::AppRewarded)
    .seed(Seed::new(42))
    .impressions(ImpressionCount::new(2))
    .deals(Deals::new(3))
    .floor(Floor::cpm(1.50, Currency::USD))
    .build();

let json: String = req.to_json_string();
```

Same seed and config produce byte-identical output; the output is covered by a versioned
golden corpus. Knobs widen a profile, they never conflict with it, so `build()` cannot fail.

## Which call do I need?

| You want | Do this |
|---|---|
| A valid request for a test | `Request::builder(Profile::SiteDisplay).seed(Seed::new(1)).build()` |
| A specific shape (video, native, PMP, app, multi-impression) | pick the matching `Profile` |
| An OpenRTB 2.5 request | add `.spec(SpecVersion::V25)`; 2.6 is the default |
| Several impressions | `.impressions(ImpressionCount::new(2))`, clamped to 1..=3 |
| A private marketplace | `.deals(Deals::new(3))`, on any profile |
| A bid floor | `.floor(Floor::cpm(1.50, Currency::USD))` |
| An exact bug reproduction | keep the `Seed` in the test, not the JSON |
| Profiles as data | `Profile::from_name("app_rewarded")` and `Profile::ALL` |
| Requests for a load test | `rtbgen --profile app_rewarded --seed 42 --count 10000 --out ./req/` |
| Fuzzing where every byte matters | the `arbitrary` feature, then `Request::arbitrary(&mut Unstructured::new(bytes))` |
| Property tests that shrink | the `proptest` feature, then `rtbgen::proptest::request()` |
| Serializing through `serde` | the `serde` feature, then `serde_json::to_string(&request)` |
| Your own deterministic source | `Request::from_entropy(profile, spec, &mut source)` |

## Profiles

| Profile | Shape | Use for |
|---|---|---|
| `SiteDisplay` | site, banner | default web display |
| `SiteVideo` | site, video | pre-roll, outstream |
| `SiteNative` | site, native | native asset rules |
| `AppRewarded` | app, video | in-app rewarded |
| `AppBanner` | app, banner | in-app display |
| `PmpDeals` | site, banner + pmp | deal priority, deal floors |
| `MultiImp` | site, mixed media | multi-impression auctions, truncation |
| `Sparse` | `id` + one banner `imp` | presence rules (missing device/site) |
| `Maximal` | every field the model can emit | the largest valid request |

`Maximal` alternates `site` and `app` with the seed: OpenRTB forbids both at once, so the seed
chooses which supply source the request keeps. Every other field appears in either variant.

## Determinism

Same seed + same config + same `rtbgen` version ⇒ byte-identical output. The PRNG is `rtbgen`'s
own (not `rand`, whose algorithm is not guaranteed stable across versions), key order is fixed,
and `corpus/v1/` is checked in and verified by the test suite.

**Changing the bytes of existing output is a semver-major change.**

The corpus lives in [`crates/rtbgen/corpus/v1`](https://github.com/rsbid/rtbgen/tree/main/crates/rtbgen/corpus/v1): `manifest.toml` names
every entry and the JSON files are the bytes. Regenerate them with `rtbgen corpus --write`;
never edit them by hand.

## Fuzzing and property tests

With the `arbitrary` feature, any byte slice becomes one request:

```rust
use arbitrary::{Arbitrary, Unstructured};

let mut input = Unstructured::new(bytes);
let request = rtbgen::Request::arbitrary(&mut input)?;
```

With the `proptest` feature, the strategy shrinks on the typed knobs:

```rust
proptest! {
    #[test]
    fn every_request_is_valid(request in rtbgen::proptest::request()) {
        let json = request.to_json_string();
        // parse and assert
    }
}
```

With the `serde` feature, `serde_json::to_string(&request)` is byte-identical to
`request.to_json_string()`. For your own deterministic source, implement `rtbgen::Entropy` and
call `rtbgen::Request::from_entropy(profile, spec, &mut source)`; such a request has no seed.

## Add to your AGENTS.md

If an agent writes tests in your repository, put this in its context:

```md
## OpenRTB tests
Fixtures are generated with `rtbgen` (dev-dependency); never hand-write bid-request JSON.
`Request::builder(Profile::X).seed(Seed::new(42)).build().to_json_string()`
The output is byte-stable; do not sort keys or post-process it.
```

## Installation

```toml
[dev-dependencies]
rtbgen = "0.2"
```

`to_json_string` needs the default `alloc` feature. `default-features = false` gives a `no_std`
writer: `write_json(&mut impl fmt::Write)`. The core has no unconditional runtime dependencies.

Optional features add adapters, never new bytes:

```toml
rtbgen = { version = "0.2", features = ["arbitrary", "proptest", "serde"] }
```

| Feature | Adds |
|---|---|
| `arbitrary` | `arbitrary::Arbitrary for Request`: every byte slice becomes one valid request |
| `proptest` | `rtbgen::proptest::request()`: a strategy that shrinks on the typed knobs |
| `serde` | `serde::Serialize for Request`: `serde_json::to_string` equals `to_json_string` |

## Command line

```text
rtbgen --profile app_rewarded --seed 42 --count 10000 --out ./requests/
rtbgen --profile pmp_deals --spec 2.5 --seed 42
rtbgen --list-profiles
rtbgen corpus --verify
```

`--count` increments the seed for each request, so one seed still reproduces the whole series.
`--out` writes `request-NNNNNN.json` files; without it the requests go to stdout.

## Roadmap

The 0.2 release adds the public `Entropy` seam with the `arbitrary`, `proptest` and `serde`
adapters; none of them changes the bytes of an existing profile. Planned next: the `degrade`
knob that emits one deliberate OpenRTB violation per request. The canonical snippet and the
bytes of existing profiles will not change when it lands.

## License

MIT OR Apache-2.0.
