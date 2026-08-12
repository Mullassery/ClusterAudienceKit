//! Revenue intelligence engine: revenue metrics, ROI, attribution, alerts, forecasting

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 1. SegmentRevenue - Track revenue per segment
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentRevenue {
    pub segment_id: String,
    pub segment_name: String,
    pub total_revenue: f64,
    pub member_count: usize,
    pub revenue_per_member: f64,
    pub revenue_concentration: f64,
    pub top_customer_contribution: f64,
    pub currency: String,
}

pub struct SegmentRevenueCalculator;

impl SegmentRevenueCalculator {
    pub fn calculate_segment_revenue(
        segment_id: &str,
        segment_name: &str,
        member_revenues: &[f64],
    ) -> Result<SegmentRevenue> {
        if member_revenues.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No members in segment".to_string(),
            ));
        }

        let total_revenue: f64 = member_revenues.iter().sum();
        let member_count = member_revenues.len();
        let revenue_per_member = total_revenue / member_count as f64;

        let max_revenue = member_revenues
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let top_contribution = if total_revenue > 0.0 {
            max_revenue / total_revenue
        } else {
            0.0
        };

        let concentration = Self::calculate_concentration(member_revenues);

        Ok(SegmentRevenue {
            segment_id: segment_id.to_string(),
            segment_name: segment_name.to_string(),
            total_revenue,
            member_count,
            revenue_per_member,
            revenue_concentration: concentration,
            top_customer_contribution: top_contribution,
            currency: "USD".to_string(),
        })
    }

    fn calculate_concentration(revenues: &[f64]) -> f64 {
        let total: f64 = revenues.iter().sum();
        if total == 0.0 {
            return 0.0;
        }

        let sorted_revenues: Vec<_> = {
            let mut v = revenues.to_vec();
            v.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            v
        };

        let top_20_pct = (revenues.len() / 5).max(1);
        let top_20_revenue: f64 = sorted_revenues.iter().take(top_20_pct).sum();

        top_20_revenue / total
    }
}

// ============================================================================
// 2. SegmentROI - Calculate return on investment per segment
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentROI {
    pub segment_id: String,
    pub revenue: f64,
    pub investment: f64,
    pub roi_percentage: f64,
    pub payback_period_days: f64,
    pub profitability_index: f64,
    pub risk_level: String,
}

pub struct ROICalculator;

impl ROICalculator {
    pub fn calculate_roi(
        segment_id: &str,
        revenue: f64,
        investment: f64,
        _clv: f64,
    ) -> Result<SegmentROI> {
        let roi = if investment > 0.0 {
            ((revenue - investment) / investment) * 100.0
        } else {
            0.0
        };

        let payback_period = if revenue > 0.0 && investment > 0.0 {
            (investment / revenue) * 365.0
        } else {
            f64::INFINITY
        };

        let profitability_index = if investment > 0.0 {
            revenue / investment
        } else {
            0.0
        };

        let risk_level = if roi > 100.0 {
            "low".to_string()
        } else if roi > 20.0 {
            "medium".to_string()
        } else if roi > 0.0 {
            "medium-high".to_string()
        } else {
            "high".to_string()
        };

        Ok(SegmentROI {
            segment_id: segment_id.to_string(),
            revenue,
            investment,
            roi_percentage: roi,
            payback_period_days: payback_period.min(10000.0),
            profitability_index,
            risk_level,
        })
    }
}

// ============================================================================
// 3. RevenueAttribution - Track revenue sources and attribution
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueAttribution {
    pub segment_id: String,
    pub channel_attributions: HashMap<String, f64>,
    pub product_attributions: HashMap<String, f64>,
    pub cohort_attributions: HashMap<String, f64>,
    pub first_touch_revenue: f64,
    pub last_touch_revenue: f64,
    pub multi_touch_revenue: f64,
    pub attribution_model: String,
}

pub struct AttributionEngine;

