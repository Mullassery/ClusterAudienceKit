# ClusterAudienceKit Architecture

## Overview

ClusterAudienceKit is a high-performance Python library for audience segmentation. The performance-critical engine runs natively for speed, but the entire interface is Python. This document describes the system architecture.

## High-Level Design

```
┌──────────────────────────────────────────────┐
│              Python API Layer                │
│  - accept pandas/polars DataFrames           │
│  - return DataFrames                         │
│  - sklearn-compatible Estimator protocol     │
└────────────────┬─────────────────────────────┘
                 │
┌──────────────────────────────────────────────┐
│      Rust Core Modules                       │
│  ┌─────────────┬─────────────┬─────────────┐ │
│  │  RFM Engine │ Clustering  │ Streaming   │ │
│  │             │             │             │ │
│  │ - Calc R/F/M│ - KMeans    │ - Updates   │ │
│  │ - Normalize │ - KProto    │ - State mgmt│ │
│  │ - Decay fns │ - Metrics   │ - Drift det │ │
│  │ - Profiles  │             │             │ │
│  └─────────────┴─────────────┴─────────────┘ │
└──────────────────────────────────────────────┘
                 │
┌──────────────────────────────────────────────┐
│   Arrow RecordBatch (Zero-Copy Layer)        │
└──────────────────────────────────────────────┘
```

## Module Structure

### `engine/`
Core segmentation algorithms:
- **rfm.rs** — Recency-Frequency-Monetary calculation
- **clustering.rs** — KMeans and K-Prototypes implementations
- **metrics.rs** — Silhouette, Davies-Bouldin, Inertia scores

### `streaming/`
Incremental/streaming support:
- State persistence (save/load)
- Incremental RFM updates
- Segment stability tracking

### `utils/`
Utility functions:
- **validation.rs** — Configuration validation
- **conversions.rs** — Data type conversions (pandas  Arrow  Rust)

### `python.rs`
Python API

## Data Flow

### Batch Workflow
```
pandas DataFrame
      ↓
Arrow RecordBatch (zero-copy)
      ↓
Rust types (RFM calculation)
      ↓
Clustering (KMeans/KProto)
      ↓
Metrics computation
      ↓
pandas DataFrame (results)
```

### Streaming Workflow
```
Event stream (daily/hourly)
      ↓
Incremental RFM update
      ↓
Update cluster assignments
      ↓
Check stability
      ↓
Optionally retrain if drift detected
```

## Performance Considerations

### Parallelization
- Uses **Rayon** for multi-core parallelization
- RFM calculation parallelized across customers
- Distance computations parallelized in clustering

### Memory Efficiency
- **Arrow format** for zero-copy data handling
- Lazy evaluation where possible
- Streaming state doesn't keep full customer history

### Computational Efficiency
- SIMD operations where applicable
- KMeans++ initialization for better convergence
- Efficient distance calculations (Euclidean for KMeans)

## API Design

### Python Interface
Follows **scikit-learn conventions**:
- `fit(data)` — Train on data
- `predict(data)` — Get cluster assignments
- `transform(data)` — Get features
- `fit_predict(data)` — One-step workflow

### Configuration
```python
segmenter = AudienceSegmenter(
    method='rfm_kmeans',
    n_clusters=4,
    recency_window_days=90,
    decay_function='linear',
    random_state=42,
    n_jobs=-1,
)
```

## Extensibility

Future enhancements:
- Additional clustering algorithms (DBSCAN, Hierarchical)
- Customer Lifetime Value (CLV) integration
- Time-series segmentation
- Integration with MLflow for experiment tracking
- GPU acceleration for large-scale operations

## Dependencies

### Rust
- `ndarray` — Matrix operations
- `rayon` — Parallelization
- `arrow` — Data format
- `pyo3` — Python bindings
- `chrono` — Date handling
- `serde` — Serialization

### Python
- `pandas` — Data manipulation
- `numpy` — Numerical operations
- `pyarrow` — Arrow format support
