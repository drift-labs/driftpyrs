# driftpyrs

> [!NOTE]
> This is an experimental project to create Python bindings for the drift-rs crate.
> It is not yet ready for production use.

The idea here is to create python bindings for the [drift-rs crate](https://github.com/drift-labs/drift-rs) using [pyo3](https://github.com/PyO3/pyo3) in python and slowly replace
most of [driftpy](https://github.com/drift-labs/driftpy)'s internals with the rust sdk, giving us a sort of a single source of truth for
the drift sdk, since the [drift-rs crate](https://github.com/drift-labs/drift-rs) uses the contract code thru [drift-ffi-sys](https://github.com/drift-labs/drift-ffi-sys).

## Usage

You can just ignore the rust part of the project and use driftpy as you would normally if you want.

```python
from driftpyrs import driftpy

print(driftpy.get_vault_program_id())
```

## Development

This project uses [maturin](https://github.com/PyO3/maturin) to build the rust code and install it in the python environment, [uv](https://github.com/astral-sh/uv) to manage the python environment, and [cargo](https://github.com/rust-lang/cargo) for the rust code.

To build the project and install it in the python environment, run:

```bash
uvx maturin develop
```

To add new python dependencies, run:

```bash
uv add <dependency>
```

To add a new rust dependency, run:

```bash
cargo add <dependency>
```
