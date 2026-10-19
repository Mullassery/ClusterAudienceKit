"""Exposed temporal-analytics, price-intelligence, and revenue-intelligence APIs."""

from __future__ import annotations

import pytest

from clusteraudiencekit import (
    ElasticityAnalysis,
    SegmentROI,
    SizeForecast,
    analyze_price_elasticity,
    calculate_segment_roi,
    forecast_segment_sizes,
)


def test_size_forecast_growth_is_linear():
    forecast = forecast_segment_sizes([(0, 100), (10, 200)], 3)
    assert isinstance(forecast, SizeForecast)
    assert forecast.forecast_days == 3
    assert forecast.trend_type == "growth"
    assert forecast.r_squared == pytest.approx(1.0)
    assert [period for period, _ in forecast.predictions] == [1, 2, 3]
    assert [value for _, value in forecast.predictions] == pytest.approx([210.0, 220.0, 230.0])
    assert forecast.confidence_interval_lower == pytest.approx([189.0, 198.0, 207.0])
    assert forecast.confidence_interval_upper == pytest.approx([231.0, 242.0, 253.0])


def test_size_forecast_decline_and_stability():
    declining = forecast_segment_sizes([(0, 100), (10, 50)], 2)
    assert declining.trend_type == "decline"
    assert [value for _, value in declining.predictions] == pytest.approx([45.0, 40.0])

    stable = forecast_segment_sizes([(0, 100), (10, 100)], 2)
    assert stable.trend_type == "stable"
    assert stable.r_squared == 0.0
    assert [value for _, value in stable.predictions] == pytest.approx([100.0, 100.0])


def test_size_forecast_zero_periods_is_empty_not_an_error():
    forecast = forecast_segment_sizes([(0, 100), (10, 200)], 0)
    assert forecast.forecast_days == 0
    assert forecast.predictions == []
    assert forecast.confidence_interval_lower == []
    assert forecast.confidence_interval_upper == []


def test_size_forecast_rejects_too_few_points():
    with pytest.raises(ValueError):
        forecast_segment_sizes([(0, 100)], 3)
    with pytest.raises(ValueError):
        forecast_segment_sizes([], 3)


def test_size_forecast_rejects_zero_variance_days():
    with pytest.raises(ValueError):
        forecast_segment_sizes([(5, 100), (5, 200)], 2)


def test_price_elasticity_single_step_and_classification():
    analysis = analyze_price_elasticity("seg-1", [(100.0, 1000.0), (110.0, 900.0)])
    assert isinstance(analysis, ElasticityAnalysis)
    assert analysis.segment_id == "seg-1"
    assert analysis.price_elasticity == pytest.approx(1.0)
    assert analysis.elasticity_classification == "inelastic"
    assert analysis.revenue_sensitivity == pytest.approx(0.5)
    assert analysis.demand_curve == [(110.0, 900.0)]
    assert analysis.optimal_price == pytest.approx(100.0)


def test_price_elasticity_averages_absolute_steps():
    analysis = analyze_price_elasticity(
        "seg-1", [(100.0, 1000.0), (110.0, 900.0), (120.0, 700.0)]
    )
    expected = (1.0 + abs((-200.0 / 900.0) / (10.0 / 110.0))) / 2.0
    assert analysis.price_elasticity == pytest.approx(expected)
    assert analysis.elasticity_classification == "highly_elastic"
    assert analysis.demand_curve == [(110.0, 900.0), (120.0, 700.0)]


def test_price_elasticity_zero_price_change_contributes_zero():
    analysis = analyze_price_elasticity("seg-1", [(100.0, 1000.0), (100.0, 900.0)])
    assert analysis.price_elasticity == 0.0
    assert analysis.elasticity_classification == "highly_inelastic"
    assert analysis.optimal_price == pytest.approx(100.0)


def test_price_elasticity_rejects_too_few_points():
    with pytest.raises(ValueError):
        analyze_price_elasticity("seg-1", [(100.0, 1000.0)])
    with pytest.raises(ValueError):
        analyze_price_elasticity("seg-1", [])


def test_segment_roi_numeric_values():
    roi = calculate_segment_roi("seg-1", 10000.0, 5000.0, 15000.0)
    assert isinstance(roi, SegmentROI)
    assert roi.segment_id == "seg-1"
    assert roi.roi_percentage == pytest.approx(100.0)
    assert roi.payback_period_days == pytest.approx(182.5)
    assert roi.profitability_index == pytest.approx(2.0)
    assert roi.risk_level == "medium"


def test_segment_roi_risk_levels():
    assert calculate_segment_roi("s", 30000.0, 10000.0, 0.0).risk_level == "low"
    assert calculate_segment_roi("s", 11000.0, 10000.0, 0.0).risk_level == "medium-high"
    assert calculate_segment_roi("s", 9000.0, 10000.0, 0.0).risk_level == "high"


def test_segment_roi_with_zero_investment_is_infinite_payback():
    roi = calculate_segment_roi("seg-1", 1000.0, 0.0, 0.0)
    assert roi.roi_percentage == 0.0
    assert roi.profitability_index == 0.0
    assert roi.payback_period_days == pytest.approx(10000.0)
    assert roi.risk_level == "high"


def test_classes_exported_in_all():
    import clusteraudiencekit as cak

    for name in (
        "SizeForecast",
        "ElasticityAnalysis",
        "SegmentROI",
        "forecast_segment_sizes",
        "analyze_price_elasticity",
        "calculate_segment_roi",
    ):
        assert name in cak.__all__
    assert SizeForecast.__name__ == "PySizeForecast"
    assert ElasticityAnalysis.__name__ == "PyElasticityAnalysis"
    assert SegmentROI.__name__ == "PySegmentROI"
