"""Exposed pattern-discovery slice: trends and emerging audiences."""

from __future__ import annotations

import pytest

from clusteraudiencekit import EmergingAudience, SegmentTrend


def test_increasing_trend():
    trend = SegmentTrend.from_time_series(1, [1.0, 2.0, 3.0, 4.0, 5.0])
    assert trend.trend_direction == "increasing"
    assert trend.trend_strength > 0


def test_decreasing_trend():
    trend = SegmentTrend.from_time_series(1, [5.0, 4.0, 3.0, 2.0, 1.0])
    assert trend.trend_direction == "decreasing"


def test_stable_trend():
    trend = SegmentTrend.from_time_series(1, [3.0, 3.0, 3.0, 3.0])
    assert trend.trend_direction == "stable"


def test_short_series_rejected():
    with pytest.raises(ValueError):
        SegmentTrend.from_time_series(1, [1.0])


def test_emerging_audience_growth():
    audience = EmergingAudience(1, 150, 100, [50, 80, 100])
    assert audience.growth_rate == pytest.approx(0.5)
    assert audience.current_size == 150
    assert audience.emergence_status in {"accelerating", "emerging", "plateauing"}
