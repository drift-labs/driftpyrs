# driftpyrs

> [!NOTE]
> This is an experimental project to create Python bindings for the drift-rs crate.
> It is not yet ready for production use.

The idea here is to create python bindings for the [drift-rs crate](https://github.com/drift-labs/drift-rs) using [pyo3](https://github.com/PyO3/pyo3) in python and slowly replace
most of [driftpy](https://github.com/drift-labs/driftpy)'s internals with the rust sdk, giving us a sort of a single source of truth for
the drift sdk, since the [drift-rs crate](https://github.com/drift-labs/drift-rs) uses the contract code thru [drift-ffi-sys](https://github.com/drift-labs/drift-ffi-sys).

## Development

This project uses [maturin](https://github.com/PyO3/maturin) to build the rust code and install it in the python environment, [uv](https://github.com/astral-sh/uv) to manage the python environment, and [cargo](https://github.com/rust-lang/cargo) for the rust code.

### Setup

Before running `uvx maturin develop`, make sure you have:

- Python 3.10+ available on your PATH
- Rust toolchain installed (via [rustup](https://rustup.rs/))
- [uv](https://github.com/astral-sh/uv) installed

Install rust dependencies required for the driftrs:

```bash
rustup install 1.85.0-x86_64-apple-darwin 1.76.0-x86_64-apple-darwin --force-non-host
rustup override set 1.85.0-x86_64-apple-darwin
```

Then install the Python dependencies:

```bash
uv sync
```

To build the project and install it in the python environment, run:

```bash
uvx maturin develop
```

### drift-ffi-sys + Rosetta / x86_64 on Apple Silicon

Some Drift oracle functionality in `drift-rs` goes through the `drift-ffi-sys` dynamic library and requires `libdrift_ffi_sys.dylib` to be linkable at build time and loadable at runtime. On Apple Silicon, the easiest dev path right now is to run **everything x86_64 under Rosetta** (x86_64 Python + x86_64 extension + x86_64 `libdrift_ffi_sys.dylib`).

One-time setup (Rosetta shell):

```bash
arch -x86_64 zsh

# x86_64 OpenSSL for the x86_64 target build
/usr/local/bin/brew install openssl@3 pkg-config

# Build drift-ffi-sys (produces libdrift_ffi_sys.dylib)
git clone https://github.com/drift-labs/drift-ffi-sys /tmp/drift-ffi-sys
cd /tmp/drift-ffi-sys
cargo build --release

# Make it discoverable without env vars (default link path used by build.rs)
ln -sf /tmp/drift-ffi-sys/target/release/libdrift_ffi_sys.dylib /usr/local/lib/libdrift_ffi_sys.dylib
```

After that, you can build with a single command:

```bash
./scripts/dev-rosetta.sh
```

Notes:

- `.cargo/config.toml` sets the `X86_64_APPLE_DARWIN_OPENSSL_*` env vars so `openssl-sys` can find x86_64 OpenSSL automatically.
- `build.rs` links `drift_ffi_sys` and defaults to `/usr/local/lib` if `CARGO_DRIFT_FFI_PATH` is not set (and hard-fails if the dylib is missing).
- `scripts/dev-rosetta.sh` auto-selects the uv-managed x86_64 Python interpreter and builds `--target x86_64-apple-darwin`.

To add new python dependencies, run:

```bash
uv add <dependency>
```

To add a new rust dependency, run:

```bash
cargo add <dependency>
```

## Development Notes

### Async

You have some async logs by enabling the `observability` feature. This will enable tracing and logging of the async operations. To add more, you'll need to add more tracing macros to the rust code.

```bash
maturin develop -F observability

 RUST_LOG=info python examples/<SCRIPT_NAME>.py
```

You can also install the `tokio-console` crate to get the console output for the async operations.
Make sure you build with the `tokio-console` feature enabled.

```bash
cargo install tokio-console
RUSTFLAGS="--cfg tokio_unstable" maturin develop -F tokio-console
```

You can check your current build info by running:

```python
import driftpyrs
print(driftpyrs.build_info())
```

To make sure you have the right flags enabled for development.

## Testing

You can test the async operations by running the `examples/test_async_bridge.py` file.
