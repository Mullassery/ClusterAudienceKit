"""Exposed historical-reconstruction, momentum, tier-migration, category-affinity, and CAC APIs."""

from __future__ import annotations

import pytest

from clusteraudiencekit import (
    CategoryAffinity,
    CustomerAcquisitionCost,
    HistoricalSegmentState,
    TierMigrationForecast,
    TrendMomentum,
    analyze_category_affinity,
    analyze_trend_momentum,
    calculate_cac,
    predict_tier_migration,
    reconstruct_segment_history,
)


def test_reconstruct_segment_history_replays_events_to_target_date():
    events = [
        ("2026-01-01", "added", "a"),
        ("2026-01-02", "added", "b"),
        ("2026-01-03", "removed", "a"),
        ("2026-02-01", "added", "c"),
    ]
    state = reconstruct_segment_history("seg-1", "2026-01-31", events)
    assert isinstance(state, HistoricalSegmentState)
    assert state.segment_id == "seg-1"
    assert state.date == "2026-01-31"
    assert state.members == ["b"]
    assert state.size_trend == []
    assert state.composition_changes == []


def test_reconstruct_segment_history_skips_readds_and_keeps_new_members():
    events = [
        ("2026-01-01", "added", "a"),
        ("2026-01-02", "added", "a"),
        ("2026-01-03", "added", "b"),
    ]
    state = reconstruct_segment_history("seg-1", "2026-12-31", events)
    assert state.members == ["a", "b"]


def test_reconstruct_segment_history_rejects_invalid_events():
    with pytest.raises(ValueError):
        reconstruct_segment_history("", "2026-01-01", [])
    with pytest.raises(ValueError):
        reconstruct_segment_history("seg", "", [])
    with pytest.raises(ValueError):
        reconstruct_segment_history("seg", "2026-01-01", [("", "added", "a")])
    with pytest.raises(ValueError):
        reconstruct_segment_history("seg", "2026-01-01", [("2026-01-01", "moved", "a")])
    with pytest.raises(ValueError):
        reconstruct_segment_history("seg", "2026-01-01", [("2026-01-01", "added", "")])


def test_trend_momentum_numeric_values():
    momentum = analyze_trend_momentum([10.0, 20.0, 30.0, 40.0, 50.0], 3)
    assert isinstance(momentum, TrendMomentum)
    assert momentum.metric_name == "trend_metric"
    assert momentum.current_value == pytest.approx(50.0)
    assert momentum.momentum_score == pytest.approx(20.0)
    assert momentum.acceleration == pytest.approx(0.0)
    assert momentum.trend_strength == "strong"
    assert momentum.continuation_probability == pytest.approx(0.8)
    assert momentum.reversal_risk == pytest.approx(0.2)


def test_trend_momentum_declining_series():
    momentum = analyze_trend_momentum([50.0, 40.0, 30.0, 20.0], 2)
    assert momentum.momentum_score == pytest.approx(-10.0)
    assert momentum.acceleration == pytest.approx(10.0)
    assert momentum.trend_strength == "strong"
    assert momentum.continuation_probability == pytest.approx(10.0 / 15.0)


def test_trend_momentum_rejects_invalid_input():
    with pytest.raises(ValueError):
        analyze_trend_momentum([1.0, 2.0], 2)
    with pytest.raises(ValueError):
        analyze_trend_momentum([1.0, 2.0, 3.0], 0)
    with pytest.raises(ValueError):
        analyze_trend_momentum([1.0, float("nan"), 3.0], 2)


def test_tier_migration_upgrade_forecast():
    forecast = predict_tier_migration("seg-1", "basic", 120.0, [100.0, 100.0])
    assert isinstance(forecast, TierMigrationForecast)
    assert forecast.segment_id == "seg-1"
    assert forecast.current_tier == "basic"
    assert forecast.tier_migration_probabilities == pytest.approx(
        {"stay": 0.8, "upgrade": 0.2, "downgrade": 0.0}
    )
    assert forecast.most_likely_tier == "upgrade"
    assert forecast.migration_timeline_months == 3
    assert forecast.expansion_likelihood == pytest.approx(0.2)
    assert forecast.churn_risk == pytest.approx(0.2)


def test_tier_migration_downgrade_and_stay():
    downgrade = predict_tier_migration("seg-1", "pro", 50.0, [100.0])
    assert downgrade.most_likely_tier == "downgrade"
    assert downgrade.churn_risk == pytest.approx(0.8)
    assert downgrade.expansion_likelihood == 0.0

    stay = predict_tier_migration("seg-1", "pro", 100.0, [100.0])
    assert stay.most_likely_tier == "stay"
    assert stay.tier_migration_probabilities["stay"] == pytest.approx(1.0)


