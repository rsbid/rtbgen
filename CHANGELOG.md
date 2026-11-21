# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The emitted bytes are part of the public API: changing them for an existing
`(spec, profile, seed, knobs)` is a breaking change. `corpus/v1` is the initial
byte contract.

## [0.1.1] - 2026-11-21

### Fixed

- The installation snippets in `README.md` asked for `rtbgen = "0.3"`, a version that never
  existed; `0.1.0` is the first published version. The roadmap no longer describes that release
  as upcoming, and it states that `corpus/v1` is the published byte contract. No emitted byte
  changes: `corpus/v1` is unchanged.

## [0.1.0] - 2026-11-21

The first public release: valid OpenRTB 2.5/2.6 requests for every shape, the typed builder, the
`degrade` knob, coherent privacy scenarios, and the `arbitrary`, `proptest` and `serde` adapters.
`corpus/v1` is the initial byte contract; there is no earlier published version, so there is no
second contract.

### Added

- `Request::builder(Profile)` with the typed knobs `Seed`, `SpecVersion`, `ImpressionCount`,
  `Deals`, `Floor` and `Currency`. Knobs widen a profile instead of conflicting with it, so
  `build()` is infallible.
- Nine profiles: `SiteDisplay`, `SiteVideo`, `SiteNative`, `AppRewarded`, `AppBanner`, `PmpDeals`,
  `MultiImp`, `Sparse`, `Maximal`.
- A `no_std` writer with a fixed key order: `write_json(&mut impl fmt::Write)` and
  `to_json_string()` under the default `alloc` feature.
- The versioned golden corpus `corpus/v1` with a manifest, verified byte for byte by the test
  suite, plus `rtbgen::corpus::ALL`.
- The `rtbgen` command-line interface: profiles, seeds, counts, impressions, deals, floors,
  currencies, 2.5/2.6, stdout or a directory of files, and `rtbgen corpus --verify|--write`.
- `README.md`, `llms.txt`, `skills/rtbgen/SKILL.md` and six examples sharing one canonical
  snippet.
- A public entropy seam: `Entropy`, `Share`, `Span`, `RngEntropy` and `BytesEntropy`.
  `Request::from_entropy(profile, spec, &mut source)` generates from any deterministic source
  through the same core as the seeded builder.
- `Request::from_entropy_with(profile, spec, rate, entropy)` for a custom deterministic source
  and a degrade rate.
- The `arbitrary` feature: `arbitrary::Arbitrary for Request`. The adapter draws the typed knobs,
  feeds every remaining byte to `BytesEntropy`, and never panics on an empty or truncated input.
- The `proptest` feature: `rtbgen::proptest::request()`, a strategy that varies the profile, seed,
  revision and typed knobs and shrinks on them. The impression count, the deal count and the floor
  are optional, so the strategy also produces requests without a `pmp`.
- The `serde` feature: `serde::Serialize for Request`, targeting `serde_json`. Serialization
  reuses the writer's own JSON, so `serde_json::to_string(&request)` is byte-identical to
  `request.to_json_string()`; another serializer sees the raw-value marker, not the object.
- Byte goldens for the entropy adapters: `Request::from_entropy` and `arbitrary` output is pinned
  by a test, so a change to a byte source or to the draw order is a visible diff.
- `DegradeRate` and `RequestBuilder::degrade`: about one request in the given share carries
  **exactly one** deliberate OpenRTB violation. `percent(0)` is byte-identical to not calling the
  knob, `percent(100)` always mutates, and the mutation is drawn after the profile body, so a
  request without a mutation keeps the plain bytes.
- `Mutation`, the closed set of 28 defects: auction-type bounds, `bidfloor` as a string, invalid
  `bidfloorcur`, IPv6 `device.ip`, empty/blank/non-ASCII request ids, `site` plus `app`, over-cap
  and 10 000-impression bodies, duplicate `imp[].id`, `tmax` 120000/0/absent, missing or dotted
  `consent`, negative or missing banner size, deeply nested `ext`, wrong-shaped `cur`, and the
  three geo code formats (`country` alpha-2, `city` name, `region` without its country prefix).
  `Mutation::ALL` lists the set and `Request::mutation()` names the defect a request carries.
