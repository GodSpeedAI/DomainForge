# domainforge-python — Python Distribution

## Overview
Distribution root for the PyPI package `domainforge` (version `0.21.2`,
`requires-python = ">=3.11"`, Apache-2.0). `pyproject.toml` pins the build
backend to `maturin` (`requires = ["maturin>=1.7,<2.0"]`, `build-backend =
"maturin"`) and points `manifest-path` at `../domainforge-core/Cargo.toml` with
the `python` and `three_valued_logic` cargo features, so the wheel is compiled
from the canonical Rust core rather than containing Python logic. Declared test
dependencies live in the `dev` extra (`pytest>=7.4`, `mypy>=1.0`); pytest is
driven from the repository root (`pytest.ini` sets `testpaths = tests`,
`python_files = test_*.py`) via `just python-test`. `dist/` holds build output
(`domainforge-0.13.0.tar.gz`) and is gitignored. `README.md`, `MIGRATING.md`,
and `CHANGELOG-python.md` are documentation only.

## Subdirectories & Boundaries
- domainforge: `./domainforge/EGM.md` — importable package root: `__init__.py` re-export shim over the compiled PyO3 extension plus its `domainforge.pyi` type stub.
