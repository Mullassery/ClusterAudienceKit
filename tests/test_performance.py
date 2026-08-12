"""Performance tests comparing ClusterAudienceKit with sklearn alternatives."""

import pytest
import pandas as pd
import numpy as np
from datetime import datetime, timedelta
import time

try:
    from sklearn.preprocessing import StandardScaler
    from sklearn.cluster import KMeans as SklearnKMeans
    from sklearn.metrics import silhouette_score as sklearn_silhouette
    HAS_SKLEARN = True
except ImportError:
    HAS_SKLEARN = False


def generate_transaction_data(n_customers=1000, transactions_per_customer=5):
    """Generate realistic transaction data for testing."""
    base_date = datetime(2026, 1, 1)
    data = []

    for cust_id in range(n_customers):
        for i in range(transactions_per_customer):
            date = base_date - timedelta(days=i * 20 + (cust_id % 30))
            amount = (cust_id + 1) * (i + 1) * 10.0
            data.append({
                'customer_id': f'cust_{cust_id:04d}',
                'transaction_date': date.strftime('%Y-%m-%d'),
                'amount': float(amount),
            })

    return pd.DataFrame(data)


@pytest.mark.benchmark
class TestClusterAudienceKitPerformance:
    """Performance tests for ClusterAudienceKit."""

    @pytest.fixture
    def sample_data_small(self):
        """100 customers for quick tests."""
        return generate_transaction_data(n_customers=100, transactions_per_customer=5)

    @pytest.fixture
    def sample_data_medium(self):
        """1000 customers for comprehensive tests."""
        return generate_transaction_data(n_customers=1000, transactions_per_customer=5)

    @pytest.fixture
    def sample_data_large(self):
        """10000 customers for stress tests."""
        return generate_transaction_data(n_customers=10000, transactions_per_customer=3)

    @staticmethod
    def _rfm_features(df):
        """Turn a raw-transactions DataFrame into an RFM numeric feature
        matrix via the real Rust-backed `calculate_rfm`."""
        from clusteraudiencekit import RFMConfig, calculate_rfm

        transactions = list(
            df[["customer_id", "transaction_date", "amount"]]
            .assign(transaction_date=lambda d: d["transaction_date"] + "T00:00:00+00:00")
            .itertuples(index=False, name=None)
        )
        scores = calculate_rfm(transactions, RFMConfig())
        return [[s.recency, s.frequency, s.monetary] for s in scores]

    def test_segmenter_creation(self):
        """Test segmenter instantiation time."""
        from clusteraudiencekit import AudienceSegmenter

        start = time.time()
        segmenter = AudienceSegmenter(4)
        elapsed = time.time() - start
        assert segmenter.get_n_clusters() == 4
        assert elapsed < 0.1  # Should be < 100ms

    def test_fit_performance_small(self, sample_data_small):
        """Test fit performance on 100 customers."""
        from clusteraudiencekit import AudienceSegmenter

        features = self._rfm_features(sample_data_small)
        segmenter = AudienceSegmenter(4)
        start = time.time()
        segmenter.fit(features)
        elapsed = time.time() - start
        print(f"\nFit (100 customers): {elapsed*1000:.1f}ms")
        assert elapsed < 1.0  # Should be < 1 second

    def test_fit_performance_medium(self, sample_data_medium):
        """Test fit performance on 1000 customers."""
        from clusteraudiencekit import AudienceSegmenter

        features = self._rfm_features(sample_data_medium)
        segmenter = AudienceSegmenter(4)
        start = time.time()
        segmenter.fit(features)
        elapsed = time.time() - start
        print(f"\nFit (1000 customers): {elapsed*1000:.1f}ms")
        assert elapsed < 2.0  # Should be < 2 seconds

    def test_predict_performance(self, sample_data_small):
        """Test predict performance."""
        from clusteraudiencekit import AudienceSegmenter

        features = self._rfm_features(sample_data_small)
        segmenter = AudienceSegmenter(4)
        segmenter.fit(features)
        start = time.time()
        segments = segmenter.predict(features)
        elapsed = time.time() - start
        print(f"\nPredict (100 customers): {elapsed*1000:.1f}ms")
        assert len(segments) == len(features)

    def test_silhouette_performance(self, sample_data_small):
        """Test silhouette score calculation."""
        from clusteraudiencekit import AudienceSegmenter, silhouette_score

        features = self._rfm_features(sample_data_small)
        segmenter = AudienceSegmenter(4)
        segmenter.fit(features)
        labels = segmenter.predict(features)
        start = time.time()
        score = silhouette_score(features, labels)
        elapsed = time.time() - start
        print(f"\nSilhouette (100 customers): {elapsed*1000:.1f}ms")
        assert -1 <= score <= 1


@pytest.mark.skipif(not HAS_SKLEARN, reason="scikit-learn not installed")
@pytest.mark.benchmark
class TestSklearnComparison:
    """Compare performance against scikit-learn baseline."""

    @pytest.fixture
    def sklearn_data_small(self):
        """Prepare data in sklearn format."""
        df = generate_transaction_data(n_customers=100, transactions_per_customer=5)

        # Manually calculate RFM (sklearn doesn't have this built-in)
        rfm_data = []
        for cust_id in df['customer_id'].unique():
            cust_df = df[df['customer_id'] == cust_id]
            recency = (pd.Timestamp('2026-01-01') - pd.to_datetime(cust_df['transaction_date']).max()).days
            frequency = len(cust_df)
            monetary = cust_df['amount'].sum()
            rfm_data.append([recency, frequency, monetary])

        return np.array(rfm_data)

    def test_sklearn_kmeans_performance(self, sklearn_data_small):
        """Benchmark scikit-learn KMeans."""
        scaler = StandardScaler()
        scaled_data = scaler.fit_transform(sklearn_data_small)

        start = time.time()
        kmeans = SklearnKMeans(n_clusters=4, random_state=42, n_init=10)
        labels = kmeans.fit_predict(scaled_data)
        elapsed = time.time() - start

        print(f"\nscikit-learn KMeans (100 customers): {elapsed*1000:.1f}ms")
        assert len(labels) == len(sklearn_data_small)

    def test_sklearn_silhouette_performance(self, sklearn_data_small):
        """Benchmark scikit-learn silhouette calculation."""
        scaler = StandardScaler()
        scaled_data = scaler.fit_transform(sklearn_data_small)

        kmeans = SklearnKMeans(n_clusters=4, random_state=42, n_init=10)
        labels = kmeans.fit_predict(scaled_data)

        start = time.time()
        score = sklearn_silhouette(scaled_data, labels)
        elapsed = time.time() - start

        print(f"\nscikit-learn Silhouette (100 customers): {elapsed*1000:.1f}ms")
        assert 0 <= score <= 1


# Performance expectations
PERFORMANCE_TARGETS = {
    'import': 0.1,  # <100ms
    'create': 0.05,  # <50ms
    'fit_small': 0.5,  # <500ms (100 customers)
    'fit_medium': 1.0,  # <1s (1000 customers)
    'predict_small': 0.1,  # <100ms
    'silhouette_small': 0.2,  # <200ms
}
