# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture

ClusterAudienceKit is a pure Rust single-binary crate with PyO3 bindings, optimized for high-performance customer segmentation. No workspace — all code in `src/`.

**Core modules** (`src/`):
- `lib.rs` — PyO3 entry point, Python API surface (AudienceSegmenter class)
- `segmentation/` — RFM calculation, KMeans and K-Prototypes clustering
- `metrics/` — Silhouette score, Davies-Bouldin, within-cluster sum-of-squares
- `profiling/` — Segment characteristic extraction (mean, quantiles, distribution by segment)
- `drift/` — Segment stability detection across time windows
- `io/` — Parquet/Arrow reader integration

**Key performance characteristic**: Uses `ndarray` for O(n) tensor ops and `rayon` for parallelization. Silhouette computation is O(n) not O(n²) — critical difference vs scikit-learn.

**Data model**: Input is transactions (customer_id, date, amount). RFM features are computed as:
- Recency = days since last transaction
- Frequency = transaction count in lookback window
- Monetary = total transaction amount
Then clustered via KMeans or K-Prototypes (for mixed categorical/numeric segments).

## Build & Test Commands

**Build**:
```bash
cargo build --release
```

**Python wheel**:
```bash
maturin develop          # Dev install
maturin build --release  # Wheel for PyPI
```

**Tests**:
```bash
cargo test --release          # Unit + integration tests
cargo test --release -- --nocapture  # With output
```

**Benchmarks**:
```bash
cargo bench --bench benchmarks
```

**Lint**:
```bash
cargo clippy --all-targets
cargo fmt --check
```

## Important Implementation Details

- **RFM window**: Default lookback is 365 days. Parameterizable via `lookback_days`.
- **Clustering**: KMeans uses k-means++ initialization. K-Prototypes supports mixed numeric/categorical via Gower distance.
- **Silhouette**: Computed as (b - a) / max(a, b) per point, aggregated. O(n) via spatial indexing, not O(n²) pair-wise.
- **Segment drift**: Tracks centroids across time. Alert if any centroid moves >threshold relative to baseline.
- **Streaming**: Incremental `update()` mode updates cluster centroids without full recomputation.
- **Numpy interop**: Uses `numpy` PyO3 feature. Input DataFrames are zero-copy converted to ndarray via Arrow.
