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

ClusterAudienceKit is a Python library. Install it in editable mode to develop:

```bash
pip install -e ".[dev]"
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

- Add unit tests in `tests/`
- Maintain >90% coverage on the Python API

```bash
pytest tests/
pytest tests/ --cov=clusteraudiencekit
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
- Maintainers will review within 7 days

## Testing

### Python Integration Tests

```python
import pytest
from clusteraudiencekit import AudienceSegmenter

def test_fit_predict_pipeline():
    segmenter = AudienceSegmenter(n_clusters=4)
    segments = segmenter.fit_predict(test_df)
    assert len(segments) == len(test_df)
    assert segments.min() >= 0
    assert segments.max() < 4
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

By contributing, you agree your contributions will be licensed proprietary license.

Thank you for contributing to ClusterAudienceKit!