impl AttributionEngine {
    pub fn calculate_attribution(
        segment_id: &str,
        channel_revenue: &[(String, f64)],
        product_revenue: &[(String, f64)],
        cohort_revenue: &[(String, f64)],
    ) -> Result<RevenueAttribution> {
        let mut channel_map = HashMap::new();
        for (channel, revenue) in channel_revenue {
            channel_map.insert(channel.clone(), *revenue);
        }

        let mut product_map = HashMap::new();
        for (product, revenue) in product_revenue {
            product_map.insert(product.clone(), *revenue);
        }

        let mut cohort_map = HashMap::new();
        for (cohort, revenue) in cohort_revenue {
            cohort_map.insert(cohort.clone(), *revenue);
        }

        let total_revenue: f64 = channel_revenue.iter().map(|(_, r)| r).sum();
        let first_touch = (total_revenue * 0.25).min(total_revenue);
        let last_touch = (total_revenue * 0.25).min(total_revenue);
        let multi_touch = (total_revenue * 0.50).max(0.0);

        Ok(RevenueAttribution {
            segment_id: segment_id.to_string(),
            channel_attributions: channel_map,
            product_attributions: product_map,
            cohort_attributions: cohort_map,
            first_touch_revenue: first_touch,
            last_touch_revenue: last_touch,
            multi_touch_revenue: multi_touch,
            attribution_model: "multi-touch".to_string(),
        })
    }
}

// ============================================================================
// 4. RevenueAlert - Real-time revenue anomaly detection
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueAlert {
    pub alert_id: String,
    pub segment_id: String,
    pub alert_type: String,
    pub current_value: f64,
    pub baseline_value: f64,
    pub variance_percentage: f64,
    pub severity: String,
    pub recommended_action: String,
}

pub struct RevenueAlerter;

impl RevenueAlerter {
    pub fn detect_revenue_anomaly(
        segment_id: &str,
        current_revenue: f64,
        baseline_revenue: f64,
        revenue_history: &[f64],
    ) -> Result<Option<RevenueAlert>> {
        let variance = if baseline_revenue > 0.0 {
            ((current_revenue - baseline_revenue) / baseline_revenue) * 100.0
        } else {
            0.0
        };

        let mean = revenue_history.iter().sum::<f64>() / revenue_history.len() as f64;
        let std_dev = Self::calculate_std_dev(revenue_history, mean);

        let z_score = if std_dev > 0.0 {
            (current_revenue - mean) / std_dev
        } else {
            0.0
        };

        if z_score.abs() > 2.0 || variance.abs() > 30.0 {
            let severity = if z_score.abs() > 3.0 || variance.abs() > 50.0 {
                "critical".to_string()
            } else if z_score.abs() > 2.5 || variance.abs() > 40.0 {
                "high".to_string()
            } else {
                "medium".to_string()
            };

            let alert_type = if current_revenue < baseline_revenue {
                "revenue_decline".to_string()
            } else {
                "revenue_spike".to_string()
            };

            let action = if alert_type == "revenue_decline" {
                "Investigate churn, pricing changes, or campaign underperformance".to_string()
            } else {
                "Capitalize on momentum; investigate sustainability".to_string()
            };

            Ok(Some(RevenueAlert {
                alert_id: format!("alert_{}", segment_id),
                segment_id: segment_id.to_string(),
                alert_type,
                current_value: current_revenue,
                baseline_value: baseline_revenue,
                variance_percentage: variance,
                severity,
                recommended_action: action,
            }))
        } else {
            Ok(None)
        }
    }

    fn calculate_std_dev(values: &[f64], mean: f64) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }
}

// ============================================================================
// 5. RevenueForecast - Predict future segment revenue
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueForecast {
    pub segment_id: String,
    pub forecast_periods: u32,
    pub predicted_revenue: Vec<f64>,
    pub confidence_intervals_lower: Vec<f64>,
    pub confidence_intervals_upper: Vec<f64>,
    pub growth_rate: f64,
    pub forecast_accuracy: f64,
}

pub struct RevenueForecaster;

