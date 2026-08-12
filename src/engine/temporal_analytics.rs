//! Temporal analytics engine: time machine, forecasting, what-if modeling, scenario planning

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 1. TemporalSnapshot - Capture segment state at a point in time
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentSnapshot {
    pub segment_id: String,
    pub segment_name: String,
    pub timestamp_days_ago: i32,
    pub member_count: usize,
    pub avg_rfm_score: f64,
    pub avg_clv: f64,
    pub churn_rate: f64,
    pub confidence_score: f64,
    pub key_characteristics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalSnapshot {
    pub snapshot_date: String,
    pub days_ago: i32,
    pub total_customers: usize,
    pub segments: Vec<SegmentSnapshot>,
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub data_quality: f64,
    pub completeness: f64,
    pub interpolation_used: bool,
    pub source: String,
}

// ============================================================================
// 2. HistoricalReconstruction - Rebuild past segments from event logs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSegmentState {
    pub segment_id: String,
    pub date: String,
    pub members: Vec<String>,
    pub size_trend: Vec<(i32, usize)>,
    pub composition_changes: Vec<CompositionChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionChange {
    pub days_ago: i32,
    pub change_type: String,
    pub affected_members: usize,
    pub reason: String,
}

pub struct HistoricalReconstruction;

impl HistoricalReconstruction {
    pub fn reconstruct_at_date(
        segment_id: &str,
        target_date: &str,
        historical_events: &[(String, String, String)],
    ) -> Result<HistoricalSegmentState> {
        let mut state = HistoricalSegmentState {
            segment_id: segment_id.to_string(),
            date: target_date.to_string(),
            members: Vec::new(),
            size_trend: Vec::new(),
            composition_changes: Vec::new(),
        };

        for (event_date, event_type, member_id) in historical_events {
            if event_date.as_str() <= target_date {
                match event_type.as_str() {
                    "added" => {
                        if !state.members.contains(member_id) {
                            state.members.push(member_id.clone());
                        }
                    }
                    "removed" => {
                        state.members.retain(|m| m != member_id);
                    }
                    _ => {}
                }
            }
        }

        Ok(state)
    }

    pub fn build_composition_timeline(
        _segment_id: &str,
        start_date: &str,
        end_date: &str,
        historical_events: &[(String, String, String)],
    ) -> Result<Vec<CompositionChange>> {
        let mut changes = Vec::new();
        let mut current_members = 0;

        for (event_date, event_type, _member_id) in historical_events {
            if event_date.as_str() >= start_date && event_date.as_str() <= end_date {
                let change = match event_type.as_str() {
                    "added" => {
                        current_members += 1;
                        CompositionChange {
                            days_ago: 0,
                            change_type: "member_added".to_string(),
                            affected_members: 1,
                            reason: "Segment assignment".to_string(),
                        }
                    }
                    "removed" => {
                        current_members = (current_members as i32 - 1).max(0) as usize;
                        CompositionChange {
                            days_ago: 0,
                            change_type: "member_removed".to_string(),
                            affected_members: 1,
                            reason: "Segment reassignment".to_string(),
                        }
                    }
                    _ => continue,
                };
                changes.push(change);
            }
        }

        Ok(changes)
    }
}

// ============================================================================
// 3. SegmentSizeForecasting - Predict future segment sizes
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeForecast {
    pub segment_id: String,
    pub forecast_days: u32,
    pub predictions: Vec<(u32, f64)>,
    pub confidence_interval_lower: Vec<f64>,
    pub confidence_interval_upper: Vec<f64>,
    pub trend_type: String,
    pub r_squared: f64,
}

pub struct SegmentSizeForecaster;

impl SegmentSizeForecaster {
    pub fn forecast_sizes(
        historical_sizes: &[(i32, usize)],
        forecast_periods: u32,
    ) -> Result<SizeForecast> {
        if historical_sizes.len() < 2 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Need at least 2 data points for forecasting".to_string(),
            ));
        }

        let x: Vec<f64> = historical_sizes.iter().map(|(d, _)| *d as f64).collect();
        let y: Vec<f64> = historical_sizes.iter().map(|(_, s)| *s as f64).collect();

        let (slope, intercept) = Self::linear_regression(&x, &y)?;
        let r_squared = Self::calculate_r_squared(&x, &y, slope, intercept)?;

        let mut predictions = Vec::new();
        let mut lower = Vec::new();
        let mut upper = Vec::new();

        let last_day = historical_sizes.last().unwrap().0;

        for i in 1..=forecast_periods {
            let future_day = (last_day + i as i32) as f64;
            let predicted_value = (slope * future_day + intercept).max(0.0);
            let ci_width = 0.1 * predicted_value.abs();

            predictions.push((i, predicted_value));
            lower.push((predicted_value - ci_width).max(0.0));
            upper.push(predicted_value + ci_width);
        }

        let trend_type = if slope > 0.1 {
            "growth".to_string()
        } else if slope < -0.1 {
            "decline".to_string()
        } else {
            "stable".to_string()
        };

        Ok(SizeForecast {
            segment_id: "forecast".to_string(),
            forecast_days: forecast_periods,
            predictions,
            confidence_interval_lower: lower,
            confidence_interval_upper: upper,
            trend_type,
            r_squared,
        })
    }

    fn linear_regression(x: &[f64], y: &[f64]) -> Result<(f64, f64)> {
        let n = x.len() as f64;
        let x_mean = x.iter().sum::<f64>() / n;
        let y_mean = y.iter().sum::<f64>() / n;

        let numerator: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| (xi - x_mean) * (yi - y_mean))
            .sum();

        let denominator: f64 = x.iter().map(|xi| (xi - x_mean).powi(2)).sum();

        if denominator.abs() < 1e-10 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Cannot fit linear regression (zero variance in X)".to_string(),
            ));
        }

        let slope = numerator / denominator;
        let intercept = y_mean - slope * x_mean;

        Ok((slope, intercept))
    }

    fn calculate_r_squared(x: &[f64], y: &[f64], slope: f64, intercept: f64) -> Result<f64> {
        let y_mean = y.iter().sum::<f64>() / y.len() as f64;

        let ss_tot: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum();
        let ss_res: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| (yi - (slope * xi + intercept)).powi(2))
            .sum();

        if ss_tot.abs() < 1e-10 {
            return Ok(0.0);
        }

        Ok(1.0 - (ss_res / ss_tot))
    }
}

