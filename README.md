# ClusterAudienceKit

**A Python library for customer segmentation in marketing data pipelines — RFM analysis, clustering, segment profiling, and streaming updates in one import.**

ClusterAudienceKit is a Python library that replaces the scikit-learn + pandas + lifetimes stack for customer segmentation. If you've built this before, you've probably written hundreds of lines of boilerplate glue and still ended up with a pipeline that can't handle 100k customers in any reasonable time. ClusterAudienceKit does it in a single import, backed by a Rust engine.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Python](https://img.shields.io/badge/python-3.8%20%7C%203.9%20%7C%203.10%20%7C%203.11%20%7C%203.12%20%7C%203.13-blue)](pyproject.toml)
[![PyPI](https://img.shields.io/badge/pypi-clusteraudiencekit-orange)](https://pypi.org/project/clusteraudiencekit/)
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

Beyond performance, you also get RFM scoring, segment profiling, drift detection, and streaming updates — none of which scikit-learn or pandas provide out of the box.

| Capability | scikit-learn | pandas | lifetimes | ClusterAudienceKit |
|------------|:---:|:---:|:---:|:---:|
| RFM calculation | — | manual | — | ✓ |
| KMeans clustering | ✓ | — | — | ✓ |
| K-Prototypes (mixed data) | — | — | — | ✓ |
| Marketing segment profiles | — | manual | — | ✓ |
| Silhouette + quality metrics | ✓ | — | — | ✓ |
| Streaming / incremental updates | — | — | — | ✓ |
| Segment drift detection | — | — | — | ✓ |
| Save / load model state | — | — | ✓ | ✓ |
| Multi-core by default | partial | — | — | ✓ |
| Customer lifetime value (CLV) | — | — | ✓ | planned |

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

## Roadmap

**Segmentation**
- [ ] Customer lifetime value (CLV) — BG/NBD and Gamma-Gamma models, matching `lifetimes` parity
- [ ] DBSCAN and HDBSCAN — density-based clustering for audiences with irregular shapes
- [ ] Hierarchical clustering — dendrogram output for exploratory segment discovery
- [ ] Auto-cluster selection — silhouette + elbow method to recommend optimal `n_clusters`
- [ ] Geographic segmentation — cluster on lat/lon fields with haversine distance

**RFM and features**
- [ ] Engagement RFM — adapt RFM for non-transactional signals (email opens, app sessions, ad clicks)
- [ ] Weighted RFM — configurable weights per dimension rather than equal thirds
- [ ] Custom feature columns — include arbitrary numeric columns alongside RFM in clustering

**Pipeline integrations**
- [ ] dbt macro — expose `segment_profiles()` as a dbt model after each run
- [ ] Airflow operator — `AudienceSegmenterOperator` for scheduled retraining
- [ ] Kafka input — ingest streaming events and update segments without batch jobs
- [ ] Export to CRM — direct push of segment assignments to Salesforce, HubSpot, Braze

**Ad platform exports**
- [ ] Google Ads — upload segment lists to Google Customer Match (CRM, engagement, similar audiences)
- [ ] Meta Ads — export to Meta Conversions API and Audience Manager (pixel events, custom audiences)
- [ ] Amazon Ads — push audiences to Amazon DSP and sponsored ads platform
- [ ] TikTok Ads — segment export to TikTok Custom Audience API
- [ ] LinkedIn Ads — matched audience export for B2B account-based marketing

**Data warehouse & CDP**
- [ ] Segment.com integration — push segments to 100+ downstream tools via Segment protocol
- [ ] mParticle — native export with segment attributes and metadata
- [ ] Customer Data Platforms (Treasure Data, Tealium, Lytics) — streaming and batch sync
- [ ] Snowflake native app — segment creation and management within Snowflake UI

**Marketing automation & email**
- [ ] Klaviyo — export audiences for email campaigns, SMS, push notifications with segment traits
- [ ] Mailchimp — segment sync with dynamic tag assignment
- [ ] SendGrid — audience upload for email marketing workflows
- [ ] Iterable — streaming segment membership updates for triggered campaigns

**Advanced audience features**
- [ ] Lookalike audience generation — find similar high-value customers outside current segments
- [ ] Segment expansion — identify upsell/cross-sell opportunities within each segment
- [ ] Churn prediction within segments — risk-score customers by segment for retention campaigns
- [ ] Real-time segment API — serve `customer_id → segment` membership via REST/gRPC

**Privacy & compliance**
- [ ] Differential privacy — add noise to segments to protect individual privacy
- [ ] K-anonymity enforcement — ensure segments contain at least k customers
- [ ] GDPR/CCPA cleanup — auto-remove opted-out customer IDs from segments
- [ ] Audit logs — track all segment exports and access for compliance

**Output and observability**
- [ ] Segment naming — auto-label segments ("high-value loyalists", "at-risk") based on profile stats
- [ ] Cohort tracking — compare how individual customers move between segments over time
- [ ] HTML segment report — shareable one-page visual summary for marketing teams
- [ ] Prometheus metrics — expose segment health and drift as scrapeable endpoints

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

ClusterAudienceKit includes comprehensive security and error handling:

- **Input Validation**: Pydantic models validate all segmentation requests
- **Resource Limits**: DoS protection with limits on customers (10M), clusters (1000), and features (100)
- **Memory-Safe**: Execution time estimation and automatic limits
- **Detailed Error Messages**: See `clusteraudiencekit/error_messages.py` for recovery steps

### Security Roadmap

- ✅ v1.0.0: Input validation with Pydantic, resource limits
- ✅ v1.0.1: Dependencies pinned, DoS protection
- 🔄 v1.1.0: Advanced segmentation algorithms
- 🔄 v1.2.0: Distributed clustering for large datasets
- 📋 v1.3.0: Real-time segment updates

Full roadmap: [ROADMAP.md](ROADMAP.md)
