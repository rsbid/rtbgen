#!/usr/bin/env bash
# Every gate that CI runs, in the order CI runs them.
#
# Overrides:
#   MSRV=1.85 ./ci/local-gates.sh    also checks the minimum supported Rust version
set -euo pipefail

cd "$(dirname "$0")/.."

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

if [[ -n "${MSRV:-}" ]]; then
  echo "== msrv $MSRV"
  cargo "+$MSRV" check --workspace --all-features --locked
fi

echo "== docs"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked

echo "all gates passed"
