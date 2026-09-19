# API Reference

> **Note (post-audit honesty pass):** this document predates a remediation
> pass that fixed two fabricated-data modules, wired 10 previously
> Rust-only modules into the Python API, and rewrote the broken test suite.
> Some API details below (e.g. constructor signatures, algorithm names, or
> performance numbers) may be aspirational or out of date. For the current,
> verified-accurate picture of what's real and callable, see
> [`README.md`](../README.md) and [`docs/ROADMAP_HONEST.md`](ROADMAP_HONEST.md).


Complete API documentation for ClusterAudienceKit.

## AudienceSegmenter

Main class for customer audience segmentation.

### Constructor

```python
AudienceSegmenter(
    method: str = "rfm_kmeans",
    n_clusters: int = 4,
    recency_window_days: int = 90,
    decay_function: str = "linear",
    decay_half_life_days: int = 30,
    frequency_threshold: int = 1,
    monetary_threshold: float = 0.0,
    random_state: int = 42,
    n_jobs: int = -1
)
```

**Parameters:**

- `method` (str): Segmentation algorithm
  - `"rfm_kmeans"` (default): RFM features + KMeans clustering
  - `"rfm_kprototypes"`: RFM + K-Prototypes for mixed data
  - `"kmeans_only"`: Pure KMeans without RFM

- `n_clusters` (int): Number of segments to create
  - Range: 1-100
  - Default: 4
  - Tip: Use silhouette_score() to optimize

- `recency_window_days` (int): Days to look back for RFM
  - Default: 90
  - Shorter for high-frequency (e-commerce): 30
  - Longer for low-frequency (B2B): 180+

- `decay_function` (str): How to weight recent transactions
  - `"linear"`: Linear decay (default)
  - `"exponential"`: Exponential decay
  - `"inverse"`: Inverse distance decay

- `decay_half_life_days` (int): For exponential/inverse decay
  - Default: 30
  - Ignored for linear decay

- `frequency_threshold` (int): Minimum transaction count
  - Default: 1
  - Increase to filter out one-time customers

- `monetary_threshold` (float): Minimum transaction value
  - Default: 0.0
  - Use to filter low-value transactions

- `random_state` (int): For reproducibility
  - Default: 42
  - Use same value for consistent results

- `n_jobs` (int): Parallel jobs
  - `-1` (default): Use all CPU cores
  - `1`: Single-threaded (for debugging)
  - `N`: Use N specific cores

**Example:**

```python
from clusteraudiencekit import AudienceSegmenter

# Basic segmentation
segmenter = AudienceSegmenter()

# Custom configuration
segmenter = AudienceSegmenter(
    method="rfm_kmeans",
    n_clusters=5,
    recency_window_days=60,
    decay_function="exponential",
    n_jobs=-1
)
```

---

## Methods

### fit()

Train segmenter on transaction data.

```python
fit(
    df: pd.DataFrame,
    date_column: str = "transaction_date",
    customer_column: str = "customer_id",
    transaction_column: str = "transaction_amount",
    categorical_columns: Optional[List[str]] = None
) -> AudienceSegmenter
```

**Parameters:**

- `df` (pd.DataFrame): Transaction data
  - Columns: customer_id, transaction_date, amount (+ optional categorical)
  - Format example:
    ```
    customer_id | transaction_date | amount
    ------------|------------------|--------
    cust_001    | 2026-01-15      | 150.00
    cust_002    | 2026-01-20      | 75.50
    ```

- `date_column` (str): Column name for dates
  - Default: "transaction_date"
  - Format: ISO 8601 (YYYY-MM-DD)

- `customer_column` (str): Column name for customer IDs
  - Default: "customer_id"
  - Must be unique per customer

- `transaction_column` (str): Column name for amounts
  - Default: "transaction_amount"
  - Must be numeric (int or float)

- `categorical_columns` (Optional[List[str]]): Categorical features
  - Only for K-Prototypes method
  - Example: ["region", "product_category"]

**Returns:** `AudienceSegmenter` (self for method chaining)

**Raises:**

- `ValueError`: Invalid date format or non-numeric amounts
- `KeyError`: Column not found in DataFrame
- `TypeError`: DataFrame is wrong type

**Time Complexity:** O(n * k * i) where n=customers, k=clusters, i=iterations

**Space Complexity:** O(n + k*d) where d=dimensions (3 for RFM)

**Example:**

```python
import pandas as pd
from clusteraudiencekit import AudienceSegmenter

# Load data
df = pd.read_csv('transactions.csv')

# Create and fit
segmenter = AudienceSegmenter(n_clusters=4)
segmenter.fit(
    df,
    date_column='transaction_date',
    customer_column='customer_id',
    transaction_column='amount'
)

# Method chaining
segments = segmenter.fit(df).predict(df)
```

---

### predict()

Get segment assignments for customers.

```python
predict(
    df: pd.DataFrame,
    date_column: str = "transaction_date",
    customer_column: str = "customer_id",
    transaction_column: str = "transaction_amount"
) -> pd.Series
```

**Parameters:** Same as `fit()`

**Returns:** `pd.Series` with segment IDs (0, 1, 2, ...)

**Raises:**

- `ValueError`: Model not fitted yet
- `KeyError`: Column not found

**Example:**

```python
# After fitting
segments = segmenter.predict(test_df)
print(segments)
# Output: Series([0, 2, 1, 0, 3, ...], name='segment')

# Filter high-value segment
high_value = test_df[segments == 0]
```

---

### fit_predict()

Fit and predict in one call (convenience method).