impl RevenueForecaster {
    pub fn forecast_revenue(
        historical_revenue: &[f64],
        forecast_periods: u32,
    ) -> Result<RevenueForecast> {
        if historical_revenue.len() < 3 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Need at least 3 data points for revenue forecasting".to_string(),
            ));
        }

        let x: Vec<f64> = (0..historical_revenue.len()).map(|i| i as f64).collect();
        let y = historical_revenue;

        let (slope, intercept) = Self::linear_regression(&x, y)?;
        let r_squared = Self::calculate_r_squared(&x, y, slope, intercept)?;

        let mut predictions = Vec::new();
        let mut lower = Vec::new();
        let mut upper = Vec::new();

        let last_x = (historical_revenue.len() - 1) as f64;
        let std_error = Self::calculate_std_error(&x, y, slope, intercept)?;

        for i in 1..=forecast_periods {
            let future_x = last_x + i as f64;
            let pred = (slope * future_x + intercept).max(0.0);
            let ci = 1.96 * std_error;

            predictions.push(pred);
            lower.push((pred - ci).max(0.0));
            upper.push(pred + ci);
        }

        let growth_rate = if historical_revenue.len() > 1 {
            (historical_revenue.last().unwrap() - historical_revenue.first().unwrap())
                / historical_revenue.first().unwrap()
                * 100.0
        } else {
            0.0
        };

        Ok(RevenueForecast {
            segment_id: "revenue_forecast".to_string(),
            forecast_periods,
            predicted_revenue: predictions,
            confidence_intervals_lower: lower,
            confidence_intervals_upper: upper,
            growth_rate,
            forecast_accuracy: r_squared,
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
                "Cannot fit linear regression".to_string(),
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

    fn calculate_std_error(x: &[f64], y: &[f64], slope: f64, intercept: f64) -> Result<f64> {
        let n = y.len() as f64;
        let residuals: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| (yi - (slope * xi + intercept)).powi(2))
            .sum();

        let mse = residuals / (n - 2.0).max(1.0);
        Ok(mse.sqrt())
    }
}

// ============================================================================
// 6. CustomerAcquisitionCost - Track CAC per segment
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAcquisitionCost {
    pub segment_id: String,
    pub total_acquisition_spend: f64,
    pub customers_acquired: usize,
    pub cac_per_customer: f64,
    pub payback_period_months: f64,
    pub efficiency_score: f64,
}

pub struct CACCalculator;

impl CACCalculator {
    pub fn calculate_cac(
        segment_id: &str,
        acquisition_spend: f64,
        customers_acquired: usize,
        avg_monthly_revenue_per_customer: f64,
    ) -> Result<CustomerAcquisitionCost> {
        if customers_acquired == 0 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No customers acquired".to_string(),
            ));
        }

        let cac = acquisition_spend / customers_acquired as f64;
        let payback = if avg_monthly_revenue_per_customer > 0.0 {
            cac / avg_monthly_revenue_per_customer
        } else {
            f64::INFINITY
        };

        let efficiency = if payback < 12.0 && payback > 0.0 {
            (12.0 - payback) / 12.0
        } else if payback == 0.0 {
            1.0
        } else {
            0.0
        };

        Ok(CustomerAcquisitionCost {
            segment_id: segment_id.to_string(),
            total_acquisition_spend: acquisition_spend,
            customers_acquired,
            cac_per_customer: cac,
            payback_period_months: payback.min(1000.0),
            efficiency_score: efficiency.clamp(0.0, 1.0),
        })
    }
}

// ============================================================================
// 7. MarginAnalysis - Profit margin per segment
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginAnalysis {
    pub segment_id: String,
    pub revenue: f64,
    pub cost_of_goods_sold: f64,
    pub operating_costs: f64,
    pub gross_margin_percentage: f64,
    pub operating_margin_percentage: f64,
    pub net_margin_percentage: f64,
}

pub struct MarginAnalyzer;

