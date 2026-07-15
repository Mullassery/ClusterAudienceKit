//! Price intelligence engine: elasticity, tier migration, pricing strategies, optimization

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 1. PriceElasticity - Measure price sensitivity
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticityAnalysis {
    pub segment_id: String,
    pub price_elasticity: f64,
    pub elasticity_classification: String,
    pub revenue_sensitivity: f64,
    pub demand_curve: Vec<(f64, f64)>,
    pub optimal_price: f64,
}

pub struct ElasticityCalculator;

impl ElasticityCalculator {
    pub fn analyze_elasticity(
        segment_id: &str,
        price_changes: &[(f64, f64)],
    ) -> Result<ElasticityAnalysis> {
        if price_changes.len() < 2 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Need at least 2 price points".to_string(),
            ));
        }

        let mut total_elasticity = 0.0;
        let mut demand_curve = Vec::new();

        for i in 1..price_changes.len() {
            let price_change =
                (price_changes[i].0 - price_changes[i - 1].0) / price_changes[i - 1].0;
            let quantity_change =
                (price_changes[i].1 - price_changes[i - 1].1) / price_changes[i - 1].1;

            let elasticity = if price_change.abs() > 1e-10 {
                quantity_change / price_change
            } else {
                0.0
            };

            total_elasticity += elasticity.abs();
            demand_curve.push((price_changes[i].0, price_changes[i].1));
        }

        let avg_elasticity = total_elasticity / (price_changes.len() - 1) as f64;

        let classification = if avg_elasticity > 1.5 {
            "highly_elastic".to_string()
        } else if avg_elasticity > 1.0 {
            "elastic".to_string()
        } else if avg_elasticity > 0.5 {
            "inelastic".to_string()
        } else {
            "highly_inelastic".to_string()
        };

        let optimal_price = price_changes
            .iter()
            .max_by(|a, b| (a.0 * a.1).partial_cmp(&(b.0 * b.1)).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(p, _)| *p)
            .unwrap_or(0.0);

        Ok(ElasticityAnalysis {
            segment_id: segment_id.to_string(),
            price_elasticity: avg_elasticity,
            elasticity_classification: classification,
            revenue_sensitivity: avg_elasticity * 0.5,
            demand_curve,
            optimal_price,
        })
    }
}

// ============================================================================
// 2. TierMigration - Predict customer movement across pricing tiers
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierMigrationForecast {
    pub segment_id: String,
    pub current_tier: String,
    pub tier_migration_probabilities: HashMap<String, f64>,
    pub most_likely_tier: String,
    pub migration_timeline_months: u32,
    pub expansion_likelihood: f64,
    pub churn_risk: f64,
}

pub struct TierMigrationPredictor;

impl TierMigrationPredictor {
    pub fn predict_migration(
        segment_id: &str,
        current_tier: &str,
        current_usage: f64,
        historical_usage: &[f64],
    ) -> Result<TierMigrationForecast> {
        let avg_historical = historical_usage.iter().sum::<f64>() / historical_usage.len() as f64;
        let growth_rate = if avg_historical > 0.0 {
            (current_usage - avg_historical) / avg_historical
        } else {
            0.0
        };

        let mut probabilities = HashMap::new();
        probabilities.insert("stay".to_string(), (1.0 - growth_rate.abs()).clamp(0.0, 1.0));
        probabilities.insert("upgrade".to_string(), growth_rate.max(0.0));
        probabilities.insert("downgrade".to_string(), (-growth_rate).max(0.0));

        let most_likely = if growth_rate > 0.1 {
            "upgrade".to_string()
        } else if growth_rate < -0.1 {
            "downgrade".to_string()
        } else {
            "stay".to_string()
        };

        let expansion = growth_rate.max(0.0).min(1.0);
        let churn = if growth_rate < -0.3 { 0.8 } else { 0.2 };

        Ok(TierMigrationForecast {
            segment_id: segment_id.to_string(),
            current_tier: current_tier.to_string(),
            tier_migration_probabilities: probabilities,
            most_likely_tier: most_likely,
            migration_timeline_months: 3,
            expansion_likelihood: expansion,
            churn_risk: churn,
        })
    }
}

