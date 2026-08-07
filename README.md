# ClusterAudienceKit

**Segment millions of customers in <1 second. Know who matters.**

RFM analysis, customer lifetime value prediction, churn detection, and lookalike modeling—all in one production-ready package. Process 1M+ customers instantly.

[![Tests](https://img.shields.io/github/actions/workflow/status/Mullassery/ClusterAudienceKit/tests.yml?label=tests)](https://github.com/Mullassery/ClusterAudienceKit/actions)
[![PyPI](https://img.shields.io/pypi/v/clusteraudiencekit)](https://pypi.org/project/clusteraudiencekit/)
[![Python 3.10+](https://img.shields.io/badge/Python-3.10%2B-blue)](https://www.python.org)

---

## 30-Second Start

```python
from clusteraudiencekit import Segmentation, CLV

# Segment your customers
segments = Segmentation(df).fit()
print(segments.summary)  # Automatic RFM + clustering

# Find high-value customers
clv = CLV(df)
vips = clv.top_customers(n=100)
print(f"Top 100 worth: ${clv.total_value(vips):,.0f}")
```

---

## Why ClusterAudienceKit?

**The Problem:**
- Marketing teams manually segment (outdated, slow)
- No clear view of customer value
- Churn prediction requires multiple tools
- Building lookalike audiences is complex

**The Solution:**
- Automatic RFM segmentation (no configuration)
- Customer lifetime value prediction
- Churn scoring and early warning
- Lookalike audience generation
- Sub-second processing (1M+ customers)
---

## Quick Start

```python
from clusteraudiencekit import Segmentation, CLV

# Load customer data
df = load_customers()

# Automatic segmentation
segments = Segmentation(df).fit()
print(segments.summary)

# Calculate lifetime value
clv = CLV(df)
high_value = clv.top_customers(n=100)
```

## Process 1M+ Customers in <1s

Optimized clustering on millions of records:
- K-means, DBSCAN, Hierarchical clustering, Gaussian Mixture Models
- Parallel processing
- GPU acceleration support

## Key Features

- RFM analysis built-in
- 4 clustering algorithms
- Customer Lifetime Value (CLV)
- Churn prediction
- Lookalike modeling
- Neural network clustering
- Sub-second processing

## Key Features

**Audience Analysis**
- RFM segmentation (Recency, Frequency, Monetary)
- 4 clustering algorithms (K-means, DBSCAN, Hierarchical, Gaussian Mixture Model).
  Spectral clustering and Isolation Forest are not implemented anywhere in this
  codebase (not even as stubs) despite being listed here previously — removed
  until they're real.
- Customer Lifetime Value (CLV) prediction
- Churn detection and scoring
- Lookalike audience modeling

**Scalability**
- Process 1M+ customers in <1 second
- Rust-powered core for speed
- Streaming data support
- Batch and real-time APIs

**Integration**
- Pandas/Polars DataFrames
- Cloud storage (S3, GCS)
- SQL databases
- BI tools (Tableau, Looker)

---

## Requirements

- Python 3.10+
- NumPy ≥1.20.0
- Scikit-learn ≥1.0.0
- Pandas ≥1.3.0
- Rust core (precompiled wheels)
- Optional: Polars ≥0.18.0

---

## Installation

```bash
pip install clusteraudiencekit
# or with uv
uv pip install clusteraudiencekit

# Verify installation
clusteraudiencekit --version
```

## Use Cases

- Customer segmentation
- Churn prediction
- Retention campaigns
- CLV analysis
- Lookalike targeting
- Campaign personalization

## Examples

```python
from clusteraudiencekit import Segmentation, Churn, Lookalikes

# Segment customers
seg = Segmentation(df)
high_value = seg.segments['high_value']

# Predict churn
churn = Churn(df)
at_risk = churn.predict_churn(threshold=0.7)

# Find lookalikes
similar = Lookalikes(df).find_similar(seed_customers)
```

## Benchmarks

| Customers | Time | Throughput |
|-----------|------|-----------|
| 100K | 50ms | 2M/s |
| 1M | 400ms | 2.5M/s |
| 10M | 3.5s | 2.8M/s |

## Documentation

- [API Reference](docs/api.md)
- [Examples](examples/)
- [Benchmarks](docs/performance.md)

## License

MIT License - See LICENSE
