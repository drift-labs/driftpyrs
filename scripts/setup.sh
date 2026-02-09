#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------------------------------------------
# scripts/setup.sh — one-command dev setup for driftpyrs
#
# Usage:  ./scripts/setup.sh
#
# Idempotent: safe to re-run. Detects platform, installs prerequisites,
# downloads the drift-ffi-sys shared library, builds the extension, and
# verifies the install.
# ---------------------------------------------------------------------------

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Pinned drift-ffi-sys version (update manually when bumping drift-rs)
FFI_VERSION="v2.156.3"
FFI_REPO="drift-labs/drift-ffi-sys"

# ---------- helpers --------------------------------------------------------

info()  { printf "\033[1;34m==> %s\033[0m\n" "$*"; }
warn()  { printf "\033[1;33mWARN: %s\033[0m\n" "$*" >&2; }
die()   { printf "\033[1;31mERROR: %s\033[0m\n" "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "'$1' not found. Please install it first."
}

# ---------- detect platform ------------------------------------------------

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) PLATFORM="macos" ;;
  Linux)  PLATFORM="linux" ;;
  *)      die "Unsupported OS: $OS" ;;
esac

info "Platform: $PLATFORM / $ARCH"

# ---------- step 1: prerequisites ------------------------------------------

need_cmd curl
need_cmd uv

# Rust / rustup
if ! command -v rustup >/dev/null 2>&1; then
  info "Installing rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

if [[ "$PLATFORM" == "macos" && "$ARCH" == "arm64" ]]; then
  # -- Apple Silicon specifics --

  # Rosetta 2
  if ! /usr/bin/pgrep -q oahd 2>/dev/null; then
    info "Installing Rosetta 2..."
    softwareupdate --install-rosetta --agree-to-license || true
  fi

  # x86_64 Rust toolchain
  info "Ensuring x86_64 Rust toolchain..."
  rustup target add x86_64-apple-darwin 2>/dev/null || true
  rustup install stable-x86_64-apple-darwin --force-non-host 2>/dev/null || true

  # x86_64 Python via uv
  X86_PYTHON="$(ls -1d "$HOME/.local/share/uv/python"/cpython-*-macos-x86_64-none/bin/python3.* 2>/dev/null | tail -n 1 || true)"
  if [[ -z "$X86_PYTHON" ]]; then
    info "Installing x86_64 Python via uv..."
    uv python install cpython-3.13-macos-x86_64-none
  fi

  # Ensure venv uses x86_64 Python (required for x86_64 extension to load)
  info "Syncing venv with x86_64 Python..."
  uv sync --python cpython-3.13-macos-x86_64-none
fi

# ---------- step 2: drift-ffi-sys shared library ---------------------------

if [[ "$PLATFORM" == "macos" ]]; then
  FFI_LIB="/usr/local/lib/libdrift_ffi_sys.dylib"
  FFI_EXT="dylib"
  FFI_ASSET="libdrift_ffi_sys.dylib"
else
  FFI_LIB="/usr/local/lib/libdrift_ffi_sys.so"
  FFI_EXT="so"
  FFI_ASSET="libdrift_ffi_sys.so"
fi

if [[ -f "$FFI_LIB" ]]; then
  info "drift-ffi-sys already present at $FFI_LIB"
  # Check if install name needs fixing (pre-built has CI paths)
  if [[ "$PLATFORM" == "macos" ]]; then
    CURRENT_ID=$(otool -D "$FFI_LIB" | tail -1)
    if [[ "$CURRENT_ID" != "$FFI_LIB" ]]; then
      info "Fixing dylib install name..."
      sudo install_name_tool -id "$FFI_LIB" "$FFI_LIB"
    fi
  fi
else
  DOWNLOAD_URL="https://github.com/$FFI_REPO/releases/download/$FFI_VERSION/$FFI_ASSET"
  info "Downloading drift-ffi-sys $FFI_VERSION ($FFI_ASSET)..."

  TMPFILE="$(mktemp)"
  HTTP_CODE=$(curl -sL -o "$TMPFILE" -w "%{http_code}" "$DOWNLOAD_URL")

  if [[ "$HTTP_CODE" != "200" ]]; then
    rm -f "$TMPFILE"
    if [[ "$PLATFORM" == "macos" ]]; then
      die "No pre-built macOS dylib available for drift-ffi-sys $FFI_VERSION.
Build from source:
  git clone https://github.com/$FFI_REPO /tmp/drift-ffi-sys
  cd /tmp/drift-ffi-sys && cargo build --release
  sudo cp /tmp/drift-ffi-sys/target/release/libdrift_ffi_sys.dylib /usr/local/lib/"
    else
      die "Failed to download drift-ffi-sys (HTTP $HTTP_CODE). URL: $DOWNLOAD_URL"
    fi
  fi

  info "Installing $FFI_ASSET to /usr/local/lib (may require sudo)..."
  sudo mkdir -p /usr/local/lib
  sudo mv "$TMPFILE" "$FFI_LIB"
  sudo chmod 755 "$FFI_LIB"

  # Fix the dylib's install name (the pre-built has CI runner paths baked in)
  if [[ "$PLATFORM" == "macos" ]]; then
    info "Fixing dylib install name..."
    sudo install_name_tool -id "$FFI_LIB" "$FFI_LIB"
  fi
fi

# On Linux, ensure the linker can find the library
if [[ "$PLATFORM" == "linux" ]]; then
  export CARGO_DRIFT_FFI_PATH="/usr/local/lib"
  if ! ldconfig -p 2>/dev/null | grep -q libdrift_ffi_sys; then
    info "Running ldconfig..."
    sudo ldconfig /usr/local/lib 2>/dev/null || true
  fi
fi

# ---------- step 3: build the extension ------------------------------------

info "Building driftpyrs extension..."

if [[ "$PLATFORM" == "macos" && "$ARCH" == "arm64" ]]; then
  # Cross-compile to x86_64 (uvx/maturin are arm64, but cargo cross-compiles)
  uvx maturin develop --target x86_64-apple-darwin
else
  uvx maturin develop
fi

# Fix the extension's reference to libdrift_ffi_sys (pre-built dylib has CI paths)
if [[ "$PLATFORM" == "macos" ]]; then
  # Find the built extension (works for any Python version)
  EXT_PATH=$(find "$ROOT/python/driftpyrs" -name "_driftpyrs*.so" -type f 2>/dev/null | head -1)
  if [[ -n "$EXT_PATH" && -f "$EXT_PATH" ]]; then
    # Find the old path baked into the extension
    OLD_PATH=$(otool -L "$EXT_PATH" | grep libdrift_ffi_sys | awk '{print $1}')
    if [[ -n "$OLD_PATH" && "$OLD_PATH" != "$FFI_LIB" ]]; then
      info "Fixing extension's dylib reference..."
      install_name_tool -change "$OLD_PATH" "$FFI_LIB" "$EXT_PATH"
    fi
  fi
fi

# ---------- step 4: verify -------------------------------------------------

info "Verifying installation..."

# On Apple Silicon, the venv now has x86_64 Python, so uv run will use it
if uv run python -c "import driftpyrs; print('driftpyrs loaded:', driftpyrs.build_info())"; then
  info "Setup complete!"
else
  die "Verification failed. Check the output above for errors."
fi
