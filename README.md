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
    .impressions_exact(ImpressionCount::new(2))
    .deals(Deals::new(3))
    .floor(Floor::cpm(1.50, Currency::USD))
    .build();

let json: String = req.to_json_string();
```

Same seed and config produce byte-identical output; the output is covered by a versioned
golden corpus. Every knob is a typed value, so `build()` cannot fail.

## Which call do I need?

| You want | Do this |
|---|---|
| A valid request for a test | `Request::builder(Profile::SiteDisplay).seed(Seed::new(1)).build()` |
| A specific shape (video, native, PMP, app, multi-impression) | pick the matching `Profile` |
| An OpenRTB 2.5 request | add `.spec(SpecVersion::V25)`; 2.6 is the default |
| Several impressions | `.impressions_exact(ImpressionCount::new(2))`: an exact count in 1..=3, over a profile's default |
| A private marketplace | `.deals(Deals::new(3))`, on any profile |
| A bid floor | `.floor(Floor::cpm(1.50, Currency::USD))` |
| A GDPR or COPPA scenario | `.privacy(Privacy::Gdpr { consent: Consent::Granted })`, `.privacy(Privacy::Coppa)` |
| A request that violates the spec | `.degrade(DegradeRate::percent(5))` |
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
| `MultiImp` | site, mixed media | multi-impression auctions, truncation (2..=3 imp by default) |
| `Sparse` | `id` + one banner `imp` | presence rules (missing device/site) |
| `Maximal` | every field the model can emit | the largest valid request |

`Maximal` alternates `site` and `app` with the seed: OpenRTB forbids both at once, so the seed
chooses which supply source the request keeps. Every other field appears in either variant.

## Privacy

`regs`, `user.consent` and `regs.gpp` are not drawn independently: one `Privacy` scenario owns
all of them, so the request cannot claim `gdpr=1` from the USA or send a GPP string that differs
from `user.consent`. For every profile that draws a request body (all except `Sparse` and
`Maximal`), the seed draws a scenario (no regulation, GDPR granted/denied/without consent, GDPR
disabled, or COPPA); `.privacy()` replaces it on any profile. The CLI takes the same names:
`--privacy gdpr_granted`, `--privacy coppa`, and so on.

- `Privacy::Gdpr { consent }` emits a syntactically valid TCF v2.2 TC string
  (`TcfPolicyVersion=4`, `IsServiceSpecific=1`, a terminated vendor section; `CmpId=0` marks it
  as a synthetic fixture), the matching `regs.gpp` and `gpp_sid`, and a location in the EU/UK.
  Under `OpenRTB` 2.5 the first-class fields do not exist, so the TC string and the flag travel
  in `user.ext.consent` and `regs.ext.gdpr`, the legacy locations 2.6 promoted. A denied string
  carries no storage identifiers: `device.ifa`, `user.id` and `user.buyeruid` are absent.
- `Privacy::Coppa` emits `coppa=1` with a US location, no `user`, no `device.ifa`, no
  `device.ip` and no precise location.
- `Privacy::GdprWithoutConsent` is `gdpr=1` with no TC string, the case a bidder must handle;
  it carries no storage identifiers. Under 2.5 the flag is in `regs.ext.gdpr`.
- `Privacy::GdprDisabled` and `Privacy::None` use non-EU locations on profiles with a device; the legacy
  `regs.ext.us_privacy` string appears only for a US request. `Privacy::ALL`, `name()` and
  `from_name()` are the stable set used by the CLI and the corpus manifest.

Locations carry their real UTC offset, region and UN/LOCODE, and the language follows the
country.

## Malformed requests

Parsers fail on real traffic, not only on valid fixtures. `.degrade(DegradeRate::percent(5))`
gives about one request in twenty **exactly one** deliberate OpenRTB violation; the other
requests keep the profile's bytes. `percent(0)` is byte-identical to not calling the knob, and
`percent(100)` always mutates. The mutation is drawn from the seed, so a seed and a rate
reproduce the same defect, and the document is always valid JSON.

| Mutation | What it breaks |
|---|---|
| `at` 0, 5 or 499 | auction-type code bounds |
| `bidfloor` as `"1.50"` | money typing |
| `bidfloorcur` `"RUR"`, `"usd"` or `""` | currency validation |
| `device.ip` as IPv6 | address family |
| `id` empty, blank or non-ASCII | empty vs blank id, byte length in parsers |
| `site` and `app` together | mutually exclusive supply sources |
| 4..=10 and 10 000 impressions | impression cap and oversized body |
| duplicate `imp[].id` | impression-id uniqueness |
| `tmax` 120000, 0 or absent | deadline cap and "zero = no deadline" |
| `user` without `consent`, or `consent` with a dot | presence rules and TCF decoding |
| negative or missing `banner.w`/`banner.h` | banner size |
| deeply nested `ext` | raw-JSON handling |
| `cur` as `"USD"` or `["US1"]` | list typing and currency validation |

`Mutation::ALL` is the closed set, and `request.mutation()` returns the one defect a request
carries. A mutation never breaks JSON: it breaks OpenRTB.

## Parser check

The mutation table was measured, not guessed. On 2026-11-21, `rtbgen` 0.3.0 generated 180
requests (nine profiles, both revisions, ten seeds each) and one accepted baseline per mutation
and implementation, searched over seeds 0..=499. Each implementation below parsed every request;
a mutation counts as detected only when its baseline was **accepted** before the mutation and
**rejected** after it.

| Implementation | Version | Valid requests accepted | Defects detected |
|---|---|---|---|
| [`rtblint-core`](https://crates.io/crates/rtblint-core) | 0.14.8 | 180/180 | 7/28 |
| [`openrtb26`](https://crates.io/crates/openrtb26) | 0.1.1 | 180/180 | 2/28 |
| [`iab-specs-openrtb`](https://crates.io/crates/iab-specs-openrtb) | 0.5.1 | 132/180 | 2/28 |
| [`openrtb`](https://crates.io/crates/openrtb) | 0.2.1 | 10/180 | 4/26 |

Read the table as a map of the layer each tool checks:

- `rtblint-core` is the only one that rejects `at: 0/5/499`, `bidfloor` as a string, both
  `site` and `app`, `tmax: 0` and `cur` as a string. It warns on, but accepts, `"usd"`/`""`
  in `bidfloorcur`, an implausible `tmax`, a malformed currency code and a missing or dotted
  `consent`. It is silent on IPv6 in `device.ip`, duplicate impression ids, an empty request
  id, a negative banner width and the three geo code formats.
- The serde parsers catch a wrong JSON type (`"1.50"` as money, `"USD"` instead of `["USD"]`)
  and little else. `openrtb26` accepts every request; `iab-specs-openrtb` adds two type checks;
  `openrtb` also rejects a missing `tmax` and a negative banner size.
- `rtblint-core` accepts all 180 valid requests; the gaps in its detection column are the
  semantic checks listed above, not missing fields.
- `openrtb` 0.2.1 rejects 170 of the 180 valid requests because its `User.data` has no serde
  default; its detection column covers the 26 mutations whose baseline it could parse.
  `iab-specs-openrtb` 0.5.1 reads nested `ext` and `pmp` as `Vec<u8>`, so map-valued extensions
  fail there regardless of validity.

This matrix is why 0.3.0 emits `device.ifa` (not `idfa`), a native request at the root,
`skipmin`/`skipafter`/`gpp_sid` only with their parent field, and `source.schain` only under 2.6.

## Determinism

Same seed + same config + same `rtbgen` version ⇒ byte-identical output, including the mutation
a degraded request carries. The PRNG is `rtbgen`'s own (not `rand`, whose algorithm is not
guaranteed stable across versions), key order is fixed, and `corpus/v1/` is checked in and
verified by the test suite.

**Changing the bytes of existing output is a semver-major change.**

The corpus lives in [`crates/rtbgen/corpus/v1`](https://github.com/rsbid/rtbgen/tree/main/crates/rtbgen/corpus/v1): `manifest.toml` names
every entry and the JSON files are the bytes. Regenerate them with `rtbgen corpus --write`;
never edit them by hand.

## Fuzzing and property tests

With the `arbitrary` feature, any byte slice becomes one request. The adapter also draws a
degrade rate, so a fuzz target covers malformed input as well as valid requests:

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
`request.to_json_string()`. `Serialize` targets `serde_json`: it embeds the writer's JSON as a
raw value, which only `serde_json` understands; another serializer sees a private marker, not the
request object. For your own deterministic source, implement `rtbgen::Entropy` and
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
rtbgen = "0.3"
```