- `Privacy` and `Consent`, and `RequestBuilder::privacy`: one scenario owns `regs`,
  `user.consent`, `regs.gpp`, the request location and whether identifiers may appear.
  `Privacy::Gdpr` emits a syntactically valid TCF v2.2 TC string (verified by a `lib_tcstring`
  dev-dependency; `CmpId` is 0, the string is a synthetic fixture), `Privacy::Coppa` drops the
  user, `device.ifa`, `device.ip` and precise geolocation, and `Privacy::GdprWithoutConsent` is
  the no-consent case a bidder must handle.
- `Privacy::ALL`, `Privacy::name()` and `Privacy::from_name()`: the stable snake_case scenario
  names (`none`, `coppa`, `gdpr_disabled`, `gdpr_granted`, `gdpr_denied`, `gdpr_no_consent`) used
  by the CLI and the corpus manifest.
- `DeprecatedFields` and `RequestBuilder::deprecated_fields`: the default `Omit` drops fields the
  target revision deprecated and writes the revision's replacement instead; `Include` keeps the
  deprecated field next to it. OpenRTB 2.5 bytes do not depend on the policy.
- CLI flags for every typed knob: `--privacy <NAME>`, `--degrade <PERCENT>` (accepting `5` and
  `5%`) and `--include-deprecated`. `rtbgen corpus --write` writes `manifest.toml` as well as the
  JSON fixtures.
- `Privacy` in the `arbitrary` adapter, and the privacy scenario in the `proptest` strategy.
- `examples/05_malformed_input.rs`.

### Wire format

- `device.ifa`, not `device.idfa`: `idfa` is not an OpenRTB field, and `rtblint-core` rejects
  every request that carries it.
- `imp.native.request` is the Native Markup Request root; there is no top-level `"native"`
  wrapper.
- `imp.video.skipmin` and `skipafter` appear only when `skip` is set, as the spec requires.
- `regs.gpp_sid` appears only when `regs.gpp` is present, so the section ids are scoped.
- `source.schain.nodes[].n` is not emitted: OpenRTB 2.6 defines no `n` attribute on
  `SupplyChainNode`.
- `source.schain` appears only under OpenRTB 2.6, because the `SupplyChain` object does not exist
  in 2.5: `Maximal` emits it under `V26` only.
- `source.schain` follows Supply Chain 1.1 as of OpenRTB 2.6-202611: `ver` is `"1.1"`, and the
  one-node maximal chain declares `complete: 2` (payment and non-payment complete).
- `device.geo.country` is an ISO-3166-1 alpha-3 code (`RUS`, `USA`, `DEU`, `GBR`, `FRA`, `ESP`,
  `ITA`, `NLD`).
- `device.geo.city` is a UN/LOCODE (`RUMOW`, `USNYC`, `DEBER`, `GBLON`, `FRPAR`, `ESMAD`,
  `ITROM`, `NLAMS`).
- `device.geo.region` uses ISO-3166-2, with the 2-letter state code for the USA as the spec
  requires (`RU-MOW`, `NY`, `DE-BE`, `GB-ENG`, `FR-IDF`, `ES-MD`, `IT-RM`, `NL-NH`).
- `source.pchain` is a valid TAG Payment ID chain (`XYZ01234:ABCD56789`).
- OpenRTB 2.6 video impressions write `imp.video.plcmt`, the field that replaced the deprecated
  `imp.video.placement` in 2.6-202303. The legacy value is mapped onto the new list (`In-Stream` →
  `Instream`, `Interstitial` → `Interstitial`, `In-Article`/`In-Feed` →
  `No Content/Standalone`). OpenRTB 2.5 keeps `placement`, which 2.6 deprecated.
  `.deprecated_fields(DeprecatedFields::Include)` (CLI `--include-deprecated`) also emits
  `placement` next to `plcmt`, the migration pattern for a parser that still reads it. `Maximal`
  follows the policy.