// ============================================================================
// 4. CompositionForecasting - Predict segment composition changes
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionForecast {
    pub segment_id: String,
    pub forecast_periods: u32,
    pub high_value_ratio_forecast: Vec<f64>,
    pub churn_risk_forecast: Vec<f64>,
    pub new_member_forecast: Vec<f64>,
    pub composition_stability: f64,
}

pub struct CompositionForecaster;

impl CompositionForecaster {
    pub fn forecast_composition(
        current_high_value_ratio: f64,
        current_churn_risk_ratio: f64,
        current_new_member_ratio: f64,
        historical_ratios: &[(f64, f64, f64)],
        forecast_periods: u32,
    ) -> Result<CompositionForecast> {
        let hv_trend = Self::calculate_trend(
            &historical_ratios
                .iter()
                .map(|(hv, _, _)| *hv)
                .collect::<Vec<_>>(),
        );
        let cr_trend = Self::calculate_trend(
            &historical_ratios
                .iter()
                .map(|(_, cr, _)| *cr)
                .collect::<Vec<_>>(),
        );
        let nm_trend = Self::calculate_trend(
            &historical_ratios
                .iter()
                .map(|(_, _, nm)| *nm)
                .collect::<Vec<_>>(),
        );

        let mut hv_forecast = vec![current_high_value_ratio];
        let mut cr_forecast = vec![current_churn_risk_ratio];
        let mut nm_forecast = vec![current_new_member_ratio];

        for _ in 1..forecast_periods {
            let next_hv = (hv_forecast.last().unwrap() + hv_trend).clamp(0.0, 1.0);
            let next_cr = (cr_forecast.last().unwrap() + cr_trend).clamp(0.0, 1.0);
            let next_nm = (nm_forecast.last().unwrap() + nm_trend).clamp(0.0, 1.0);

            hv_forecast.push(next_hv);
            cr_forecast.push(next_cr);
            nm_forecast.push(next_nm);
        }

        let stability = Self::calculate_stability(historical_ratios);

        Ok(CompositionForecast {
            segment_id: "composition".to_string(),
            forecast_periods,
            high_value_ratio_forecast: hv_forecast,
            churn_risk_forecast: cr_forecast,
            new_member_forecast: nm_forecast,
            composition_stability: stability,
        })
    }

