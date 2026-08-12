"""Basic integration tests for ClusterAudienceKit.

`AudienceSegmenter`'s real constructor (`PyAudienceSegmenter::new` in
`src/python.rs`) takes a single positional `n_clusters: usize` — there is no
`method` keyword argument, and `fit`/`predict` take a plain numeric feature
matrix (`Vec<Vec<f64>>`), not a raw transactions DataFrame. The previous
version of this file called `AudienceSegmenter(method="rfm_kmeans",
n_clusters=4)` and then `.fit(sample_transactions)` with a DataFrame of raw
transactions — neither of which the real binding accepts — and skipped the
one test that would have exercised the real fit/predict path. This rewrite
uses the real API end-to-end: `calculate_rfm` to turn raw transactions into
RFM features, then `AudienceSegmenter.fit`/`.predict` on the resulting
numeric matrix.
"""

import pandas as pd
import pytest
from datetime import datetime, timedelta


@pytest.fixture
def sample_transactions():
    """Generate sample transaction data."""
    base_date = datetime(2026, 1, 1)
    data = []

    for cust_id in range(100):
        for i in range(5):
            date = base_date - timedelta(days=i * 20)
            amount = (cust_id + 1) * (i + 1) * 10
            data.append({
                "customer_id": f"cust_{cust_id:03d}",
                "transaction_date": date.strftime("%Y-%m-%dT00:00:00+00:00"),
                "amount": float(amount),
            })

    return pd.DataFrame(data)


def test_import():
    """Test that ClusterAudienceKit can be imported."""
    from clusteraudiencekit import AudienceSegmenter
    assert AudienceSegmenter is not None


def test_segmenter_creation():
    """Test creating a segmenter instance."""
    from clusteraudiencekit import AudienceSegmenter

    segmenter = AudienceSegmenter(4)
    assert segmenter is not None
    assert segmenter.get_n_clusters() == 4


def test_calculate_rfm_from_transactions(sample_transactions):
    """Test the real RFM calculation over raw transaction rows."""
    from clusteraudiencekit import RFMConfig, calculate_rfm

    transactions = list(
        sample_transactions[["customer_id", "transaction_date", "amount"]].itertuples(
            index=False, name=None
        )
    )
    scores = calculate_rfm(transactions, RFMConfig())

    assert len(scores) == 100  # one RFMScore per distinct customer_id
    for score in scores:
        assert 1 <= score.recency_score <= 5
        assert 1 <= score.frequency_score <= 5
        assert 1 <= score.monetary_score <= 5
        assert score.rfm_segment  # non-empty segment label


def test_fit_predict_end_to_end(sample_transactions):
    """Full pipeline: raw transactions -> RFM features -> KMeans segments."""
    from clusteraudiencekit import AudienceSegmenter, RFMConfig, calculate_rfm

    transactions = list(
        sample_transactions[["customer_id", "transaction_date", "amount"]].itertuples(
            index=False, name=None
        )
    )
    scores = calculate_rfm(transactions, RFMConfig())
    features = [[s.recency, s.frequency, s.monetary] for s in scores]

    segmenter = AudienceSegmenter(4)
    segmenter.fit(features)
    segments = segmenter.predict(features)

    assert len(segments) == len(features)
    assert min(segments) >= 0
    assert max(segments) < 4