impl MarginAnalyzer {
    pub fn analyze_margins(
        segment_id: &str,
        revenue: f64,
        cogs: f64,
        operating_costs: f64,
    ) -> Result<MarginAnalysis> {
        let gross_profit = revenue - cogs;
        let gross_margin = if revenue > 0.0 {
            (gross_profit / revenue) * 100.0
        } else {
            0.0
        };

        let operating_profit = gross_profit - operating_costs;
        let operating_margin = if revenue > 0.0 {
            (operating_profit / revenue) * 100.0
        } else {
            0.0
        };

        let net_profit = operating_profit;
        let net_margin = if revenue > 0.0 {
            (net_profit / revenue) * 100.0
        } else {
            0.0
        };

        Ok(MarginAnalysis {
            segment_id: segment_id.to_string(),
            revenue,
            cost_of_goods_sold: cogs,
            operating_costs,
            gross_margin_percentage: gross_margin,
            operating_margin_percentage: operating_margin,
            net_margin_percentage: net_margin.clamp(-100.0, 100.0),
        })
    }
}

// ============================================================================
// 8. UpsellOpportunities - Identify cross-sell/upsell potential
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsellOpportunity {
    pub segment_id: String,
    pub product_name: String,
    pub current_penetration: f64,
    pub market_size: usize,
    pub addressable_market: usize,
    pub opportunity_revenue: f64,
    pub priority_score: f64,
    pub recommended_action: String,
}

pub struct UpsellIdentifier;

impl UpsellIdentifier {
    pub fn identify_opportunities(
        segment_id: &str,
        product: &str,
        penetration_rate: f64,
        segment_size: usize,
        avg_order_value: f64,
    ) -> Result<UpsellOpportunity> {
        let addressable = (segment_size as f64 * (1.0 - penetration_rate)) as usize;
        let opportunity_revenue = addressable as f64 * avg_order_value;

        let priority = if penetration_rate < 0.2 {
            0.9
        } else if penetration_rate < 0.5 {
            0.7
        } else if penetration_rate < 0.8 {
            0.4
        } else {
            0.1
        };

        let action = if penetration_rate < 0.2 {
            "Aggressive expansion campaign".to_string()
        } else if penetration_rate < 0.5 {
            "Targeted marketing to non-adopters".to_string()
        } else if penetration_rate < 0.8 {
            "Retention and engagement focus".to_string()
        } else {
            "Monitor for saturation".to_string()
        };

        Ok(UpsellOpportunity {
            segment_id: segment_id.to_string(),
            product_name: product.to_string(),
            current_penetration: penetration_rate,
            market_size: segment_size,
            addressable_market: addressable,
            opportunity_revenue,
            priority_score: priority,
            recommended_action: action,
        })
    }
}

// ============================================================================
// 9. RevenueConcentrationRisk - Measure revenue dependency risk
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueConcentrationRisk {
    pub segment_id: String,
    pub herfindahl_index: f64,
    pub top_10_pct_contribution: f64,
    pub gini_coefficient: f64,
    pub risk_level: String,
    pub recommended_action: String,
}

pub struct ConcentrationAnalyzer;

impl ConcentrationAnalyzer {
    pub fn analyze_concentration(
        segment_id: &str,
        member_revenues: &[f64],
    ) -> Result<RevenueConcentrationRisk> {
        if member_revenues.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No revenue data".to_string(),
            ));
        }

        let total: f64 = member_revenues.iter().sum();
        if total == 0.0 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Total revenue is zero".to_string(),
            ));
        }

        let herfindahl = member_revenues
            .iter()
            .map(|r| (r / total).powi(2))
            .sum::<f64>();

        let mut sorted = member_revenues.to_vec();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let top_10_pct_count = (member_revenues.len() / 10).max(1);
        let top_10_revenue: f64 = sorted.iter().take(top_10_pct_count).sum();
        let top_10_contribution = top_10_revenue / total;

        let gini = Self::calculate_gini(&sorted);

        let risk_level = if herfindahl > 0.5 {
            "critical".to_string()
        } else if herfindahl > 0.25 {
            "high".to_string()
        } else if herfindahl > 0.21 {
            "medium".to_string()
        } else {
            "low".to_string()
        };

        let action = if herfindahl > 0.25 {
            "Diversify customer base; reduce dependency on top accounts".to_string()
        } else {
            "Monitor top accounts; maintain healthy mix".to_string()
        };

        Ok(RevenueConcentrationRisk {
            segment_id: segment_id.to_string(),
            herfindahl_index: herfindahl,
            top_10_pct_contribution: top_10_contribution,
            gini_coefficient: gini,
            risk_level,
            recommended_action: action,
        })
    }

    fn calculate_gini(sorted_revenues: &[f64]) -> f64 {
        if sorted_revenues.is_empty() {
            return 0.0;
        }

        let total: f64 = sorted_revenues.iter().sum();
        if total == 0.0 {
            return 0.0;
        }

        let n = sorted_revenues.len() as f64;
        let sum: f64 = sorted_revenues
            .iter()
            .enumerate()
            .map(|(i, r)| (2.0 * (i as f64 + 1.0) - n - 1.0) * r)
            .sum();

        (sum / (n * total)).max(0.0)
    }
}

