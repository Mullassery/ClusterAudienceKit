# Getting Started (Simple Version)

> **Note (post-audit honesty pass):** this document predates a remediation
> pass that fixed two fabricated-data modules, wired 10 previously
> Rust-only modules into the Python API, and rewrote the broken test suite.
> Some API details below (e.g. constructor signatures, algorithm names, or
> performance numbers) may be aspirational or out of date. For the current,
> verified-accurate picture of what's real and callable, see
> [`README.md`](../README.md) and [`docs/ROADMAP_HONEST.md`](ROADMAP_HONEST.md).


**For non-technical users who want to segment customers without deep technical knowledge.**

---

## What Is Audience Segmentation?

Dividing your customers into groups based on their buying behavior so you can:
- Send better emails (different messages to different groups)
- Offer targeted discounts (to customers most likely to buy)
- Focus retention efforts (on your best customers)
- Understand customer types (who are your customers?)

---

## Installation (Choose One)

### Option 1: Easiest Way (pip)

**Step 1:** Open terminal/command prompt and type:
```bash
pip install clusteraudiencekit
```

**Step 2:** Press Enter and wait for it to finish

**Verify it worked:**
```bash
python -c "from clusteraudiencekit import AudienceSegmenter; print('Success!')"
```

Should see: `Success!`

### Option 2: Faster Way (uv)

If you want installation to be really fast:

```bash
pip install uv
uv pip install clusteraudiencekit
```

**That's it!** You're done.

---

## Get Your Data Ready

You need a file with customer transactions. It should look like this:

| customer_id | transaction_date | amount |
|------------|------------------|--------|
| customer_123 | 2026-01-15 | 50.00 |
| customer_456 | 2026-01-20 | 75.50 |
| customer_123 | 2026-01-25 | 30.00 |

**Where to get this:**
- Export from your store system (Shopify, WooCommerce, etc.)
- Export from your database
- Save as CSV file (Excel → Save As → CSV)

**Column names must be exactly:**
- `customer_id` (or can change this)
- `transaction_date` (must be YYYY-MM-DD format)
- `amount` (the purchase amount)

---

## Segment Your Customers (5 Minutes)

**Step 1:** Open Python and load your data

```python
import pandas as pd
from clusteraudiencekit import AudienceSegmenter

# Load your CSV file
df = pd.read_csv('transactions.csv')

# Take a quick look at your data
print(df.head())  # Shows first 5 rows
```

**Step 2:** Create segments

```python
# Create the segmenter (divides customers into 4 groups)
segmenter = AudienceSegmenter(n_clusters=4)

# Train it on your data
segmenter.fit(df)

# Get segment for each transaction
segments = segmenter.predict(df)
print(segments)  # Shows which segment each transaction belongs to
```

**Step 3:** Understand your segments

```python
# Get summary of each segment
profiles = segmenter.segment_profiles()
print(profiles)
```

Output looks like:
```
segment | size  | avg_recency | avg_frequency | avg_monetary
--------|-------|-------------|---------------|-------------
0       | 1500  | 10 days     | 5.2 purchases | $85.50
1       | 2300  | 45 days     | 2.1 purchases | $35.20
2       | 800   | 90 days     | 1.0 purchases | $15.00
3       | 1200  | 60 days     | 3.5 purchases | $52.00
```

**What this means:**
- **Segment 0:** Your best customers (bought recently, often, and spent most)
- **Segment 1:** Regular customers (moderate activity)
- **Segment 2:** One-time or dormant customers
- **Segment 3:** Occasional customers

---

## Use Your Segments

### Example 1: Target Best Customers for VIP Program

```python
# Get customers in segment 0 (best customers)
best_customers = df[segments == 0]['customer_id'].unique()

# Send them a VIP offer email
send_email_to_list(best_customers, template='vip_offer')
```

### Example 2: Win Back Dormant Customers

```python
# Get segment 2 (dormant customers)
dormant = df[segments == 2]['customer_id'].unique()

# Send them a "Come back!" discount
send_email_to_list(dormant, template='come_back_discount')
```

### Example 3: Increase Frequency of Occasional Buyers

```python
# Get segment 3 (occasional buyers)
occasional = df[segments == 3]['customer_id'].unique()

# Send them a "Complete your collection" email
send_email_to_list(occasional, template='cross_sell')
```

---

## Need Different Segments?

**Too many groups or too few?** Try different number:

```python
# Instead of 4 groups, try 5
segmenter = AudienceSegmenter(n_clusters=5)
segmenter.fit(df)
```

**Want to focus on recent customers?** Change the time window:

```python
# Only look at last 60 days instead of 90
segmenter = AudienceSegmenter(recency_window_days=60)
segmenter.fit(df)
```

---

## Common Questions

### Q: What dates do I need?
**A:** Just include all historical data you have. More is better (1+ years ideal).

### Q: What if I'm missing some data?
**A:** That's okay. Just clean it first:
```python
# Remove rows with missing values
df = df.dropna()

# Remove transactions with $0 amount
df = df[df['amount'] > 0]
```

### Q: How often should I re-run this?
**A:** Depends on business. Options:
- Daily (if you want latest segments)
- Weekly (common)
- Monthly (if you prefer less frequent updates)

### Q: My segments look weird?
**A:** Check your data first:
```python
# Look at some examples
print(df.head(20))

# Check how many customers you have
print(f"Total customers: {df['customer_id'].nunique()}")

# Check date range
print(f"Date range: {df['transaction_date'].min()} to {df['transaction_date'].max()}")
```

---

## Troubleshooting

### Error: "Column not found"
**Solution:** Make sure your column names are EXACTLY:
- `customer_id` (not "CustomerID" or "cust_id")
- `transaction_date` (not "date" or "purchase_date")
- `amount` (not "price" or "sale_amount")

OR rename them:
```python
df = df.rename(columns={'CustomerID': 'customer_id'})
```

### Error: "Invalid date format"
**Solution:** Make sure dates are in YYYY-MM-DD format
```python
# Convert if needed
df['transaction_date'] = pd.to_datetime(df['transaction_date'])
```

### Error: "Amount must be numeric"
**Solution:** Make sure amounts are numbers (not text):
```python
# Convert if needed
df['amount'] = pd.to_numeric(df['amount'])

# Remove $ symbols if present
df['amount'] = df['amount'].astype(str).str.replace('$', '')
df['amount'] = pd.to_numeric(df['amount'])
```

---

## Next Steps

1. **Try it:** Run the examples above with your data
2. **Validate:** Do the segments make sense for your business?
3. **Deploy:** Export segments and use them in your marketing
4. **Monitor:** Re-run monthly to keep segments fresh

---

## Need More Help?

- See full [API Reference](api-reference.md) for advanced options
- Check [Troubleshooting Guide](troubleshooting.md) for detailed errors

---

**That's it! You're ready to segment your customers.** 