```python
fit_predict(
    df: pd.DataFrame,
    date_column: str = "transaction_date",
    customer_column: str = "customer_id",
    transaction_column: str = "transaction_amount",
    categorical_columns: Optional[List[str]] = None
) -> pd.Series
```

**Returns:** `pd.Series` with segment assignments

**Example:**

```python
segments = segmenter.fit_predict(df)
```

---

### transform()

Get RFM features (Recency, Frequency, Monetary).

```python
transform(
    df: pd.DataFrame,
    date_column: str = "transaction_date",
    customer_column: str = "customer_id",
    transaction_column: str = "transaction_amount"
) -> pd.DataFrame
```

**Returns:** DataFrame with columns: recency, frequency, monetary (normalized)

**Example:**

```python
features = segmenter.transform(df)
print(features)
#    recency  frequency  monetary
# 0      0.45       0.82      0.61
# 1      0.12       0.30      0.25
```

---

### segment_profiles()

Get aggregate statistics per segment.

```python
segment_profiles() -> pd.DataFrame
```

**Returns:** DataFrame with columns:
- `segment`: Segment ID
- `size`: Number of customers
- `avg_recency`: Average days since purchase
- `avg_frequency`: Average transaction count
- `avg_monetary`: Average spending
- Plus categorical stats if applicable

**Example:**

```python
profiles = segmenter.segment_profiles()
print(profiles)
#   segment | size  | avg_recency | avg_frequency | avg_monetary
#   0       | 250k  | 15.3 days   | 8.2 txns      | $450.20
#   1       | 180k  | 45.2 days   | 3.1 txns      | $120.50
```

---

### silhouette_score()

Measure cluster cohesion (higher is better).

```python
silhouette_score() -> float
```

**Returns:** Float in range [-1, 1]
- Close to 1: Well-separated clusters
- Close to 0: Overlapping clusters
- Negative: Clusters poorly defined

**Example:**

```python
score = segmenter.silhouette_score()
print(f"Silhouette: {score:.3f}")  # Output: Silhouette: 0.654
```

---

### davies_bouldin_score()

Measure cluster separation (lower is better).

```python
davies_bouldin_score() -> float
```

**Returns:** Float >= 0
- Lower values indicate better separation
- Typical range: 0-2

**Example:**

```python
score = segmenter.davies_bouldin_score()
print(f"Davies-Bouldin: {score:.3f}")  # Output: Davies-Bouldin: 0.485
```

---

### inertia()

Sum of squared distances within clusters (lower is better).

```python
inertia() -> float
```

**Returns:** Float representing cluster compactness

---

### update()

Incremental update for streaming data.

```python
update(
    df: pd.DataFrame,
    date_column: str = "transaction_date",
    customer_column: str = "customer_id",
    transaction_column: str = "transaction_amount",
    refit: bool = False
) -> AudienceSegmenter
```

**Parameters:**

- `df`: New transaction data
- `refit` (bool): If True, retrain clusters; if False, just update RFM
  - False (default): Fast incremental update
  - True: Full retraining (slower but more accurate)

**Example:**

```python
# Daily incremental updates
for day in range(30):
    daily_events = get_daily_events(day)
    segmenter.update(daily_events)  # Fast (~1-2ms)

    # Check for drift
    current = segmenter.predict(customers)
    stability = segmenter.segment_stability(previous)

    if stability < 0.85:  # Significant drift detected
        segmenter.update(all_data, refit=True)  # Full retrain

    previous = current
```

---

### segment_stability()

Measure how many customers stayed in same segment.

```python
segment_stability(previous_segments: pd.Series) -> float
```

**Parameters:**

- `previous_segments`: Previous segment assignments

**Returns:** Float in range [0, 1]
- 1.0: All customers stayed in same segment
- 0.5: Half changed segments
- 0.0: All customers changed segments

**Example:**

```python
stability = segmenter.segment_stability(yesterday_segments)
if stability < 0.85:
    print("Significant drift detected, retraining...")
```

---

### save_state()

Save segmenter to disk for later use.

```python
save_state(path: str) -> None
```

**Parameters:**

- `path` (str): File path to save to (e.g., "model.state")

**Example:**

```python
segmenter.save_state('segments.state')
```

---

### load_state()

Load previously saved segmenter.

```python
load_state(path: str) -> None
```

**Parameters:**

- `path` (str): File path to load from

**Example:**

```python
segmenter.load_state('segments.state')
segments = segmenter.predict(new_data)
```

---

### get_config()

Get segmenter configuration and state.

```python
get_config() -> dict
```

**Returns:** Dictionary with:
- `method`: Segmentation algorithm
- `n_clusters`: Number of segments
- `recency_window_days`: RFM window
- `cluster_centers`: Current cluster centers
- `segment_sizes`: Number of customers per segment

**Example:**

```python
config = segmenter.get_config()
print(f"Using {config['n_clusters']} clusters")
print(f"Segment sizes: {config['segment_sizes']}")
```

---

## Error Reference

See [troubleshooting guide](troubleshooting.md) for detailed error explanations and solutions.

Common errors:

- `ValueError`: Invalid parameters or data format
- `KeyError`: Column not found in DataFrame
- `TypeError`: Wrong data type for parameter
- `ImportError`: Package not installed correctly

---

## Performance Notes

For 1M customers:
- `fit()`: 400-500ms
- `predict()`: 10-20ms
- `silhouette_score()`: 150-200ms
- `update()` (incremental): 1-2ms

See [performance guide](../docs/performance-comparison.md) for optimization tips.

---

## Related Documentation

- [Quick Start](../README.md#quick-start)
- [Examples](../examples/)
- [Troubleshooting](troubleshooting.md)
- [Architecture](architecture.md)
- [Performance Guide](performance-comparison.md)
