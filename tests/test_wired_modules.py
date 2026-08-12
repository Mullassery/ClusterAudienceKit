"""Tests for the engine modules wired into the Python API in this release:
privacy (differential privacy / k-anonymity), streaming, drift_detection,
lookalike, cohorts, lifecycle, cluster quality metrics, k-estimation,
behavioral rule-based segmentation, and segment profiling.

These modules previously had real Rust-level unit tests but were never
exposed through `#[pymodule]` in `src/python.rs`; each class/function here
now has at least one Python-level test exercising the real PyO3 binding
(not a mock).
"""

import pytest
from clusteraudiencekit import (
    PyPrivacyBudget,
    add_laplace_noise,
    add_gaussian_noise,
    check_k_anonymity,
    suppress_to_k_anonymous,
    generalize_numeric,
    calculate_information_loss,
    PyStreamingEvent,
    PyStreamingConfig,
    PyStreamingSegmentationEngine,
    kolmogorov_smirnov,
    hellinger_distance,
    chi_square_drift,
    detect_feature_drift,
    detect_segment_composition_change,
    PySeedCustomer,
    generate_lookalike,
    find_similar_customers,
    cosine_similarity,
    cohort_id_for,
    create_cohort,
    compare_cohorts,
    aggregate_cohorts_by_period,
    cohort_retention_table,
    cohort_performance_ranking,
    classify_lifecycle_stage,
    lifecycle_retention_actions,
    lifecycle_stage_distribution,
    silhouette_score,
    davies_bouldin_score,
    calinski_harabasz_score,
    assess_cluster_quality,
    estimate_k_elbow,
    estimate_k_gap_statistic,
    estimate_k_silhouette,
    estimate_k_combined,
    PyCondition,
    PyBehavioralRule,
    PyBehavioralSegment,
    PyBehavioralSegmenter,
    profile_segment,
    kmeans,
)


class TestPrivacy:
    def test_privacy_budget_consumption(self):
        budget = PyPrivacyBudget(1.0, 0.01)
        assert budget.consume(0.4)
        assert not budget.budget_exhausted()
        assert budget.consume(0.6)
        assert budget.budget_exhausted()
        assert not budget.consume(0.01)  # over budget, refused

    def test_laplace_noise_preserves_length_and_nonnegativity(self):
        noisy = add_laplace_noise([10.0, 20.0, 30.0], epsilon=1.0, sensitivity=1.0)
        assert len(noisy) == 3
        assert all(v >= 0.0 for v in noisy)

    def test_gaussian_noise_preserves_length(self):
        noisy = add_gaussian_noise([5.0, 5.0], epsilon=0.5, delta=0.01, sensitivity=1.0)
        assert len(noisy) == 2

    def test_k_anonymity_detects_small_groups(self):
        data = [{"age": "20-30", "gender": "M"}] * 2 + [{"age": "40-50", "gender": "F"}] * 1
        result = check_k_anonymity(data, ["age", "gender"], k=2)
        assert result.k_value == 2
        assert not result.anonymized  # the 1-row group violates k=2
        assert result.suppressed_rows == 1

    def test_suppress_to_k_anonymous_drops_small_groups(self):
        data = [{"age": "20-30"}] * 3 + [{"age": "unique"}] * 1
        kept = suppress_to_k_anonymous(data, ["age"], k=2)
        assert len(kept) == 3

    def test_generalize_numeric_and_information_loss(self):
        values = [10.0, 20.0, 30.0, 40.0, 50.0]
        bins = generalize_numeric(values, intervals=5)
        assert len(bins) == 5
        assert all(0 <= b < 5 for b in bins)
        loss = calculate_information_loss(values, bins, 5)
        assert loss >= 0.0


class TestStreaming:
    def test_engine_processes_event_and_assigns_segment(self):
        config = PyStreamingConfig(batch_size=10, window="hour")
        engine = PyStreamingSegmentationEngine(config)
        event = PyStreamingEvent("cust_1", "purchase", 500.0, 1704067200)
        update = engine.process_event(event)
        assert update is not None
        assert update.customer_id == "cust_1"
        assert engine.get_segment("cust_1") is not None
        assert engine.customer_count() == 1

    def test_engine_processes_batch(self):
        config = PyStreamingConfig()
        engine = PyStreamingSegmentationEngine(config)
        events = [
            PyStreamingEvent("cust_1", "purchase", 100.0, 1704067200),
            PyStreamingEvent("cust_2", "engagement", 0.0, 1704067200),
        ]
        updates = engine.process_batch(events)
        assert len(updates) == 2
        dist = engine.segment_distribution()
        assert sum(dist.values()) == 2

    def test_unknown_event_type_raises(self):
        with pytest.raises(Exception):
            PyStreamingEvent("cust_1", "not_a_real_type", 1.0, 0)


