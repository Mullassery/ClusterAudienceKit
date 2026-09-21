# ClusterAudienceKit Roadmap (Honest)

**Current Version:** 7.3.0
**Last Updated:** 2026-09-21 (quick-fix pass — see "Quick-fix pass
(2026-09-21)" below: 33 ruff findings, a clippy-blocking compile error, dead
code, a non-snake-case rename, and the macOS `cargo test` crash all fixed;
no version bump. Previously 2026-09-20's OSS-standardization/
documentation-honesty pass — see "Documentation and structural issues found
(2026-09-20)" below; no version bump, no functional code changes then
either)
**Status:** Real, tested Rust core for RFM + KMeans/K-Prototypes clustering,
churn prediction, CLV, SQL export, and now 10 additional analytics modules —
all exposed through the Python API and covered by both Rust unit tests and
Python integration tests.

This file previously described a `v1.0.0` state (streaming "not implemented",
CLV "not shipped", drift detection "skeleton only", clustering "5 TODOs").
That description predates an 2026-08-07 commit that implemented real
KMeans/K-Prototypes clustering and is now materially wrong about the current
codebase. This is the corrected version.

---

## What's real and shipping in the Python API today

- **RFM analysis** (`calculate_rfm`, `RFMConfig`) — real decay functions
  (linear/exponential/inverse), quintile/decile/percentile scoring,
  13-segment RFM classification. Per-customer computation is
  rayon-parallelized.
- **Clustering** (`kmeans`, `AudienceSegmenter`) — real Lloyd's-algorithm
  KMeans with k-means++ initialization, deterministic for a given
  `random_state`, empty-cluster re-seeding. K-Prototypes (mixed
  numeric/categorical) exists in the Rust core (`engine::clustering::
  kprototypes`) but `AudienceSegmenter.fit()` currently only takes a numeric
  feature matrix, so it runs K-Prototypes in numeric-only mode
  (`= KMeans`) when selected — full categorical support needs a richer
  Python-facing input type and is a real follow-up, not a silent bug.
  The nearest-center assignment step (the dominant per-iteration cost) is
  rayon-parallelized in both `kmeans` and `kprototypes`.
- **Cluster quality metrics** (`silhouette_score`, `davies_bouldin_score`,
  `calinski_harabasz_score`, `assess_cluster_quality`) — real
  implementations, not placeholders.
- **K estimation** (`estimate_k_elbow`, `estimate_k_gap_statistic`,
  `estimate_k_silhouette`, `estimate_k_combined`) — real elbow/gap-statistic/
  silhouette/ensemble methods for choosing K automatically.
- **Churn prediction** (`ChurnPrediction`, `ChurnRiskLevel`) — logistic and
  ensemble heuristic scoring. As of this release, `evaluate_model_performance`
  computes a **real trapezoidal-rule AUC-ROC** from the actual
  prediction/label pairs, replacing a previous hardcoded `auc_roc: 0.82`.
- **Customer Lifetime Value** (`CustomerLTV`, `calculate_simple_ltv`) — real,
  shipped (this file previously said CLV was "not implemented"; it has been
  for several releases now).
- **SQL export** (`export_segment_sql`, `export_all_segments_sql`,
  8 dialects) — table/column identifiers are now validated against an
  alphanumeric+underscore(+dot-qualified) allow-list before interpolation
  into generated SQL; see `docs/SECURITY_AUDIT.md`.
- **Differential privacy / k-anonymity** (`PyPrivacyBudget`,
  `add_laplace_noise`, `add_gaussian_noise`, `check_k_anonymity`,
  `suppress_to_k_anonymous`, `generalize_numeric`) — newly wired this
  release. Closes a real PII-handling gap: previously there was no way to
  anonymize or add DP noise to customer data through the Python API at all.
- **Real-time streaming segmentation** (`PyStreamingSegmentationEngine`,
  `PyStreamingEvent`, `PyStreamingConfig`) — newly wired this release. There
  was previously a near-duplicate, genuinely-empty stub module at
  `src/streaming/mod.rs` (`pub mod streaming;` in `lib.rs`) sitting alongside
  the real, fully-implemented `engine::streaming` (673 lines, originally 14
  passing tests, all real logic — incremental RFM state, buffering,
  windowed aggregation, segment reassignment). The stub has been deleted;
  only the real implementation remains, and it's now Python-callable.
- **Drift detection** (`kolmogorov_smirnov`, `hellinger_distance`,
  `chi_square_drift`, `detect_feature_drift`,
  `detect_segment_composition_change`) — newly wired this release. Real KS
  test, Hellinger distance, and chi-square statistics with severity
  classification.
- **Lookalike audiences** (`generate_lookalike`, `find_similar_customers`,
  `cosine_similarity`, `PySeedCustomer`) — newly wired this release.
- **Cohort analytics** (`create_cohort`, `cohort_id_for`, `compare_cohorts`,
  `aggregate_cohorts_by_period`, `cohort_retention_table`,
  `cohort_performance_ranking`, `PyCohort`) — newly wired this release.
