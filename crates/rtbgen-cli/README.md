# rtbgen-cli

Command-line interface for [`rtbgen`](https://crates.io/crates/rtbgen), a deterministic
OpenRTB 2.5/2.6 bid request generator.

## Installation

```bash
cargo install rtbgen-cli
```

The package installs one binary, `rtbgen`.

## Usage

```text
rtbgen --profile site_display --seed 42
rtbgen --profile app_rewarded --seed 42 --count 10000 --out ./requests/
rtbgen --profile site_display --seed 42 --count 100 --degrade 5% --out ./malformed/
rtbgen --profile site_video --spec 2.5 --privacy gdpr_granted
rtbgen --profile pmp_deals --seed 7 --deals 3 --floor 1.50 --currency EUR
rtbgen --list-profiles
rtbgen corpus --verify
```

`rtbgen --help` lists every flag: profile and revision, seed and count, impression and deal
counts, floor and currency, the privacy scenario, the degrade rate, the deprecated-field policy,
and `--out` for a directory of `request-NNNNNN.json` files.

`--count` increments the seed for each request, so one seed reproduces the whole series.
`corpus --verify` regenerates the golden corpus in memory and compares it byte for byte;
`corpus --write` rewrites it after a reviewed generator change.

## Relationship to the library

The binary and the library share one generator: `rtbgen --profile X --seed N` writes the same
bytes as `Request::builder(Profile::X).seed(Seed::new(N))`. Use the library when the fixtures are
generated in-process; use the binary when another tool needs files.

## License

MIT OR Apache-2.0.
