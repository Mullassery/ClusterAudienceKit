# ClusterAudienceKit Roadmap (Honest)

**Current Version:** 7.1.x
**Last Updated:** August 2026 (post remediation pass)
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

- `cargo fmt --check` is now clean across the whole repository (it wasn't —
  effectively every file had unapplied formatting deltas before this pass;
  `cargo fmt` was run repo-wide).
- `cargo clippy --workspace -- -D warnings` is **not** fully clean
  repo-wide. Before this pass: 214 errors. After fixing the modules touched/
  wired this release (python.rs, clustering.rs, rfm.rs, sql_export.rs,
  churn_prediction.rs, heuristic_score_estimator.rs, mod.rs, cohorts.rs,
  streaming.rs, quality_metrics.rs, k_estimation.rs, clv.rs, segments.rs):
  **43 remain**, concentrated entirely in modules that are either explicitly
  deferred (`b2b_governance`, `dashboard`, `activation`,
  `activation_orchestrator`) or Rust-only/not-yet-wired this pass
  (`neural_networks`, `pattern_discovery`, `temporal_analytics`,
  `segment_intelligence`, `price_intelligence`, `algorithms`, `metrics`,
  `b2b_segmentation`). None are in code paths reachable from the Python API.

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

---

## Not Planned

- Hierarchical clustering (removed from README/tests; never implemented
  anywhere in this codebase).
- GPU acceleration.
- Real-time streaming *ingestion* from external message queues/Kafka/etc
  (the in-process streaming *segmentation engine* described above is real
  and shipped; connecting it to an external event source is a different,
  larger integration).
