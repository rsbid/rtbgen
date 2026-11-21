# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The emitted bytes are part of the public API: changing them for an existing
`(spec, profile, seed, knobs)` is a breaking change. `corpus/v1` is the initial
byte contract.

## [0.3.0] - 2026-11-20

The `degrade` knob, the closed mutation set, coherent GDPR/COPPA scenarios, and the fields the
parser matrix and the OpenRTB 2.6 audit caught. Existing profiles change bytes, so `corpus/v1` is
regenerated in place. No version has been released yet, so there is no second corpus contract.

### Added

- `DegradeRate` and `RequestBuilder::degrade`: about one request in the given share carries
  **exactly one** deliberate OpenRTB violation. `percent(0)` is byte-identical to not calling
  the knob, `percent(100)` always mutates, and the mutation is drawn after the profile body, so
  a request without a mutation keeps the plain bytes.
- `Mutation`, the closed set of 28 defects: auction-type bounds, `bidfloor` as a string, invalid
  `bidfloorcur`, IPv6 `device.ip`, empty/blank/non-ASCII request ids, `site` plus `app`,
  over-cap and 10 000-impression bodies, duplicate `imp[].id`, `tmax` 120000/0/absent, missing or
  dotted `consent`, negative or missing banner size, deeply nested `ext`, wrong-shaped `cur`,
  and the three geo code formats (`country` alpha-2, `city` name, `region` without its country
  prefix).
  `Mutation::ALL` lists the set and `Request::mutation()` names the defect a request carries.
- `Request::from_entropy_with(profile, spec, rate, entropy)` for a custom deterministic source.
- `Privacy` and `Consent`, and `RequestBuilder::privacy`: one scenario owns `regs`,
  `user.consent`, `regs.gpp`, the request location and whether identifiers may appear.
  `Privacy::Gdpr` emits a syntactically valid TCF v2.2 TC string (verified by a `lib_tcstring`
  dev-dependency; `CmpId` is 0, the string is a synthetic fixture), `Privacy::Coppa` drops the
  user, `device.ifa`, `device.ip` and precise geolocation, and `Privacy::GdprWithoutConsent` is
  the no-consent case a bidder must handle.
- `Privacy::ALL`, `Privacy::name()` and `Privacy::from_name()`: the stable snake_case scenario
  names (`none`, `coppa`, `gdpr_disabled`, `gdpr_granted`, `gdpr_denied`, `gdpr_no_consent`)
  used by the CLI and the corpus manifest.
- `--privacy <NAME>` in the CLI, `Privacy` in the `arbitrary` adapter, and the privacy scenario
  in the `proptest` strategy.
- `rtbgen corpus --write` writes `manifest.toml` as well as the JSON fixtures.
- `--degrade <PERCENT>` in the CLI, accepting `5` and `5%`.
- `examples/05_malformed_input.rs`.

### Changed (breaking for snapshots)

- `device.ifa`, not `device.idfa`: `idfa` is not an OpenRTB field, and `rtblint-core` rejects
  every request that carries it. Every profile that emits a device changes bytes.
- `imp.native.request` is the Native Markup Request root; the obsolete top-level `"native"`
  wrapper is gone.
- `imp.video.skipmin` and `skipafter` appear only when `skip` is set, as the spec requires.
- `regs.gpp_sid` appears only when `regs.gpp` is present, so the section ids are scoped.
- `source.schain.nodes[].n` is no longer emitted: OpenRTB 2.6 defines no `n` attribute on
  `SupplyChainNode`.
- `source.schain` appears only under OpenRTB 2.6: the `SupplyChain` object does not exist in 2.5,
  so `Maximal` loses it under `V25` and keeps it under `V26`.
- `device.geo.country` is an ISO-3166-1 alpha-3 code (`RUS`, `USA`, `DEU`, `GBR`, `FRA`, `ESP`,
  `ITA`, `NLD`), not alpha-2.
- `device.geo.city` is a UN/LOCODE (`RUMOW`, `USNYC`, `DEBER`, `GBLON`, `FRPAR`, `ESMAD`,
  `ITROM`, `NLAMS`), not a city name.
- `device.geo.region` uses ISO-3166-2, with the 2-letter state code for the USA as the spec
  requires (`RU-MOW`, `NY`, `DE-BE`, `GB-ENG`, `FR-IDF`, `ES-MD`, `IT-RM`, `NL-NH`).
