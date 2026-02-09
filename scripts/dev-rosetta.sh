#!/usr/bin/env bash
set -euo pipefail

# Quick build script for Apple Silicon. For full setup, run ./scripts/setup.sh first.
# This cross-compiles to x86_64 without needing Rosetta for the build tools.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

exec uvx maturin develop \
  --target x86_64-apple-darwin \
  "$@"