`to_json_string` needs the default `alloc` feature. `default-features = false` gives a `no_std`
writer: `write_json(&mut impl fmt::Write)`. The core has no unconditional runtime dependencies.

Optional features add adapters, never new bytes:

```toml
rtbgen = { version = "0.3", features = ["arbitrary", "proptest", "serde"] }
```

| Feature | Adds |
|---|---|
| `arbitrary` | `arbitrary::Arbitrary for Request`: every byte slice becomes one valid request |
| `proptest` | `rtbgen::proptest::request()`: a strategy that shrinks on the typed knobs |
| `serde` | `serde::Serialize for Request` for `serde_json`: `to_string` equals `to_json_string`; other formats are not supported |

## Command line

```text
rtbgen --profile app_rewarded --seed 42 --count 10000 --out ./requests/
rtbgen --profile pmp_deals --spec 2.5 --seed 42
rtbgen --profile site_display --seed 42 --count 100 --degrade 5% --out ./malformed/
rtbgen --list-profiles
rtbgen corpus --verify
```

`--count` increments the seed for each request, so one seed still reproduces the whole series.
`--out` writes `request-NNNNNN.json` files; without it the requests go to stdout.

## Roadmap

The 0.3 release adds the `degrade` knob and its mutation table, and fixes the fields the parser
matrix and the OpenRTB 2.6 audit caught: `device.ifa` (not the non-spec `idfa`), the Native
Markup Request root, `skipmin`/`skipafter` only with `skip`, `gpp_sid` only with `gpp`, no
`source.schain.nodes[].n`, `source.schain` only under 2.6, and standard code lists for
`device.geo.country` (ISO-3166-1 alpha-3), `device.geo.region` (ISO-3166-2, 2-letter state code
for the USA), `device.geo.city` (UN/LOCODE) and `source.pchain` (TAG Payment ID). Existing
snapshots must be regenerated; `corpus/v1` is regenerated in place because no
version has been released. The `arbitrary` adapter now draws a degrade rate, which changes its
byte mapping too. After a period without behaviour changes, 1.0 freezes the API and the bytes.

## License

MIT OR Apache-2.0.
