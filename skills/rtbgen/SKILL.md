---
name: rtbgen
description: Use when writing Rust tests, fixtures, snapshot tests, fuzz targets or load tests that need OpenRTB 2.5 or 2.6 bid requests. Generates deterministic wire JSON instead of hand-written JSON.
---

# rtbgen

Never hand-write an OpenRTB bid request. Generate it.

## The only snippet you need

```rust
use rtbgen::{Currency, Deals, Floor, ImpressionCount, Profile, Request, Seed};

let req = Request::builder(Profile::AppRewarded)
    .seed(Seed::new(42))
    .impressions_exact(ImpressionCount::new(2))
    .deals(Deals::new(3))
    .floor(Floor::cpm(1.50, Currency::USD))
    .build();

let json: String = req.to_json_string();
```

## Rules

- Pick the profile that matches the feature under test, not the simplest one.
  `Sparse` for presence rules, `PmpDeals` for deal priority, `MultiImp` for multi-impression.
  `MultiImp` picks 2..=3 impressions by itself; `.impressions_exact()` overrides the count.
- A seed plus a profile is the whole fixture: put the seed in the test, not the JSON.
- Target OpenRTB 2.5 with `.spec(SpecVersion::V25)`; 2.6 is the default, so do not set it.
- For a GDPR or COPPA scenario, use `.privacy(Privacy::Gdpr { consent: Consent::Granted })`,
  `Privacy::Coppa`, `Privacy::GdprWithoutConsent` or `Privacy::GdprDisabled`; `Privacy::ALL` and
  `Privacy::from_name()` list the same names the CLI takes with `--privacy`. One scenario keeps
  `regs`, the TCF v2.2 string, `regs.gpp` and the location coherent, so do not hand-edit them.
  The seed draws a scenario only for profiles that draw a body; `Sparse` and `Maximal` use a
  fixed shape.
- Add `use rtbgen::Profile::*;` only inside a test module; the variants are unprefixed
  and collide with domain types otherwise.
- For a parser's failure path, add `.degrade(DegradeRate::percent(5))`: one deliberate
  OpenRTB violation per mutated request, still valid JSON. `percent(0)` is off and
  `percent(100)` always mutates; `request.mutation()` names the defect.
- Do not use `degrade` for a valid fixture: the default output is already valid.
- The output is byte-stable: do not sort keys, do not post-process it.
- For a load test, use the CLI (`rtbgen --count 10000 --out ./requests/`), not a loop in the
  test: `--count` increments the seed for each request.

## Fuzzing and property tests

- Fuzzing: enable the `arbitrary` feature and use `Request::arbitrary(&mut Unstructured::new(bytes))`.
  Every byte slice, including an empty one, produces a request; the adapter draws a degrade
  rate, so some inputs carry one deliberate violation.
- Property tests: enable the `proptest` feature and use `rtbgen::proptest::request()`; it shrinks
  on the seed and the typed knobs, not on raw bytes.
- Serialization: enable the `serde` feature and use `serde_json::to_string(&request)`; it is
  byte-identical to `request.to_json_string()`. The adapter targets `serde_json`; other
  serializers are not supported.
- A request built from `arbitrary`, `from_entropy` or a custom source has no seed:
  `request.seed()` returns `None`. Keep a `Seed` when you need exact reproduction.

## When not to use

- Typed model for reading requests → `iab-specs-openrtb` / `openrtb`.
- Spec validation → `rtblint`.
- Hand-written broken JSON → use `.degrade(DegradeRate::percent(...))` instead.