// ============================================================================
// 10. RevenueEfficiency - Measure revenue generation efficiency
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueEfficiency {
    pub segment_id: String,
    pub revenue_per_marketing_spend: f64,
    pub revenue_per_employee_hour: f64,
    pub revenue_per_customer: f64,
    pub efficiency_score: f64,
    pub benchmark_comparison: String,
}

pub struct EfficiencyScorer;

impl EfficiencyScorer {
    pub fn calculate_efficiency(
        segment_id: &str,
        total_revenue: f64,
        marketing_spend: f64,
        employee_hours: f64,
        member_count: usize,
    ) -> Result<RevenueEfficiency> {
        let revenue_per_marketing = if marketing_spend > 0.0 {
            total_revenue / marketing_spend
        } else {
            0.0
        };

        let revenue_per_employee = if employee_hours > 0.0 {
            total_revenue / employee_hours
        } else {
            0.0
        };

        let revenue_per_member = if member_count > 0 {
            total_revenue / member_count as f64
        } else {
            0.0
        };

        let efficiency = (revenue_per_marketing + revenue_per_employee + revenue_per_member) / 3.0;
        let normalized_efficiency = (efficiency / 100.0).min(1.0);

        let comparison = if normalized_efficiency > 0.75 {
            "best-in-class".to_string()
        } else if normalized_efficiency > 0.5 {
            "above-average".to_string()
        } else if normalized_efficiency > 0.25 {
            "average".to_string()
        } else {
            "below-average".to_string()
        };

        Ok(RevenueEfficiency {
            segment_id: segment_id.to_string(),
            revenue_per_marketing_spend: revenue_per_marketing,
            revenue_per_employee_hour: revenue_per_employee,
            revenue_per_customer: revenue_per_member,
            efficiency_score: normalized_efficiency,
            benchmark_comparison: comparison,
        })
    }
}

// ============================================================================
// 11. RevenueTrend - Analyze revenue trends and momentum
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueTrend {
    pub segment_id: String,
    pub period_count: u32,
    pub trend_direction: String,
    pub growth_rate: f64,
    pub volatility: f64,
    pub trend_strength: f64,
    pub recommendation: String,
}

pub struct TrendAnalyzer;