// ============================================================================
// 3. CategoryAffinity - Cross-category purchasing patterns
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAffinity {
    pub segment_id: String,
    pub category_pairs: Vec<(String, String, f64)>,
    pub affinity_scores: HashMap<String, f64>,
    pub recommended_bundles: Vec<String>,
    pub cross_sell_opportunities: f64,
}

pub struct AffiniCalculator;

impl AffiniCalculator {
    pub fn analyze_category_affinity(
        segment_id: &str,
        category_purchases: &[(String, f64)],
    ) -> Result<CategoryAffinity> {
        let mut affinity_scores = HashMap::new();
        let total_purchases: f64 = category_purchases.iter().map(|(_, p)| p).sum();

        for (category, purchases) in category_purchases {
            let score = if total_purchases > 0.0 {
                purchases / total_purchases
            } else {
                0.0
            };
            affinity_scores.insert(category.clone(), score);
        }

        let mut pairs = Vec::new();
        for i in 0..category_purchases.len() {
            for j in (i + 1)..category_purchases.len() {
                let affinity = (category_purchases[i].1 * category_purchases[j].1).sqrt();
                pairs.push((
                    category_purchases[i].0.clone(),
                    category_purchases[j].0.clone(),
                    affinity,
                ));
            }
        }

        pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        let bundles: Vec<String> = pairs
            .iter()
            .take(3)
            .map(|(a, b, _)| format!("{}-{}", a, b))
            .collect();

        let cross_sell = affinity_scores.values().map(|s| s.max(0.5)).sum::<f64>()
            / affinity_scores.len().max(1) as f64;

        Ok(CategoryAffinity {
            segment_id: segment_id.to_string(),
            category_pairs: pairs,
            affinity_scores,
            recommended_bundles: bundles,
            cross_sell_opportunities: cross_sell.min(1.0),
        })
    }
}

// ============================================================================
// 4. PriceSensitivity - Measure customer price sensitivity
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSensitivity {
    pub segment_id: String,
    pub price_sensitivity_score: f64,
    pub sensitivity_classification: String,
    pub discount_response_rate: f64,
    pub price_point_preference: String,
    pub willingness_to_pay: f64,
}

pub struct SensitivityAnalyzer;

impl SensitivityAnalyzer {
    pub fn analyze_price_sensitivity(
        segment_id: &str,
        discount_history: &[(f64, f64)],
    ) -> Result<PriceSensitivity> {
        if discount_history.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No discount history".to_string(),
            ));
        }

        let avg_discount = discount_history.iter().map(|(d, _)| d).sum::<f64>()
            / discount_history.len() as f64;
        let avg_response = discount_history.iter().map(|(_, r)| r).sum::<f64>()
            / discount_history.len() as f64;

        let sensitivity = if avg_discount > 0.0 {
            avg_response / avg_discount
        } else {
            0.0
        };

        let classification = if sensitivity > 2.0 {
            "highly_sensitive".to_string()
        } else if sensitivity > 1.0 {
            "sensitive".to_string()
        } else if sensitivity > 0.5 {
            "moderate".to_string()
        } else {
            "insensitive".to_string()
        };

        let willingness = (1.0 - sensitivity / 3.0).clamp(0.0, 1.0);

        Ok(PriceSensitivity {
            segment_id: segment_id.to_string(),
            price_sensitivity_score: sensitivity,
            sensitivity_classification: classification,
            discount_response_rate: avg_response,
            price_point_preference: "mid-range".to_string(),
            willingness_to_pay: willingness,
        })
    }
}

// ============================================================================
// 5. DiscountOptimization - Find optimal discount strategies
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountStrategy {
    pub segment_id: String,
    pub optimal_discount: f64,
    pub expected_conversion_lift: f64,
    pub revenue_impact: f64,
    pub customer_acquisition_benefit: f64,
    pub recommendation: String,
}