- Privacy is coherent: one `Privacy` scenario owns `regs`, `user.consent`, `regs.gpp`, the
  request location and identifier eligibility. The TC strings are syntactically valid TCF v2.2 (a
  `Created` equal to `LastUpdated`, `TcfPolicyVersion=4`, valid language and publisher country, a
  terminated vendor section; `CmpId=0` marks them as synthetic), `regs.gpp` carries the same
  string as `user.consent`, `gpp_sid` accompanies `gpp`, and `coppa=1` never carries identifiers.
  When GDPR applies, a request carries storage identifiers only with a positive consent signal:
  `device.ifa`, `user.id`, `user.buyeruid` and `user.ext.eids` are absent when consent is denied
  or missing, so a parser never sees identifiers it may not use. The GPP envelope uses the
  canonical `DBABM~…` header for version 1 with TCF EU v2 section id 2. `gdpr=1` uses an EU/UK
  location, non-GDPR traffic uses another location, and the legacy `us_privacy` string appears
  only for a US request. Every location carries its real UTC offset and language. Under OpenRTB 2.5
  the writer omits the first-class `user.consent`, `regs.gdpr`, `regs.gpp` and `regs.gpp_sid`, and
  carries the same signals in the legacy `user.ext.consent` and `regs.ext.gdpr` fields that 2.6
  promoted; a `Privacy::Gdpr` override on 2.5 changes the location, emits `regs.coppa=0`, and
  keeps the identifiers only for a granted consent.
- `device.geo.utcoffset` follows the location (`180` for Moscow, `-300` for New York, `0` for
  London, `60` for the EU).
- `degrade(100)` selects only a mutation the request can carry: `TmaxMissing` requires a `tmax`
  and `ConsentMissing` requires a `consent`. `ConsentMissing` and `ConsentDotted` reach OpenRTB
  2.5 through `user.ext.consent`.

### API

- `Request::seed()` returns `Option<Seed>`: `Some` for a request built from a seed and `None` for
  one generated from an entropy source (`from_entropy`, `arbitrary`).
- `RequestBuilder::impressions_exact` overrides a profile's default count (`MultiImp` picks
  `2..=3` by itself) instead of suggesting one; the name says so.
- The `arbitrary` adapter draws a degrade rate and a privacy scenario as typed knobs, so a fuzz
  input can produce a malformed or a specific privacy request. The `proptest` strategy varies the
  rate, the scenario and the deprecated-field policy and shrinks them toward zero/absence; the
  `arbitrary` adapter uses the default policy and does not draw it.

### Notes

- `corpus/v1` is the only byte contract. The privacy entries cover `Maximal`, `Sparse`, a V25
  GDPR override and a privacy + degrade combination; `knobs-deprecated-site-video-v26-6` is the
  `Include` twin of `knobs-site-video-v26-6`, differing only by the kept `placement`. Regenerate
  the corpus with `rtbgen corpus --write` and review the diff as a byte change of the public API;
  never hand-edit the JSON fixtures.
- The writer carries the raw wire value a mutation needs (a number or a string, a code or a
  literal, a code array or a literal, a generated id or a literal).
- A mutation never breaks JSON syntax: it breaks OpenRTB.
- The core has no unconditional dependency and builds with `--no-default-features`; the adapters
  are optional features. The corpus test runs under every feature configuration.
- The minimum supported Rust version is 1.88, required by `proptest` 1.12.
- `Maximal` alternates `site` and `app` with the seed, because OpenRTB forbids both supply
  sources in one request.

[0.1.1]: https://github.com/rsbid/rtbgen/releases/tag/v0.1.1
[0.1.0]: https://github.com/rsbid/rtbgen/releases/tag/v0.1.0
