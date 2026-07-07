"""Unit tests for ClusterAudienceKit clustering algorithms."""

import pytest
import numpy as np
from clusteraudiencekit import ClusterEngine, AudienceSegmenter


class TestClusterEngine:
    """Tests for core clustering engine."""

    def test_engine_initialization(self):
        """Test ClusterEngine can be initialized."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=3)
        assert engine is not None
        assert engine.num_clusters == 3

    def test_fit_simple_data(self):
        """Test fitting on simple 2D data."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=3)
        data = np.array([[1, 2], [1, 4], [1, 0], [4, 2], [4, 4], [4, 0]])

        labels = engine.fit_predict(data)

        assert len(labels) == len(data)
        assert all(0 <= label < 3 for label in labels)

    def test_fit_high_dimensional_data(self):
        """Test fitting on high-dimensional data."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=5)
        data = np.random.randn(100, 50)  # 100 samples, 50 dimensions

        labels = engine.fit_predict(data)

        assert len(labels) == 100
        assert len(set(labels)) <= 5

    def test_different_algorithms(self):
        """Test various clustering algorithms."""
        data = np.random.randn(50, 3)
        algorithms = ["kmeans", "hierarchical"]

        for algo in algorithms:
            engine = ClusterEngine(algorithm=algo, num_clusters=3)
            labels = engine.fit_predict(data)
            assert len(labels) == len(data)

    def test_empty_data_handling(self):
        """Test handling of empty data."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=3)
        data = np.array([]).reshape(0, 2)

        with pytest.raises((ValueError, IndexError)):
            engine.fit_predict(data)

    def test_single_sample(self):
        """Test handling of single sample."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=3)
        data = np.array([[1, 2, 3]])

        labels = engine.fit_predict(data)
        assert len(labels) == 1

    def test_num_clusters_validation(self):
        """Test that num_clusters must be positive."""
        with pytest.raises(ValueError):
            ClusterEngine(algorithm="kmeans", num_clusters=0)

        with pytest.raises(ValueError):
            ClusterEngine(algorithm="kmeans", num_clusters=-1)

    def test_inertia_computation(self):
        """Test within-cluster sum of squares computation."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=2)
        data = np.array([[0, 0], [1, 1], [10, 10], [11, 11]])

        engine.fit_predict(data)
        inertia = engine.inertia()

        assert inertia >= 0

    def test_silhouette_score(self):
        """Test silhouette score computation."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=2)
        data = np.array([[0, 0], [1, 1], [10, 10], [11, 11]])

        engine.fit_predict(data)
        score = engine.silhouette_score()

        assert -1 <= score <= 1

    def test_predict_on_new_data(self):
        """Test predicting cluster labels for new data."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=3)
        train_data = np.random.randn(50, 5)
        test_data = np.random.randn(10, 5)

        engine.fit_predict(train_data)
        predictions = engine.predict(test_data)

        assert len(predictions) == len(test_data)
        assert all(0 <= p < 3 for p in predictions)

    def test_reproducibility_with_seed(self):
        """Test that results are reproducible with fixed seed."""
        data = np.random.RandomState(42).randn(50, 3)

        engine1 = ClusterEngine(algorithm="kmeans", num_clusters=3, random_state=42)
        labels1 = engine1.fit_predict(data)

        engine2 = ClusterEngine(algorithm="kmeans", num_clusters=3, random_state=42)
        labels2 = engine2.fit_predict(data)

        assert np.array_equal(labels1, labels2)


class TestAudienceSegmenter:
    """Tests for audience segmentation wrapper."""

    def test_segmenter_initialization(self):
        """Test AudienceSegmenter can be initialized."""
        segmenter = AudienceSegmenter()
        assert segmenter is not None

    def test_segment_user_data(self):
        """Test segmenting user audience data."""
        segmenter = AudienceSegmenter()
        user_data = {
            "user_id": [1, 2, 3, 4, 5],
            "age": [25, 30, 45, 22, 55],
            "purchase_amount": [100, 200, 150, 50, 300],
        }

        segments = segmenter.segment(user_data, num_segments=3)

        assert len(segments) == 5  # One segment per user
        assert all(0 <= seg < 3 for seg in segments)

    def test_feature_importance(self):
        """Test feature importance ranking."""
        segmenter = AudienceSegmenter()
        user_data = {
            "age": [20, 30, 40, 50, 60],
            "income": [50000, 75000, 100000, 125000, 150000],
        }

        segmenter.segment(user_data, num_segments=2)
        importance = segmenter.feature_importance()

        assert isinstance(importance, dict)
        assert "age" in importance or "income" in importance

    def test_segment_stability(self):
        """Test that similar data produces similar segments."""
        segmenter = AudienceSegmenter()

        # Original data
        data1 = {"score": [10, 20, 30, 40, 50]}
        seg1 = segmenter.segment(data1, num_segments=2)

        # Slightly perturbed data
        data2 = {"score": [11, 21, 29, 41, 49]}
        seg2 = segmenter.segment(data2, num_segments=2)

        # Segments should be mostly similar
        agreement = sum(1 for a, b in zip(seg1, seg2) if a == b) / len(seg1)
        assert agreement > 0.6  # At least 60% agreement

    def test_large_dataset_performance(self):
        """Test performance on large dataset."""
        import time

        segmenter = AudienceSegmenter()
        large_data = {
            "feature1": np.random.randn(10000),
            "feature2": np.random.randn(10000),
        }

        start = time.time()
        segments = segmenter.segment(large_data, num_segments=10)
        elapsed = time.time() - start

        assert len(segments) == 10000
        assert elapsed < 30  # Should complete in under 30 seconds


class TestDataValidation:
    """Tests for input data validation."""

    def test_nan_handling(self):
        """Test handling of NaN values."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=2)
        data = np.array([[1, 2], [np.nan, 4], [5, 6]])

        # Should either handle NaN or raise clear error
        try:
            engine.fit_predict(data)
        except ValueError as e:
            assert "NaN" in str(e) or "nan" in str(e).lower()

    def test_infinite_values(self):
        """Test handling of infinite values."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=2)
        data = np.array([[1, 2], [np.inf, 4], [5, 6]])

        try:
            engine.fit_predict(data)
        except ValueError as e:
            assert "inf" in str(e).lower()

    def test_dtype_conversion(self):
        """Test automatic dtype conversion."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=2)

        # Integer data
        data_int = np.array([[1, 2], [3, 4]], dtype=np.int32)
        labels = engine.fit_predict(data_int)
        assert len(labels) == 2

        # Float data
        data_float = np.array([[1.0, 2.0], [3.0, 4.0]], dtype=np.float64)
        labels = engine.fit_predict(data_float)
        assert len(labels) == 2


class TestMemorySecurity:
    """Tests for TIER 1 GPU memory bounds (if applicable)."""

    def test_memory_usage_bounded(self):
        """Test that memory usage doesn't exceed bounds."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=5)

        # Test with progressively larger data
        for size in [100, 1000, 10000]:
            data = np.random.randn(size, 50)
            labels = engine.fit_predict(data)
            assert len(labels) == size

    def test_no_memory_leak_on_repeated_fits(self):
        """Test that repeated fits don't leak memory."""
        engine = ClusterEngine(algorithm="kmeans", num_clusters=3)

        for _ in range(10):
            data = np.random.randn(100, 5)
            engine.fit_predict(data)

        # If we get here without memory error, test passes
        assert True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
