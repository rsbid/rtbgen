# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The emitted bytes are part of the public API: changing them for an existing
`(spec, profile, seed, knobs)` is a breaking change. `corpus/v1` is the initial
byte contract.

## [0.1.0] - 2026-11-18

The first release: valid OpenRTB 2.5/2.6 requests for every shape.

### Added

- `Request::builder(Profile)` with the typed knobs `Seed`, `SpecVersion`,
  `ImpressionCount`, `Deals`, `Floor` and `Currency`.
- Nine profiles: `SiteDisplay`, `SiteVideo`, `SiteNative`, `AppRewarded`,
  `AppBanner`, `PmpDeals`, `MultiImp`, `Sparse`, `Maximal`. Knobs widen a
  profile instead of conflicting with it, so `build()` is infallible.
- A `no_std` writer with a fixed key order: `write_json(&mut impl fmt::Write)`
  and `to_json_string()` under the default `alloc` feature.
- The versioned golden corpus `corpus/v1` with a manifest, verified byte for
  byte by the test suite, plus `rtbgen::corpus::ALL`.
- The `rtbgen` command-line interface: profiles, seeds, counts, impressions,
  deals, floors, currencies, 2.5/2.6, stdout or a directory of files, and
  `rtbgen corpus --verify|--write`.
- `README.md`, `llms.txt`, `skills/rtbgen/SKILL.md` and four examples sharing
  one canonical snippet.

### Notes

- `degrade`, the `arbitrary`/`proptest` adapters and the `serde` feature are
  planned for later releases. Adding them will not change the bytes of the
  profiles shipped here.
- `Maximal` alternates `site` and `app` with the seed, because OpenRTB forbids
  both supply sources in one request.

[0.1.0]: https://github.com/rsbid/rtbgen/releases/tag/v0.1.0