    fn calculate_trend(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        (values.last().unwrap() - values.first().unwrap()) / (values.len() - 1) as f64
    }

    fn calculate_stability(ratios: &[(f64, f64, f64)]) -> f64 {
        if ratios.is_empty() {
            return 1.0;
        }

        let avg_hv = ratios.iter().map(|(hv, _, _)| hv).sum::<f64>() / ratios.len() as f64;
        let variance: f64 = ratios
            .iter()
            .map(|(hv, _, _)| (hv - avg_hv).powi(2))
            .sum::<f64>()
            / ratios.len() as f64;

        1.0 / (1.0 + variance)
    }
}

// ============================================================================
// 5. MembershipForecasting - Predict individual member movement
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipForecast {
    pub member_id: String,
    pub current_segment: String,
    pub probability_stay: f64,
    pub probability_move_high_value: f64,
    pub probability_churn: f64,
    pub most_likely_next_segment: String,
    pub forecast_confidence: f64,
}

pub struct MembershipForecaster;

impl MembershipForecaster {
    pub fn forecast_member_movement(
        member_id: &str,
        current_segment: &str,
        _member_history: &[(String, String)],
        segment_transition_probs: &HashMap<(String, String), f64>,
    ) -> Result<MembershipForecast> {
        let mut stay_prob = 0.6;
        let mut move_prob = 0.25;
        let mut churn_prob = 0.15;

        if let Some((_, &prob)) = segment_transition_probs
            .iter()
            .find(|((from, to), _)| from.as_str() == current_segment && to == "*")
        {
            stay_prob = prob;
        }

        let (next_segment, move_prob_val) =
            Self::find_most_likely_transition(current_segment, segment_transition_probs);

        move_prob = move_prob_val;
        churn_prob = 1.0 - stay_prob - move_prob;

        let confidence = (stay_prob.max(move_prob) - 0.33).max(0.0);

        Ok(MembershipForecast {
            member_id: member_id.to_string(),
            current_segment: current_segment.to_string(),
            probability_stay: stay_prob,
            probability_move_high_value: move_prob,
            probability_churn: churn_prob.max(0.0),
            most_likely_next_segment: next_segment,
            forecast_confidence: confidence,
        })
    }

    fn find_most_likely_transition(
        current_segment: &str,
        transitions: &HashMap<(String, String), f64>,
    ) -> (String, f64) {
        let current_transitions: Vec<_> = transitions
            .iter()
            .filter(|((from, _), _)| from.as_str() == current_segment)
            .collect();

        if let Some(((_from, to), prob)) = current_transitions
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            (to.clone(), **prob)
        } else {
            ("unknown".to_string(), 0.0)
        }
    }
}

// ============================================================================
// 6. WhatIfScenario - Simulate parameter/rule changes
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatIfScenario {
    pub scenario_name: String,
    pub description: String,
    pub parameters_changed: HashMap<String, (String, String)>,
    pub projected_segment_sizes: Vec<(String, usize)>,
    pub projected_revenue_impact: f64,
    pub projected_churn_impact: f64,
    pub feasibility_score: f64,
}

pub struct WhatIfSimulator;

