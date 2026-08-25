# ClusterAudienceKit

**A Rust-powered customer segmentation engine with Python bindings.**

RFM analysis, KMeans/K-Prototypes clustering, churn prediction, customer
lifetime value, SQL export to 8 warehouse dialects, differential privacy /
k-anonymity, real-time streaming segmentation, drift detection, lookalike
audiences, cohort analytics, lifecycle tracking, rule-based behavioral
segmentation, and segment profiling — all real, tested, and callable from
Python today.

[![Tests](https://img.shields.io/github/actions/workflow/status/Mullassery/ClusterAudienceKit/tests.yml?label=tests)](https://github.com/Mullassery/ClusterAudienceKit/actions)
[![PyPI](https://img.shields.io/pypi/v/clusteraudiencekit)](https://pypi.org/project/clusteraudiencekit/)
[![Python 3.8+](https://img.shields.io/badge/Python-3.8%2B-blue)](https://www.python.org)

---

## 30-Second Start

```python
from clusteraudiencekit import AudienceSegmenter, RFMConfig, calculate_rfm

# transactions: list of (customer_id, iso8601_date, amount).
# n_clusters must be <= the number of distinct customers — use your real,
# larger transaction history here; this toy example has 3 customers.
transactions = [
    ("cust_1", "2026-06-01T00:00:00+00:00", 120.0),
    ("cust_1", "2026-07-15T00:00:00+00:00", 80.0),
    ("cust_2", "2026-01-10T00:00:00+00:00", 15.0),
    ("cust_3", "2026-08-01T00:00:00+00:00", 500.0),
]

# Real RFM scoring (recency/frequency/monetary, quintile-scored, 13-segment
# classification), not a mock.
scores = calculate_rfm(transactions, RFMConfig())

# Cluster customers by their RFM features with real KMeans (k-means++ init,
# rayon-parallelized assignment step, deterministic for a given seed).
features = [[s.recency, s.frequency, s.monetary] for s in scores]
segmenter = AudienceSegmenter(2)
segmenter.fit(features)
segments = segmenter.predict(features)
```

---

## Why ClusterAudienceKit?

Marketing/data teams need RFM segmentation, clustering, churn scoring, and
CLV estimation, usually stitched together from several tools. This package
does the core numeric work in Rust (fast, deterministic, real unit-tested
algorithms — not scikit-learn wrappers) with a Python API, so you get one
dependency instead of five, and you can inspect exactly what's real (see
[`docs/ROADMAP_HONEST.md`](docs/ROADMAP_HONEST.md) — this project tracks its
own honesty about what's implemented vs. planned, on purpose).

---

## What's real today

Everything below is backed by real Rust logic with `cargo test` coverage
**and** exposed through the compiled Python extension (`import
clusteraudiencekit`) with its own Python-level tests — not a stub, not a
mock, not aspirational documentation.

| Capability | Python entry points |
|---|---|
| RFM analysis | `calculate_rfm`, `RFMConfig`, `RFMScore` |
| KMeans / K-Prototypes clustering | `kmeans`, `AudienceSegmenter` |
| Cluster quality metrics | `silhouette_score`, `davies_bouldin_score`, `calinski_harabasz_score`, `assess_cluster_quality` |
| Automatic K selection | `estimate_k_elbow`, `estimate_k_gap_statistic`, `estimate_k_silhouette`, `estimate_k_combined` |
| Churn prediction (incl. real AUC-ROC) | `ChurnPrediction`, `ChurnRiskLevel` |
| Customer lifetime value | `CustomerLTV`, `calculate_simple_ltv` |
| SQL export (8 dialects, injection-safe) | `export_segment_sql`, `export_all_segments_sql`, `get_supported_sql_dialects` |
| Differential privacy & k-anonymity | `PyPrivacyBudget`, `add_laplace_noise`, `add_gaussian_noise`, `check_k_anonymity`, `suppress_to_k_anonymous`, `generalize_numeric` |
| Real-time streaming segmentation | `PyStreamingSegmentationEngine`, `PyStreamingEvent`, `PyStreamingConfig` |
| Drift detection | `kolmogorov_smirnov`, `hellinger_distance`, `chi_square_drift`, `detect_feature_drift`, `detect_segment_composition_change` |
| Lookalike audiences | `generate_lookalike`, `find_similar_customers`, `cosine_similarity` |
| Cohort analytics | `create_cohort`, `cohort_id_for`, `compare_cohorts`, `aggregate_cohorts_by_period`, `cohort_retention_table` |
| Lifecycle tracking | `classify_lifecycle_stage`, `lifecycle_retention_actions`, `lifecycle_stage_distribution` |
| Rule-based behavioral segmentation | `PyBehavioralSegmenter`, `PyBehavioralSegment`, `PyBehavioralRule`, `PyCondition` |
| Segment profiling | `profile_segment` |

**Segmentation output**: 13 named RFM segments (Champions, Loyal Customers,
Potential Loyalists, At Risk, Cannot Lose Them, About to Sleep, New
Customers, Promising, Need Attention, Lost, At Risk - Sleeping, Hibernating,
VIP).

### What's real but not yet exposed to Python

`segment_intelligence`, `pattern_discovery`, `temporal_analytics`,
`price_intelligence`, `revenue_intelligence`, and `neural_networks` are real,
tested Rust modules (not stubs) that are large enough we deferred wiring
them to a follow-up release rather than rush it. See
[`docs/ROADMAP_HONEST.md`](docs/ROADMAP_HONEST.md) for specifics on each.

### What's explicitly out of scope

External platform activation (pushing segments to ad/CRM platforms),
B2B governance/workflow tooling, a dashboard UI, and a plugin framework are
deliberately not part of this library — see
[`docs/ROADMAP_HONEST.md`](docs/ROADMAP_HONEST.md) for why.

---

## Requirements

- Python 3.8+
- NumPy, Pandas, PyArrow (see `pyproject.toml` for exact ranges)
- Precompiled Rust core (ships as a platform wheel; no local Rust toolchain
  needed to install) — **but see the platform caveat below**

## Installation

```bash
pip install clusteraudiencekit
```

The only wheel currently published to PyPI is macOS ARM64 (cp39). There is
no source distribution and no CI pipeline building Linux/Windows wheels yet,
so `pip install` will fail on other platforms today — see
[Known Issues](#known-issues). To use this on Linux/Windows/other Python
versions in the meantime, clone the repo and build locally with
[`maturin`](https://github.com/PyO3/maturin) (`pip install maturin && maturin
develop --release`), which does require a Rust toolchain.

## Documentation

- [Honest roadmap](docs/ROADMAP_HONEST.md) — what's real, what's deferred,
  and why.
- [Security audit](docs/SECURITY_AUDIT.md)
- [SQL export reference](docs/SQL_EXPORT.md)
- [Examples](examples/)

## Known Issues

Verified as of this audit (August 2026):

- **Published wheels are single-platform.** The latest PyPI release
  (7.2.0) ships only a macOS ARM64 / cp39 wheel, with no source
  distribution, same as `7.1.1` and `v7.1.0` before it (macOS ARM64 /
  cp313). There is currently no CI job that builds Linux or Windows wheels
  (`.github/workflows/` only runs tests on `ubuntu-latest`, not a release
  build matrix), despite `pyproject.toml` classifying the package as
  `OS Independent` and supporting Python 3.8–3.12. In practice, `pip
  install clusteraudiencekit` only works out of the box on macOS ARM64
  with a matching Python; everyone else needs to build from source with
  `maturin` (see Installation above).
- **`v7.0.0` remains installable from PyPI despite a confirmed
  import-crashing bug.** It was never yanked. If you have it pinned,
  upgrade to `>=7.2.0`.
- **K-Prototypes categorical support is partial.** `AudienceSegmenter.fit()`
  only accepts a numeric feature matrix today, so selecting K-Prototypes
  through that class currently runs in numeric-only mode (effectively
  KMeans). The underlying `engine::clustering::kprototypes` Rust
  implementation does support mixed numeric/categorical data; it just
  isn't reachable from that Python entry point yet. Details in
  [`docs/ROADMAP_HONEST.md`](docs/ROADMAP_HONEST.md).
- **Two Rust utility functions are unimplemented stubs.**
  `pandas_to_arrow`/`arrow_to_pandas` in `src/utils/conversions.rs` return
  `Err("Not implemented")`. They are not called from anywhere else in the
  crate and are not exposed to Python, so they don't affect any documented
  functionality — noted here for completeness.
- **Registry check:** local version (`7.2.0`, in `Cargo.toml` and
  `pyproject.toml`) matches the latest version actually published on PyPI.
  No drift.
- **No open GitHub issues** at the time of this audit.
- Six real, tested Rust modules are implemented but not yet wired to the
  Python API (`segment_intelligence`, `pattern_discovery`,
  `temporal_analytics`, `price_intelligence`, `revenue_intelligence`,
  `neural_networks`) — see "What's real but not yet exposed to Python"
  above.

## License

Proprietary — free to use with explicit attribution. See
[`LICENSE`](LICENSE) for the full terms.