pub struct DiscountOptimizer;

impl DiscountOptimizer {
    pub fn optimize_discount(
        segment_id: &str,
        baseline_conversion: f64,
        price_sensitivity: f64,
        customer_ltv: f64,
    ) -> Result<DiscountStrategy> {
        let optimal_discount = (0.15 * price_sensitivity).clamp(0.0, 0.5);
        let conversion_lift = optimal_discount * 2.0 * price_sensitivity;
        let revenue_impact = conversion_lift * customer_ltv * (1.0 - optimal_discount);
        let acq_benefit = conversion_lift * 0.3;

        let recommendation = if revenue_impact > 0.1 {
            format!(
                "Apply {:.1}% discount to maximize revenue",
                optimal_discount * 100.0
            )
        } else {
            "Maintain current pricing".to_string()
        };

        Ok(DiscountStrategy {
            segment_id: segment_id.to_string(),
            optimal_discount,
            expected_conversion_lift: conversion_lift,
            revenue_impact,
            customer_acquisition_benefit: acq_benefit,
            recommendation,
        })
    }
}

// ============================================================================
// 6. RevenueMaximization - Identify highest revenue pricing strategy
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueOptimization {
    pub segment_id: String,
    pub current_price: f64,
    pub optimized_price: f64,
    pub price_adjustment: f64,
    pub projected_revenue_increase: f64,
    pub customer_impact: String,
}

pub struct RevenueMaximizer;

impl RevenueMaximizer {
    pub fn maximize_revenue(
        segment_id: &str,
        current_price: f64,
        elasticity: f64,
        current_volume: f64,
    ) -> Result<RevenueOptimization> {
        let optimal_price = if elasticity > 0.0 {
            current_price * (1.0 + (1.0 / (elasticity + 1.0)) * 0.1)
        } else {
            current_price * 1.1
        };

        let price_adjustment = ((optimal_price - current_price) / current_price * 100.0).clamp(-20.0, 30.0);
        let volume_change = price_adjustment / elasticity / 100.0;
        let new_volume = current_volume * (1.0 + volume_change);
        let revenue_increase = ((optimal_price * new_volume) - (current_price * current_volume))
            / (current_price * current_volume);

        let impact = if volume_change < -0.1 {
            "Volume decline risk".to_string()
        } else if volume_change > 0.1 {
            "Volume growth opportunity".to_string()
        } else {
            "Minimal volume impact".to_string()
        };

        Ok(RevenueOptimization {
            segment_id: segment_id.to_string(),
            current_price,
            optimized_price: optimal_price.max(0.0),
            price_adjustment,
            projected_revenue_increase: (revenue_increase * 100.0).clamp(-30.0, 50.0),
            customer_impact: impact,
        })
    }
}

// ============================================================================
// 7. CompetitivePricing - Benchmark against competitors
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitivePricingAnalysis {
    pub segment_id: String,
    pub your_price: f64,
    pub competitor_avg_price: f64,
    pub price_competitiveness: String,
    pub price_gap: f64,
    pub market_position: String,
}

pub struct CompetitiveAnalyzer;

impl CompetitiveAnalyzer {
    pub fn analyze_competitive_pricing(
        segment_id: &str,
        your_price: f64,
        competitor_prices: &[f64],
    ) -> Result<CompetitivePricingAnalysis> {
        if competitor_prices.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No competitor data".to_string(),
            ));
        }

        let avg_competitor = competitor_prices.iter().sum::<f64>() / competitor_prices.len() as f64;
        let gap = ((your_price - avg_competitor) / avg_competitor * 100.0).clamp(-50.0, 100.0);

        let competitiveness = if gap < -20.0 {
            "very_competitive".to_string()
        } else if gap < 0.0 {
            "competitive".to_string()
        } else if gap < 20.0 {
            "in_line".to_string()
        } else {
            "premium".to_string()
        };

        let position = if your_price < competitor_prices.iter().cloned().fold(f64::INFINITY, f64::min) {
            "market_leader".to_string()
        } else if your_price > competitor_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max) {
            "premium_tier".to_string()
        } else {
            "middle_market".to_string()
        };

        Ok(CompetitivePricingAnalysis {
            segment_id: segment_id.to_string(),
            your_price,
            competitor_avg_price: avg_competitor,
            price_competitiveness: competitiveness,
            price_gap: gap,
            market_position: position,
        })
    }
}

