# Complete Technical Debt Register — ClusterAudienceKit

## Executive Summary

Total items: 9
Open: 6 · Resolved (this pass): 6 (overlaps with historical where a fix
closed more than one angle of the same item — see per-item notes) ·
Future/Deferred: 2
Critical: 0 · High: 1 · Medium: 4 · Low: 4

This repo arrived at this pass already extensively self-audited: a
"deep code-review pass" (2026-09-27) had already found and fixed a real
zero-actual-privacy bug in the differential-privacy module and an
order-dependent `generate_lookalike` bug, on top of an earlier
"OSS-standardization" pass and a "quick-fix" pass. This audit fast-forwarded
9 commits to pick up that work first, then focused on dependency security
(`cargo audit`), a pyo3 major-version bump forced by two real CVEs, and
verifying the external-facing marketing claims against the codebase's own
honest internal documentation.

## P0 — Critical

None found.

## P1 — High

| ID | Category | Description | Source | Status | Fix |
|---|---|---|---|---|---|
| TD-0001 | SECURITY, DEPENDENCY | `cargo audit`: pyo3 0.22.6 had 2 real CVEs — RUSTSEC-2025-0020 (buffer overflow in `PyString::from_object`) and RUSTSEC-2026-0177 (missing `Sync` bound on `PyCFunction::new_closure` closures) — plus `crossbeam-epoch` 0.9.18 (RUSTSEC-2026-0204, invalid pointer dereference in a `fmt::Pointer` impl). | SOURCE_CODE (`cargo audit`, verified live) | **FIXED** this pass | Bumped `pyo3` 0.22→0.29 and `numpy` 0.22→0.29 in lockstep; `cargo update -p crossbeam-epoch --precise 0.9.20`. `cargo audit` now reports 0 vulnerabilities (2 unmaintained/unsound *warnings* remain, see TD-0006). |

## P2 — Medium