- **Lifecycle tracking** (`classify_lifecycle_stage`,
  `lifecycle_retention_actions`, `lifecycle_stage_distribution`) — newly
  wired this release.
- **Behavioral rule-based segmentation** (`PyBehavioralSegmenter`,
  `PyBehavioralSegment`, `PyBehavioralRule`, `PyCondition`) — newly wired
  this release. A deterministic, business-rule-defined alternative/companion
  to statistical clustering, with SQL export.
- **Segment profiling** (`profile_segment`) — newly wired this release.
  Per-segment statistics, plain-language business description, and a
  stability/cohesion/separation health score.

Every module in this list has both pre-existing Rust `#[cfg(test)]` unit
tests (run via `cargo test --lib`) **and** new Python-level tests in
`tests/test_wired_modules.py` (or `tests/test_basic.py` /
`tests/test_clustering.py` for the core RFM/clustering path) exercising the
actual PyO3 binding, not a mock.

---

## Fixed for honesty this release

- **`xgboost_models.rs` → `heuristic_score_estimator.rs`**: this module's
  `train()` never trained anything (`train_score = 0.85 +
  (learning_rate * 10.0).min(0.1)` — a formula, not a fit metric) and
  `predict()` computed a linear combination of feature-magnitude
  "importances," not a decision-tree ensemble traversal — despite being
  named/typed as if it were a real XGBoost wrapper (`XGBModel`, `XGBParams`,
  `train_score`, `validation_score`). We checked for a real, actively
  maintained Rust XGBoost binding crate; the viable options require linking
  a system-installed `libxgboost` via a C++ build step that isn't available
  in this environment/CI, so a real integration wasn't feasible in this
  pass. Rather than ship the fake version silently, the module and its
  types were renamed to drop all XGBoost/gradient-boosting terminology
  (`HeuristicScoreEstimator`, `HeuristicEstimatorParams`,
  `heuristic_fit_score`, `heuristic_holdout_score`, with an explicit doc
  comment explaining what it actually is), and "xgboost"/"gradient-boosting"
  were removed from `pyproject.toml`'s keyword list. **It is not exposed in
  the Python API** — it never was, and shouldn't be presented as a trained
  ML model.
- **`churn_prediction.rs` AUC-ROC**: was hardcoded `0.82 // Simulated`.
  Now computed via a real trapezoidal-rule integration over the ROC curve
  traced from the same prediction/label pairs already used for the
  confusion-matrix metrics a few lines above. See
  `compute_auc_roc()` and its dedicated tests (perfect separation → 1.0,
  perfectly inverted → 0.0, no discrimination → 0.5, hand-computed case →
  0.75).

---

## Rust-only, tested, deliberately not yet Python-exposed

These modules are **real and tested** (not stubs, not fabricated) — they
were simply large enough (900–1200+ lines each) that binding them properly
in this pass would have meant rushing the binding work rather than doing it
carefully. Deferred to a follow-up release, not "not started":

- `engine::segment_intelligence` (1103 lines)
- `engine::pattern_discovery` (1228 lines)
- `engine::temporal_analytics` (1101 lines)
- `engine::price_intelligence` (926 lines)
- `engine::revenue_intelligence` (1185 lines)

## Real but reasonably deferred (heavier ML surface)

- `engine::neural_networks` — **this one is real**, not fabricated like the
  old xgboost module: it's a genuine from-scratch forward/backward-prop
  dense-layer network, autoencoder, and simple RNN layer with real gradient
  descent training. It doesn't require a new external dependency (it's pure
  Rust), but wiring its full training/prediction/autoencoder/RNN API into
  Python is a substantially larger new product surface (configurable
  architectures, training loops, model serialization) than the other
  modules in this pass, and is closer to "ship a deep learning framework"
  than "expose an existing statistical capability." Deferred for the same
  reason XGBoost integration was deferred: scope, not feasibility.

## Genuinely deferred (explicitly out of scope for this pass)

- `engine::governance`, `engine::b2b_governance` — organizational
  policy/workflow tooling, not a statistical capability.