// ============================================================================
// 8. PriceThresholdDetection - Identify price breaking points
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceThreshold {
    pub segment_id: String,
    pub acceptable_threshold: f64,
    pub breakeven_threshold: f64,
    pub premium_threshold: f64,
    pub conversion_drop_point: f64,
    pub threshold_confidence: f64,
}

pub struct ThresholdDetector;

impl ThresholdDetector {
    pub fn detect_thresholds(segment_id: &str, price_demand: &[(f64, f64)]) -> Result<PriceThreshold> {
        if price_demand.len() < 3 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Need at least 3 data points".to_string(),
            ));
        }

        let avg_price = price_demand.iter().map(|(p, _)| p).sum::<f64>() / price_demand.len() as f64;
        let avg_demand = price_demand.iter().map(|(_, d)| d).sum::<f64>() / price_demand.len() as f64;

        let acceptable = avg_price * 0.8;
        let breakeven = avg_price;
        let premium = avg_price * 1.5;
        let drop_point = avg_price * 1.3;

        Ok(PriceThreshold {
            segment_id: segment_id.to_string(),
            acceptable_threshold: acceptable,
            breakeven_threshold: breakeven,
            premium_threshold: premium,
            conversion_drop_point: drop_point,
            threshold_confidence: 0.75,
        })
    }
}

// ============================================================================
// 9. DemandForecasting - Forecast demand at different price points
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandForecast {
    pub segment_id: String,
    pub forecast_periods: u32,
    pub price_points: Vec<f64>,
    pub demand_predictions: Vec<Vec<f64>>,
    pub demand_elasticity: f64,
    pub forecast_confidence: f64,
}

pub struct DemandForecaster;

impl DemandForecaster {
    pub fn forecast_demand(
        segment_id: &str,
        current_demand: f64,
        historical_demand: &[f64],
        elasticity: f64,
    ) -> Result<DemandForecast> {
        let price_points = vec![0.8, 0.9, 1.0, 1.1, 1.2];
        let mut demand_predictions = Vec::new();

        for _ in 0..4 {
            let mut period_demand = Vec::new();
            for price_mult in &price_points {
                let demand = current_demand * (1.0 + (1.0 - price_mult) * elasticity * 0.5);
                period_demand.push(demand.max(0.0));
            }
            demand_predictions.push(period_demand);
        }

        let confidence = if historical_demand.len() > 5 { 0.85 } else { 0.65 };

        Ok(DemandForecast {
            segment_id: segment_id.to_string(),
            forecast_periods: 4,
            price_points,
            demand_predictions,
            demand_elasticity: elasticity,
            forecast_confidence: confidence,
        })
    }
}

// ============================================================================
// 10. MarginOptimization - Maximize profit margins
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginOptimization {
    pub segment_id: String,
    pub current_margin_percentage: f64,
    pub optimized_margin_percentage: f64,
    pub cost_per_unit: f64,
    pub optimal_price: f64,
    pub profit_improvement: f64,
}

pub struct MarginOptimizer;

impl MarginOptimizer {
    pub fn optimize_margin(
        segment_id: &str,
        current_price: f64,
        cost_per_unit: f64,
        volume: usize,
    ) -> Result<MarginOptimization> {
        let current_margin = ((current_price - cost_per_unit) / current_price * 100.0).max(0.0);
        let optimal_price = cost_per_unit * 2.2;
        let optimal_margin = ((optimal_price - cost_per_unit) / optimal_price * 100.0).max(0.0);
        let profit_improvement = optimal_margin - current_margin;

        Ok(MarginOptimization {
            segment_id: segment_id.to_string(),
            current_margin_percentage: current_margin,
            optimized_margin_percentage: optimal_margin,
            cost_per_unit,
            optimal_price: optimal_price.max(current_price),
            profit_improvement,
        })
    }
}

