#!/usr/bin/env bash
# Every gate that CI runs, in the order CI runs them.
#
# Requires `cargo-machete` and `cargo-deny` (cargo install cargo-machete cargo-deny)
# and the toolchain named by `rust-version` in Cargo.toml.
#
# Overrides:
#   MSRV=1.90 ./ci/local-gates.sh   use another toolchain for the MSRV check
set -euo pipefail

cd "$(dirname "$0")/.."

msrv="${MSRV:-$(sed -n 's/^rust-version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)}"
if [[ -z "$msrv" ]]; then
  echo "no rust-version in Cargo.toml"
  exit 1
fi

echo "== snippet"
bash ci/check-snippet.sh

echo "== fmt"
cargo fmt --all --check

echo "== clippy"
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

echo "== test"
cargo test --workspace --all-features --locked --no-fail-fast

echo "== no_std"
cargo build -p rtbgen --no-default-features --locked
cargo test -p rtbgen --no-default-features --features alloc --locked

echo "== feature matrix"
cargo test -p rtbgen --no-default-features --features arbitrary --locked
cargo test -p rtbgen --no-default-features --features proptest --locked
cargo test -p rtbgen --no-default-features --features serde --locked
cargo test -p rtbgen --test corpus --no-default-features --features arbitrary --locked
cargo test -p rtbgen --test corpus --no-default-features --features proptest --locked
cargo test -p rtbgen --test corpus --no-default-features --features serde --locked

echo "== msrv $msrv"
cargo "+$msrv" check --workspace --all-features --locked

echo "== dependencies"
cargo machete --with-metadata
cargo deny check

echo "== docs"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked

echo "all gates passed"