impl TrendAnalyzer {
    pub fn analyze_trend(segment_id: &str, revenue_history: &[f64]) -> Result<RevenueTrend> {
        if revenue_history.len() < 2 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Need at least 2 data points".to_string(),
            ));
        }

        let growth = if revenue_history[0] != 0.0 {
            (revenue_history.last().unwrap() - revenue_history.first().unwrap())
                / revenue_history.first().unwrap()
                * 100.0
        } else {
            0.0
        };

        let mean = revenue_history.iter().sum::<f64>() / revenue_history.len() as f64;
        let variance = revenue_history
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>()
            / revenue_history.len() as f64;
        let std_dev = variance.sqrt();
        let volatility = if mean != 0.0 {
            (std_dev / mean) * 100.0
        } else {
            0.0
        };

        let x: Vec<f64> = (0..revenue_history.len()).map(|i| i as f64).collect();
        let slope = if revenue_history.len() > 1 {
            let x_mean = x.iter().sum::<f64>() / x.len() as f64;
            let y_mean = mean;
            let num: f64 = x
                .iter()
                .zip(revenue_history.iter())
                .map(|(xi, yi)| (xi - x_mean) * (yi - y_mean))
                .sum();
            let denom: f64 = x.iter().map(|xi| (xi - x_mean).powi(2)).sum();
            if denom != 0.0 {
                num / denom
            } else {
                0.0
            }
        } else {
            0.0
        };

        let direction = if slope > 0.01 {
            "up".to_string()
        } else if slope < -0.01 {
            "down".to_string()
        } else {
            "flat".to_string()
        };

        let trend_strength = (slope.abs() / (mean.abs() + 1.0)).min(1.0);

        let recommendation = match direction.as_str() {
            "up" => "Capitalize on momentum; invest in growth".to_string(),
            "down" => "Investigate decline; increase engagement".to_string(),
            _ => "Maintain current strategy; monitor closely".to_string(),
        };

        Ok(RevenueTrend {
            segment_id: segment_id.to_string(),
            period_count: revenue_history.len() as u32,
            trend_direction: direction,
            growth_rate: growth,
            volatility,
            trend_strength,
            recommendation,
        })
    }
}

// ============================================================================
// 12. CohortRevenueAnalysis - Track revenue by cohort
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortRevenue {
    pub cohort_id: String,
    pub cohort_age_months: u32,
    pub member_count: usize,
    pub total_revenue: f64,
    pub revenue_per_member: f64,
    pub cumulative_revenue: f64,
    pub retention_rate: f64,
}

pub struct CohortRevenueAnalyzer;

impl CohortRevenueAnalyzer {
    pub fn analyze_cohort_revenue(
        cohort_id: &str,
        cohort_age_months: u32,
        current_members: usize,
        total_revenue: f64,
        original_members: usize,
        cumulative_revenue: f64,
    ) -> Result<CohortRevenue> {
        let revenue_per_member = if current_members > 0 {
            total_revenue / current_members as f64
        } else {
            0.0
        };

        let retention = if original_members > 0 {
            current_members as f64 / original_members as f64
        } else {
            0.0
        };

        Ok(CohortRevenue {
            cohort_id: cohort_id.to_string(),
            cohort_age_months,
            member_count: current_members,
            total_revenue,
            revenue_per_member,
            cumulative_revenue,
            retention_rate: retention,
        })
    }
}

// ============================================================================
// 13. ProductMixAnalysis - Analyze product revenue composition
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMixAnalysis {
    pub segment_id: String,
    pub products: Vec<(String, f64, f64)>,
    pub herfindahl_index: f64,
    pub diversification_score: f64,
    pub top_product_concentration: f64,
}

pub struct ProductMixAnalyzer;

impl ProductMixAnalyzer {
    pub fn analyze_product_mix(
        segment_id: &str,
        product_revenue: &[(String, f64)],
    ) -> Result<ProductMixAnalysis> {
        let total: f64 = product_revenue.iter().map(|(_, r)| r).sum();
        if total == 0.0 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No product revenue".to_string(),
            ));
        }

        let mut products = Vec::new();
        let mut herfindahl = 0.0;

        for (product, revenue) in product_revenue {
            let share = revenue / total;
            products.push((product.clone(), *revenue, share));
            herfindahl += share.powi(2);
        }

        products.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_concentration = products.first().map(|p| p.2).unwrap_or(0.0);
        let diversification = 1.0 - herfindahl;

        Ok(ProductMixAnalysis {
            segment_id: segment_id.to_string(),
            products,
            herfindahl_index: herfindahl,
            diversification_score: diversification,
            top_product_concentration: top_concentration,
        })
    }
}

// ============================================================================
// 14. RevenueGrowthRate - Calculate segment growth metrics
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueGrowthRate {
    pub segment_id: String,
    pub period: String,
    pub previous_revenue: f64,
    pub current_revenue: f64,
    pub absolute_growth: f64,
    pub percentage_growth: f64,
    pub cagr_3year: f64,
    pub growth_classification: String,
}

