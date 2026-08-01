# ClusterAudienceKit

Enterprise audience intelligence at scale. RFM analysis, clustering, CLV, churn detection, lookalikes.

[![Tests](https://img.shields.io/github/actions/workflow/status/Mullassery/ClusterAudienceKit/tests.yml?label=tests)](https://github.com/Mullassery/ClusterAudienceKit/actions)
[![PyPI](https://img.shields.io/pypi/v/clusteraudiencekit)](https://pypi.org/project/clusteraudiencekit/)

Segment millions of customers. RFM analysis, neural networks, lookalike modeling—all in one package.

## Overview

ClusterAudienceKit provides enterprise-grade audience intelligence at scale. RFM analysis, 
advanced clustering (K-means, DBSCAN, spectral), CLV prediction, churn detection, lookalike 
modeling, and neural network clustering. Process 1M+ customers in <1 second.
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
- K-means, DBSCAN, Hierarchical clustering
- 6 algorithm options
- Parallel processing
- GPU acceleration support

## Key Features

- RFM analysis built-in
- 6 clustering algorithms
- Customer Lifetime Value (CLV)
- Churn prediction
- Lookalike modeling
- Neural network clustering
- Sub-second processing

## Key Features

**Audience Analysis**
- RFM segmentation (Recency, Frequency, Monetary)
- 6 clustering algorithms (K-means, DBSCAN, Spectral, Hierarchical, GMM, Isolation Forest)
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