impl WhatIfSimulator {
    pub fn simulate_parameter_change(
        scenario_name: &str,
        baseline_sizes: &[(String, usize)],
        parameter_changes: &HashMap<String, f64>,
    ) -> Result<WhatIfScenario> {
        let mut projected_sizes = baseline_sizes.to_vec();

        if let Some(&rfm_threshold) = parameter_changes.get("rfm_threshold") {
            for (_segment, size) in &mut projected_sizes {
                *size = (*size as f64 * (1.0 + rfm_threshold * 0.1)) as usize;
            }
        }

        if let Some(&cluster_count) = parameter_changes.get("n_clusters") {
            for (_segment, size) in &mut projected_sizes {
                *size = (*size as f64 / cluster_count) as usize;
            }
        }

        let revenue_impact = Self::calculate_revenue_impact(&projected_sizes, baseline_sizes);
        let churn_impact = Self::calculate_churn_impact(parameter_changes);
        let feasibility = Self::calculate_feasibility(parameter_changes);

        let mut params_changed = HashMap::new();
        for (key, &val) in parameter_changes {
            params_changed.insert(key.clone(), ("old".to_string(), format!("{:.2}", val)));
        }

        Ok(WhatIfScenario {
            scenario_name: scenario_name.to_string(),
            description: format!(
                "Scenario with {} parameter changes",
                parameter_changes.len()
            ),
            parameters_changed: params_changed,
            projected_segment_sizes: projected_sizes,
            projected_revenue_impact: revenue_impact,
            projected_churn_impact: churn_impact,
            feasibility_score: feasibility,
        })
    }

    fn calculate_revenue_impact(
        projected: &[(String, usize)],
        baseline: &[(String, usize)],
    ) -> f64 {
        let proj_total: usize = projected.iter().map(|(_, s)| s).sum();
        let base_total: usize = baseline.iter().map(|(_, s)| s).sum();

        if base_total == 0 {
            return 0.0;
        }

        ((proj_total as f64 - base_total as f64) / base_total as f64) * 100.0
    }

    fn calculate_churn_impact(parameters: &HashMap<String, f64>) -> f64 {
        parameters.get("churn_threshold").copied().unwrap_or(0.0)
    }

    fn calculate_feasibility(parameters: &HashMap<String, f64>) -> f64 {
        let count = parameters.len() as f64;
        (1.0 - (count / 10.0)).clamp(0.0, 1.0)
    }
}

// ============================================================================
// 7. ScenarioComparison - Compare multiple what-if scenarios
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioComparison {
    pub scenarios: Vec<(String, f64, f64)>,
    pub best_revenue_scenario: String,
    pub best_churn_mitigation: String,
    pub overall_recommendation: String,
    pub comparison_summary: String,
}

pub struct ScenarioAnalyzer;

impl ScenarioAnalyzer {
    pub fn compare_scenarios(scenarios: &[WhatIfScenario]) -> Result<ScenarioComparison> {
        if scenarios.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No scenarios to compare".to_string(),
            ));
        }

        let mut scenario_data = Vec::new();
        for scenario in scenarios {
            scenario_data.push((
                scenario.scenario_name.clone(),
                scenario.projected_revenue_impact,
                scenario.projected_churn_impact,
            ));
        }

        let best_revenue = scenario_data
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|s| s.0.clone())
            .unwrap_or_default();

        let best_churn = scenario_data
            .iter()
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .map(|s| s.0.clone())
            .unwrap_or_default();

        let recommendation = if scenario_data.len() > 1 {
            format!(
                "Recommend {} for revenue, {} for churn mitigation",
                best_revenue, best_churn
            )
        } else {
            format!("Recommend {}", scenario_data[0].0)
        };

        Ok(ScenarioComparison {
            scenarios: scenario_data,
            best_revenue_scenario: best_revenue,
            best_churn_mitigation: best_churn,
            overall_recommendation: recommendation,
            comparison_summary: format!("{} scenarios analyzed", scenarios.len()),
        })
    }
}

// ============================================================================
// 8. SensitivityAnalysis - Analyze impact of parameter changes
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityResult {
    pub parameter: String,
    pub baseline_value: f64,
    pub sensitivity_coefficient: f64,
    pub elasticity: f64,
    pub impact_on_segments: f64,
    pub impact_on_revenue: f64,
    pub risk_level: String,
}

pub struct SensitivityAnalyzer;