- `engine::dashboard` — a UI concern; different product surface entirely.
  (Note: `DASHBOARD_SHORTCUTS.md`, which described a fictional
  auto-starting terminal dashboard daemon that doesn't exist anywhere in
  this codebase, has been deleted as part of this pass's honesty cleanup.)
- `engine::plugins` — an extensibility framework with no concrete plugins
  to support yet; premature.
- `engine::platform_adapters`, `engine::activation`,
  `engine::activation_orchestrator` — pushing segments to external ad/
  marketing platforms; requires real external API integrations and
  credentials this environment doesn't have.
- MCP (Model Context Protocol) connector — `MCP_QUICKSTART.md` and
  `clusteraudiencekit.toml` described an MCP server (`start_mcp_connector()`
  on `AudienceSegmenter`) that doesn't exist on the real, exported
  `AudienceSegmenter` class at all; the code it depended on
  (`python/clusteraudiencekit/_mcp_connector.py`) lived in a stale,
  never-actually-packaged duplicate directory (see Repo Hygiene below) and
  has been deleted along with the misleading docs.

---

## Repo hygiene done this release

- Deleted `clusteraudiencekit.bak/`, `README.md.bak`,
  `python/clusteraudiencekit/__init__.py.bak`.
- There were **three** candidate Python package directories:
  `./clusteraudiencekit/`, `./python/clusteraudiencekit/`, and
  `./src/clusteraudiencekit/`. Per `pyproject.toml`'s
  `[tool.maturin]` config (`python-packages = ["clusteraudiencekit"]`, no
  `python-source` override — confirmed against how CI actually installs the
  package, `pip install -e ".[dev]"`, which invokes maturin with this exact
  config), only `./clusteraudiencekit/` (a thin `__init__.py` re-exporting
  the compiled extension with friendly aliases) is the real,
  maturin-packaged source. `./python/clusteraudiencekit/` and
  `./src/clusteraudiencekit/` were stale, never-wired duplicates (one used
  a hand-rolled `.so` loader for SQL-export-only fallback behavior, the
  other had extra helper modules like `logging_config.py`/`validation.py`
  that were never imported by the real package). Both have been deleted.
- `.gitignore` updated to explicitly ignore `*.so`/`*.pyd`/`*.dylib` (the
  compiled extension `maturin develop` builds in-place inside
  `clusteraudiencekit/`) and `wheels/`/`target/wheels/`. `Cargo.lock`'s
  stale ignore entry was removed — it's intentionally tracked (was already
  committed) for reproducible builds.
- Deleted `MCP_QUICKSTART.md`, `DASHBOARD_SHORTCUTS.md`,
  `FINAL_REPORT.txt`, and `clusteraudiencekit.toml` — all described
  product surfaces (an MCP connector, an auto-starting dashboard daemon)
  that don't exist in the real, exported API.

---

## Tests fixed this release

`tests/test_clustering.py` previously imported `ClusterEngine` and a
`hierarchical` clustering algorithm — `ClusterEngine` was never implemented
anywhere in this codebase (the real class is `PyAudienceSegmenter`/
`AudienceSegmenter`), and hierarchical clustering is explicitly listed under
"Not Planned" below. The file has been rewritten against the real exported
API (`AudienceSegmenter`, `kmeans`, `silhouette_score`,
`assess_cluster_quality`) with the hierarchical-clustering case removed
rather than faked. `tests/test_basic.py` had the same
`AudienceSegmenter(method="rfm_kmeans", ...)` mismatch (the real constructor
takes a single positional `n_clusters`) and a skipped fit/predict test; both
are now real, passing, end-to-end tests. `tests/test_performance.py`'s main
performance-test class was five `pytest.skip("Placeholder - implementation
coming in Phase 1")` stubs with commented-out bodies; they're now real,
running performance assertions against the real Rust-backed pipeline.

---

## Security

See `docs/SECURITY_AUDIT.md` for the SQL-injection fix in `sql_export.rs`
and current dependency-pinning status.

---

## Performance