// ============================================================================
// 11. BundleRecommendation - Recommend product bundles
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleRecommendation {
    pub segment_id: String,
    pub bundle_name: String,
    pub products: Vec<String>,
    pub individual_total_price: f64,
    pub bundle_price: f64,
    pub bundle_discount: f64,
    pub expected_lift: f64,
}

pub struct BundleRecommender;

impl BundleRecommender {
    pub fn recommend_bundle(
        segment_id: &str,
        bundle_products: &[(String, f64)],
    ) -> Result<BundleRecommendation> {
        let individual_total: f64 = bundle_products.iter().map(|(_, p)| p).sum();
        let bundle_price = individual_total * 0.85;
        let discount = ((individual_total - bundle_price) / individual_total * 100.0).clamp(0.0, 30.0);

        let products: Vec<String> = bundle_products.iter().map(|(p, _)| p.clone()).collect();

        Ok(BundleRecommendation {
            segment_id: segment_id.to_string(),
            bundle_name: format!("Bundle-{}", products.len()),
            products,
            individual_total_price: individual_total,
            bundle_price,
            bundle_discount: discount,
            expected_lift: 0.25,
        })
    }
}

// ============================================================================
// 12. PriceCeilingFloor - Define acceptable price ranges
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub segment_id: String,
    pub minimum_acceptable_price: f64,
    pub maximum_acceptable_price: f64,
    pub recommended_price: f64,
    pub price_range_width: f64,
    pub flexibility_score: f64,
}

pub struct PriceRangeAnalyzer;

impl PriceRangeAnalyzer {
    pub fn analyze_price_range(
        segment_id: &str,
        market_price: f64,
        cost_per_unit: f64,
        price_sensitivity: f64,
    ) -> Result<PriceRange> {
        let min_price = cost_per_unit * 1.2;
        let max_price = market_price * (1.0 + price_sensitivity * 0.3);
        let recommended = (min_price + max_price) / 2.0;
        let width = max_price - min_price;
        let flexibility = (width / market_price).clamp(0.0, 1.0);

        Ok(PriceRange {
            segment_id: segment_id.to_string(),
            minimum_acceptable_price: min_price,
            maximum_acceptable_price: max_price,
            recommended_price: recommended,
            price_range_width: width,
            flexibility_score: flexibility,
        })
    }
}

// ============================================================================
// 13. ChurnByPricePoint - Analyze churn at different price levels
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePointChurnAnalysis {
    pub segment_id: String,
    pub price_tiers: Vec<(String, f64, f64)>,
    pub highest_churn_tier: String,
    pub lowest_churn_tier: String,
    pub price_churn_correlation: f64,
}

pub struct ChurnAnalyzer;

impl ChurnAnalyzer {
    pub fn analyze_churn_by_price(
        segment_id: &str,
        tier_churn_data: &[(String, f64, f64)],
    ) -> Result<PricePointChurnAnalysis> {
        if tier_churn_data.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No tier data".to_string(),
            ));
        }

        let highest = tier_churn_data
            .iter()
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(n, _, _)| n.clone())
            .unwrap_or_default();

        let lowest = tier_churn_data
            .iter()
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(n, _, _)| n.clone())
            .unwrap_or_default();

        let correlation = if tier_churn_data.len() > 1 {
            0.65
        } else {
            0.0
        };

        Ok(PricePointChurnAnalysis {
            segment_id: segment_id.to_string(),
            price_tiers: tier_churn_data.to_vec(),
            highest_churn_tier: highest,
            lowest_churn_tier: lowest,
            price_churn_correlation: correlation,
        })
    }
}

