"""Exposed temporal-analytics, price-intelligence, and revenue-intelligence APIs."""

from __future__ import annotations

import pytest

from clusteraudiencekit import (
    CompositionForecast,
    DiscountStrategy,
    ElasticityAnalysis,
    RevenueAttribution,
    RevenueForecast,
    SegmentROI,
    SizeForecast,
    WhatIfScenario,
    analyze_price_elasticity,
    calculate_revenue_attribution,
    calculate_segment_roi,
    forecast_composition,
    forecast_revenue,
    forecast_segment_sizes,
    optimize_discount,
    simulate_what_if,
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


def test_composition_forecast_projects_each_ratio():
    forecast = forecast_composition(
        0.7,
        0.15,
        0.05,
        [(0.5, 0.2, 0.1), (0.7, 0.15, 0.05)],
        3,
    )
    assert isinstance(forecast, CompositionForecast)
    assert forecast.forecast_periods == 3
    assert forecast.high_value_ratio_forecast == pytest.approx([0.7, 0.9, 1.0])
    assert forecast.churn_risk_forecast == pytest.approx([0.15, 0.10, 0.05])
    assert forecast.new_member_forecast == pytest.approx([0.05, 0.0, 0.0])
    assert forecast.composition_stability == pytest.approx(1.0 / 1.01)


def test_composition_forecast_rejects_invalid_inputs():
    with pytest.raises(ValueError):
        forecast_composition(0.5, 0.2, 0.1, [], 0)
    with pytest.raises(ValueError):
        forecast_composition(1.5, 0.2, 0.1, [], 2)
    with pytest.raises(ValueError):
        forecast_composition(0.5, -0.2, 0.1, [], 2)
    with pytest.raises(ValueError):
        forecast_composition(0.5, 0.2, 0.1, [(0.5, float("nan"), 0.1)], 2)


def test_what_if_scenario_projects_sizes_and_impacts():
    scenario = simulate_what_if(
        "scale",
        [("a", 100), ("b", 200)],
        {"rfm_threshold": 0.5, "n_clusters": 2.0, "churn_threshold": 0.1},
    )
    assert isinstance(scenario, WhatIfScenario)
    assert scenario.scenario_name == "scale"
    assert scenario.description == "Scenario with 3 parameter changes"
    assert scenario.projected_segment_sizes == [("a", 52), ("b", 105)]
    assert scenario.projected_revenue_impact == pytest.approx((157.0 - 300.0) / 300.0 * 100.0)
    assert scenario.projected_churn_impact == pytest.approx(0.1)
    assert scenario.feasibility_score == pytest.approx(0.7)
    assert set(scenario.parameters_changed) == {
        "rfm_threshold",
        "n_clusters",
        "churn_threshold",
    }
    assert scenario.parameters_changed["n_clusters"] == ("old", "2.00")


def test_what_if_scenario_rejects_invalid_inputs():
    with pytest.raises(ValueError):
        simulate_what_if("", [("a", 10)], {})
    with pytest.raises(ValueError):
        simulate_what_if("scale", [], {})
    with pytest.raises(ValueError):
        simulate_what_if("scale", [("a", 10)], {"n_clusters": 0.0})
    with pytest.raises(ValueError):
        simulate_what_if("scale", [("a", 10)], {"rfm_threshold": float("inf")})


def test_optimize_discount_numeric_values():
    strategy = optimize_discount("seg-1", 0.05, 2.0, 100.0)
    assert isinstance(strategy, DiscountStrategy)
    assert strategy.optimal_discount == pytest.approx(0.3)
    assert strategy.expected_conversion_lift == pytest.approx(1.2)
    assert strategy.revenue_impact == pytest.approx(84.0)
    assert strategy.customer_acquisition_benefit == pytest.approx(0.36)
    assert strategy.recommendation == "Apply 30.0% discount to maximize revenue"


def test_optimize_discount_recommends_holding_price_when_impact_is_small():
    strategy = optimize_discount("seg-1", 0.05, 0.05, 100.0)
    assert strategy.optimal_discount == pytest.approx(0.0075)
    assert strategy.revenue_impact == pytest.approx(0.0744375)
    assert strategy.recommendation == "Maintain current pricing"


def test_optimize_discount_rejects_invalid_inputs():
    with pytest.raises(ValueError):
        optimize_discount("", 0.05, 1.0, 100.0)
    with pytest.raises(ValueError):
        optimize_discount("seg", 1.5, 1.0, 100.0)
    with pytest.raises(ValueError):
        optimize_discount("seg", 0.05, -1.0, 100.0)
    with pytest.raises(ValueError):
        optimize_discount("seg", 0.05, 1.0, -100.0)
    with pytest.raises(ValueError):
        optimize_discount("seg", 0.05, float("nan"), 100.0)


def test_revenue_attribution_preserves_maps_and_splits():
    attribution = calculate_revenue_attribution(
        "seg-1",
        [("email", 100.0), ("ads", 300.0)],
        [("product-a", 150.0)],
        [("cohort-a", 250.0)],
    )
    assert isinstance(attribution, RevenueAttribution)
    assert attribution.channel_attributions == {"email": 100.0, "ads": 300.0}
    assert attribution.product_attributions == {"product-a": 150.0}
    assert attribution.cohort_attributions == {"cohort-a": 250.0}
    assert attribution.first_touch_revenue == pytest.approx(100.0)
    assert attribution.last_touch_revenue == pytest.approx(100.0)
    assert attribution.multi_touch_revenue == pytest.approx(200.0)
    assert attribution.attribution_model == "multi-touch"


def test_revenue_attribution_rejects_invalid_inputs():
    with pytest.raises(ValueError):
        calculate_revenue_attribution("", [], [], [])
    with pytest.raises(ValueError):
        calculate_revenue_attribution("seg", [("", 1.0)], [], [])
    with pytest.raises(ValueError):
        calculate_revenue_attribution("seg", [("email", -1.0)], [], [])
    with pytest.raises(ValueError):
        calculate_revenue_attribution("seg", [], [("p", float("nan"))], [])


def test_revenue_forecast_is_linear_with_full_accuracy():
    forecast = forecast_revenue([100.0, 120.0, 140.0], 3)
    assert isinstance(forecast, RevenueForecast)
    assert forecast.forecast_periods == 3
    assert forecast.predicted_revenue == pytest.approx([160.0, 180.0, 200.0])
    assert forecast.confidence_intervals_lower == pytest.approx([160.0, 180.0, 200.0])
    assert forecast.confidence_intervals_upper == pytest.approx([160.0, 180.0, 200.0])
    assert forecast.growth_rate == pytest.approx(40.0)
    assert forecast.forecast_accuracy == pytest.approx(1.0)


def test_revenue_forecast_zero_periods_is_empty_not_an_error():
    forecast = forecast_revenue([100.0, 120.0, 140.0], 0)
    assert forecast.forecast_periods == 0
    assert forecast.predicted_revenue == []


def test_revenue_forecast_rejects_invalid_inputs():
    with pytest.raises(ValueError):
        forecast_revenue([100.0, 120.0], 2)
    with pytest.raises(ValueError):
        forecast_revenue([0.0, 120.0, 140.0], 2)
    with pytest.raises(ValueError):
        forecast_revenue([100.0, -120.0, 140.0], 2)
    with pytest.raises(ValueError):
        forecast_revenue([100.0, float("nan"), 140.0], 2)


def test_additional_classes_and_functions_exported_in_all():
    import clusteraudiencekit as cak

    for name in (
        "CompositionForecast",
        "WhatIfScenario",
        "DiscountStrategy",
        "RevenueAttribution",
        "RevenueForecast",
        "forecast_composition",
        "simulate_what_if",
        "optimize_discount",
        "calculate_revenue_attribution",
        "forecast_revenue",
    ):
        assert name in cak.__all__
    assert CompositionForecast.__name__ == "PyCompositionForecast"
    assert WhatIfScenario.__name__ == "PyWhatIfScenario"
    assert DiscountStrategy.__name__ == "PyDiscountStrategy"
    assert RevenueAttribution.__name__ == "PyRevenueAttribution"
    assert RevenueForecast.__name__ == "PyRevenueForecast"
