#!/usr/bin/env bash
# The canonical snippet must be character-for-character identical in every
# artifact that teaches the API. A divergence makes a model synthesise a third
# form instead of reusing the one it was given.
set -euo pipefail

cd "$(dirname "$0")/.."

extract() {
  awk '
    /^[[:space:]]*(\/\/! )?use rtbgen::\{/ { capture = 1 }
    capture {
      line = $0
      sub(/^[[:space:]]*\/\/! ?/, "", line)
      sub(/^[[:space:]]*/, "", line)
      if (line == "" || line ~ /^fn main/ || line ~ /^#\[/ || line ~ /^clippy::/ || line ~ /^reason =/ || line ~ /^\)\]/) next
      print line
    }
    capture && /to_json_string\(\);/ { exit }
  ' "$1"
}

reference=$(extract README.md)
if [[ -z "$reference" ]]; then
  echo "no canonical snippet found in README.md"
  exit 1
fi

for file in \
  llms.txt \
  skills/rtbgen/SKILL.md \
  crates/rtbgen/examples/01_quickstart.rs \
  crates/rtbgen/src/lib.rs
do
  other=$(extract "$file")
  if [[ "$other" != "$reference" ]]; then
    echo "canonical snippet differs in $file:"
    diff <(printf '%s\n' "$reference") <(printf '%s\n' "$other") || true
    exit 1
  fi
done

echo "canonical snippet is identical in every artifact"
