# Troubleshooting Guide

Solutions for common ClusterAudienceKit issues.

---

## Installation Issues

### Error: "ModuleNotFoundError: No module named 'clusteraudiencekit'"

**Cause:** Package not installed or installed in wrong environment

**Solution:**

```bash
# 1. Check if installed
pip show clusteraudiencekit

# 2. If not installed, install it
pip install clusteraudiencekit

# 3. If installed but not found, check Python path
python -c "import sys; print(sys.path)"

# 4. Make sure you're in the right virtual environment
which python  # Should show venv path
```

**Note:** Don't import from the source directory. Work from a different directory:

```bash
cd ~  # Leave the ClusterAudienceKit source directory
python -c "from clusteraudiencekit import AudienceSegmenter"
```

---

### Error: "ImportError: DLL load failed"

**Platform:** Windows only

**Cause:** Missing Visual C++ redistributables

**Solution:**

1. Download from Microsoft: https://support.microsoft.com/en-us/help/2977003/
2. Or reinstall Python (includes VC++ runtime)
3. Restart your IDE/terminal

---

### Error: "wheel incompatible with this Python"

**Cause:** Python version mismatch

**Solution:**

```bash
# Check your Python version
python --version

# Install compatible version
pip install --upgrade clusteraudiencekit

# Or specify Python version
pip install clusteraudiencekit -v
```

Supported versions: Python 3.8 - 3.13

---

## Data Issues

### Error: "KeyError: 'column_name' not found"

**Cause:** Column name doesn't exist in DataFrame

**Solution:**

```python
# 1. Check available columns
print(df.columns)

# 2. Use correct column names (case-sensitive!)
segmenter.fit(
    df,
    date_column='transaction_date',  # Exact name!
    customer_column='customer_id',
    transaction_column='amount'
)

# 3. Or rename columns first
df = df.rename(columns={'Date': 'transaction_date'})
```

---

### Error: "ValueError: Column 'date' contains invalid values"

**Cause:** Date column has non-date values

**Solution:**

```python
# 1. Check for non-dates
print(df['transaction_date'].dtype)
print(df['transaction_date'].unique()[:10])

# 2. Convert to datetime
df['transaction_date'] = pd.to_datetime(df['transaction_date'])

# 3. Check for NaT (null) values
print(df['transaction_date'].isna().sum())

# 4. Remove invalid rows
df = df.dropna(subset=['transaction_date'])

# 5. Filter to valid date range
df = df[df['transaction_date'] <= pd.Timestamp.now()]
```

---

### Error: "TypeError: unsupported operand type(s) for +: 'str' and 'float'"

**Cause:** Amount column contains non-numeric values

**Solution:**

```python
# 1. Check amount column
print(df['amount'].dtype)
print(df['amount'].unique()[:10])

# 2. Convert to numeric
df['amount'] = pd.to_numeric(df['amount'], errors='coerce')

# 3. Remove rows with invalid amounts
df = df.dropna(subset=['amount'])

# 4. Check for negative amounts
print((df['amount'] < 0).sum())  # Should be 0
```

---

## Configuration Issues

### Error: "ValueError: n_clusters must be >= 1"

**Cause:** Invalid cluster count

**Solution:**

```python
# Use valid range: 1-100
segmenter = AudienceSegmenter(n_clusters=4)  # Valid: 2-10 recommended

# For optimal clusters, try multiple values
for n in [3, 4, 5, 6]:
    seg = AudienceSegmenter(n_clusters=n)
    seg.fit(df)
    score = seg.silhouette_score()
    print(f"n_clusters={n}: silhouette={score:.3f}")
```

---

### Error: "ValueError: recency_window_days must be > 0"

**Cause:** Invalid window size

**Solution:**

```python
# Use positive integer (in days)
segmenter = AudienceSegmenter(recency_window_days=90)  # Default

# Choose based on business:
# - E-commerce: 30 days
# - Retail: 90 days
# - B2B/SaaS: 180+ days
```

---

### Error: "ValueError: categorical_columns only for K-Prototypes"

**Cause:** Used categorical columns with wrong method

**Solution:**

```python
# For categorical features, use K-Prototypes
segmenter = AudienceSegmenter(
    method='rfm_kprototypes',  # Must use this!
    categorical_columns=['region', 'product_category']
)

# For RFM KMeans, don't use categorical_columns
segmenter = AudienceSegmenter(method='rfm_kmeans')
```

---

## Runtime Issues

### Segmentation is Very Slow (>10 seconds for 1M customers)

**Diagnosis:**

```python
# 1. Check customer count
n_customers = df['customer_id'].nunique()
print(f"Customers: {n_customers}")

# 2. Check transaction count
print(f"Transactions: {len(df)}")

# 3. Time individual operations
import time

start = time.time()
segmenter.fit(df)
print(f"Fit time: {(time.time()-start)*1000:.1f}ms")
```

**Solutions:**