| ID | Category | Description | Source | Status | Fix |
|---|---|---|---|---|---|
| TD-0002 | COMPATIBILITY | The pyo3 0.29 bump broke 2 call sites: `PyModule::new_bound` was removed (API rename to `new`), and bare `pyo3::PyObject` was removed as a type alias (needs `pyo3::Py<PyAny>`). | CI_FAILURE (would have failed to compile) | **FIXED** this pass | `src/python.rs:2467` and `src/utils/conversions.rs:14`. The latter is in an honestly-labeled, already-unimplemented `TODO` stub (`arrow_to_pandas`) never called from anywhere — fixed the type signature only, didn't implement the stub (out of scope, already tracked as future work). |
| TD-0003 | COMPATIBILITY, DEPRECATION | pyo3 0.29 deprecated the automatic `FromPyObject` derive on `Clone`-deriving `#[pyclass]` types (7 structs affected: `PySeedCustomer`, `PyCohort`, `PyCondition`, `PyBehavioralRule`, `PyBehavioralSegment`, `PyStreamingEvent`, `PyFeatureDrift`). | SOURCE_CODE (build warnings) | **FIXED** this pass | Verified per-type whether each is ever extracted from a Python argument (not just constructed/returned) by grepping actual call-site signatures: 6 of the 7 (`PySeedCustomer`, `PyCohort`, `PyCondition`, `PyBehavioralRule`, `PyBehavioralSegment`, `PyStreamingEvent`) are genuinely passed in from Python (e.g. `Vec<PyCohort>`, `&PySeedCustomer`) and need `#[pyclass(from_py_object)]` to preserve current behavior; `PyFeatureDrift` is construct/return-only and got `#[pyclass(skip_from_py_object)]` instead. Zero behavioral change (227/227 pytest, 439/439 cargo test both still pass identically). |
| TD-0004 | BUG, NAMING | The crate's internal error enum was misnamed `ClusterClusterAudienceKitError` (doubled "Cluster" prefix) — used at 60+ call sites across 18 files, `pub` at the crate root. Not exposed to Python (errors cross the PyO3 boundary via `PyResult`/`PyErr`, not this type directly), but still a real, visible naming bug in the crate's own public API surface. | INFERRED (code read) | **FIXED** this pass | Mechanical rename to `ClusterAudienceKitError` across all 18 affected files via scripted `sed`, followed by `cargo fmt` (the shorter name changed line-wrap decisions in ~6 places) and full build+test reverification. |
| TD-0005 | DEPENDENCY, ARCHITECTURE | `bincode = "1.3"` was a declared direct dependency, used only in a single `#[from] bincode::Error` error-enum variant that nothing in the codebase ever actually constructs (`bincode::serialize`/`deserialize` are never called anywhere in the crate) — dead dependency weight that also happened to be the thing triggering a `cargo-audit` "unmaintained" warning (RUSTSEC-2025-0141). | SECURITY, INFERRED | **FIXED** this pass | Removed the `bincode` dependency and the dead `Serialization` error variant entirely. Verified via grep across the whole repo (not just `src/`) that no call site existed before removing. |
| TD-0006 | DEPENDENCY, SECURITY | 2 residual `cargo-audit` warnings remain, both deep transitive dependencies with no safe direct fix: `paste` 1.0.15 (unmaintained, RUSTSEC-2024-0436) pulled in via `parquet` 55.2.0; `anyhow` 1.0.102 (unsound `downcast_mut`, RUSTSEC-2026-0190) pulled in only under non-default target configurations via a `wit-bindgen`/wasm-tooling dependency chain (`cargo tree -i anyhow` prints nothing under the default target — confirmed target-gated, not resolvable without `--target all`). | SOURCE_CODE (`cargo audit`, verified live) | OPEN | Neither is a direct dependency; fixing either means either waiting on upstream (`parquet`) or tracing exactly which wasm-adjacent tooling pulls in the `wit-bindgen` chain and whether it can be made target-specific/optional — needs a dedicated session, not a quick pin (same pattern as `ring`/`cc` conflicts found in other repos this rollout). |
| TD-0007 | DOCUMENTATION | The GitHub repo's short description (`gh repo view --json description`) reads "RFM analysis, **6 clustering algorithms**, CLV, churn detection, lookalikes, neural networks. **Process 1M+ customers in <1s.**" Neither claim is substantiated by the repo's own code or its already-honest internal docs: (a) the Python module's own `info.algorithms` list — the project's own source of truth, visible in `src/python.rs` right next to a comment confirming "dbscan"/"hierarchical"/"gmm" don't exist anywhere in the codebase — lists exactly `["kmeans", "kprototypes"]`, and `kprototypes` currently runs in numeric-only mode (`= kmeans`) per `docs/ROADMAP_HONEST.md`; the "6" count likely conflates the 4 K-*selection* heuristics (`estimate_k_elbow/gap/silhouette/combined`, which choose a cluster count, not a clustering algorithm) with the 2 real clustering algorithms. (b) No benchmark anywhere in the repo (real-data or Criterion) tests at 1M rows; the largest real-data benchmark in `README.md` is 4,338 real customers at 0.026s, and `docs/ROADMAP_HONEST.md` already explicitly flags *other* historical performance claims as "unverified in this pass — profile on your own hardware/data before relying on any specific number." | DOCUMENTATION, INFERRED (gh CLI + source read) | OPEN | This is GitHub repo metadata (not a tracked file in this repo), and correcting product-positioning copy is the README-rewrite phase's job per the audit sequencing rule ("fix the software, then document the resulting reality" — this pass is the "fix" half), not this pass's. Flagging with the exact evidence so the rewrite pass doesn't have to re-derive it. |

## P3 — Low