pub struct GrowthRateCalculator;

impl GrowthRateCalculator {
    pub fn calculate_growth(
        segment_id: &str,
        previous_revenue: f64,
        current_revenue: f64,
        revenue_3year_ago: f64,
    ) -> Result<RevenueGrowthRate> {
        let absolute = current_revenue - previous_revenue;
        let percentage = if previous_revenue > 0.0 {
            (absolute / previous_revenue) * 100.0
        } else {
            0.0
        };

        let cagr = if revenue_3year_ago > 0.0 && current_revenue > 0.0 {
            ((current_revenue / revenue_3year_ago).powf(1.0 / 3.0) - 1.0) * 100.0
        } else {
            0.0
        };

        let classification = if percentage > 20.0 {
            "high-growth".to_string()
        } else if percentage > 5.0 {
            "moderate-growth".to_string()
        } else if percentage > -5.0 {
            "stable".to_string()
        } else if percentage > -20.0 {
            "slow-decline".to_string()
        } else {
            "rapid-decline".to_string()
        };

        Ok(RevenueGrowthRate {
            segment_id: segment_id.to_string(),
            period: "quarterly".to_string(),
            previous_revenue,
            current_revenue,
            absolute_growth: absolute,
            percentage_growth: percentage,
            cagr_3year: cagr,
            growth_classification: classification,
        })
    }
}

// ============================================================================
// 15. RevenueHealthScore - Comprehensive revenue health metric
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueHealthScore {
    pub segment_id: String,
    pub overall_score: f64,
    pub profitability_score: f64,
    pub growth_score: f64,
    pub efficiency_score: f64,
    pub concentration_risk_score: f64,
    pub health_status: String,
}

pub struct HealthScorer;

