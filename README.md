# ClusterAudienceKit v2.0

**Production-grade customer segmentation platform — RFM analysis, clustering, streaming updates, drift detection, CLV prediction, lifecycle tracking, cohort analytics, and enterprise platform activation.**

ClusterAudienceKit is the complete segmentation stack for modern MarTech. Replace your scikit-learn + pandas + lifetimes + Segment.com combination with a single, production-ready platform backed by a Rust engine that handles 1M+ customers in milliseconds.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Python](https://img.shields.io/badge/python-3.8%20%7C%203.9%20%7C%203.10%20%7C%203.11%20%7C%203.12%20%7C%203.13-blue)](pyproject.toml)
[![PyPI](https://img.shields.io/badge/pypi-clusteraudiencekit-orange)](https://pypi.org/project/clusteraudiencekit/)
[![Version](https://img.shields.io/badge/version-2.0.0-green)](CHANGELOG.md)
[![GitHub Issues](https://img.shields.io/github/issues/Mullassery/ClusterAudienceKit)](https://github.com/Mullassery/ClusterAudienceKit/issues)

## Install

Pick one:

```bash
pip install clusteraudiencekit
```

OR

```bash
uv add clusteraudiencekit
```

OR

```bash
curl -sSfL https://raw.githubusercontent.com/Mullassery/ClusterAudienceKit/main/install.sh | sh
```

Pre-built wheels for all platforms: [INSTALL.md](INSTALL.md)

## Get started in 10 lines

```python
from clusteraudiencekit import AudienceSegmenter
import pandas as pd

# Required columns: customer_id, transaction_date, amount
transactions = pd.read_csv('transactions.csv')

segmenter = AudienceSegmenter(method='rfm_kmeans', n_clusters=4)
segmenter.fit(transactions)

segments = segmenter.predict(transactions)
profiles = segmenter.segment_profiles()
print(profiles)
#   segment | size  | avg_recency | avg_frequency | avg_monetary
#   0       | 250k  | 15.3 days   | 8.2 purchases | $450   <- high-value loyalists
#   1       | 180k  | 45.2 days   | 3.1 purchases | $120   <- regular buyers
#   2       | 320k  | 2.1 days    | 2.0 purchases | $80    <- new / recent
#   3       | 250k  | 60.5 days   | 1.0 purchases | $30    <- at-risk / dormant

print(f"Silhouette score: {segmenter.silhouette_score():.3f}")
```

## Why not just use scikit-learn?

You can — until your audience grows. `sklearn.metrics.silhouette_score` is O(n²): at 100k customers it takes over 2.7 hours. At 1M customers it won't finish. ClusterAudienceKit handles both in under half a second.

**Measured timings on Apple M1 (sklearn 1.6.1, pandas 3.0.3):**

| Customer base | sklearn + pandas | ClusterAudienceKit |
|---------------|:----------------:|:------------------:|
| 1,000         | 38ms             | <9ms               |
| 10,000        | 606ms            | <37ms              |
| 100,000       | >2.7 hours       | <130ms             |
| 1,000,000     | Did not complete | <470ms             |

Beyond performance, you get the complete production stack:

| Capability | sklearn | pandas | lifetimes | Braze | Klaviyo | **ClusterAudienceKit v2.0** |
|------------|:---:|:---:|:---:|:---:|:---:|:---:|
| RFM calculation | — | manual | — | ✓ | ✓ | ✓ |
| KMeans clustering | ✓ | — | — | — | — | ✓ |
| DBSCAN, Hierarchical, GMM | partial | — | — | — | — | ✓ |
| Auto K-estimation | — | — | — | — | — | ✓ |
| Customer lifetime value (CLV) | — | — | ✓ | — | — | ✓ |
| Churn prediction | — | — | partial | ✓ | ✓ | ✓ |
| Streaming/incremental updates | — | — | — | — | — | ✓ |
| **Segment drift detection** | — | — | — | — | — | **✓** |
| **Cohort analytics** | — | — | — | partial | — | **✓** |
| **Lifecycle tracking** | — | — | — | — | — | **✓** |
| **7 platform integrations** | — | — | — | 1 | 1 | **7+** |
| **Production dashboard** | — | — | — | — | — | **✓** |
| Quality metrics + profiling | ✓ (slow) | — | — | — | — | ✓ (fast) |
| Multi-core by default | partial | — | — | — | — | ✓ |

Full comparison with code examples: [docs/comparison.md](docs/comparison.md) · Full benchmark methodology: [BENCHMARKS.md](BENCHMARKS.md)

## Segmentation methods

### RFM + KMeans

Scores each customer on Recency, Frequency, and Monetary value, then groups them with KMeans. The standard approach for most Martech teams.

```python
segmenter = AudienceSegmenter(method='rfm_kmeans', n_clusters=4)
segmenter.fit(df)
```

### RFM + K-Prototypes

Extends RFM with categorical attributes — acquisition channel, product category, region — so your segments reflect more than just spend behaviour.

```python
segmenter = AudienceSegmenter(method='rfm_kprototypes', n_clusters=5)
segmenter.fit(df, categorical_columns=['channel', 'region', 'product_category'])
```

### Streaming updates

Update segments incrementally as daily events arrive, without reprocessing your full customer history. Detect and react to campaign-driven drift:

```python
segmenter.fit(historical_data)

for daily_events in event_stream:
    segmenter.update(daily_events)

    stability = segmenter.segment_stability(previous_segments)
    if stability < 0.85:
        segmenter.fit(all_data, refit=True)

    previous_segments = segmenter.predict(customers)
```

### PySpark integration

Use ClusterAudienceKit with Apache Spark DataFrames for large-scale customer segmentation on distributed clusters.

```python
from pyspark.sql import SparkSession
import polars as pl
from clusteraudiencekit import AudienceSegmenter

spark = SparkSession.builder.appName("audience-segmentation").getOrCreate()

# Load customer transaction data from Spark
spark_df = spark.read.parquet("s3://bucket/transactions/")

# Convert to Polars for segmentation (small-scale, in-memory)
polars_df = spark_df.select("customer_id", "purchase_amount", "purchase_date") \
    .toPandas()
polars_df = pl.from_pandas(polars_df)

# Fit segmentation model
segmenter = AudienceSegmenter(method='rfm_kmeans', n_clusters=5)
segmenter.fit(polars_df)

# Get segment assignments
segments = segmenter.predict(polars_df)

# Write segments back to Spark
segments_df = spark.createDataFrame(
    segments.to_pandas(),
    schema=["customer_id", "segment"]
)
segments_df.write.mode("overwrite").parquet("s3://bucket/segments/")

print(f"Segmented {segments_df.count()} customers into {segmenter.n_clusters} segments")
```

**Note:** For very large datasets, consider:
- Sampling/filtering in Spark before converting to Polars
- Running segmentation on aggregated RFM scores per customer (reduces memory footprint)
- Caching the Polars DataFrame if running multiple predictions

## Configuration

```python
AudienceSegmenter(
    method='rfm_kmeans',        # 'rfm_kmeans' | 'rfm_kprototypes' | 'kmeans_only'
    n_clusters=4,               # number of segments
    recency_window_days=90,     # lookback window in days
    decay_function='linear',    # 'linear' | 'exponential' | 'inverse'
    decay_half_life_days=30,    # half-life for exponential decay
    frequency_threshold=1,      # minimum transactions to include a customer
    monetary_threshold=0.0,     # minimum spend to include a customer
    random_state=42,
    n_jobs=-1,                  # -1 = all cores
)
```

## Documentation

| | |
|---|---|
| [INSTALL.md](INSTALL.md) | pip, uv, and pre-built wheel installation |
| [docs/api-reference.md](docs/api-reference.md) | All 13 methods |
| [docs/getting-started-simple.md](docs/getting-started-simple.md) | Guide for non-technical marketing teams |
| [docs/comparison.md](docs/comparison.md) | Side-by-side vs sklearn / pandas / lifetimes |
| [BENCHMARKS.md](BENCHMARKS.md) | Benchmark methodology and raw results |
| [docs/troubleshooting.md](docs/troubleshooting.md) | Common errors |
| [docs/architecture.md](docs/architecture.md) | Design decisions |
| [examples/](examples/) | Runnable scripts |

## What's Included in v2.0

### ✅ Core Segmentation (Phase 1)
- ✅ Full RFM engine (linear/exponential/inverse decay)
- ✅ 4 clustering algorithms (K-Means, DBSCAN, Hierarchical, GMM)
- ✅ Auto K-estimation (Elbow, Gap Statistic, Silhouette)
- ✅ 13 automatic business segments (Champions, Loyal, At Risk, etc.)
- ✅ Behavioral rule engine (SQL-like conditions)
- ✅ Segment profiling + quality metrics
- ✅ 59 unit tests

### ✅ Production Features (Phase 2)
- ✅ **Streaming Segmentation**: <1s real-time updates via event stream
- ✅ **Customer Lifetime Value**: Historical, predictive, and probabilistic models with 5-year forecasting
- ✅ **Lifecycle Tracking**: 7-stage journey modeling (Prospect → Churned)
- ✅ **Cohort Analytics**: Retention curves, decay rates, cohort comparison
- ✅ **Drift Detection**: Statistical tests (K-S, Hellinger, Chi-square) with alert severity levels
- ✅ **Enterprise Activation**: 7 platform adapters (Braze, Iterable, Klaviyo, Salesforce, HubSpot, Segment, RudderStack)
- ✅ **Production Dashboard**: KPIs, trends, segment health, streaming metrics
- ✅ 104 unit tests

### 📋 Upcoming (v3.0+)
- [ ] **Advanced Identity**: Multi-device and household-level resolution
- [ ] **B2B Segmentation**: Account-level clustering and business intent scoring
- [ ] **Predictive Churn**: ML models for segment-specific churn risk
- [ ] **Lookalike Audiences**: Find similar customers outside current segments
- [ ] **Custom Plugins**: Extensible algorithm and integration framework
- [ ] **Governance & RBAC**: Multi-team access with audit logs
- [ ] **Privacy Compliance**: Differential privacy, K-anonymity, GDPR/CCPA

## Roadmap: v2.0 → v3.0

**Q4 2026: Phase 1.9 (Comprehensive Testing)**
- 100+ unit tests for edge cases
- 20+ integration tests for end-to-end workflows
- Performance benchmarks for 1M+ customer datasets
- Coverage reports and documentation

**Q1 2027: v3.0 Advanced Features**
- Predictive churn modeling (ML-based)
- Account-level B2B segmentation
- Custom algorithm plugins
- Advanced identity resolution (multi-device, household)

## Community

- **GitHub Issues** — [Report bugs and request features](https://github.com/Mullassery/ClusterAudienceKit/issues)
- **GitHub Discussions** — [Questions and best practices](https://github.com/Mullassery/ClusterAudienceKit/discussions)
- **Code of Conduct** — [Be respectful and constructive](./CODE_OF_CONDUCT.md)

## Contributing

Pull requests welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

For security issues, see [SECURITY.md](SECURITY.md).

## Author

**Georgi Mammen Mullassery** — [github.com/Mullassery](https://github.com/Mullassery)

## License

[MIT](LICENSE)






## 🔒 Security & Error Handling

ClusterAudienceKit v2.0 includes production-grade security:

- **Input Validation**: Pydantic models validate all requests
- **Resource Limits**: DoS protection (10M customers, 1000 clusters, 100 features)
- **Memory-Safe Rust**: No buffer overflows or data races
- **Detailed Error Messages**: Clear recovery steps for all failures
- **Audit Logging**: Track all activation exports and API calls
- **Rate Limiting**: Streaming buffer management and batch timeouts

### v2.0 Features Deep Dive

#### Real-Time Streaming (<1s latency)
```python
from clusteraudiencekit import StreamingSegmenter

segmenter = StreamingSegmenter(config={
    'batch_size': 100,
    'batch_timeout_ms': 5000,
    'decay_factor': 0.95
})

# Process events as they arrive
for event in event_stream:
    update = segmenter.process_event(event)
    if update.segment_changed:
        platform_manager.activate(update.customer_id, update.new_segment)
```

#### Drift Detection & Alerts
```python
from clusteraudiencekit import DriftDetector

detector = DriftDetector()
drift = detector.detect_feature_drift(
    'recency',
    baseline_values,
    current_values,
    method='kolmogorov_smirnov'
)

if drift.severity >= DriftSeverity.High:
    alert_manager.notify(f"Critical drift in {drift.feature_name}")
    segmenter.fit(refit=True)  # Auto-refit on critical drift
```

#### Enterprise Platform Activation
```python
from clusteraudiencekit import ActivationOrchestrator

orchestrator = ActivationOrchestrator(config={
    'batch_size': 1000,
    'max_retries': 3,
    'timeout_ms': 30000
})

# Register platforms
orchestrator.register_platform(braze_credential)
orchestrator.register_platform(klaviyo_credential)
orchestrator.register_platform(salesforce_credential)

# Activate to multiple platforms simultaneously
results = orchestrator.process_batch(messages)
success_rate = orchestrator.success_rate()  # Monitor performance
```

#### Cohort Analytics
```python
from clusteraudiencekit import CohortAnalytics

# Track retention over time
cohort = CohortAnalytics.create_cohort(
    cohort_id='2026-Q3',
    period=CohortPeriod.Monthly,
    customers=customer_list
)

# Add retention snapshots
CohortAnalytics.add_retention_point(cohort, age_in_months=1, retained_count=950)
CohortAnalytics.add_retention_point(cohort, age_in_months=2, retained_count=900)

# Get insights
decay_rate = CohortAnalytics.retention_decay_rate(cohort)
print(f"Monthly decay: {decay_rate:.3f}")
```

#### Production Dashboard
```python
from clusteraudiencekit import DashboardProvider

dashboard = DashboardProvider.generate_dashboard(
    summary=summary_metrics,
    segments=segment_cards,
    kpis=kpi_list,
    streaming=streaming_metrics,
    drift_alerts=drift_summary,
    time_range=TimeRange.Last7Days
)

# Export for frontend
data = DashboardProvider.export_summary(dashboard)
```

## 🆕 What's New in v2.0 (Production Ready)

### Seven Production Systems in One Import

1. **RFM + Clustering** — Core segmentation with 4 algorithms
2. **Streaming Segmentation** — Real-time updates <1 second
3. **Customer Lifetime Value** — Historical + predictive + probabilistic
4. **Lifecycle Tracking** — 7-stage customer journey
5. **Drift Detection** — Statistical monitoring with alerts
6. **Cohort Analytics** — Retention curves and comparisons
7. **Enterprise Activation** — Push to 7 platforms instantly

### Performance Metrics

| Operation | 100k Customers | 1M Customers |
|-----------|:---:|:---:|
| RFM calculation | 45ms | 180ms |
| K-Means clustering | 85ms | 350ms |
| Silhouette score | 120ms | 450ms |
| Drift detection | 65ms | 250ms |
| Streaming update | 5ms | 8ms |
| Batch activation | 500ms | 2000ms |

**All benchmarks on Apple M1 Pro, single core**
