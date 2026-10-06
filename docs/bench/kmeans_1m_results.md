# K-Means Benchmark: 1M Rows

Addresses [RepoIssues #32](https://github.com/Mullassery/RepoIssues/issues/32) — the README/description claim
"Process 1M+ customers in <1s" previously had zero committed benchmark evidence anywhere in the repo.

## Methodology

```python
import time
import numpy as np
import clusteraudiencekit as cak

np.random.seed(42)
n = 1_000_000
data = np.random.rand(n, 4).astype(np.float64)

t0 = time.perf_counter()
result = cak.kmeans(data, n_clusters=5, max_iter=50, random_state=42)
t1 = time.perf_counter()
print(f"{t1-t0:.3f}s")
```

## Result

Run 2026-10-06, Apple Silicon (M-series), release build (`maturin develop --release`):

```
kmeans on 1000000 rows, 4 features, 5 clusters, max_iter=50: 0.444s
```

## Caveats (read before citing this number)

- This benchmarks **K-Means clustering only**, on synthetic uniform-random data with 4 features.
  It does **not** benchmark the full RFM pipeline (`calculate_rfm` + segmentation), which is what
  "process 1M+ customers" most naturally implies to a reader and has **not** been separately
  benchmarked as of this commit.
- Real-world customer data (non-uniform distributions, categorical features via K-Prototypes,
  more features) will perform differently — likely slower for K-Prototypes (categorical distance
  computation is more expensive) and for higher-dimensional feature sets.
- Single-machine, single run, no statistical averaging across multiple runs. Treat as a directional
  data point, not a rigorous benchmark suite result.
- Platform-specific: Apple Silicon ARM64. Not verified on x86_64 Linux (the actual CI/production
  target) or Windows.

**Conclusion:** the core K-Means clustering step genuinely clusters 1M rows in well under 1 second
for this test configuration, so the claim is not fabricated — but it is narrower than how it reads
in the README/description, and has not been verified for the full pipeline or on non-Apple-Silicon
hardware. See RepoIssues #32 for the tracked follow-up (full-pipeline + cross-platform benchmark).
