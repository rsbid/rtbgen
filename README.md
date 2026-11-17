# rtbgen

Deterministic OpenRTB 2.5/2.6 bid request generator for tests, fuzzing and load runs.

This repository is the skeleton: the Cargo workspace, the profile, seed and spec-version
types, and the CLI entry point. The request model, writer, corpus and `degrade` land next.

Both 2.x revisions share one wire model: 2.5 drops the fields the 2.6 revision added, and 2.6
is the default. OpenRTB 3.0 is a different object model and is planned for a future version of
this project; it will not change 2.x output.

The core crate is `no_std` and has no unconditional runtime dependencies.