class TestDriftDetection:
    def test_kolmogorov_smirnov_no_drift_vs_drift(self):
        same = kolmogorov_smirnov([1.0, 2.0, 3.0], [1.0, 2.0, 3.0])
        assert same < 0.1
        drifted = kolmogorov_smirnov([1.0, 2.0, 3.0], [100.0, 200.0, 300.0])
        assert drifted > 0.5

    def test_hellinger_distance_bounded(self):
        d = hellinger_distance([1.0, 2.0, 3.0], [4.0, 5.0, 6.0])
        assert 0.0 <= d <= 1.0

    def test_chi_square_drift_positive_when_distributions_differ(self):
        chi2 = chi_square_drift({"A": 50, "B": 50}, {"A": 90, "B": 10})
        assert chi2 > 0.0

    def test_detect_feature_drift_classifies_severity(self):
        drift = detect_feature_drift("recency", [1.0, 2.0, 3.0], [100.0, 200.0, 300.0], "ks")
        assert drift.feature_name == "recency"
        assert drift.severity in {"none", "low", "medium", "high", "critical"}
        assert drift.severity == "critical"

    def test_segment_composition_change(self):
        changes = detect_segment_composition_change(
            {"Champions": 100, "Lost": 50}, {"Champions": 120, "Lost": 30}
        )
        names = {c.segment_name for c in changes}
        assert names == {"Champions", "Lost"}


class TestLookalike:
    def test_cosine_similarity_identical_vectors(self):
        assert cosine_similarity([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]) == pytest.approx(1.0)

    def test_generate_lookalike_audience(self):
        seed = PySeedCustomer("seed_1", [1.0, 1.0, 1.0], ltv=1000.0)
        candidates = [PySeedCustomer(f"c{i}", [1.0 + i * 0.05, 1.0, 1.0]) for i in range(10)]
        audience = generate_lookalike([seed], candidates, metric="cosine", percentile_threshold=0.5)
        assert audience.seed_count == 1
        assert audience.lookalike_count > 0
        assert 0.0 <= audience.avg_similarity <= 1.0

    def test_find_similar_customers_returns_ranked_results(self):
        seed = PySeedCustomer("seed_1", [1.0, 0.0])
        candidates = [
            PySeedCustomer("near", [1.0, 0.1]),
            PySeedCustomer("far", [-1.0, 5.0]),
        ]
        results = find_similar_customers(seed, candidates, n=2, metric="cosine")
        assert results[0].customer_id == "near"


class TestCohorts:
    def test_cohort_id_is_deterministic_for_same_month(self):
        id_a = cohort_id_for("monthly", 1704067200)  # 2024-01-01
        id_b = cohort_id_for("monthly", 1704153600)  # 2024-01-02
        assert id_a == id_b

    def test_create_cohort_and_summary(self):
        cohort = create_cohort(
            "test_cohort", "monthly", 1704067200,
            [("c1", 100.0, True), ("c2", 200.0, False), ("c3", 300.0, True)],
        )
        assert cohort.size == 3
        assert cohort.revenue == 600.0
        summary = cohort.summary()
        assert "retention_rate" in summary

    def test_add_retention_point_and_decay_rate(self):
        cohort = create_cohort("c", "monthly", 0, [("c1", 100.0, True), ("c2", 100.0, True)])
        cohort.add_retention_point(1, 1)
        assert len(cohort.retention_curve) == 2
        # decay rate should be a finite number (no exception)
        rate = cohort.retention_decay_rate()
        assert isinstance(rate, float)

    def test_compare_cohorts(self):
        a = create_cohort("a", "monthly", 0, [("c1", 100.0, False)])
        b = create_cohort("b", "monthly", 0, [("c1", 100.0, True)])
        better, size_diff, revenue_diff, ltv_diff, retention_diff = compare_cohorts(a, b)
        assert better == "b"  # b has higher retention
        assert retention_diff > 0.0

    def test_aggregate_and_rank(self):
        a = create_cohort("a", "monthly", 0, [("c1", 100.0, True)])
        b = create_cohort("b", "monthly", 0, [("c1", 100.0, False)])
        aggregates = aggregate_cohorts_by_period([a, b])
        assert "a" in aggregates and "b" in aggregates
        table = cohort_retention_table([a, b])
        assert len(table) == 2
        best, worst = cohort_performance_ranking([a, b])
        assert best == "a"
        assert worst == "b"