// ============================================================================
// 14. CustomerValueByTier - Segment value analysis by pricing tier
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierValueAnalysis {
    pub segment_id: String,
    pub tier_values: Vec<(String, f64, f64)>,
    pub highest_value_tier: String,
    pub tier_concentration: f64,
    pub growth_tier: String,
}

pub struct TierValueAnalyzer;

impl TierValueAnalyzer {
    pub fn analyze_tier_value(
        segment_id: &str,
        tier_data: &[(String, f64, f64)],
    ) -> Result<TierValueAnalysis> {
        if tier_data.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No tier data".to_string(),
            ));
        }

        let total_value: f64 = tier_data.iter().map(|(_, v, _)| v).sum();
        let highest = tier_data
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(n, _, _)| n.clone())
            .unwrap_or_default();

        let concentration = if total_value > 0.0 {
            let top_value = tier_data
                .iter()
                .map(|(_, v, _)| v)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(&0.0);
            top_value / total_value
        } else {
            0.0
        };

        let growth_tier = tier_data
            .iter()
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(n, _, _)| n.clone())
            .unwrap_or_default();

        Ok(TierValueAnalysis {
            segment_id: segment_id.to_string(),
            tier_values: tier_data.to_vec(),
            highest_value_tier: highest,
            tier_concentration: concentration,
            growth_tier,
        })
    }
}

// ============================================================================
// 15. PriceChangeImpactAnalysis - Forecast impact of price changes
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChangeImpact {
    pub segment_id: String,
    pub price_change_percentage: f64,
    pub projected_volume_change: f64,
    pub projected_revenue_change: f64,
    pub churn_risk_increase: f64,
    pub recommendation: String,
}

pub struct ImpactAnalyzer;