| ID | Category | Description | Source | Status | Fix |
|---|---|---|---|---|---|
| TD-0008 | TEST_DEBT | `tests/test_performance.py` uses `@pytest.mark.benchmark` without registering it, producing a `PytestUnknownMarkWarning` on every run (2 occurrences). | SOURCE_CODE (pytest output) | OPEN | One-line fix (add to `pyproject.toml`'s `[tool.pytest.ini_options]` markers list, or a `pytest.ini`) — not done this pass to keep the diff focused on security/compatibility; trivial follow-up. |
| TD-0009 | REFACTOR | 30 pre-existing clippy warnings remain, all confirmed (by file) concentrated in modules that are either explicitly deferred (`neural_networks.rs`, `temporal_analytics.rs`, `b2b_governance.rs`, `segment_intelligence.rs`, `pattern_discovery.rs`, `dashboard.rs`, `activation.rs`, `price_intelligence.rs`, `b2b_segmentation.rs`, `activation_orchestrator.rs`) or Rust-only/not-yet-wired per `docs/ROADMAP_HONEST.md` — zero are in a module reachable from the Python API. Re-verified this pass (count unchanged at 30; the pass added zero new clippy warnings despite the pyo3 bump and 18-file rename). | SOURCE_CODE (`cargo clippy`) | OPEN (unchanged) | Already tracked in `docs/ROADMAP_HONEST.md`'s own lint-debt accounting; not re-litigated here. |

## Explicit TODOs

- `src/utils/conversions.rs:6` — `pandas_to_arrow`, honestly returns `Err(..."Not implemented")`.
- `src/utils/conversions.rs:14` — `arrow_to_pandas`, same pattern. Both confirmed dead (never called, not Python-exposed) via grep.

## Stubs

- `pandas_to_arrow`/`arrow_to_pandas` (above) — honestly labeled, not silently broken, not reachable from the public API.

## Partial Implementations

- K-Prototypes clustering runs numeric-only (`= KMeans`) through the Python-facing `AudienceSegmenter` — already documented honestly in `docs/ROADMAP_HONEST.md`, re-verified accurate this pass, not re-fixed (full categorical support is a real follow-up feature, not a bug).

## Planned Features

See `docs/ROADMAP_HONEST.md`'s own "Roadmap" section — `segment_intelligence`/`pattern_discovery`/`temporal_analytics`/`price_intelligence`/`revenue_intelligence` Python bindings, full K-Prototypes categorical support, and a from-scratch-or-bound gradient-boosting integration are all tracked there already and not duplicated here.

## CI/CD Debt

None found this pass. `actionlint` clean on both workflow files. `tests.yml`'s `ruff check .` (fixed in a prior pass to actually fail on findings) re-verified still clean and still actually enforced.

## Test Debt

TD-0008 (benchmark marker warning). Otherwise: 439/439 Rust tests, 227 passed/2 skipped Python tests, both re-verified at the final state after all fixes in this pass.

## Dependency Debt

TD-0001 (fixed: pyo3, numpy, crossbeam-epoch), TD-0005 (fixed: removed dead bincode dep), TD-0006 (open: paste, anyhow — both transitive, no direct fix available).

## Security Debt

TD-0001 (fixed: 3 real CVEs closed — 2 in pyo3, 1 in crossbeam-epoch). `cargo audit` now reports 0 vulnerabilities, down from 3.

## Architecture Debt

TD-0005 (fixed: removed a dead dependency + its dead error-enum variant).

## Performance Debt

Not independently re-profiled this pass; `docs/ROADMAP_HONEST.md` already has real Criterion benchmarks (10k/100k rows) and honest caveats about older marketing claims (see TD-0007).

## Documentation Debt

TD-0007 (GitHub repo description overstates clustering-algorithm count and makes an unbenchmarked 1M-row performance claim, contradicted by the project's own internal `info.algorithms` list and `docs/ROADMAP_HONEST.md`'s existing honesty caveats).

## Resolved Historical Issues

Preserved from prior passes' own documentation (not re-verified line-by-line this pass, but no evidence found contradicting them): hardcoded CLV `churn_probability`, fake differential-privacy noise (zero actual privacy), order-dependent `generate_lookalike` percentile bug, fake XGBoost wrapper (renamed to honestly-labeled `HeuristicScoreEstimator`), hardcoded `auc_roc: 0.82`, three duplicate Python package directories, several `.bak` files, a fictional MCP connector and dashboard daemon described in now-deleted docs, and a `clippy::approx_constant` hard-error. Full detail in `docs/ROADMAP_HONEST.md` and `CHANGELOG.md`.