- `source.pchain` is a valid TAG Payment ID chain (`XYZ01234:ABCD56789`), not `pchain-1`.
- Privacy is coherent instead of three independent draws. The TC strings are now syntactically
  valid TCF v2.2 (a `Created` equal to `LastUpdated`, `TcfPolicyVersion=4`, valid language and
  publisher country, a terminated vendor section; `CmpId=0` marks them as synthetic),
  `regs.gpp` always carries the same string as `user.consent`, `gpp_sid` always accompanies
  `gpp`, and `coppa=1` never carries identifiers. When GDPR applies, a request carries storage
  identifiers only with a positive consent signal: `device.ifa`, `user.id`, `user.buyeruid` and
  `user.ext.eids` are absent when consent is denied or missing, so a parser never sees
  identifiers it may not use. The GPP envelope uses the canonical
  `DBABM~…` header for version 1 with TCF EU v2 section id 2, not the previous `DBABMA~…`
  placeholder. `gdpr=1` uses an EU/UK location, non-GDPR traffic uses another location, and the
  legacy `us_privacy` string appears only for a US request. The location table gains `FRA`,
  `ESP`, `ITA` and `NLD`, and every location carries its real UTC offset and language.
  Under OpenRTB 2.5 the writer omits the first-class `user.consent`, `regs.gdpr`, `regs.gpp`
  and `regs.gpp_sid`, and carries the same signals in the legacy `user.ext.consent` and
  `regs.ext.gdpr` fields that 2.6 promoted; a `Privacy::Gdpr` override on 2.5 changes the
  location, emits `regs.coppa=0`, and keeps the identifiers only for a granted consent.
- `device.geo.utcoffset` follows the location (`180` for Moscow, `-300` for New York, `0` for
  London, `60` for the EU), instead of the constant `180` or the inconsistent `-180` fixture.
- `degrade(100)` no longer selects a mutation the request cannot carry: `TmaxMissing` requires
  a `tmax` and `ConsentMissing` requires a `consent`. `ConsentMissing` and `ConsentDotted` now
  reach OpenRTB 2.5 through `user.ext.consent`, so a seed can select a different defect than
  before.
- Three geo mutations join the set: `geo_country_alpha2`, `geo_city_name` and
  `geo_region_without_country`. The set grows from 25 to 28, so a seed can select a different
  defect than before; `corpus/v1` is regenerated in place.
- The corpus is regenerated in `corpus/v1`. There is no `corpus/v2`: the crate has not been
  released, so the initial contract is the only one. The privacy entries now cover `Maximal`,
  `Sparse`, a V25 GDPR override and a privacy + degrade combination. Regenerate `insta`
  snapshots: every existing `(spec, profile, seed, knobs)` tuple produced new bytes.
- The `arbitrary` adapter draws a degrade rate and a privacy scenario as typed knobs, so a fuzz
  input can produce a malformed or a specific privacy request. This changes the adapter's byte
  mapping for an existing input; the pinned uniform fixture keeps its bytes. The `proptest`
  strategy varies the rate and the scenario and shrinks them toward zero/absence.

### Notes

- The writer now carries the raw wire value a mutation needs (a number or a string, a code or a
  literal, a code array or a literal, a generated id or a literal). The typed variants write
  exactly the bytes they wrote before the union existed.
- A mutation never breaks JSON syntax: it breaks OpenRTB.
- The core still has no unconditional dependency and builds with `--no-default-features`.

## [0.2.0] - 2026-11-20

The entropy seam and the adapters that use it. No emitted byte changes: `corpus/v1` is
unchanged, and every existing `(spec, profile, seed, knobs)` tuple keeps its output.

### Added

- A public entropy seam: `Entropy`, `Share`, `Span`, `RngEntropy` and `BytesEntropy`.
  `Request::from_entropy(profile, spec, &mut source)` generates from any deterministic
  source through the same core as the seeded builder.
- The `arbitrary` feature: `arbitrary::Arbitrary for Request`. The adapter draws the typed
  knobs, feeds every remaining byte to `BytesEntropy`, and never panics on an empty or
  truncated input.
- The `proptest` feature: `rtbgen::proptest::request()`, a strategy that varies the profile,
  seed, revision and typed knobs and shrinks on them. The impression count, the deal count and
  the floor are optional, so the strategy also produces requests without a `pmp`.
- The `serde` feature: `serde::Serialize for Request`, targeting `serde_json`. Serialization
  reuses the writer's own JSON, so `serde_json::to_string(&request)` is byte-identical to
  `request.to_json_string()`; another serializer sees the raw-value marker, not the object.
- Byte goldens for the entropy adapters: `Request::from_entropy` and `arbitrary` output is
  pinned by a test, so a change to a byte source or to the draw order is a visible diff.

### Changed

- `Request::seed()` now returns `Option<Seed>`. It is `Some` for a request built from a
  seed and `None` for one generated from an entropy source (`from_entropy`, `arbitrary`).
- `RequestBuilder::impressions` is now `RequestBuilder::impressions_exact`. The method
  overrides a profile's default count (`MultiImp` picks `2..=3` by itself) instead of
  suggesting one; the name now says so.
- The minimum supported Rust version is 1.88, required by `proptest` 1.12.

### Notes

- The adapters are optional features; the core keeps zero unconditional runtime dependencies
  and still builds with `--no-default-features`.
- No feature changes the bytes of an existing request. The corpus test runs under every
  feature configuration.

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

[0.3.0]: https://github.com/rsbid/rtbgen/releases/tag/v0.3.0
[0.2.0]: https://github.com/rsbid/rtbgen/releases/tag/v0.2.0
[0.1.0]: https://github.com/rsbid/rtbgen/releases/tag/v0.1.0
