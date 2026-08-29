"""Unit tests for ClusterAudienceKit clustering algorithms.

Rewritten against the real exported API (`AudienceSegmenter` /
`PyAudienceSegmenter`, `kmeans`, `assess_cluster_quality`, `estimate_k_elbow`,
etc. from `clusteraudiencekit`). The previous version of this file imported
`ClusterEngine`/`AudienceSegmenter` in a shape that was never implemented and
included a "hierarchical" clustering algorithm test — hierarchical
clustering is explicitly listed as "Not Planned" in
`docs/ROADMAP_HONEST.md`, so that case has been removed rather than faked.
"""

import numpy as np
import pytest
from clusteraudiencekit import AudienceSegmenter, KMeansResult, kmeans


class TestKMeansFunction:
    """Tests for the free `kmeans()` function (real Lloyd's-algorithm KMeans)."""

    def test_kmeans_recovers_well_separated_blobs(self):
        data = [
            [1.0, 1.0], [1.1, 0.9], [0.9, 1.1],
            [10.0, 10.0], [10.1, 9.9], [9.9, 10.1],
        ]
        result = kmeans(data, n_clusters=2, random_state=42)
        assert isinstance(result, KMeansResult)
        labels = result.labels
        assert len(labels) == 6
        assert labels[0] == labels[1] == labels[2]
        assert labels[3] == labels[4] == labels[5]
        assert labels[0] != labels[3]

    def test_kmeans_result_has_centers_and_inertia(self):
        data = [[0.0, 0.0], [1.0, 1.0], [10.0, 10.0], [11.0, 11.0]]
        result = kmeans(data, n_clusters=2, random_state=0)
        assert len(result.centers) == 2
        assert len(result.centers[0]) == 2
        assert result.inertia >= 0.0
        assert result.n_iter >= 1

    def test_kmeans_is_deterministic_for_a_given_seed(self):
        data = [[float(i), float(i) * 2] for i in range(20)]
        a = kmeans(data, n_clusters=3, random_state=7)
        b = kmeans(data, n_clusters=3, random_state=7)
        assert a.labels == b.labels
        assert a.inertia == b.inertia

    def test_kmeans_rejects_more_clusters_than_points(self):
        data = [[1.0, 1.0], [2.0, 2.0]]
        with pytest.raises(Exception):
            kmeans(data, n_clusters=5, random_state=0)

    def test_kmeans_rejects_zero_clusters(self):
        data = [[1.0, 1.0], [2.0, 2.0]]
        with pytest.raises(Exception):
            kmeans(data, n_clusters=0, random_state=0)

    def test_kmeans_single_sample(self):
        data = [[1.0, 2.0, 3.0]]
        result = kmeans(data, n_clusters=1, random_state=0)
        assert len(result.labels) == 1


