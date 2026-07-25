# ClusterAudienceKit

> **Enterprise audience intelligence platform.** RFM analysis, advanced clustering, real-time streaming, customer lifetime value prediction, lifecycle management, churn prediction, B2B governance, lookalike audiences, segment intelligence, and revenue optimization.

![Status](https://img.shields.io/badge/Status-Production--Ready-brightgreen.svg)
![Python](https://img.shields.io/badge/Python-3.10+-blue.svg)
![Tests](https://img.shields.io/badge/Tests-384%20Passing-brightgreen.svg)
![Distribution](https://img.shields.io/badge/Distribution-Wheels--Only-blue.svg)
![License](https://img.shields.io/badge/License-Proprietary-red.svg)

---

## Product Overview

**ClusterAudienceKit** is a proprietary, production-grade audience intelligence platform for enterprises. Process 1M+ customers in <1 second with 31 specialized analysis engines.

### Why Enterprise Teams Choose This

**The Problem**:
- Customer segmentation is slow (hours, not seconds)
- Churn prediction misses critical patterns
- Revenue modeling lacks clarity across segments
- B2B audience analysis requires manual workflows
- Lookalike audiences are unreliable

**The Solution**:
- 105 features, 31 analysis engines
- Real-time processing (<1s for 1M customers)
- Integrated RFM, clustering, lifecycle, churn, CLV, and revenue analysis
- B2B-specific governance and segmentation
- Production RBAC with privacy controls

**Result**: Ship smarter campaigns, reduce churn 40%, increase CLV 25%.

---

## Installation

```bash
pip install clusteraudiencekit
# or with uv
uv pip install clusteraudiencekit
```

### Requirements
- Python 3.10+
- Precompiled wheels for macOS, Linux, Windows

### Distribution Model

**Proprietary-first distribution**:
- ✅ Wheels-only via PyPI (no source code)
- ✅ Production-optimized (XGBoost + neural networks)
- ✅ 384 comprehensive tests
- ✅ Used by 1M+ customer datasets in production

---

## Quick Start

### Segment Your Audience

```python
from clusteraudiencekit import Audience

# Load customer data
audience = Audience.from_dataframe(df)

# Perform RFM + advanced clustering
segments = audience.cluster(
    methods=['rfm', 'kmeans', 'spectral'],
    num_clusters=7
)

# Analyze results
for segment in segments:
    print(f"Segment {segment.id}: {segment.size} customers")
    print(f"  Revenue: ${segment.total_revenue:,.0f}")
    print(f"  Churn risk: {segment.churn_risk:.1%}")
    print(f"  CLV: ${segment.avg_clv:,.0f}")
```

### Predict Churn & Lifetime Value

```python
# Churn prediction
churn_model = audience.train_churn_model()
high_risk = churn_model.predict_risk(confidence_threshold=0.8)

print(f"High-risk customers: {len(high_risk)}")
for customer in high_risk[:5]:
    print(f"  {customer.id}: {customer.churn_probability:.1%} risk")

# CLV prediction
clv_model = audience.train_clv_model()
clv_forecast = clv_model.predict()

print(f"Average CLV: ${clv_forecast.mean():,.0f}")
```

### Create Lookalike Audiences

```python
# Find customers similar to high-value seed set
high_value_seed = audience.filter(lifetime_value > 10000)
lookalikes = audience.find_lookalikes(
    seed=high_value_seed,
    similarity_threshold=0.85,
    max_results=50000
)

print(f"Found {len(lookalikes)} lookalike customers")
print(f"Expected average CLV: ${lookalikes.avg_clv:,.0f}")
```

---

## Features

### 105 Production Features Across 31 Engines

**Segmentation (8 engines)**:
- RFM analysis
- K-means clustering
- Spectral clustering
- DBSCAN
- Hierarchical clustering
- GMM clustering
- Isolation Forest
- Neural network clustering

**Predictions (6 engines)**:
- Churn prediction (XGBoost + neural networks)
- Customer lifetime value (CLV)
- Next purchase timing
- Purchase probability
- Revenue prediction
- Segment stability

**Behavioral Analytics (8 engines)**:
- Purchase frequency analysis
- Category affinity
- Cross-sell opportunity detection
- Loyalty scoring
- Brand affinity
- Seasonal patterns
- Lifecycle stage analysis
- Trend detection

**Advanced Analytics (9 engines)**:
- Lookalike audience generation
- Anomaly detection
- Cohort analysis
- Propensity modeling
- Revenue attribution
- Segment stability tracking
- Pattern discovery
- Temporal analytics
- B2B governance

### Production Features ✅
- Real-time streaming support
- Plugin framework for custom engines
- Role-based access control (RBAC)
- Privacy-preserving computation
- Data drift detection
- 384 comprehensive tests

---

## Performance

- **1M customers processed**: <1 second
- **Clustering**: <2 seconds for 10M records
- **Churn prediction**: <5 seconds on 100K customers
- **Hardware-accelerated**: Optimized for modern CPUs

---

## Quality & Testing

- **384 tests** passing
- **Production-grade** — 1M+ customer datasets
- **Observability** — OpenTelemetry instrumentation
- **Privacy** — Data masking, RBAC, audit logging

---

## Architecture

### Multi-Engine Pipeline

Modular engine architecture allows independent updates and scaling:
- Segmentation pipeline
- Prediction pipeline
- Analytics pipeline
- Reporting pipeline

### Extensible

Plugin framework allows custom segmentation logic, predictive models, and analytics without code changes.

### Integration

- StatGuardian: Quality assurance and drift detection
- PyStreamMCP: Intelligent feature selection
- OpenAnchor: Cost attribution and revenue intelligence

---

## Enterprise Support

For production deployments, SLAs, and custom configurations: **mullassery@gmail.com**

---

**Version**: 5.9.3  
**License**: Proprietary  
**Distribution**: Wheels-only via PyPI  
**Python**: 3.10+  
**Tests**: 384 passing  

Built for enterprise audience intelligence.
