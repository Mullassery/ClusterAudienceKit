# Contributing to ClusterAudienceKit

Thank you for your interest in contributing to ClusterAudienceKit!

## Code of Conduct

Be respectful and constructive in all interactions. We're committed to providing a welcoming and inclusive environment.

## Getting Started

### Fork and Clone

```bash
git clone https://github.com/YOUR_USERNAME/clusteraudiencekit.git
cd clusteraudiencekit
```

### Set Up Development Environment

ClusterAudienceKit is a Rust core (`src/`) with PyO3 Python bindings — you
need a Rust toolchain (see `rust-toolchain.toml`, currently `stable`) as well
as Python 3.8+.

Install in editable mode, which builds the Rust extension via `maturin` and
installs the Python package:

```bash
pip install -e ".[dev]"
```

**macOS linker note:** a bare `cargo build`/`cargo test` (not going through
`maturin`) needs this linker flag, because the crate's PyO3 dependency uses
the `extension-module` feature (which deliberately doesn't link against
`libpython`, since normally a Python interpreter provides those symbols at
import time):

```bash
export RUSTFLAGS="-C link-args=-undefined -C link-args=dynamic_lookup"
cargo build --release
```

**Known gap:** even with that flag, `cargo test` currently cannot run at all
on macOS — it builds, but the test binary SIGABRTs immediately
(`dyld: symbol not found in flat namespace '_PyBaseObject_Type'`) because a
standalone test binary isn't loaded inside a Python process, so those Python
C-API symbols are genuinely unavailable at runtime. This is a real,
unresolved gap (see `docs/ROADMAP_HONEST.md`), not something you're doing
wrong. Until it's fixed, validate Rust logic on macOS via the Python test
suite instead (below), which builds the extension through `maturin` and
exercises it from a real Python process — verified working:

```bash
maturin develop --release
pytest tests/ -v          # 227 passed, 2 skipped, last verified 2026-09-19
```

Rust-only checks that do work directly on macOS:

```bash
cargo fmt --all -- --check   # verified clean
cargo clippy --workspace --all-targets   # NOT clean repo-wide — see
                                          # docs/ROADMAP_HONEST.md for the
                                          # current count and which modules
cargo bench --bench benchmarks
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Follow [PEP 8](https://www.python.org/dev/peps/pep-0008/) for Python code
- Use type hints on all public functions
- Use Black for formatting and Ruff for linting

```bash
black .
ruff check .
mypy .
```

### 3. Write Tests

- Add Rust unit tests (`#[cfg(test)]`) in the module you changed under
  `src/engine/`, and a Python-level test in `tests/` exercising the same
  change through the actual PyO3 binding (not a mock).
- There's no coverage tooling wired up for the Rust side (no tarpaulin/
  grcov/similar), and `pytest --cov` only measures
  `clusteraudiencekit/__init__.py` (a 24-line re-export shim, already at
  100%) — Python's `coverage.py` can't instrument the compiled Rust
  extension, so a "% coverage" number for this project wouldn't mean much.
  Judge test adequacy by whether the new/changed logic has a real test that
  would fail if the logic were wrong, not by a coverage percentage.

```bash
pytest tests/
```

### 4. Update Documentation

- Update docstrings for any changed public API
- Update README.md if the user-facing API changes
- Add examples in `examples/`

### 5. Commit and Push

```bash
git add .
git commit -m "feat: add streaming drift detection"
git push origin feature/your-feature-name
```

Follow [conventional commits](https://www.conventionalcommits.org/):
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation only
- `test:` Test changes
- `perf:` Performance improvement
- `refactor:` Code refactoring
- `chore:` Build or dependency changes

### 6. Open a Pull Request

Create a pull request with a clear description of:
- What the change does
- Why it's needed
- How it was tested

## Pull Request Guidelines

- Keep PRs focused on a single feature or fix
- Include tests for all new functionality
- Ensure all tests pass before requesting review
- This is currently maintained by one person (see `docs/CONTRIBUTORS.md`);
  there's no guaranteed review turnaround time

## Testing

### Python Integration Tests

The real constructor is `AudienceSegmenter(n_clusters, n_jobs=None)`, and it
takes a numeric feature matrix (`list[list[float]]`), not a DataFrame; there
is no `fit_predict()` — call `fit()` then `predict()` separately:

```python
import pytest
from clusteraudiencekit import AudienceSegmenter

def test_fit_predict_pipeline():
    features = [[10.0, 2.0], [12.0, 1.0], [200.0, 40.0], [190.0, 38.0]]
    segmenter = AudienceSegmenter(2)
    segmenter.fit(features)
    segments = segmenter.predict(features)
    assert len(segments) == len(features)
    assert min(segments) >= 0
    assert max(segments) < 2
```

## Documentation

- Update docstrings for all changed public APIs
- Include usage examples in docstrings
- Update README.md for user-facing changes
- Add or update examples in `examples/`

## Reporting Issues

Use the [GitHub issue tracker](https://github.com/Mullassery/clusteraudiencekit/issues):

1. Check existing issues first
2. Include:
   - Clear description of the issue
   - Steps to reproduce
   - Expected vs actual behaviour
   - Environment (OS, Python version, ClusterAudienceKit version)

## Suggesting Features

- Open an issue with the "Feature Request" label
- Describe the feature and the use case it addresses
- Link to similar features in other libraries if applicable

## License

By contributing, you agree your contributions will be licensed under the
project's Apache License 2.0 (see [`LICENSE`](LICENSE)).

Thank you for contributing to ClusterAudienceKit!
