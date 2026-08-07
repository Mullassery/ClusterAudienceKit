"""Basic segmentation example."""

import pandas as pd
from datetime import datetime, timedelta

# Example 1: Create sample data
def generate_sample_data():
    """Generate sample transaction data for demonstration."""
    base_date = datetime(2026, 1, 1)
    data = []

    for cust_id in range(1000):
        # Generate 3-10 transactions per customer
        num_transactions = (cust_id % 8) + 3
        for i in range(num_transactions):
            date = base_date - timedelta(days=i * 15 + (cust_id % 30))
            amount = (cust_id + 1) * (i + 1) * 10
            data.append({
                "customer_id": f"cust_{cust_id:04d}",
                "transaction_date": date.strftime("%Y-%m-%d"),
                "amount": float(amount),
            })

    return pd.DataFrame(data)


def main():
    """Run basic segmentation example."""
    print("ClusterAudienceKit - Basic Segmentation Example")
    print("=" * 50)

    # Generate data
    print("\n1. Generating sample transaction data...")
    transactions = generate_sample_data()
    print(f"   Generated {len(transactions)} transactions for {len(transactions.groupby('customer_id'))} customers")
    print(f"   Columns: {list(transactions.columns)}")

    # Aggregate to one (frequency, total_spend) feature vector per customer.
    # This is a stand-in for full RFM feature engineering — calculate_rfm_py
    # is exposed separately for that — kept simple here since the point of
    # this example is showing the clustering API, not RFM computation.
    print("\n2. Building per-customer feature vectors...")
    per_customer = transactions.groupby("customer_id")["amount"].agg(["count", "sum"])
    feature_vectors = per_customer.values.tolist()
    print(f"   {len(feature_vectors)} customers, features = [transaction_count, total_spend]")

    from clusteraudiencekit import PyAudienceSegmenter, kmeans_py

    print("\n3. Creating and fitting AudienceSegmenter (k=4)...")
    segmenter = PyAudienceSegmenter(4)
    segmenter.fit(feature_vectors)

    print("\n4. Getting segments...")
    segments = segmenter.predict(feature_vectors)
    print(f"   Segment sizes: {[segments.count(s) for s in sorted(set(segments))]}")

    print("\n5. Cluster quality (via the lower-level kmeans_py + metrics)...")
    result = kmeans_py(feature_vectors, n_clusters=4, random_state=42)
    print(f"   Inertia: {result.inertia:.2f}, converged in {result.n_iter} iterations")

    print("\n" + "=" * 50)
    print("Example complete")


if __name__ == "__main__":
    main()