impl SensitivityAnalyzer {
    pub fn analyze_parameter_sensitivity(
        parameter_name: &str,
        baseline_value: f64,
        test_value: f64,
        baseline_segments: usize,
        test_segments: usize,
        baseline_revenue: f64,
        test_revenue: f64,
    ) -> Result<SensitivityResult> {
        let value_change = test_value - baseline_value;
        let segment_change = test_segments as f64 - baseline_segments as f64;
        let revenue_change = test_revenue - baseline_revenue;

        let sensitivity_coef = if value_change.abs() > 1e-10 {
            segment_change / value_change
        } else {
            0.0
        };

        let elasticity = if baseline_value.abs() > 1e-10 && baseline_segments > 0 {
            (segment_change / baseline_segments as f64) / (value_change / baseline_value)
        } else {
            0.0
        };

        let impact_segments = (segment_change / baseline_segments as f64 * 100.0).abs();
        let impact_revenue = (revenue_change / baseline_revenue.abs() * 100.0).abs();

        let risk_level = if sensitivity_coef.abs() > 2.0 {
            "high".to_string()
        } else if sensitivity_coef.abs() > 1.0 {
            "medium".to_string()
        } else {
            "low".to_string()
        };

        Ok(SensitivityResult {
            parameter: parameter_name.to_string(),
            baseline_value,
            sensitivity_coefficient: sensitivity_coef,
            elasticity,
            impact_on_segments: impact_segments,
            impact_on_revenue: impact_revenue,
            risk_level,
        })
    }

    pub fn tornado_analysis(
        _baseline_metrics: &HashMap<String, f64>,
        parameter_ranges: &HashMap<String, (f64, f64)>,
    ) -> Result<Vec<(String, f64)>> {
        let mut impacts = Vec::new();

        for (param, &(low, high)) in parameter_ranges {
            let range = (high - low).abs();
            impacts.push((param.clone(), range));
        }

        impacts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(impacts)
    }
}

// ============================================================================
// 9. ExpansionPlanning - Scenario planning for growth
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionPlan {
    pub plan_name: String,
    pub target_customer_count: usize,
    pub target_segments: Vec<String>,
    pub growth_rate_per_quarter: f64,
    pub required_resources: HashMap<String, f64>,
    pub timeline_quarters: u32,
    pub roi_projection: f64,
    pub risk_factors: Vec<String>,
}

pub struct ExpansionPlanner;

impl ExpansionPlanner {
    pub fn plan_expansion(
        current_customer_count: usize,
        target_customer_count: usize,
        quarters: u32,
        current_segments: usize,
    ) -> Result<ExpansionPlan> {
        if quarters == 0 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Timeline must be > 0 quarters".to_string(),
            ));
        }

        let total_growth = target_customer_count as f64 - current_customer_count as f64;
        let growth_per_quarter = total_growth / quarters as f64;
        let growth_rate = (growth_per_quarter / current_customer_count as f64 * 100.0).abs();

        let mut resources = HashMap::new();
        resources.insert("engineering_hours".to_string(), 500.0 * quarters as f64);
        resources.insert("marketing_budget".to_string(), 50000.0 * quarters as f64);
        resources.insert("infrastructure_cost".to_string(), 10000.0 * quarters as f64);

        let roi = (total_growth / 100.0) / resources.values().sum::<f64>() * 10000.0;

        let risk_factors = vec![
            "Market saturation".to_string(),
            "Competitive pressure".to_string(),
            "Resource constraints".to_string(),
        ];

        let target_segments = (0..current_segments + 2)
            .map(|i| format!("Segment_{}", i))
            .collect();

        Ok(ExpansionPlan {
            plan_name: "Growth Strategy".to_string(),
            target_customer_count,
            target_segments,
            growth_rate_per_quarter: growth_rate,
            required_resources: resources,
            timeline_quarters: quarters,
            roi_projection: roi,
            risk_factors,
        })
    }
}

