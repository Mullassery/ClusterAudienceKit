# Repository Audit — ClusterAudienceKit

## Health

**GREEN.** This repo arrived already in unusually good shape — three prior
dedicated passes (OSS-standardization, quick-fix, and a "deep code-review"
pass that found a real zero-actual-privacy differential-privacy bug) had
already been merged to `origin/main`, 9 commits ahead of this audit's
starting clone. This pass fast-forwarded onto that work first, then closed
a real security gap (3 CVEs via a forced pyo3 major-version bump) and
verified the external-facing marketing claims against the codebase's own
source of truth.

## Before Audit (this pass's baseline, after fast-forward to `51cffd2`)

Open debt: 9
Critical: 0 · High: 1 · Medium: 7 (6 immediately fixable, 1 deep-transitive) · Low: 2

## After Audit

Open debt: 3
Critical: 0 · High: 0 · Medium: 2 (TD-0006 transitive deps, TD-0007 external description) · Low: 1 (TD-0008 benchmark marker)

## Items Fixed

1. **TD-0001** — 3 real `cargo-audit` CVEs closed: pyo3 buffer-overflow (RUSTSEC-2025-0020), pyo3 missing-`Sync`-bound (RUSTSEC-2026-0177), crossbeam-epoch invalid-pointer-deref (RUSTSEC-2026-0204). Required bumping pyo3 0.22→0.29 and numpy 0.22→0.29.
2. **TD-0002** — 2 real compile breaks from the pyo3 bump (`PyModule::new_bound`→`new`, `pyo3::PyObject`→`pyo3::Py<PyAny>`).
3. **TD-0003** — 7 pyo3 0.29 `FromPyObject`-on-`Clone` deprecation warnings, resolved individually per-type (not a blanket fix) based on verified actual usage: 6 types opted into `#[pyclass(from_py_object)]` because they're genuinely extracted from Python arguments; 1 type (`PyFeatureDrift`, construct/return-only) opted out with `#[pyclass(skip_from_py_object)]`.
4. **TD-0004** — Fixed a real naming bug in the crate's own public API: `ClusterClusterAudienceKitError` → `ClusterAudienceKitError`, mechanically across 18 files and 60+ call sites.
5. **TD-0005** — Removed a dead `bincode` dependency (zero actual call sites anywhere in the repo) and its dead error-enum variant, closing an "unmaintained crate" `cargo-audit` warning as a side effect.

## Items Remaining

- **TD-0006** (P2) — 2 residual `cargo-audit` *warnings* (not vulnerabilities): `paste` (unmaintained, transitive via `parquet`) and `anyhow` (unsound, transitive via a non-default-target wasm-tooling chain). Neither is a direct dependency; no safe one-line fix exists.
- **TD-0007** (P2) — The GitHub repo's short description overstates clustering-algorithm count ("6 clustering algorithms" vs. the project's own `info.algorithms` list of `["kmeans", "kprototypes"]`) and makes an unbenchmarked "1M+ customers in <1s" claim. Documented with exact evidence for the subsequent README/positioning pass; not corrected in this pass per the audit's own sequencing rule (fix the software first, document reality after).
- **TD-0008** (P3) — Unregistered `pytest.mark.benchmark` produces a harmless warning on every test run.

## Future Phase Work

Already comprehensively tracked in the repo's own `docs/ROADMAP_HONEST.md` (5 large Rust-only modules awaiting Python bindings, full K-Prototypes categorical support, gradient-boosting integration) — not duplicated here.

## CI Status

Both workflows (`ci.yml`, `tests.yml`) pass `actionlint` clean. `ruff check .` re-verified clean and genuinely enforced (a prior pass fixed it from silently discarding findings).

## Test Status

Rust: 439/439 passing (re-verified at every step of the pyo3 bump — before, immediately after the compile fixes, and after the final version bump). Python: 227 passed, 2 skipped, re-verified against a freshly `maturin develop`-built extension after all Rust changes, confirming zero behavioral drift from the pyo3/numpy major-version bump.

## Build Status

`cargo build --release`, `cargo test --release --no-default-features --lib`, `cargo clippy --workspace --all-targets` (30 warnings, all pre-existing and confirmed unreachable from the Python API — zero new warnings introduced), `cargo fmt --all -- --check`, and `maturin develop --release` all verified clean at the final commit.

## Security Status

`cargo audit`: 0 vulnerabilities (down from 3), 2 unmaintained/unsound warnings remaining on transitive-only dependencies (documented, not force-fixable).

## Dependency Status

pyo3/numpy bumped to current major versions (0.29) with zero downstream behavioral breakage beyond the 2 expected compile-time API changes. `crossbeam-epoch` patched via `cargo update --precise`. `bincode` removed entirely (was dead weight).

## Final Assessment

This repo's own prior audit passes had already done most of the honesty and bug-fixing work; this pass's genuine contribution is closing a real security gap that required a disruptive major-version dependency bump (not a quick pin) across pyo3/numpy, verified with zero behavioral regression across both the Rust and Python test suites, plus fixing two smaller but real bugs (a dead dependency, a public-API naming typo) this project's own prior passes hadn't caught. The one new finding worth flagging loudly: the GitHub repo's external-facing description makes claims ("6 clustering algorithms", "1M+ customers in <1s") that the project's own internal honesty documentation doesn't support — evidence is captured in `TECHNICAL_DEBT.md` TD-0007 for whoever runs the README/positioning pass next. Version bumped to 7.3.2 for this release; PyPI publish intentionally left to the coordinator to handle directly.
