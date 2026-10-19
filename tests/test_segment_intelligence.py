"""Exposed segment-intelligence slice: confidence, entropy, differentiation,
predictability, and aging metrics."""

from __future__ import annotations

import pytest

from clusteraudiencekit import (
    SegmentAging,
    SegmentConfidence,
    SegmentDifferentiation,
    SegmentEntropy,
    SegmentPredictability,
)


def test_confidence_uses_distance_ratio():
    conf = SegmentConfidence(1, "member_0", 1.0, 3.0)
    assert conf.segment_id == 1
    assert conf.member_id == "member_0"
    assert conf.confidence_score == pytest.approx(0.75)


def test_confidence_is_one_when_no_competing_segment():
    conf = SegmentConfidence(0, "member_0", 5.0, 0.0)
    assert conf.confidence_score == pytest.approx(1.0)


def test_shannon_entropy_uniform_and_homogeneous():
    assert SegmentEntropy.compute_shannon([1, 1]) == pytest.approx(1.0)
    assert SegmentEntropy.compute_shannon([1, 1, 1, 1]) == pytest.approx(2.0)
    assert SegmentEntropy.compute_shannon([7]) == 0.0


def test_shannon_entropy_empty_is_zero():
    assert SegmentEntropy.compute_shannon([]) == 0.0


def test_gini_concentrated_and_uniform():
    assert SegmentEntropy.compute_gini([100, 0, 0]) == pytest.approx(1.0)
    assert SegmentEntropy.compute_gini([1, 1, 1]) == pytest.approx(0.0)


def test_gini_empty_is_zero():
    assert SegmentEntropy.compute_gini([]) == 0.0


def test_entropy_constructor_defaults():
    entropy = SegmentEntropy(2, 10)
    assert entropy.segment_id == 2
    assert entropy.member_count == 10
    assert entropy.shannon_entropy == 0.0
    assert entropy.gini_coefficient == 0.0


def test_negative_frequencies_rejected():
    with pytest.raises(OverflowError):
        SegmentEntropy.compute_shannon([-1, 2])


def test_differentiation_inverts_nearest_similarity():
    diff = SegmentDifferentiation(3, 0.2, 0.9)
    assert diff.differentiation_score == pytest.approx(0.8)
    assert diff.similarity_to_nearest == pytest.approx(0.2)
    assert diff.similarity_to_farthest == pytest.approx(0.9)


def test_predictability_stable_segment():
    pred = SegmentPredictability(0, 0.1, 0.02)
    assert pred.predictability_score == pytest.approx(0.94)
    assert pred.stability_trend == "stable"


def test_predictability_declining_segment():
    pred = SegmentPredictability(0, 0.5, 0.2)
    assert pred.predictability_score == pytest.approx(0.65)
    assert pred.stability_trend == "declining"


def test_predictability_neutral_trend():
    assert SegmentPredictability(0, 0.0, 0.1).stability_trend == "neutral"


def test_aging_statistics_odd_and_even():
    odd = SegmentAging(1, [10.0, 20.0, 30.0, 40.0, 50.0], 0.05)
    assert odd.avg_member_tenure_days == pytest.approx(30.0)
    assert odd.median_member_tenure_days == pytest.approx(30.0)
    assert odd.newest_member_days_ago == pytest.approx(10.0)
    assert odd.oldest_member_days_ago == pytest.approx(50.0)

    even = SegmentAging(1, [10.0, 20.0, 30.0, 40.0], 0.05)
    assert even.median_member_tenure_days == pytest.approx(25.0)


def test_aging_lifecycle_stages():
    assert SegmentAging(0, [10.0, 20.0], 0.0).segment_lifespan_stage == "emerging"
    assert SegmentAging(0, [100.0, 100.0], 0.1).segment_lifespan_stage == "growth"
    assert SegmentAging(0, [200.0, 200.0], 0.15).segment_lifespan_stage == "mature"
    assert SegmentAging(0, [200.0, 200.0], 0.25).segment_lifespan_stage == "declining"


def test_aging_rejects_empty_tenure():
    with pytest.raises(ValueError):
        SegmentAging(0, [], 0.0)


def test_metrics_exported_in_all():
    import clusteraudiencekit as cak

    for name in (
        "SegmentAging",
        "SegmentConfidence",
        "SegmentDifferentiation",
        "SegmentEntropy",
        "SegmentPredictability",
    ):
        assert name in cak.__all__
        assert getattr(cak, name).__name__ == f"Py{name}"
