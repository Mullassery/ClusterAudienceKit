# ClusterAudienceKit

Enterprise audience intelligence at scale. RFM analysis, clustering, CLV, churn detection, lookalikes.

[![Tests](https://img.shields.io/github/actions/workflow/status/Mullassery/ClusterAudienceKit/tests.yml?label=tests)](https://github.com/Mullassery/ClusterAudienceKit/actions)
[![PyPI](https://img.shields.io/pypi/v/clusteraudiencekit)](https://pypi.org/project/clusteraudiencekit/)

Segment millions of customers. RFM analysis, neural networks, lookalike modeling—all in one package.

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

## Installation

```bash
pip install clusteraudiencekit
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