impl ImpactAnalyzer {
    pub fn analyze_price_change_impact(
        segment_id: &str,
        price_change: f64,
        elasticity: f64,
        current_revenue: f64,
    ) -> Result<PriceChangeImpact> {
        let volume_change = price_change * elasticity;
        let revenue_change = ((1.0 + price_change) * (1.0 + volume_change) - 1.0) * 100.0;
        let churn_increase = (-volume_change).max(0.0) * 50.0;

        let recommendation = if revenue_change > 5.0 {
            "Price increase justified by revenue growth".to_string()
        } else if revenue_change < -5.0 {
            "Price increase will damage revenue; not recommended".to_string()
        } else {
            "Minimal revenue impact; proceed with caution".to_string()
        };

        Ok(PriceChangeImpact {
            segment_id: segment_id.to_string(),
            price_change_percentage: price_change * 100.0,
            projected_volume_change: volume_change * 100.0,
            projected_revenue_change: revenue_change,
            churn_risk_increase: churn_increase,
            recommendation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_elasticity() {
        let prices = vec![(100.0, 1000.0), (110.0, 900.0)];
        let elasticity = ElasticityCalculator::analyze_elasticity("seg1", &prices).unwrap();
        assert!(elasticity.price_elasticity > 0.0);
    }

    #[test]
    fn test_tier_migration() {
        let history = vec![100.0, 110.0, 120.0];
        let migration = TierMigrationPredictor::predict_migration("seg1", "pro", 130.0, &history).unwrap();
        assert_eq!(migration.most_likely_tier, "upgrade");
    }

    #[test]
    fn test_category_affinity() {
        let purchases = vec![
            ("Electronics".to_string(), 5000.0),
            ("Software".to_string(), 3000.0),
        ];
        let affinity = AffiniCalculator::analyze_category_affinity("seg1", &purchases).unwrap();
        assert!(affinity.cross_sell_opportunities > 0.0);
    }

    #[test]
    fn test_price_sensitivity() {
        let discounts = vec![(0.1, 1.2), (0.2, 1.5)];
        let sensitivity = SensitivityAnalyzer::analyze_price_sensitivity("seg1", &discounts).unwrap();
        assert!(sensitivity.price_sensitivity_score > 0.0);
    }

    #[test]
    fn test_discount_optimization() {
        let strategy =
            DiscountOptimizer::optimize_discount("seg1", 0.3, 1.5, 5000.0).unwrap();
        assert!(strategy.optimal_discount >= 0.0);
    }

    #[test]
    fn test_revenue_maximization() {
        let optimization = RevenueMaximizer::maximize_revenue("seg1", 100.0, 1.5, 1000.0).unwrap();
        assert!(optimization.optimized_price > 0.0);
    }

    #[test]
    fn test_competitive_pricing() {
        let competitors = vec![95.0, 105.0, 110.0];
        let analysis = CompetitiveAnalyzer::analyze_competitive_pricing("seg1", 100.0, &competitors).unwrap();
        assert!(analysis.price_gap >= -50.0);
    }

    #[test]
    fn test_price_threshold() {
        let demand = vec![(100.0, 1000.0), (110.0, 900.0), (120.0, 750.0)];
        let threshold = ThresholdDetector::detect_thresholds("seg1", &demand).unwrap();
        assert!(threshold.acceptable_threshold > 0.0);
    }

    #[test]
    fn test_demand_forecasting() {
        let history = vec![100.0, 110.0, 120.0];
        let forecast = DemandForecaster::forecast_demand("seg1", 130.0, &history, 1.5).unwrap();
        assert_eq!(forecast.forecast_periods, 4);
    }

    #[test]
    fn test_margin_optimization() {
        let optimization = MarginOptimizer::optimize_margin("seg1", 100.0, 50.0, 1000).unwrap();
        assert!(optimization.optimized_margin_percentage > optimization.current_margin_percentage);
    }

    #[test]
    fn test_bundle_recommendation() {
        let products = vec![("Product A".to_string(), 50.0), ("Product B".to_string(), 75.0)];
        let bundle = BundleRecommender::recommend_bundle("seg1", &products).unwrap();
        assert!(bundle.bundle_discount > 0.0);
    }

    #[test]
    fn test_price_range() {
        let range = PriceRangeAnalyzer::analyze_price_range("seg1", 100.0, 50.0, 0.5).unwrap();
        assert!(range.minimum_acceptable_price < range.maximum_acceptable_price);
    }

    #[test]
    fn test_churn_by_price() {
        let tiers = vec![
            ("Basic".to_string(), 100.0, 0.05),
            ("Pro".to_string(), 200.0, 0.02),
        ];
        let analysis = ChurnAnalyzer::analyze_churn_by_price("seg1", &tiers).unwrap();
        assert_eq!(analysis.lowest_churn_tier, "Pro");
    }

    #[test]
    fn test_tier_value() {
        let tiers = vec![
            ("Basic".to_string(), 10000.0, 100.0),
            ("Pro".to_string(), 50000.0, 500.0),
        ];
        let analysis = TierValueAnalyzer::analyze_tier_value("seg1", &tiers).unwrap();
        assert_eq!(analysis.highest_value_tier, "Pro");
    }

    #[test]
    fn test_price_change_impact() {
        let impact = ImpactAnalyzer::analyze_price_change_impact("seg1", 0.1, 1.5, 100000.0).unwrap();
        assert!(impact.projected_volume_change.abs() > 0.0);
    }

    #[test]
    fn test_elasticity_classification_elastic() {
        let prices = vec![(100.0, 1000.0), (110.0, 800.0), (120.0, 600.0)];
        let elasticity = ElasticityCalculator::analyze_elasticity("seg1", &prices).unwrap();
        assert_eq!(elasticity.elasticity_classification, "highly_elastic");
    }

    #[test]
    fn test_competitive_market_leader() {
        let competitors = vec![150.0, 160.0, 170.0];
        let analysis = CompetitiveAnalyzer::analyze_competitive_pricing("seg1", 100.0, &competitors).unwrap();
        assert_eq!(analysis.market_position, "market_leader");
    }
}
