#!/usr/bin/env bash
# Usage: set-version.sh <version>
# Sets the [package] version in Cargo.toml (run from the repository root).
set -euo pipefail

VERSION="${1:?usage: set-version.sh <version>}"

sed -i.bak '/^\[package\]/,/^\[/ s/^version = ".*"/version = "'"$VERSION"'"/' Cargo.toml
rm -f Cargo.toml.bak
grep -qx "version = \"$VERSION\"" Cargo.toml || { echo "ERROR: failed to set version in Cargo.toml"; exit 1; }