- `rayon` was previously a declared dependency with zero call sites (implying
  multi-core scaling that didn't exist). It's now genuinely used to
  parallelize the two real hot loops: the nearest-center assignment step in
  both `kmeans`/`kprototypes` (the dominant per-iteration cost), and the
  per-customer RFM computation in `calculate_rfm`.
- `benches/benchmarks.rs` previously contained only a `dummy_benchmark`
  measuring `1 + 1`. It now has real Criterion benchmarks for `kmeans` and
  `calculate_rfm` at 10k and 100k rows on synthetic blob data.
  Benchmark claims from earlier docs (e.g. "46x-1000x faster than
  scikit-learn") remain Apple M1-specific and unverified in this pass —
  profile on your own hardware/data before relying on any specific number.

---

## Known lint/format debt (honest accounting)

- `cargo fmt --all -- --check` — **verified clean** again on 2026-09-21
  (macOS, Apple Silicon, `RUSTFLAGS="-C link-args=-undefined -C
  link-args=dynamic_lookup"`).
- **Fixed (2026-09-21 quick-fix pass):** plain `cargo clippy --workspace
  --all-targets` (no `-D warnings`) had actually stopped completing at all
  since the 2026-09-20 pass — new toolchain/clippy drift turned
  `clippy::approx_constant` (deny-by-default) into a hard compile error on
  `segment_intelligence.rs`'s hand-written `0.693_147` half-life constant,
  which approximates `LN_2` closely enough to trip it. One-line fix:
  replaced the literal with `std::f64::consts::LN_2` (same value to ~1e-7,
  no behavior change, verified via the full `pytest`/`cargo test` runs
  below). Also deleted the dead/duplicate `src/engine/metrics.rs` (see
  "Documentation and structural issues found" below — confirmed unreachable
  from anywhere else in the crate before deleting) and renamed the
  non-snake-case `X` parameter (ML-paper notation) to `inputs` in
  `neural_networks.rs`'s four `train`/`anomaly_scores`/`extract_patterns`
  methods and their tests. Warning count: **30** (down from 37), still
  clean of the compile-blocking error. `cargo clippy --workspace
  --all-targets -- -D warnings` still fails to build (not clean) on the
  remaining 30 — this pass did not attempt those (see roadmap below).
  Confirmed by file, all still concentrated in modules that are either
  explicitly deferred or Rust-only/not-yet-wired — **zero** are in a module
  reachable from the Python API:
  - `neural_networks.rs` — 10 (all `needless_range_loop`; the non-snake-case
    `X` warnings are gone)
  - `temporal_analytics.rs` — 5
  - `b2b_governance.rs` — 4
  - `segment_intelligence.rs` — 2 (down from 3 — the `approx_constant` error
    above is fixed; two pre-existing `manual_clamp` warnings remain)
  - `pattern_discovery.rs` — 2
  - `dashboard.rs` — 2
  - `activation.rs` — 2
  - `price_intelligence.rs`, `b2b_segmentation.rs`, `algorithms.rs`,
    `activation_orchestrator.rs` — 1 each

---

## Roadmap

### Next release — Wire the remaining large modules
- `segment_intelligence`, `pattern_discovery`, `temporal_analytics`,
  `price_intelligence`, `revenue_intelligence` Python bindings.
- Full K-Prototypes categorical support through `AudienceSegmenter`.
- Clear the remaining 43 clippy findings in not-yet-wired modules.

### Later — Advanced Analytics
- Real gradient-boosting integration for churn/CLV, if/when a Rust XGBoost
  binding that doesn't require a system libxgboost install becomes
  practical, OR a from-scratch gradient-boosted-trees implementation
  (like `neural_networks` is a from-scratch NN) if that's preferred over an
  external binding.
- Neural network Python bindings, if there's a real use case that justifies
  the API surface.
- [x] Drift-triggered re-clustering — **Done (7.2.0).** `StreamingSegmentationEngine::process_batch`
  (`src/engine/streaming.rs`) now arms a drift baseline over the tracked
  population's (recency, frequency, monetary) distribution once enough
  customers accumulate, and checks every subsequent batch against it via
  `DriftDetector`. Drift crossing a configurable severity
  (`ReclusterConfig`) triggers a real `clustering::kmeans` re-fit over
  every tracked customer, overwriting segment assignments with the fresh
  cluster labels and re-arming the baseline. New
  `check_drift`/`maybe_recluster`/`set_recluster_baseline`/
  `recluster_history` methods, exposed to Python as
  `StreamingSegmentationEngine(config, recluster_config)` /
  `ReclusterConfig` / `ReclusterEvent`.

  Wiring this up surfaced two real, pre-existing bugs in `DriftDetector`
  itself (fixed in the same release, see `CHANGELOG.md`): `kolmogorov_smirnov`
  produced phantom drift on tied/repeated values (its baseline empirical CDF
  was computed via rank instead of counting, silently assuming no ties), and
  `hellinger_distance` short-circuited to 0 whenever either sample had exactly
  zero variance regardless of how different the other sample was, with a
  variance term that algebraically canceled to 0 for any inputs. Both were
  load-bearing for this exact feature — a streaming RFM window with
  identical purchase counts (common) or a quiet, uniform baseline window
  followed by a real shift would otherwise have produced false triggers or
  silently missed real drift.
- [x] Chunked/streaming ingestion for batch clustering — **Done (7.2.0).**
  Added `clustering::MiniBatchKMeans` (Sculley 2010): consumes one chunk at
  a time via `partial_fit`, updating centers incrementally with memory
  bounded to O(chunk_size + n_clusters) regardless of total dataset size.
  This is additive, not a replacement for `kmeans_py`/
  `AudienceSegmenter.fit`/`predict` — those still run exact Lloyd's-algorithm
  k-means for callers who already have the full dataset in memory and want
  the exact (not approximate/online) result; `MiniBatchKMeans` is for the
  specific "dataset doesn't fit in memory at once" case those can't handle.
  Exposed to Python as `MiniBatchKMeans(n_clusters, random_state)`.
- ~~License compatibility audit~~ — **Done.** This paragraph previously
  described `LICENSE` as a custom "free to use with attribution"
  source-available license treated as Proprietary per org policy. That's
  stale: the repo was relicensed to Apache License 2.0 on 2026-09-06 (see
  `LICENSE`, `Cargo.toml`'s `license = "Apache-2.0"`, and `pyproject.toml`'s
  `license = "Apache-2.0"` — all three agree, no drift). No copyleft
  (GPL/AGPL/LGPL) dependencies were found in `Cargo.toml`'s dependency list
  during this pass; a full transitive-dependency license scan (e.g. via
  `cargo-license` or `cargo-deny`) has not been run.

---

## Documentation and structural issues found (2026-09-20)

An OSS-standardization/documentation-honesty pass (README/CONTRIBUTING/
CI/docs review, no functional code changes) found the following. Everything
fixed is a documentation, CI-config, or `.gitignore` change; every bug/gap
that wasn't fixed is listed here per this file's own stated purpose.

### Fabricated or stale docs — archived to `docs/archive/`

A prior pass (see "Fixed for honesty this release" above, and
`CHANGELOG.md`'s `[7.2.0]` entry) added an honesty banner to five docs
describing a fictional `AudienceSegmenter(method="rfm_kmeans", ...)`
constructor and fabricated benchmark numbers, but left them in `docs/` as
readable reference material "out of scope for this pass." This pass found
**four more** docs with the same problem that the prior pass missed
entirely (no banner, still presented as current), and concluded a banner
isn't sufficient disclosure for any of them — a doc titled "API Reference"
sitting in `docs/` reads as current regardless of a note at the top. All
nine, plus two independently-stale architecture docs and three other stale
duplicates, were moved to `docs/archive/` (`git mv`, history preserved) with
a detailed index at `docs/archive/README.md` explaining each one. Full list
and reasoning: see that index. Short version:

- Already banner-flagged, now archived: `api-reference.md`,
  `getting-started-simple.md`, `comparison.md`, `performance-comparison.md`,
  `press-release.md`.
- **Newly found**, never flagged before this pass:
  - `docs/WORKFLOW_INTEGRATION.md` — documented a CLI
    (`clusteraudiencekit create-audience`/`refresh-audience`/`get-members`)
    and a REST server (`python -m clusteraudiencekit.server`, port 8002).
    **Neither exists anywhere in this codebase** — `pyproject.toml` has no
    `[project.scripts]` entry, there's no server module. This was the
    single most misleading doc found: unlike the other five, it had no
    honesty banner at all.
  - `docs/troubleshooting.md` and `docs/error-catalog.json` — both built
    around the same fictional DataFrame-column-based `fit()` API as the
    already-flagged docs; `error-catalog.json` is also dead weight — nothing
    in `src/` or the Python package loads it.
  - Root `ARCHITECTURE.md` — described integration with four other products
    (`StatGuardian`, `PyCustomerJourney`, `PyReverseETL`, `PyStreamMCP`) via
    Rust/Python code samples (`use pystreammcp::Discovery;`, `use
    statguardian::ValidationGate;`) referencing crates/packages this project
    has **zero dependency on** anywhere in `Cargo.toml`/`pyproject.toml`,
    and a `core/src/` module layout (`audience.rs`, `segment.rs`,
    `clustering/`, `scoring/`, `storage/`) that has never existed here. This
    wasn't "aspirational" — it was presented as `✅ CORRECT` current usage.
  - `docs/architecture.md` — a second, independently different, also-stale
    module layout (`segmentation/`, `metrics/`, `profiling/`, `drift/`,
    `io/`).
  - `docs/ROADMAP.md` — an old roadmap (last said "v2.0.0") describing the
    fictional CLI/REST integration above as an already-shipped "v2.0
    Workflow Integration" milestone. Superseded by this file.
  - `docs/CONTRIBUTING.md` — despite the filename, its actual content was a
    `CLAUDE.md`-style AI-assistant briefing describing yet another fictional
    module layout, not contributor instructions. The real one is the root
    `CONTRIBUTING.md`.
  - `docs/SECURITY.md` — a stale duplicate of the root `SECURITY.md`,
    referencing a `PRODUCTION_AUDIT_REPORT.md` that doesn't exist anywhere
    in this repo, and labeled version `0.1.0`/"NO PRODUCTION USE - beta"
    against a `7.3.0`/`Production-Stable`-classified current release.
  - `docs/PYPI_UPLOAD.md` — release instructions for `v1.5.0` via `python3
    -m build --wheel`; current process (verified from `pyproject.toml`'s
    `[build-system]`) uses `maturin`. **Nobody has written a replacement —
    there is currently no accurate release doc in this repo.** Flagged here
    as genuinely missing, not fixed.
  - `.github/CI_ERRORS.md` — an unfilled template from some org-wide
    automated CI scanner, with unsubstituted `$REPO_NAME`/`$repo` variables
    and dead links to `../../CI_ERRORS_REPORT.md` and
    `.github/TROUBLESHOOTING.md`, neither of which exists anywhere.
- A new, accurate `docs/architecture/README.md` was written in their place,
  checked directly against `src/` (module list, PyO3 boundary, a real
  request-flow Mermaid diagram), replacing both archived architecture docs.

### `examples/streaming_updates.py` was a stub describing a feature that now exists

The file was `# TODO: Implement streaming example once core functionality
is ready` with the entire example commented out and `main()` printing
"Implementation in progress..." — but `StreamingSegmentationEngine` has been
real and Python-wired since the 7.2.0 pass (see "What's real and shipping"
above); the example was just never updated. Rewritten against the real API
(mirrors `tests/test_wired_modules.py::TestStreaming`) and verified to run:
`PYTHONPATH=. python3.11 examples/streaming_updates.py` produces real output
(segment assignments, `customer_count()`, `segment_distribution()`).

### `cargo test` on macOS — **fixed (2026-09-21 quick-fix pass)**

Previously, `Cargo.toml` unconditionally enabled `pyo3`'s `extension-module`
feature (not feature-gated), which deliberately omits linking against
`libpython` on the assumption a Python interpreter process will provide
those symbols at import time via `dlopen`. A standalone `cargo test` binary
run directly is not loaded inside a Python process, so those symbols were
never available, and every `cargo test --release` invocation SIGABRTed
immediately (`dyld: symbol not found in flat namespace
'_PyBaseObject_Type'`), regardless of `--all-features`/`--lib`.

Fixed by feature-gating it in `Cargo.toml`:
```toml
[features]
default = ["extension-module"]
extension-module = ["pyo3/extension-module"]
```
`default = [...]` keeps `cargo build`/`cargo bench`/`cargo clippy`/`maturin
develop` behaving exactly as before (all verified unaffected — same
warning counts, same wheel, same `pytest` pass count). `ci.yml`'s
`rust-build` job is also unaffected either way, since it already passes
`--all-features` to both `cargo build` and `cargo test`, which forces this
feature on regardless of the default. The new, working path is:

```bash
cargo test --release --no-default-features --lib
```

This links the test binary normally against `libpython`, which requires a
Python installation with an actual linkable `libpython*.dylib` — the Xcode
Command Line Tools' bundled Python (the `python3` on `PATH` on a fresh
macOS machine) does **not** ship one, so `PYO3_PYTHON` needs to point at
one that does (Homebrew, pyenv `--enable-shared`, or a python.org install).
Verified on Apple Silicon macOS with `PYO3_PYTHON=/opt/homebrew/bin/
python3.11`: `cargo test --release --no-default-features --lib` → **435
passed, 0 failed**. On a machine with only the Xcode CLT Python, this now
fails with a clear, actionable linker error (`library 'python3.9' not
found`) instead of the previous unexplained SIGABRT — a real improvement
even where a suitable Python isn't already installed. See
`CONTRIBUTING.md` for the exact commands.

**Verified working alternative on macOS** (works regardless of which
Python is on `PATH`, since maturin does its own interpreter detection):
`maturin develop --release && pytest tests/` — this pass ran that and got
`229 passed, 0 failed` (2026-09-21, up from `227 passed, 2 skipped` on
2026-09-20 — the 2 `skipped` were `test_performance.py`'s
`skipif(not HAS_SKLEARN, ...)` cases; this pass's environment has
scikit-learn installed where the prior pass's sandbox didn't, so they ran
instead of skipping — not a change made by this pass).

### CI verification limits in this pass

This pass ran from a sandbox with restricted network access: `pip install`
to PyPI worked, but `git` fetches over HTTPS to `github.com` (used by both
`gh api`/`gh run list` and `cargo audit`'s advisory-database fetch) timed
out. As a result:
- Live GitHub Actions run status for `ci.yml`/`tests.yml` could **not** be
  checked from this session — the README's existing `tests.yml` status
  badge was left as-is (pre-existing, not added by this pass) but its
  current live state is unverified here.
- The newly-added `security-audit` job in `ci.yml` (`cargo install
  cargo-audit && cargo audit`) could not be run locally for the same
  reason. GitHub-hosted runners have full internet access and should not
  hit this, but that's inference, not a verified test run — check the
  Actions tab after this lands.
- `cargo clippy`/`cargo fmt --check`/`cargo test` results quoted throughout
  this file are real, local, macOS (Apple Silicon) runs from 2026-09-20 —
  not CI runs.

### CI structural gaps (documented, not changed — see reasoning)

- **No `clippy`/`fmt` gate in CI at all.** Neither `ci.yml` nor `tests.yml`
  runs `cargo clippy` or `cargo fmt --check`. Adding `cargo fmt --check` as
  a real gate would be safe (verified clean, see above). Adding `cargo
  clippy -- -D warnings` as a real gate would **not** be safe right now — it
  would immediately fail CI on the 30 pre-existing warnings documented
  above (down from 37 as of 2026-09-21 — see "Known lint/format debt";
  this pass fixed the mechanical subset (dead code, non-snake-case
  naming) but not the rest). Not added, to avoid either breaking CI or
  adding a fake always-passing step (the exact anti-pattern already
  called out twice in this file's own history for the pytest and ruff
  steps). A real follow-up would be: fix or `#[allow]` the remaining 30
  findings first (all in already-identified deferred/unwired modules), then
  add both gates for real.
- **`ci.yml` and `tests.yml` overlap.** Both install the package with `pip
  install -e ".[dev]"` and run `pytest` across Python 3.10–3.12 on
  `ubuntu-latest` — effectively the same job defined twice under different
  names, doubling CI minutes for no extra coverage. Not merged in this pass
  (would need to decide which workflow file is canonical and update the
  README badge accordingly — a judgment call left for a dedicated session).
- **`.pre-commit-config.yaml` hooks are unverified.** It references
  `rust-lang/rust-clippy`/`rust-lang/rustfmt` as pre-commit hook
  repositories; whether those repos actually expose the `.pre-commit-hooks.yaml`
  entries this config assumes (`id: clippy`, `id: rustfmt`) was not checked
  in this pass (would require running `pre-commit run --all-files`, which
  needs network access this session didn't reliably have). If you rely on
  `make install`/`pre-commit install`, verify it actually runs before
  trusting it as a gate.

### Technical debt inventory (not fixed — flagged per this file's purpose)

- **`src/utils/conversions.rs`**: `pandas_to_arrow`/`arrow_to_pandas` remain
  unimplemented stubs (`Err("Not implemented")`) — unchanged from previous
  audits, still not called from anywhere, still not exposed to Python.
- ~~`src/engine/metrics.rs` is dead code.~~ — **Deleted (2026-09-21).** It
  predated and duplicated part of `src/engine/quality_metrics.rs` (the
  module actually wired to Python as `silhouette_score`/
  `davies_bouldin_score`/etc). Confirmed nothing in `python.rs` or any
  other `engine::` module called into it (`grep` for `metrics::` and
  `engine::metrics` outside the file itself found nothing) before deleting
  it and its `pub mod metrics;` declaration in `src/engine/mod.rs`. Removed
  3 clippy warnings.
- **High `.unwrap()`/`.expect()` counts in Python-reachable modules**,
  meaning a bad input can trigger a Rust panic (which PyO3 converts to a
  Python `PanicException`, so it won't crash the whole interpreter, but it's
  a worse error experience than a proper `Result`/`PyErr` and wasn't audited
  for which call sites are actually reachable with attacker/user-controlled
  input vs. genuinely-impossible states). Counts from `grep -c
  '\.unwrap()'` per file, wired modules only: `clustering.rs` 41,
  `streaming.rs` 25, `cohorts.rs` 29, `mod.rs` 11, `drift_detection.rs` 15,
  `k_estimation.rs` 14, `clv.rs` 14,
  `churn_prediction.rs` 12, `quality_metrics.rs` 11, `segments.rs` 8,
  `rfm.rs` 7, `sql_export.rs` 7, `lookalike.rs` 6, `lifecycle.rs` 9,
  `heuristic_score_estimator.rs` 9, `behavioral.rs` 1. Not all of these are
  reachable with external input (many are on `Vec` indices already bounds-
  checked a few lines earlier, or on values the caller can't influence) —
  this is a raw count to prioritize a real audit, not a claim that all 219
  (233 minus the 14 that were in the now-deleted `metrics.rs`) are live
  bugs.

### Small fixes applied in this pass

- Root `CONTRIBUTING.md`: added the missing Rust/PyO3 build section
  (`RUSTFLAGS` macOS workaround, the `cargo test` macOS bug above, the
  verified `maturin develop && pytest` alternative); fixed a stale
  contributor-license line still saying "proprietary license" (repo is
  Apache-2.0); replaced a fabricated `AudienceSegmenter(...).fit_predict()`
  test example (that method doesn't exist) with a real, verified-to-run one
  using `fit()`/`predict()`; replaced an unmeasurable/unmeaningful ">90%
  coverage on the Python API" instruction (Python `coverage.py` can't
  instrument the compiled Rust extension — it only ever measured the
  24-line `__init__.py` shim, trivially at 100%) with guidance to judge test
  adequacy directly; removed an unverifiable "review within 7 days" SLA
  promise.
- Root `SECURITY.md`: removed an unverifiable "acknowledge within 24 hours"
  SLA promise (this is maintained by one person with no formal SLA).
- `.github/workflows/tests.yml`: the `Lint` step was `ruff check . 2>/dev/null
  || true` — discarded output and always exited 0 regardless of findings,
  the same "always green" anti-pattern already fixed once in this same
  file's `Test` step (see `[7.2.0]`-era `CHANGELOG.md` entry) but
  reintroduced here. `ruff check .` currently reports 33 real findings in
  `tests/`, so making this a hard gate immediately would just turn CI red on
  pre-existing lint debt unrelated to this pass. Changed to
  `continue-on-error: true` instead — output is no longer discarded and the
  step shows as failed/non-blocking in the Actions UI (visible), rather than
  silently swallowed (invisible). Also bumped `actions/setup-python@v4` to
  `@v5` (flagged by `actionlint` as too old to run).
- `.github/workflows/ci.yml`: added a new `security-audit` job running
  `cargo audit` (see "CI verification limits" above for why it's unverified
  from this session).
- `.gitignore`: added `.coverage`/`htmlcov/`/`.mypy_cache/`/`.ruff_cache/`/
  `.hypothesis/` (all correspond to already-declared dev dependencies:
  pytest-cov-style coverage output, mypy, ruff) and `.benchmarks/`/
  `.deepeval/` (untracked local tool-output directories present in this
  checkout that weren't previously ignored).

### Quick-fix pass (2026-09-21)

A follow-up pass explicitly scoped to small, verifiable fixes flagged above
— no refactors, no dependency bumps, no delete-vs-implement calls on stub
code. Every item below was verified by an actual local run (`ruff check .`,
`cargo clippy`, `cargo fmt --check`, `cargo build`, `python -m pytest`,
`cargo test`), not just read and assumed correct.

- **`ruff check .` — all 33 findings fixed** (`.github/workflows/tests.yml`'s
  `Lint` step is a real blocking gate again, see above). 10 were
  autofixable (`ruff check . --fix`: import sorting, `__all__` sort order,
  a redundant f-string prefix) with a verified no-semantic-change diff. The
  other 23 were fixed by hand, all mechanical given the actual Rust source
  each test exercises:
  - `tests/test_clustering.py` (3) and `tests/test_wired_modules.py` (2):
    `pytest.raises(Exception)` narrowed to the real exception type each
    PyO3 binding actually raises (`RuntimeError` for `kmeans`'s
    `ClusterAudienceKitError`-mapped errors in `src/python.rs`'s
    `kmeans_py`; `ValueError` for `parse_stream_event_type`/
    `parse_lifecycle_stage`'s `PyValueError`s) — confirmed by reading the
    `PyErr::new::<...>` call sites, not guessed.
  - `examples/basic_segmentation.py`, `tests/test_basic.py`,
    `tests/test_performance.py`: `datetime.datetime(...)` calls given an
    explicit `tzinfo=timezone.utc` (`DTZ001`) — safe because in all three
    files the value is only ever consumed via `.strftime(...)`, confirmed
    by grepping every use before changing it.
  - `tests/test_sql_export.py`, `tests/test_wired_modules.py`: two
    `RUF012` mutable-class-attribute warnings fixed with `ClassVar`
    annotations; one blind `except Exception: pass` in a SQL-injection
    test narrowed to `except ValueError` (same reasoning as above, verified
    against `export_segment_sql`'s error mapping) which also resolved the
    paired `S110`/`BLE001` findings; three `RUF059` unused-unpacked-variable
    findings fixed by prefixing with `_`.
  - `examples/sql_export_example.py`: two `F841` unused-variable findings
    fixed by not binding the return value; one bare `except:` narrowed to
    `except ValueError as e:` (same `export_segment_sql`/
    `get_segment_rfm_patterns` reasoning); `EXE001` fixed with `chmod +x`
    (the file already has a shebang). The top-level example-runner's
    `except Exception as e:` (deliberately generic — it must survive
    whatever any of the 10 heterogeneous example functions raises) was
    left broad but suppressed with an explicit `# noqa: BLE001` and a
    comment explaining why, rather than guessing at a narrower type that
    would defeat its purpose.
  - Full local suite after all of the above: `python -m pytest tests/ -q`
    → **229 passed, 0 failed** (`maturin develop --release` build, Python
    3.9 via the system interpreter).
- **`cargo clippy --workspace --all-targets` — was silently broken, now
  fixed, plus mechanical cleanup.** See "Known lint/format debt" above for
  the `approx_constant` compile-error fix, the `src/engine/metrics.rs`
  deletion, and the `neural_networks.rs` `X` → `inputs` rename. Warning
  count: 37 → 30.
- **`cargo test` on macOS — fixed.** See the dedicated section above.
  `Cargo.toml`'s `pyo3` dependency no longer hardcodes the
  `extension-module` feature; it's now behind a `default`-on crate feature
  of the same name, which every existing build path (`cargo build`,
  `cargo bench`, `cargo clippy`, `maturin develop`, and CI's
  `--all-features` invocations) still gets automatically, verified
  byte-for-byte unaffected (same clippy warning count, same `cargo fmt
  --check` clean result, same `pytest` pass count against the `maturin
  develop`-built wheel). `Cargo.lock` is unchanged (no dependency-graph
  impact). New working path: `cargo test --release --no-default-features
  --lib` (needs a Python with a linkable `libpython`; see `CONTRIBUTING.md`).

---

## Not Planned

- Hierarchical clustering (removed from README/tests; never implemented
  anywhere in this codebase).
- GPU acceleration.
- Real-time streaming *ingestion* from external message queues/Kafka/etc
  (the in-process streaming *segmentation engine* described above is real
  and shipped; connecting it to an external event source is a different,
  larger integration).
