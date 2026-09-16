#!/usr/bin/env bash
# Fail the release when the tag version and the package version disagree.
set -euo pipefail

EXPECTED="${1:?usage: check-version-sync.sh <version>}"
MANIFEST="crates/create_rust_app/Cargo.toml"

if [ ! -f "$MANIFEST" ]; then
  echo "Missing $MANIFEST" >&2
  exit 1
fi

ACTUAL="$(grep -E '^version = ' "$MANIFEST" | head -n 1 | sed -E 's/^version = \"([^\"]+)\"/\1/')"

if [ "$ACTUAL" != "$EXPECTED" ]; then
  echo "Version mismatch: tag wants $EXPECTED but $MANIFEST declares $ACTUAL" >&2
  exit 1
fi

echo "Version sync OK: $ACTUAL"
