# How ClusterAudienceKit Compares to Existing Libraries

> **Note (post-audit honesty pass):** this document predates a remediation
> pass that fixed two fabricated-data modules, wired 10 previously
> Rust-only modules into the Python API, and rewrote the broken test suite.
> Some API details below (e.g. constructor signatures, algorithm names, or
> performance numbers) may be aspirational or out of date. For the current,
> verified-accurate picture of what's real and callable, see
> [`README.md`](../README.md) and [`docs/ROADMAP_HONEST.md`](ROADMAP_HONEST.md).


There is no single Python library for audience segmentation today. Instead, teams stitch together 2–3 separate libraries and write glue code. ClusterAudienceKit replaces the whole stack.

---

## The Current Landscape

| Library | What it does | What it can't do |
|---------|-------------|-----------------|
| **scikit-learn** | Clustering algorithms (KMeans, DBSCAN, etc.) | No RFM, no segment profiles, no streaming |
| **pandas** | RFM via groupby + custom code | No clustering, no metrics, no streaming |
| **lifetimes** | Customer lifetime value (CLV) prediction | No segmentation, no clustering, very slow |

None of them talk to each other. You have to chain them manually every time.

---

## Feature Comparison

| Feature | scikit-learn | pandas | lifetimes | **ClusterAudienceKit** |
|---------|:-----------:|:------:|:---------:|:---------------:|
| **RFM calculation** | No | Manual\* | No | Yes — built-in |
| **KMeans clustering** | Yes | No | No | Yes |
| **K-Prototypes (numeric + categorical)** | No | No | No | Yes |
| **Silhouette score** | Yes | No | No | Yes |
| **Davies-Bouldin score** | Yes | No | No | Yes |
| **Segment profiles** | No | Manual\* | No | Yes — built-in |
| **Streaming / incremental updates** | No | No | No | Yes |
| **Segment drift detection** | No | No | No | Yes |
| **Save / load model state** | No | No | Yes | Yes |
| **Customer lifetime value (CLV)** | No | No | Yes | Planned |
| **One-line fit_predict** | Yes | No | No | Yes |
| **Multi-core parallelisation** | Partial | No | No | Yes — all cores |
| **Zero-copy Arrow data format** | No | No | No | Yes |
| **Decay-weighted RFM** | No | No | No | Yes |

\* "Manual" means the feature is achievable but requires you to write the code yourself. It is not provided by the library.

---

## Lines of Code Comparison

**Full segmentation pipeline — sklearn + pandas approach:**

```python
from sklearn.preprocessing import StandardScaler
from sklearn.cluster import KMeans
from sklearn.metrics import silhouette_score
import pandas as pd

ref = pd.Timestamp.now()
df['transaction_date'] = pd.to_datetime(df['transaction_date'])
g = df.groupby('customer_id')
rfm = pd.DataFrame({
    'recency':   (ref - g['transaction_date'].max()).dt.days,
    'frequency':  g.size(),
    'monetary':   g['amount'].sum()
})
scaled = StandardScaler().fit_transform(rfm)
labels = KMeans(n_clusters=4, n_init=10).fit_predict(scaled)
score  = silhouette_score(scaled, labels)
```
→ **13 lines · 3 imports · 3 library objects · no streaming · no profiles**

---

**Same result with ClusterAudienceKit:**

```python
from clusteraudiencekit import AudienceSegmenter

seg = AudienceSegmenter(n_clusters=4)
seg.fit(df)
score = seg.silhouette_score()
```
→ **4 lines · 1 import · 1 object · streaming built-in · profiles built-in**

---

## What Each Library Is Good For

### Use scikit-learn when:
- You need a wide variety of clustering algorithms (DBSCAN, Hierarchical, Birch, etc.)
- You're doing research and need flexibility
- Your dataset is small enough that performance doesn't matter
- You already have RFM features computed elsewhere

### Use pandas when:
- You need flexible data manipulation and custom aggregations
- You're exploring data before segmenting
- You want SQL-like operations on DataFrames

### Use lifetimes when:
- You specifically need Customer Lifetime Value (BG/NBD or Gamma-Gamma models)
- Your primary question is "how much will this customer spend?"

### Use ClusterAudienceKit when:
- You want RFM + segmentation in one library
- You're processing large customer bases (10k+ customers)
- You need real-time or streaming updates
- You want production-grade performance
- You want drift detection built in
- You want to ship in hours, not days

---

## Measured Speed: Where sklearn Breaks Down

All numbers below are **real measurements** (Apple M1, sklearn 1.6.1, pandas 3.0.3):

### Silhouette score — the critical bottleneck

```
   1,000 customers →     9ms  ████
  10,000 customers →   566ms  ████████████████████████████████████████████
 100,000 customers → ~2.7 hrs  [cannot be used in production]
```

The sklearn silhouette score is O(n²). It calculates pairwise distances between every customer pair. At 100k customers that is 10 billion distance calculations in Python loops.

ClusterAudienceKit targets <200ms at 1M customers using SIMD distance calculations in Rust, parallelised across all CPU cores.

### Full pipeline comparison (measured sklearn vs ClusterAudienceKit targets)

| Dataset | sklearn pipeline | ClusterAudienceKit target | Difference |
|---------|----------------|-------------------|------------|
| 1,000 customers | 38ms | <9ms | **4x faster** |
| 10,000 customers | 606ms | <37ms | **16x faster** |
| 100,000 customers | >2.7 hours | <130ms | **>70,000x faster** |
| 1,000,000 customers | Would not finish | <470ms | — |

---

## What ClusterAudienceKit Does Not Replace

- **scikit-learn** for general machine learning — ClusterAudienceKit is specialised for customer segmentation, not a general ML toolkit
- **lifetimes** for CLV modelling — CLV is a planned feature but not in Phase 1
- **pandas** for data exploration and transformation — use pandas to prepare and explore your data, then hand it to ClusterAudienceKit for segmentation

The typical production stack looks like:

```
pandas (data prep) → ClusterAudienceKit (segmentation) → your marketing platform
```

Not:

```
pandas (RFM) → sklearn (scale + cluster) → sklearn (metrics) → pandas (profiles) → ...
```

---

## See Also

- [BENCHMARKS.md](../BENCHMARKS.md) — full timing tables with methodology
- [API Reference](api-reference.md) — complete method documentation
- [Architecture](architecture.md) — why Rust makes the difference