class TestAudienceSegmenter:
    """Tests for the `AudienceSegmenter` fit/predict wrapper (real PyO3
    binding over the Rust KMeans core, not a stub)."""

    def test_segmenter_initialization(self):
        segmenter = AudienceSegmenter(3)
        assert segmenter is not None
        assert segmenter.get_n_clusters() == 3

    def test_fit_predict_simple_data(self):
        segmenter = AudienceSegmenter(2)
        data = [[1.0, 2.0], [1.0, 4.0], [1.0, 0.0], [4.0, 2.0], [4.0, 4.0], [4.0, 0.0]]
        segmenter.fit(data)
        labels = segmenter.predict(data)
        assert len(labels) == len(data)
        assert all(0 <= label < 2 for label in labels)

    def test_predict_on_new_data_after_fit(self):
        segmenter = AudienceSegmenter(2)
        train_data = [[1.0, 1.0], [1.1, 0.9], [10.0, 10.0], [10.1, 9.9]]
        segmenter.fit(train_data)
        predictions = segmenter.predict([[1.05, 0.95]])
        assert len(predictions) == 1

    def test_reproducibility_with_same_data(self):
        # AudienceSegmenter always uses the same fixed random_state (42)
        # internally, so fitting the same data twice must reproduce the
        # same cluster assignments.
        data = [[float(i % 5), float((i * 3) % 7)] for i in range(30)]

        segmenter1 = AudienceSegmenter(3)
        segmenter1.fit(data)
        labels1 = segmenter1.predict(data)

        segmenter2 = AudienceSegmenter(3)
        segmenter2.fit(data)
        labels2 = segmenter2.predict(data)

        assert labels1 == labels2

    def test_n_jobs_is_a_real_accepted_constructor_parameter(self):
        # n_jobs used to be silently ignored (hardcoded to -1 internally,
        # not even accepted by the constructor). It's now a real parameter:
        # this asserts it's actually settable and reflected back, not just
        # accepted-and-discarded.
        segmenter = AudienceSegmenter(3, n_jobs=2)
        assert segmenter.n_jobs == 2

        default_segmenter = AudienceSegmenter(3)
        assert default_segmenter.n_jobs == -1

    def test_fit_predict_with_n_jobs_1_and_2_match_default(self):
        # n_jobs only controls how many threads the underlying scoped rayon
        # pool may use -- it must never change fit()/predict() results.
        # Confirm construction/fit/predict all work without error for
        # n_jobs=1 (single-threaded) and n_jobs=2 (capped), and that they
        # agree with the n_jobs=-1 (all cores) baseline.
        data = [[1.0, 2.0], [1.0, 4.0], [1.0, 0.0], [4.0, 2.0], [4.0, 4.0], [4.0, 0.0]]

        baseline = AudienceSegmenter(2, n_jobs=-1)
        baseline.fit(data)
        baseline_labels = baseline.predict(data)

        for n_jobs in (1, 2):
            segmenter = AudienceSegmenter(2, n_jobs=n_jobs)
            assert segmenter.n_jobs == n_jobs
            segmenter.fit(data)
            labels = segmenter.predict(data)
            assert len(labels) == len(data)
            assert all(0 <= label < 2 for label in labels)
            assert labels == baseline_labels

    def test_predict_before_fit_raises_clear_error(self):
        segmenter = AudienceSegmenter(3)
        with pytest.raises(Exception, match="fit"):
            segmenter.predict([[1.0, 1.0]])

    def test_repr(self):
        segmenter = AudienceSegmenter(4)
        assert "4" in repr(segmenter)


class TestClusterQualityIntegration:
    """Cluster quality metrics operating on real kmeans() output."""

    def test_silhouette_score_on_well_separated_blobs_is_high(self):
        from clusteraudiencekit import silhouette_score

        data = [[0.0, 0.0], [0.1, 0.1], [10.0, 10.0], [10.1, 9.9]]
        labels = [0, 0, 1, 1]
        score = silhouette_score(data, labels)
        assert -1.0 <= score <= 1.0
        assert score > 0.9

    def test_assess_cluster_quality_end_to_end(self):
        from clusteraudiencekit import assess_cluster_quality

        data = [[0.0, 0.0], [0.2, 0.1], [10.0, 10.0], [10.1, 9.9]]
        result = kmeans(data, n_clusters=2, random_state=0)
        report = assess_cluster_quality(data, result.labels, result.centers)
        assert report.n_clusters == 2
        assert 0.0 <= report.overall_score <= 100.0


class TestDataValidation:
    """Input validation behavior of the real Rust-backed kmeans()."""

    def test_dtype_conversion_from_numpy(self):
        data_int = np.array([[1, 2], [3, 4]], dtype=np.int32).astype(float).tolist()
        result = kmeans(data_int, n_clusters=2, random_state=0)
        assert len(result.labels) == 2

        data_float = np.array([[1.0, 2.0], [3.0, 4.0]], dtype=np.float64).tolist()
        result = kmeans(data_float, n_clusters=2, random_state=0)
        assert len(result.labels) == 2

    def test_empty_data_raises(self):
        with pytest.raises(Exception):
            kmeans([], n_clusters=2, random_state=0)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