class TestLifecycle:
    def test_classify_prospect(self):
        profile = classify_lifecycle_stage("c1", 10, 0, 0.0, 10, 0.0)
        assert profile.current_stage == "prospect"
        assert profile.stage_confidence > 0.8

    def test_classify_active_customer(self):
        profile = classify_lifecycle_stage("c1", 90, 10, 1000.0, 20, 40.0)
        assert profile.current_stage == "active"

    def test_retention_actions_differ_by_stage(self):
        prospect_actions = lifecycle_retention_actions("prospect")
        at_risk_actions = lifecycle_retention_actions("at_risk")
        assert prospect_actions != at_risk_actions
        assert len(prospect_actions) > 0

    def test_unknown_stage_raises(self):
        with pytest.raises(Exception):
            lifecycle_retention_actions("not_a_stage")

    def test_stage_distribution_sums_to_100(self):
        profiles = [
            classify_lifecycle_stage("c1", 90, 10, 1000.0, 20, 40.0),
            classify_lifecycle_stage("c2", 731, 30, 5000.0, 30, 15.0),
        ]
        dist = lifecycle_stage_distribution(profiles)
        assert abs(sum(dist.values()) - 100.0) < 1e-9


class TestClusterQualityMetrics:
    def test_silhouette_score_well_separated(self):
        data = [[0.0, 0.0], [0.1, 0.1], [10.0, 10.0], [10.1, 9.9]]
        score = silhouette_score(data, [0, 0, 1, 1])
        assert score > 0.9

    def test_davies_bouldin_and_calinski_harabasz(self):
        data = [[0.0, 0.0], [0.1, 0.1], [10.0, 10.0], [10.1, 9.9]]
        centers = [[0.05, 0.05], [10.05, 9.95]]
        db = davies_bouldin_score(data, [0, 0, 1, 1], centers)
        ch = calinski_harabasz_score(data, [0, 0, 1, 1])
        assert db >= 0.0
        assert ch > 0.0

    def test_assess_cluster_quality_end_to_end(self):
        data = [[0.0, 0.0], [0.2, 0.1], [10.0, 10.0], [10.1, 9.9]]
        result = kmeans(data, n_clusters=2, random_state=0)
        report = assess_cluster_quality(data, result.labels, result.centers)
        assert report.n_clusters == 2
        assert 0.0 <= report.overall_score <= 100.0


class TestKEstimation:
    DATA = [[0.0, 0.0], [0.1, 0.1], [10.0, 10.0], [10.1, 9.9], [20.0, 0.0], [20.1, 0.1]]

    def test_elbow_method_returns_valid_k(self):
        result = estimate_k_elbow(self.DATA, (2, 4))
        assert 2 <= result.k <= 4
        assert result.method == "elbow"

    def test_gap_statistic_returns_valid_k(self):
        result = estimate_k_gap_statistic(self.DATA, (2, 4))
        assert 2 <= result.k <= 4

    def test_silhouette_estimation_returns_valid_k(self):
        result = estimate_k_silhouette(self.DATA, (2, 4))
        assert 2 <= result.k <= 4

    def test_combined_estimation_returns_valid_k(self):
        result = estimate_k_combined(self.DATA, (2, 4))
        assert 2 <= result.k <= 4


class TestBehavioralSegmentation:
    def test_rule_based_classification(self):
        high_value = PyCondition("monetary", ">", 1000.0)
        rule = PyBehavioralRule("high_value_rule", "spends a lot", [high_value])
        segment = PyBehavioralSegment("HighValue", "big spenders", [rule])
        segmenter = PyBehavioralSegmenter([segment])

        assert segmenter.classify({"monetary": 2000.0}) == ["HighValue"]
        assert segmenter.classify({"monetary": 10.0}) == []

    def test_to_sql_generates_where_clause(self):
        cond = PyCondition("frequency", ">=", 5.0)
        rule = PyBehavioralRule("frequent", "desc", [cond])
        sql = rule.to_sql()
        assert "frequency" in sql
        assert ">=" in sql

    def test_export_sql_for_segmenter(self):
        cond = PyCondition("monetary", ">", 500.0)
        rule = PyBehavioralRule("r1", "d", [cond])
        segment = PyBehavioralSegment("S1", "d", [rule])
        segmenter = PyBehavioralSegmenter([segment])
        sql = segmenter.export_sql()
        assert "monetary" in sql


class TestProfiling:
    def test_profile_segment_produces_health_and_description(self):
        members = [0, 1, 2]
        features = {0: [10.0, 20.0, 30.0], 1: [1.0, 1.0, 1.0]}
        profile = profile_segment(0, members, features, ["monetary", "frequency"])
        assert profile.segment_id == 0
        assert profile.size == 3
        assert profile.business_description
        assert 0.0 <= profile.health.health_score
