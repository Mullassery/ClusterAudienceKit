# ClusterAudienceKit

**Rust-powered customer segmentation with a Python API: RFM analysis, clustering, churn scoring, CLV, and privacy-aware audience tooling in one dependency.**

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![PyPI](https://img.shields.io/pypi/v/clusteraudiencekit)](https://pypi.org/project/clusteraudiencekit/)
[![Python 3.8+](https://img.shields.io/badge/Python-3.8%2B-blue)](https://www.python.org)
[![Tests](https://img.shields.io/github/actions/workflow/status/Mullassery/ClusterAudienceKit/tests.yml?label=tests)](https://github.com/Mullassery/ClusterAudienceKit/actions)
[![Graphify](https://img.shields.io/badge/Graphify-knowledge%20graph-blue)](https://github.com/Graphify-Labs/graphify)

## What it is

ClusterAudienceKit is a Rust-powered customer segmentation engine with Python bindings.
It covers RFM analysis, KMeans/K-Prototypes clustering, cluster-quality metrics, automatic
K selection, churn prediction, customer lifetime value, SQL export to 8 warehouse dialects,
differential privacy / k-anonymity, real-time streaming segmentation, drift detection,
lookalike audiences, cohort analytics, lifecycle tracking, rule-based behavioral
segmentation, and segment profiling — all real, tested Rust logic exposed through the
compiled Python extension (`import clusteraudiencekit`).

## Features

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

Segmentation output is 13 named RFM segments (Champions, Loyal Customers, Potential
Loyalists, At Risk, Cannot Lose Them, About to Sleep, New Customers, Promising, Need
Attention, Lost, At Risk - Sleeping, Hibernating, VIP).

## Who it's for

- **Marketing and data teams** building a segmentation pipeline on a large transaction
  table where scikit-learn's Python-level clustering is the bottleneck — the Rust core is
  rayon-parallelized and deterministic for a given seed.
- **Teams exporting customer segments directly into a warehouse** — `export_segment_sql`
  generates injection-safe SQL for 8 dialects rather than hand-writing per-warehouse
  export scripts.
- **Privacy-constrained segmentation** — differential privacy (Laplace/Gaussian noise) and
  k-anonymity suppression/generalization are real, tested primitives, not a compliance
  checkbox.

## What it is not

- **Not external platform activation.** Pushing segments to ad/CRM platforms, B2B
  governance/workflow tooling, a dashboard UI, and a plugin framework are deliberately out
  of scope for this library.
- **Not a scikit-learn wrapper.** The algorithms are real, independently unit-tested Rust
  implementations, not thin bindings over Python libraries.
- **Not feature-scaling-aware by default.** `AudienceSegmenter` does no internal
  standardization; on real data you should standardize recency/frequency/monetary yourself
  before calling `.fit()` (see [Known issues](#known-issues)).
- **K-Prototypes categorical support is partial.** `AudienceSegmenter.fit()` accepts only a
  numeric feature matrix today, so selecting K-Prototypes through that class runs
  numeric-only (effectively KMeans). The underlying Rust implementation does support mixed
  data; it just isn't reachable from that Python entry point yet.
- **Six Rust modules are real but not yet exposed to Python:** `segment_intelligence`,
  `pattern_discovery`, `temporal_analytics`, `price_intelligence`, `revenue_intelligence`,
  and `neural_networks`. They're tested, not stubs; wiring them to the Python API is a
  follow-up release.

## Project status

**Production/Stable** per the package classifier (version 7.4.1), with the honest gaps
above. This project tracks its own implementation-vs.-plan status on purpose; see
[`docs/ROADMAP_HONEST.md`](docs/ROADMAP_HONEST.md). Changes are tracked in
[CHANGELOG.md](CHANGELOG.md).

## Installation

```bash
pip install clusteraudiencekit
```

Requires Python 3.8+, with NumPy, Pandas, and PyArrow (see `pyproject.toml` for exact
ranges). The compiled Rust core ships as a platform wheel, so no local Rust toolchain is
needed — **but** the only wheel currently published to PyPI is macOS ARM64 (`cp39-abi3`),
alongside an sdist. On any other platform, `pip install` falls back to building the sdist
from source, which does require a Rust toolchain; or clone the repo and build locally with
[`maturin`](https://github.com/PyO3/maturin) (`pip install maturin && maturin develop
--release`).

## Quick start

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

## Usage

### Why ClusterAudienceKit?

Marketing/data teams need RFM segmentation, clustering, churn scoring, and CLV estimation,
usually stitched together from several tools. This package does the core numeric work in
Rust (fast, deterministic, real unit-tested algorithms — not scikit-learn wrappers) with a
Python API, so you get one dependency instead of five, and you can inspect exactly what's
real (see [`docs/ROADMAP_HONEST.md`](docs/ROADMAP_HONEST.md)).

### vs scikit-learn & lifetimes

No single OSS package covers everything here, so this compares the two pieces with direct
real competitors: clustering vs `scikit-learn`'s KMeans, and CLV vs the `lifetimes`
package's real BG/NBD probabilistic model. Tested against a real public e-commerce
dataset — the UCI "Online Retail" transaction log (541,909 real transactions, cleaned to
397,884 real rows / 4,338 real customers with valid quantity/price/customer ID) — not
synthetic data.

| | ClusterAudienceKit | scikit-learn |
|---|---|---|
| RFM computation (4,338 real customers) | 0.026s | — (not scikit-learn's job) |
| KMeans, K=5, on raw RFM features | 0.001s, silhouette=0.9502 | 0.025s (unscaled, matching CAK's real input), silhouette=0.8387, `RuntimeWarning: overflow encountered in matmul` |
| KMeans, K=5, standardized features | — (no built-in scaling) | 0.080s, silhouette=0.7744 |

**The silhouette numbers above are misleading on their own, and this is the real,
important finding, not the timing.** Inspecting the actual cluster sizes on real,
unscaled RFM features: **4,300 of 4,338 real customers (99.1%) land in one giant
cluster**, with 4 tiny outlier clusters splitting off customers with extreme real monetary
values (one real customer spent $280,206 vs. a $2,054 mean). That's a textbook
unscaled-KMeans failure — `monetary` has ~450x the numeric range of
`recency`/`frequency`, so raw Euclidean distance is dominated by it, and the "high"
silhouette score is an artifact of that degenerate split, not good segmentation.
Standardize recency/frequency/monetary yourself (e.g.
`sklearn.preprocessing.StandardScaler`) before calling `.fit()` on real data.

| | ClusterAudienceKit `calculate_simple_ltv` | `lifetimes` (BG/NBD + Gamma-Gamma) |
|---|---|---|
| Method | Deterministic: `(total_spent / days_active) * 365 * lifespan_years` | Real probabilistic model fit on repeat-purchase patterns |
| Repeat-purchaser customers scored by both | 2,790 real customers | 2,790 real customers |
| Rank correlation (Spearman ρ) between the two tools' CLV rankings | **0.7111** (p < 1e-300) | — |

A substantial, real positive correlation — the two independently-computed approaches
broadly agree on who a business's most valuable real customers are.

## Documentation

- [Honest roadmap](docs/ROADMAP_HONEST.md) — what's real, what's deferred, and why.
- [Architecture](docs/architecture/README.md) — real module layout and data flow, checked
  against `src/`.
- [Security audit](docs/SECURITY_AUDIT.md)
- [SQL export reference](docs/SQL_EXPORT.md)
- [Examples](examples/)

## Known issues

Verified as of the last audit (October 2026):

- **The only wheel on PyPI is macOS ARM64 (`cp39-abi3`), plus an sdist.** A multi-platform
  wheel-building CI workflow (`.github/workflows/wheels.yml`) builds and tests wheels on
  Linux, macOS (arm64 + x86_64), and Windows, and is wired to publish via Trusted
  Publishing on tagged releases; the PyPI-side Trusted Publisher still has to be configured
  before that job can succeed.
- **No internal feature standardization.** `AudienceSegmenter` does not standardize inputs,
  and neither the quick-start example nor `assess_cluster_quality` warns about the
  unscaled-KMeans failure described above. Standardize your features first on real data.
- **K-Prototypes categorical support is partial** (numeric-only through
  `AudienceSegmenter.fit()`; see above).
- **`v7.0.0` remains installable from PyPI despite a confirmed import-crashing bug.** If you
  have it pinned, upgrade to `>=7.2.0`.
- **`cargo audit`: 1 open advisory with no fix available.** `anyhow` 1.0.102 has an
  unsoundness advisory in `Error::downcast_mut()`
  ([RUSTSEC-2026-0190](https://rustsec.org/advisories/RUSTSEC-2026-0190)); tracked in
  [RepoIssues #34](https://github.com/Mullassery/RepoIssues/issues/34).
- **Two Rust utility functions are unimplemented stubs.** `pandas_to_arrow`/`arrow_to_pandas`
  in `src/utils/conversions.rs` return `Err("Not implemented")`; they aren't called elsewhere
  or exposed to Python.
- **Six tested Rust modules are implemented but not yet wired to the Python API** (see
  "What it is not").

## Known issues and deferred work

**Blocked / failed:**
- Linux/Windows wheels.
- CLV/churn validation against lifetimes/PyMC-Marketing.

**Rejected / deferred:**
- A full CDP or warehouse; integrate with MessageBirds/PyReverseETL.

Full shipped/blocked/rejected detail: [`QUEUE_STATUS.md`](QUEUE_STATUS.md).

## Queue status

Shipped, blocked/failed, and rejected work for this repository is documented in
[`QUEUE_STATUS.md`](QUEUE_STATUS.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Development quickstart:

```bash
maturin develop --release            # build the Rust extension + install editable
cargo test --no-default-features     # Rust unit tests
pytest tests/ -v                     # Python tests
```

Note the `--no-default-features` flag: the default `extension-module` feature omits linking
against libpython, which a standalone `cargo test` binary needs. See `CONTRIBUTING.md` and
[`docs/ROADMAP_HONEST.md`](docs/ROADMAP_HONEST.md) for details.

## Related projects

Other tools from the same family, for adjacent parts of the customer/data stack:

- [PyReverseETL](https://github.com/Mullassery/PyReverseETL) — reverse ETL runtime: move data into CRMs and marketing tools with lineage and compliance enforcement.
- [PySynthData](https://github.com/Mullassery/PySynthData) — synthetic data generation for ML training: realistic datasets for model development and testing.
- [StatGuardian](https://github.com/Mullassery/StatGuardian) — fast, declarative data quality framework (Rust): schema validation, drift detection, anomaly detection.
- [PyWeatherEnriched](https://github.com/Mullassery/PyWeatherEnriched) — real historical weather and geocoding enrichment for ML pipelines.
- [PyDBTGuard](https://github.com/Mullassery/PyDBTGuard) — analyzes a dbt project's `manifest.json` to flag risky tests, map blast radius, and estimate test cost.

[Browse all projects →](https://github.com/Mullassery?tab=repositories)

## License

Apache License 2.0. See [`LICENSE`](LICENSE) for the full terms.
