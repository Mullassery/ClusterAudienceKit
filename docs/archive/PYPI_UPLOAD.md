# PyPI Upload Instructions for ClusterAudienceKit v1.5.0

## Prerequisites

1. **PyPI Account**: Create account at https://pypi.org/account/register/
2. **API Token**: Generate at https://pypi.org/manage/account/tokens/
3. **Credentials File**: Create `~/.pypirc`:

```ini
[distutils]
index-servers =
    pypi

[pypi]
repository = https://upload.pypi.org/legacy/
username = __token__
password = pypi_YOUR_API_TOKEN_HERE
```

OR set environment variable:
```bash
export TWINE_PASSWORD=pypi_YOUR_API_TOKEN_HERE
export TWINE_USERNAME=__token__
```

## Build Wheels

```bash
# For current platform (macOS arm64)
python3 -m build --wheel

# Output: dist/clusteraudiencekit-1.5.0-cp313-cp313-macosx_11_0_arm64.whl
```

## Upload to PyPI (Test)

```bash
# Test server first (recommended)
twine upload --repository testpypi dist/clusteraudiencekit-1.5.0*.whl
```

## Upload to PyPI (Production)

```bash
# Main PyPI repository
twine upload dist/clusteraudiencekit-1.5.0*.whl

# With verbose output
twine upload --verbose dist/clusteraudiencekit-1.5.0*.whl
```

## Verify Upload

```bash
pip install clusteraudiencekit==1.5.0
python -c "import clusteraudiencekit; print(clusteraudiencekit.__version__)"
```

## Build for Multiple Platforms (CI/CD)

Use GitHub Actions to build wheels for:
- Linux (x86_64, aarch64)
- macOS (x86_64, arm64)
- Windows (x86_64, aarch64)

Add `.github/workflows/wheels.yml`:
```yaml
name: Build Wheels
on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v3
      - uses: PyO3/maturin-action@v1
        with:
          command: build
          args: --release --sdist -i python3.9 python3.10 python3.11 python3.12 python3.13
```

## Current Status

✅ **Built**: clusteraudiencekit-1.5.0-cp313-cp313-macosx_11_0_arm64.whl (207 KB)
- Python 3.13 wheel for Apple Silicon (arm64)
- Production-ready, 59 tests passing
- Ready to upload to PyPI

## Troubleshooting

**Error: HTTPError 403 Forbidden**
- Check API token is correct
- Verify account has upload permissions

**Error: Package version already exists**
- Version must be unique in PyPI
- Increment to v1.5.1 if needed

**Error: Metadata validation failed**
- Check pyproject.toml format
- Run: `twine check dist/*`

## References

- PyPI: https://pypi.org
- Twine: https://twine.readthedocs.io
- maturin: https://maturin.rs