1. **Use multi-core processing:**
   ```python
   segmenter = AudienceSegmenter(n_clusters=4, n_jobs=-1)  # Use all cores
   ```

2. **Reduce window size:**
   ```python
   segmenter = AudienceSegmenter(recency_window_days=30)  # Smaller window
   ```

3. **Filter unnecessary data:**
   ```python
   # Remove very old transactions
   df = df[df['transaction_date'] > some_cutoff_date]

   # Filter low-value transactions
   df = df[df['amount'] > 1.0]
   ```

4. **Check for duplicates:**
   ```python
   # Look for duplicate transactions
   duplicates = df.duplicated(subset=['customer_id', 'transaction_date']).sum()
   print(f"Duplicate rows: {duplicates}")

   # Aggregate if needed
   df = df.groupby(['customer_id', 'transaction_date'])['amount'].sum().reset_index()
   ```

5. **Reduce n_clusters if needed:**
   ```python
   # Fewer clusters = faster computation
   segmenter = AudienceSegmenter(n_clusters=3)  # Instead of 10
   ```

---

### Memory Error: "MemoryError: Unable to allocate X GB"

**Cause:** Dataset too large for available RAM

**Solutions:**

1. **Use sampling:**
   ```python
   # Work with sample first
   df_sample = df.sample(frac=0.1)  # 10% sample
   segmenter.fit(df_sample)
   ```

2. **Process in chunks:**
   ```python
   # Fit on subset, predict on rest
   df_train = df.sample(frac=0.5)
   df_test = df.drop(df_train.index)

   segmenter.fit(df_train)
   segments = segmenter.predict(df_test)
   ```

3. **Reduce data:**
   ```python
   # Filter to recent transactions
   df = df[df['transaction_date'] > some_recent_date]

   # Or filter to large-value transactions
   df = df[df['amount'] > threshold]
   ```

---

### Results Don't Look Right

**Poor Segment Separation:**

```python
# Check cluster quality
score = segmenter.silhouette_score()
print(f"Silhouette: {score:.3f}")

if score < 0.3:  # Poor separation
    # Try different n_clusters
    for n in [2, 3, 4, 5, 6]:
        seg = AudienceSegmenter(n_clusters=n)
        seg.fit(df)
        print(f"n={n}: silhouette={seg.silhouette_score():.3f}")
```

**Unbalanced Segments:**

```python
# Check segment sizes
profiles = segmenter.segment_profiles()
print(profiles[['segment', 'size']])

# If one segment dominates:
# - Try more clusters
# - Check for data quality issues
# - Verify RFM parameters are appropriate
```

**Same Results as Yesterday:**

```python
# Check segment stability
stability = segmenter.segment_stability(yesterday_segments)
print(f"Stability: {stability:.2%}")

if stability > 0.99:  # Too stable
    # Try different decay function
    segmenter = AudienceSegmenter(decay_function='exponential')
    segmenter.fit(df)
```

---

## Prediction Issues

### Error: "ValueError: Model not fitted"

**Cause:** Called predict() before fit()

**Solution:**

```python
# Always fit first
segmenter.fit(training_data)

# Then predict
segments = segmenter.predict(test_data)

# Or use fit_predict
segments = segmenter.fit_predict(data)
```

---

### Predictions Different Each Time

**Cause:** Non-deterministic initialization

**Solution:**

```python
# Set random_state for reproducibility
segmenter = AudienceSegmenter(random_state=42)

# Now results are consistent
seg1 = segmenter.fit(df)
segments1 = seg1.predict(df)

seg2 = AudienceSegmenter(random_state=42).fit(df)
segments2 = seg2.predict(df)

# segments1 and segments2 should be identical
assert (segments1 == segments2).all()
```

---

## Streaming Issues

### Error: "ValueError: Model not fitted for update"

**Cause:** Tried to update before initial fit

**Solution:**

```python
# Always fit first
segmenter.fit(historical_data)

# Then can update
segmenter.update(new_data)
```

---

### Segment Assignments Change Too Frequently

**Cause:** Too sensitive to new data

**Solution:**

```python
# Use refit=False for incremental updates (default)
segmenter.update(daily_events)  # Just updates RFM, not clusters

# Only refit when there's significant drift
stability = segmenter.segment_stability(previous)
if stability < 0.80:  # Refit if <80% stay same
    segmenter.update(all_data, refit=True)
```

---

## Getting Help

If you can't find a solution:

1. **Check the examples:** `examples/` directory
2. **Read the documentation:** `docs/api-reference.md`
3. **Open an issue:** https://github.com/Mullassery/clusteraudiencekit/issues

When reporting issues, include:
- Python version: `python --version`
- ClusterAudienceKit version: `pip show clusteraudiencekit`
- Error message (full traceback)
- Minimal reproducible example
- DataFrame shape and sample data

---

## Performance Optimization

For more optimization tips, see [Performance Guide](performance-comparison.md).
