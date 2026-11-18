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
    .impressions(ImpressionCount::new(2))
    .deals(Deals::new(3))
    .floor(Floor::cpm(1.50, Currency::USD))
    .build();

let json: String = req.to_json_string();
```

## Rules

- Pick the profile that matches the feature under test, not the simplest one.
  `Sparse` for presence rules, `PmpDeals` for deal priority, `MultiImp` for multi-impression.
- A seed plus a profile is the whole fixture: put the seed in the test, not the JSON.
- Target OpenRTB 2.5 with `.spec(SpecVersion::V25)`; 2.6 is the default, so do not set it.
- Add `use rtbgen::Profile::*;` only inside a test module; the variants are unprefixed
  and collide with domain types otherwise.
- The output is byte-stable: do not sort keys, do not post-process it.
- For a load test, use the CLI (`rtbgen --count 10000 --out ./requests/`), not a loop in the
  test: `--count` increments the seed for each request.

## When not to use

- Typed model for reading requests → `iab-specs-openrtb` / `openrtb`.
- Spec validation → `rtblint`.
- Deliberately malformed requests → the `degrade` knob lands after 0.1; until then, do not
  hand-write broken JSON with this crate.