def test_tier_migration_rejects_invalid_input():
    with pytest.raises(ValueError):
        predict_tier_migration("", "basic", 10.0, [10.0])
    with pytest.raises(ValueError):
        predict_tier_migration("seg", "", 10.0, [10.0])
    with pytest.raises(ValueError):
        predict_tier_migration("seg", "basic", float("inf"), [10.0])
    with pytest.raises(ValueError):
        predict_tier_migration("seg", "basic", 10.0, [])
    with pytest.raises(ValueError):
        predict_tier_migration("seg", "basic", 10.0, [float("nan")])


def test_category_affinity_scores_and_bundles():
    affinity = analyze_category_affinity("seg-1", [("books", 60.0), ("games", 40.0)])
    assert isinstance(affinity, CategoryAffinity)
    assert affinity.segment_id == "seg-1"
    assert affinity.affinity_scores == pytest.approx({"books": 0.6, "games": 0.4})
    assert len(affinity.category_pairs) == 1
    left, right, score = affinity.category_pairs[0]
    assert (left, right) == ("books", "games")
    assert score == pytest.approx(2400.0**0.5)
    assert affinity.recommended_bundles == ["books-games"]
    assert affinity.cross_sell_opportunities == pytest.approx(0.55)


def test_category_affinity_single_category_has_no_pairs():
    affinity = analyze_category_affinity("seg-1", [("books", 10.0)])
    assert affinity.category_pairs == []
    assert affinity.recommended_bundles == []
    assert affinity.cross_sell_opportunities == pytest.approx(1.0)


def test_category_affinity_rejects_invalid_input():
    with pytest.raises(ValueError):
        analyze_category_affinity("", [("books", 1.0)])
    with pytest.raises(ValueError):
        analyze_category_affinity("seg", [])
    with pytest.raises(ValueError):
        analyze_category_affinity("seg", [("", 1.0)])
    with pytest.raises(ValueError):
        analyze_category_affinity("seg", [("books", -1.0)])
    with pytest.raises(ValueError):
        analyze_category_affinity("seg", [("books", float("nan"))])


def test_calculate_cac_numeric_values():
    cac = calculate_cac("seg-1", 1000.0, 10, 50.0)
    assert isinstance(cac, CustomerAcquisitionCost)
    assert cac.segment_id == "seg-1"
    assert cac.total_acquisition_spend == pytest.approx(1000.0)
    assert cac.customers_acquired == 10
    assert cac.cac_per_customer == pytest.approx(100.0)
    assert cac.payback_period_months == pytest.approx(2.0)
    assert cac.efficiency_score == pytest.approx((12.0 - 2.0) / 12.0)


def test_calculate_cac_zero_monthly_revenue_caps_payback():
    cac = calculate_cac("seg-1", 500.0, 5, 0.0)
    assert cac.cac_per_customer == pytest.approx(100.0)
    assert cac.payback_period_months == pytest.approx(1000.0)
    assert cac.efficiency_score == 0.0


def test_calculate_cac_zero_spend_is_fully_efficient():
    cac = calculate_cac("seg-1", 0.0, 5, 20.0)
    assert cac.cac_per_customer == 0.0
    assert cac.payback_period_months == 0.0
    assert cac.efficiency_score == 1.0


def test_calculate_cac_rejects_invalid_input():
    with pytest.raises(ValueError):
        calculate_cac("", 1000.0, 10, 50.0)
    with pytest.raises(ValueError):
        calculate_cac("seg", -1.0, 10, 50.0)
    with pytest.raises(ValueError):
        calculate_cac("seg", 1000.0, 0, 50.0)
    with pytest.raises(ValueError):
        calculate_cac("seg", 1000.0, 10, -1.0)
    with pytest.raises(ValueError):
        calculate_cac("seg", float("nan"), 10, 50.0)


def test_classes_and_functions_exported_in_all():
    import clusteraudiencekit as cak

    for name in (
        "HistoricalSegmentState",
        "TrendMomentum",
        "TierMigrationForecast",
        "CategoryAffinity",
        "CustomerAcquisitionCost",
        "reconstruct_segment_history",
        "analyze_trend_momentum",
        "predict_tier_migration",
        "analyze_category_affinity",
        "calculate_cac",
    ):
        assert name in cak.__all__
    assert HistoricalSegmentState.__name__ == "PyHistoricalSegmentState"
    assert TrendMomentum.__name__ == "PyTrendMomentum"
    assert TierMigrationForecast.__name__ == "PyTierMigrationForecast"
    assert CategoryAffinity.__name__ == "PyCategoryAffinity"
    assert CustomerAcquisitionCost.__name__ == "PyCustomerAcquisitionCost"
