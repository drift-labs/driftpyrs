#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -m)" != "x86_64" ]]; then
  exec arch -x86_64 /usr/bin/env bash "$0" "$@"
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

UV_PYTHON="$(ls -1 "$HOME/.local/share/uv/python"/cpython-*-macos-x86_64-none/bin/python3.* 2>/dev/null | tail -n 1 || true)"
if [[ -z "${UV_PYTHON}" ]]; then
  echo "no uv-managed x86_64 python found. run: uv python install cpython-3.13.11-macos-x86_64-none" >&2
  exit 1
fi

if [[ ! -f /usr/local/lib/libdrift_ffi_sys.dylib ]]; then
  echo "missing /usr/local/lib/libdrift_ffi_sys.dylib (needed for oracle FFI)" >&2
  echo "build drift-ffi-sys and symlink it:" >&2
  echo "  ln -sf /tmp/drift-ffi-sys/target/release/libdrift_ffi_sys.dylib /usr/local/lib/libdrift_ffi_sys.dylib" >&2
  exit 1
fi

rustup target add x86_64-apple-darwin >/dev/null

exec uvx maturin develop \
  --target x86_64-apple-darwin \
  "$@"