// ============================================================================
// 10. ChurnForecast - Project churn rates over time
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnForecast {
    pub segment_id: String,
    pub current_churn_rate: f64,
    pub projected_churn_rates: Vec<f64>,
    pub at_risk_members: usize,
    pub intervention_opportunities: usize,
    pub estimated_revenue_at_risk: f64,
    pub confidence: f64,
}

pub struct ChurnForecaster;

impl ChurnForecaster {
    pub fn forecast_churn(
        current_churn_rate: f64,
        historical_churn_rates: &[f64],
        segment_size: usize,
        avg_customer_value: f64,
        forecast_months: u32,
    ) -> Result<ChurnForecast> {
        let trend = if historical_churn_rates.len() >= 2 {
            historical_churn_rates.last().unwrap() - historical_churn_rates.first().unwrap()
        } else {
            0.0
        };

        let trend_per_month = trend / (historical_churn_rates.len().max(1) as f64);

        let mut projections = vec![current_churn_rate];
        for i in 1..forecast_months {
            let next_rate = (current_churn_rate + trend_per_month * i as f64).clamp(0.0, 1.0);
            projections.push(next_rate);
        }

        let avg_projected_churn = projections.iter().sum::<f64>() / projections.len() as f64;
        let at_risk_count = (segment_size as f64 * avg_projected_churn) as usize;
        let intervention_count = (at_risk_count as f64 * 0.3) as usize;
        let revenue_at_risk = at_risk_count as f64 * avg_customer_value;

        let confidence = 0.65 + (historical_churn_rates.len() as f64 * 0.05).min(0.25);

        Ok(ChurnForecast {
            segment_id: "churn_forecast".to_string(),
            current_churn_rate,
            projected_churn_rates: projections,
            at_risk_members: at_risk_count,
            intervention_opportunities: intervention_count,
            estimated_revenue_at_risk: revenue_at_risk,
            confidence,
        })
    }
}

// ============================================================================
// 11. LifecycleForecasting - Predict lifecycle stage transitions
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleTransitionForecast {
    pub current_stage: String,
    pub forecast_periods: u32,
    pub stage_probabilities: HashMap<String, Vec<f64>>,
    pub expected_next_stage: String,
    pub transition_urgency: String,
    pub recommended_action: String,
}

pub struct LifecycleForecaster;