impl HealthScorer {
    pub fn calculate_health_score(
        segment_id: &str,
        roi_percentage: f64,
        revenue_growth: f64,
        efficiency: f64,
        concentration_risk: f64,
    ) -> Result<RevenueHealthScore> {
        let profitability = ((roi_percentage + 100.0) / 200.0).clamp(0.0, 1.0);
        let growth = ((revenue_growth + 50.0) / 100.0).clamp(0.0, 1.0);
        let eff = efficiency.clamp(0.0, 1.0);
        let concentration = (1.0 - concentration_risk).clamp(0.0, 1.0);

        let overall =
            (profitability * 0.35 + growth * 0.25 + eff * 0.25 + concentration * 0.15) * 100.0;

        let status = if overall >= 80.0 {
            "excellent".to_string()
        } else if overall >= 60.0 {
            "good".to_string()
        } else if overall >= 40.0 {
            "fair".to_string()
        } else {
            "poor".to_string()
        };

        Ok(RevenueHealthScore {
            segment_id: segment_id.to_string(),
            overall_score: overall,
            profitability_score: profitability * 100.0,
            growth_score: growth * 100.0,
            efficiency_score: eff * 100.0,
            concentration_risk_score: concentration * 100.0,
            health_status: status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_revenue_calculation() {
        let revenues = vec![1000.0, 2000.0, 1500.0, 3000.0];
        let result =
            SegmentRevenueCalculator::calculate_segment_revenue("seg1", "Segment 1", &revenues)
                .unwrap();
        assert_eq!(result.total_revenue, 7500.0);
        assert_eq!(result.revenue_per_member, 1875.0);
    }

    #[test]
    fn test_roi_calculation() {
        let roi = ROICalculator::calculate_roi("seg1", 10000.0, 5000.0, 15000.0).unwrap();
        assert!(roi.roi_percentage > 0.0);
        assert!(roi.profitability_index > 1.0);
    }

    #[test]
    fn test_revenue_attribution() {
        let channels = vec![("email".to_string(), 3000.0), ("web".to_string(), 4000.0)];
        let products = vec![("Pro".to_string(), 5000.0), ("Basic".to_string(), 2000.0)];
        let cohorts = vec![("2024-Q1".to_string(), 3500.0)];

        let attribution =
            AttributionEngine::calculate_attribution("seg1", &channels, &products, &cohorts)
                .unwrap();
        assert_eq!(attribution.channel_attributions.len(), 2);
    }

    #[test]
    fn test_revenue_alert_detection() {
        let history = vec![1000.0, 1050.0, 1100.0, 1150.0];
        let alert =
            RevenueAlerter::detect_revenue_anomaly("seg1", 2000.0, 1100.0, &history).unwrap();
        assert!(alert.is_some());
    }

    #[test]
    fn test_revenue_forecasting() {
        let history = vec![1000.0, 1100.0, 1200.0, 1300.0];
        let forecast = RevenueForecaster::forecast_revenue(&history, 4).unwrap();
        assert_eq!(forecast.predicted_revenue.len(), 4);
    }

    #[test]
    fn test_cac_calculation() {
        let cac = CACCalculator::calculate_cac("seg1", 10000.0, 100, 500.0).unwrap();
        assert_eq!(cac.cac_per_customer, 100.0);
    }

    #[test]
    fn test_margin_analysis() {
        let margins = MarginAnalyzer::analyze_margins("seg1", 10000.0, 6000.0, 2000.0).unwrap();
        assert!(margins.gross_margin_percentage > 0.0);
    }

    #[test]
    fn test_upsell_opportunities() {
        let opp =
            UpsellIdentifier::identify_opportunities("seg1", "Premium", 0.3, 1000, 500.0).unwrap();
        assert_eq!(opp.addressable_market, 700);
    }

    #[test]
    fn test_concentration_risk() {
        let revenues = vec![5000.0, 3000.0, 1500.0, 500.0];
        let risk = ConcentrationAnalyzer::analyze_concentration("seg1", &revenues).unwrap();
        assert!(risk.herfindahl_index > 0.0);
    }

    #[test]
    fn test_efficiency_scoring() {
        let eff =
            EfficiencyScorer::calculate_efficiency("seg1", 50000.0, 10000.0, 1000.0, 100).unwrap();
        assert!(eff.efficiency_score >= 0.0);
    }

    #[test]
    fn test_trend_analysis() {
        let history = vec![1000.0, 1100.0, 1250.0, 1450.0];
        let trend = TrendAnalyzer::analyze_trend("seg1", &history).unwrap();
        assert_eq!(trend.trend_direction, "up");
    }

    #[test]
    fn test_cohort_revenue() {
        let cohort =
            CohortRevenueAnalyzer::analyze_cohort_revenue("2024-Q1", 6, 950, 5000.0, 1000, 15000.0)
                .unwrap();
        assert!(cohort.retention_rate < 1.0);
    }

    #[test]
    fn test_product_mix_analysis() {
        let products = vec![("Pro".to_string(), 6000.0), ("Basic".to_string(), 4000.0)];
        let mix = ProductMixAnalyzer::analyze_product_mix("seg1", &products).unwrap();
        assert!(mix.diversification_score > 0.0);
    }

    #[test]
    fn test_growth_rate_calculation() {
        let growth =
            GrowthRateCalculator::calculate_growth("seg1", 10000.0, 12000.0, 9000.0).unwrap();
        assert_eq!(growth.percentage_growth, 20.0);
    }

    #[test]
    fn test_health_score() {
        let health = HealthScorer::calculate_health_score("seg1", 50.0, 15.0, 0.75, 0.3).unwrap();
        assert!(health.overall_score > 0.0);
    }

    #[test]
    fn test_revenue_concentration_low() {
        let revenues = vec![1000.0, 1000.0, 1000.0, 1000.0, 1000.0];
        let risk = ConcentrationAnalyzer::analyze_concentration("seg1", &revenues).unwrap();
        assert_eq!(risk.risk_level, "low");
    }

    #[test]
    fn test_margin_calculation_breakdown() {
        let margins = MarginAnalyzer::analyze_margins("seg1", 20000.0, 12000.0, 5000.0).unwrap();
        assert!(margins.gross_margin_percentage > margins.operating_margin_percentage);
    }
}