impl LifecycleForecaster {
    pub fn forecast_transitions(
        current_stage: &str,
        historical_transition_matrix: &HashMap<(String, String), f64>,
        forecast_periods: u32,
    ) -> Result<LifecycleTransitionForecast> {
        let mut stage_probs: HashMap<String, Vec<f64>> = HashMap::new();

        let stages = vec![
            "prospect",
            "onboarding",
            "growth",
            "mature",
            "declining",
            "churn",
        ];

        for stage in &stages {
            stage_probs.insert(stage.to_string(), vec![0.0; forecast_periods as usize]);
        }

        let mut current_dist = HashMap::new();
        for stage in &stages {
            current_dist.insert(stage.to_string(), 0.0);
        }
        current_dist.insert(current_stage.to_string(), 1.0);

        for period in 0..forecast_periods {
            let mut next_dist = HashMap::new();
            for to_stage in &stages {
                next_dist.insert(to_stage.to_string(), 0.0);
            }

            for (from_stage, prob) in &current_dist {
                for to_stage in &stages {
                    let transition_key = (from_stage.clone(), to_stage.to_string());
                    let trans_prob = historical_transition_matrix
                        .get(&transition_key)
                        .copied()
                        .unwrap_or(0.0);

                    *next_dist.get_mut(&to_stage.to_string()).unwrap() += prob * trans_prob;
                }
            }

            for (stage, probs) in stage_probs.iter_mut() {
                if (period as usize) < probs.len() {
                    probs[period as usize] = *next_dist.get(stage).unwrap_or(&0.0);
                }
            }

            current_dist = next_dist;
        }

        let expected_next = stages
            .iter()
            .max_by(|a, b| {
                current_dist
                    .get(&b.to_string())
                    .unwrap_or(&0.0)
                    .partial_cmp(current_dist.get(&a.to_string()).unwrap_or(&0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.to_string())
            .unwrap_or_default();

        let urgency = if current_stage == "declining" || current_stage == "churn" {
            "high".to_string()
        } else if current_stage == "mature" {
            "medium".to_string()
        } else {
            "low".to_string()
        };

        let action = format!("Prepare for transition to {} stage", expected_next);

        Ok(LifecycleTransitionForecast {
            current_stage: current_stage.to_string(),
            forecast_periods,
            stage_probabilities: stage_probs,
            expected_next_stage: expected_next,
            transition_urgency: urgency,
            recommended_action: action,
        })
    }
}

// ============================================================================
// 12. TrendMomentum - Momentum analysis for trend continuation
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendMomentum {
    pub metric_name: String,
    pub current_value: f64,
    pub momentum_score: f64,
    pub acceleration: f64,
    pub trend_strength: String,
    pub continuation_probability: f64,
    pub reversal_risk: f64,
}

pub struct MomentumAnalyzer;

impl MomentumAnalyzer {
    pub fn analyze_momentum(
        metric_history: &[f64],
        lookback_periods: usize,
    ) -> Result<TrendMomentum> {
        if metric_history.len() < 3 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Need at least 3 data points".to_string(),
            ));
        }

        let recent = &metric_history[metric_history.len().saturating_sub(lookback_periods)..];
        let current_value = *metric_history.last().unwrap();

        let momentum = if recent.len() >= 2 {
            recent.last().unwrap() - recent.first().unwrap()
        } else {
            0.0
        };

        let acceleration = if metric_history.len() >= 4 {
            let prev_momentum =
                metric_history[metric_history.len() - 2] - metric_history[metric_history.len() - 4];
            momentum - prev_momentum
        } else {
            0.0
        };

        let trend_strength = if momentum.abs() > 5.0 {
            "strong".to_string()
        } else if momentum.abs() > 1.0 {
            "moderate".to_string()
        } else {
            "weak".to_string()
        };

        let continuation_prob = (momentum.abs() / (momentum.abs() + 5.0)).clamp(0.0, 1.0);
        let reversal_risk = 1.0 - continuation_prob;

        Ok(TrendMomentum {
            metric_name: "trend_metric".to_string(),
            current_value,
            momentum_score: momentum,
            acceleration,
            trend_strength,
            continuation_probability: continuation_prob,
            reversal_risk,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_snapshot_creation() {
        let snapshot = TemporalSnapshot {
            snapshot_date: "2026-07-15".to_string(),
            days_ago: 0,
            total_customers: 10000,
            segments: vec![],
            metadata: SnapshotMetadata {
                data_quality: 0.99,
                completeness: 0.98,
                interpolation_used: false,
                source: "database".to_string(),
            },
        };
        assert_eq!(snapshot.total_customers, 10000);
    }

    #[test]
    fn test_segment_size_forecasting() {
        let historical = vec![(0, 1000), (-7, 950), (-14, 900)];
        let forecast = SegmentSizeForecaster::forecast_sizes(&historical, 4).unwrap();
        assert_eq!(forecast.forecast_days, 4);
        assert!(!forecast.predictions.is_empty());
    }

    #[test]
    fn test_composition_forecasting() {
        let historical = vec![(0.5, 0.2, 0.3), (0.52, 0.18, 0.3), (0.54, 0.16, 0.3)];
        let forecast =
            CompositionForecaster::forecast_composition(0.54, 0.16, 0.3, &historical, 3).unwrap();
        assert_eq!(forecast.forecast_periods, 3);
    }

    #[test]
    fn test_what_if_simulator() {
        let mut params = HashMap::new();
        params.insert("rfm_threshold".to_string(), 0.1);

        let baseline = vec![
            ("Segment_1".to_string(), 1000),
            ("Segment_2".to_string(), 500),
        ];

        let scenario =
            WhatIfSimulator::simulate_parameter_change("Test Scenario", &baseline, &params)
                .unwrap();
        assert_eq!(scenario.scenario_name, "Test Scenario");
    }

    #[test]
    fn test_churn_forecasting() {
        let historical_churn = vec![0.05, 0.06, 0.07];
        let forecast =
            ChurnForecaster::forecast_churn(0.07, &historical_churn, 5000, 500.0, 6).unwrap();
        assert_eq!(forecast.projected_churn_rates.len(), 6);
    }

    #[test]
    fn test_momentum_analysis() {
        let history = vec![100.0, 105.0, 110.0, 115.0, 120.0];
        let momentum = MomentumAnalyzer::analyze_momentum(&history, 3).unwrap();
        assert!(momentum.momentum_score > 0.0);
    }

    #[test]
    fn test_sensitivity_analysis() {
        let result = SensitivityAnalyzer::analyze_parameter_sensitivity(
            "rfm_threshold",
            0.5,
            0.6,
            1000,
            1100,
            100000.0,
            105000.0,
        )
        .unwrap();
        assert_eq!(result.parameter, "rfm_threshold");
        assert!(result.sensitivity_coefficient > 0.0);
    }

    #[test]
    fn test_expansion_planning() {
        let plan = ExpansionPlanner::plan_expansion(10000, 20000, 4, 5).unwrap();
        assert_eq!(plan.timeline_quarters, 4);
        assert_eq!(plan.target_customer_count, 20000);
    }

    #[test]
    fn test_lifecycle_forecasting() {
        let mut transitions = HashMap::new();
        transitions.insert(("prospect".to_string(), "onboarding".to_string()), 0.8);
        transitions.insert(("onboarding".to_string(), "growth".to_string()), 0.7);

        let forecast =
            LifecycleForecaster::forecast_transitions("prospect", &transitions, 3).unwrap();
        assert_eq!(forecast.forecast_periods, 3);
    }

    #[test]
    fn test_historical_reconstruction() {
        let events = vec![
            (
                "2026-07-01".to_string(),
                "added".to_string(),
                "user1".to_string(),
            ),
            (
                "2026-07-02".to_string(),
                "added".to_string(),
                "user2".to_string(),
            ),
            (
                "2026-07-03".to_string(),
                "removed".to_string(),
                "user1".to_string(),
            ),
        ];

        let state =
            HistoricalReconstruction::reconstruct_at_date("seg1", "2026-07-03", &events).unwrap();
        assert_eq!(state.members.len(), 1);
    }

    #[test]
    fn test_membership_forecasting() {
        let history = vec![
            ("2026-07-01".to_string(), "segment_1".to_string()),
            ("2026-07-08".to_string(), "segment_2".to_string()),
        ];

        let transitions = HashMap::new();
        let forecast = MembershipForecaster::forecast_member_movement(
            "user123",
            "segment_2",
            &history,
            &transitions,
        )
        .unwrap();
        assert_eq!(forecast.member_id, "user123");
    }

    #[test]
    fn test_scenario_comparison() {
        let s1 = WhatIfScenario {
            scenario_name: "Scenario_1".to_string(),
            description: "Test 1".to_string(),
            parameters_changed: HashMap::new(),
            projected_segment_sizes: vec![],
            projected_revenue_impact: 10.0,
            projected_churn_impact: -5.0,
            feasibility_score: 0.8,
        };

        let s2 = WhatIfScenario {
            scenario_name: "Scenario_2".to_string(),
            description: "Test 2".to_string(),
            parameters_changed: HashMap::new(),
            projected_segment_sizes: vec![],
            projected_revenue_impact: 15.0,
            projected_churn_impact: -8.0,
            feasibility_score: 0.7,
        };

        let comparison = ScenarioAnalyzer::compare_scenarios(&[s1, s2]).unwrap();
        assert_eq!(comparison.scenarios.len(), 2);
    }

    #[test]
    fn test_tornado_analysis() {
        let mut ranges = HashMap::new();
        ranges.insert("param1".to_string(), (0.5, 1.5));
        ranges.insert("param2".to_string(), (0.0, 2.0));

        let baseline = HashMap::new();
        let impacts = SensitivityAnalyzer::tornado_analysis(&baseline, &ranges).unwrap();
        assert_eq!(impacts.len(), 2);
    }
}
