//! Pattern Discovery: Emerging audiences, trends, intents, causality

use crate::Result;
use std::collections::HashMap;

/// Emerging audience detection (Feature 11)
#[derive(Clone, Debug)]
pub struct EmergingAudience {
    pub segment_id: usize,
    pub emergence_score: f64,    // 0-1: how new/emerging
    pub growth_rate: f64,        // % growth per period
    pub periods_since_emergence: usize,
    pub current_size: usize,
    pub projected_size_next_period: usize,
    pub emergence_status: String,  // "emerging", "accelerating", "plateauing"
}

impl EmergingAudience {
    pub fn new(
        segment_id: usize,
        current_size: usize,
        previous_size: usize,
        historical_sizes: &[usize],
    ) -> Self {
        let growth_rate = if previous_size > 0 {
            (current_size as f64 - previous_size as f64) / previous_size as f64
        } else {
            if current_size > 0 { 1.0 } else { 0.0 }
        };

        // Emergence score: high growth + young age
        let age_factor = (historical_sizes.len() as f64 / 12.0).min(1.0); // Assume max 12 periods
        let growth_factor = growth_rate.min(1.0).max(0.0);
        let emergence_score = (growth_factor + (1.0 - age_factor)) / 2.0;

        // Determine emergence status
        let emergence_status = if growth_rate > 0.2 {
            "accelerating".to_string()
        } else if growth_rate > 0.05 {
            "emerging".to_string()
        } else {
            "plateauing".to_string()
        };

        let projected_size = (current_size as f64 * (1.0 + growth_rate)) as usize;

        Self {
            segment_id,
            emergence_score,
            growth_rate,
            periods_since_emergence: historical_sizes.len(),
            current_size,
            projected_size_next_period: projected_size,
            emergence_status,
        }
    }
}

/// Trend-based discovery (Feature 15)
#[derive(Clone, Debug)]
pub struct SegmentTrend {
    pub segment_id: usize,
    pub trend_direction: String,  // "increasing", "decreasing", "stable"
    pub trend_strength: f64,      // 0-1: magnitude of trend
    pub moving_average: f64,      // Smoothed metric
    pub volatility: f64,          // Standard deviation of changes
    pub forecast_accuracy: f64,   // Confidence in trend
}

impl SegmentTrend {
    pub fn from_time_series(segment_id: usize, values: &[f64]) -> Result<Self> {
        if values.len() < 2 {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Need at least 2 data points for trend".to_string(),
            ));
        }

        // Calculate moving average
        let window_size = (values.len() / 3).max(1).min(5);
        let moving_avg = if values.len() >= window_size {
            let sum: f64 = values[values.len() - window_size..].iter().sum();
            sum / window_size as f64
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        };

        // Calculate trend direction via linear regression
        let n = values.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = (i + 1) as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let slope = if (n * sum_x2 - sum_x * sum_x) != 0.0 {
            (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
        } else {
            0.0
        };

        let trend_direction = if slope > 0.05 {
            "increasing".to_string()
        } else if slope < -0.05 {
            "decreasing".to_string()
        } else {
            "stable".to_string()
        };

        let trend_strength = slope.abs().min(1.0);

        // Calculate volatility (standard deviation)
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let volatility = variance.sqrt() / (mean + 0.001); // Normalize by mean

        // Forecast accuracy decreases with volatility
        let forecast_accuracy = (1.0 - volatility.min(1.0)).max(0.3);

        Ok(Self {
            segment_id,
            trend_direction,
            trend_strength,
            moving_average: moving_avg,
            volatility,
            forecast_accuracy,
        })
    }
}

/// Intent cluster discovery (Feature 18)
#[derive(Clone, Debug)]
pub struct IntentCluster {
    pub cluster_id: usize,
    pub intent_type: String,  // "high_churn_risk", "growth_opportunity", "dormant", etc.
    pub member_count: usize,
    pub confidence: f64,      // 0-1: how certain of this intent pattern
    pub primary_signals: Vec<(String, f64)>,  // (signal_name, strength)
}

impl IntentCluster {
    pub fn new(cluster_id: usize, intent_type: String, member_count: usize) -> Self {
        Self {
            cluster_id,
            intent_type,
            member_count,
            confidence: 0.0,
            primary_signals: Vec::new(),
        }
    }

    pub fn add_signal(&mut self, signal_name: String, strength: f64) {
        self.primary_signals.push((signal_name, strength));
    }

    pub fn calculate_confidence(&mut self) {
        if self.primary_signals.is_empty() {
            self.confidence = 0.0;
            return;
        }

        // Confidence = average strength of signals
        let avg_strength = self.primary_signals.iter().map(|(_, s)| s).sum::<f64>()
            / self.primary_signals.len() as f64;
        self.confidence = avg_strength;
    }
}

/// Growth forecasting (Feature 26)
#[derive(Clone, Debug)]
pub struct GrowthForecast {
    pub segment_id: usize,
    pub current_size: usize,
    pub forecast_periods: usize,
    pub forecasted_sizes: Vec<usize>,  // Size at each future period
    pub growth_rate_trend: String,     // "accelerating", "constant", "decelerating"
    pub confidence_interval: (f64, f64),  // (lower, upper) bounds
}

impl GrowthForecast {
    pub fn from_historical(
        segment_id: usize,
        current_size: usize,
        historical_sizes: &[usize],
        forecast_periods: usize,
    ) -> Result<Self> {
        if historical_sizes.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No historical data".to_string(),
            ));
        }

        // Calculate average growth rate
        let mut growth_rates = Vec::new();
        for i in 1..historical_sizes.len() {
            let prev = historical_sizes[i - 1] as f64;
            if prev > 0.0 {
                let rate = (historical_sizes[i] as f64 - prev) / prev;
                growth_rates.push(rate);
            }
        }

        let avg_growth = if !growth_rates.is_empty() {
            growth_rates.iter().sum::<f64>() / growth_rates.len() as f64
        } else {
            0.0
        };

        // Detect growth trend
        let growth_rate_trend = if growth_rates.len() >= 2 {
            let first_half_avg = growth_rates[..growth_rates.len() / 2]
                .iter()
                .sum::<f64>()
                / (growth_rates.len() / 2) as f64;
            let second_half_avg = growth_rates[growth_rates.len() / 2..]
                .iter()
                .sum::<f64>()
                / (growth_rates.len() - growth_rates.len() / 2) as f64;

            if second_half_avg > first_half_avg + 0.02 {
                "accelerating".to_string()
            } else if second_half_avg < first_half_avg - 0.02 {
                "decelerating".to_string()
            } else {
                "constant".to_string()
            }
        } else {
            "constant".to_string()
        };

        // Forecast using linear projection
        let mut forecasted_sizes = Vec::new();
        let mut projected = current_size as f64;
        for _ in 0..forecast_periods {
            projected = (projected * (1.0 + avg_growth)).max(0.0);
            forecasted_sizes.push(projected as usize);
        }

        // Confidence interval: ±20% based on volatility
        let volatility = if !growth_rates.is_empty() {
            let mean = avg_growth;
            let variance = growth_rates.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
                / growth_rates.len() as f64;
            variance.sqrt()
        } else {
            0.2
        };

        let confidence_margin = volatility.min(0.3);
        let confidence_interval = (
            (1.0 - confidence_margin).max(0.5),
            (1.0 + confidence_margin).min(1.5),
        );

        Ok(Self {
            segment_id,
            current_size,
            forecast_periods,
            forecasted_sizes,
            growth_rate_trend,
            confidence_interval,
        })
    }
}

/// AI Persona (Feature 14)
#[derive(Clone, Debug)]
pub struct AiPersona {
    pub persona_id: usize,
    pub name: String,              // Auto-generated: "High-Value Loyalist", "At-Risk Churn", etc.
    pub description: String,       // Business-friendly description
    pub segment_ids: Vec<usize>,   // Segments matching this persona
    pub size: usize,               // Total members in persona
    pub key_characteristics: Vec<(String, f64)>,  // (trait, score) pairs
    pub recommended_actions: Vec<String>,
    pub business_impact: String,   // "high", "medium", "low"
}

impl AiPersona {
    pub fn new(persona_id: usize, name: String, segment_ids: Vec<usize>, size: usize) -> Self {
        Self {
            persona_id,
            name,
            description: String::new(),
            segment_ids,
            size,
            key_characteristics: Vec::new(),
            recommended_actions: Vec::new(),
            business_impact: "medium".to_string(),
        }
    }

    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    pub fn add_characteristic(&mut self, trait_name: String, score: f64) {
        self.key_characteristics.push((trait_name, score));
    }

    pub fn add_action(&mut self, action: String) {
        self.recommended_actions.push(action);
    }

    pub fn set_impact(&mut self, impact: String) {
        self.business_impact = impact;
    }
}

/// Causal driver discovery (Features 31-40)
#[derive(Clone, Debug)]
pub struct CausalDriver {
    pub feature_name: String,
    pub driver_type: String,  // "revenue_driver", "churn_driver", "retention_driver", etc.
    pub effect_size: f64,     // 0-1: magnitude of causal effect
    pub statistical_significance: f64,  // p-value analog
    pub direction: String,    // "positive" or "negative"
    pub affected_segments: Vec<usize>,
    pub mechanism: String,    // How/why this causes the outcome
}

impl CausalDriver {
    pub fn new(
        feature_name: String,
        driver_type: String,
        effect_size: f64,
        significance: f64,
        direction: String,
    ) -> Self {
        Self {
            feature_name,
            driver_type,
            effect_size,
            statistical_significance: significance,
            direction,
            affected_segments: Vec::new(),
            mechanism: String::new(),
        }
    }

    pub fn with_mechanism(mut self, mechanism: String) -> Self {
        self.mechanism = mechanism;
        self
    }

    pub fn add_affected_segment(&mut self, segment_id: usize) {
        self.affected_segments.push(segment_id);
    }

    pub fn is_significant(&self) -> bool {
        self.statistical_significance < 0.05
    }
}

/// Product affinity discovery (Feature 19)
#[derive(Clone, Debug)]
pub struct ProductAffinity {
    pub product_pair: (String, String),  // (product_a, product_b)
    pub affinity_score: f64,  // 0-1: likelihood of both products
    pub co_purchase_rate: f64,  // % of users buying both
    pub correlation: f64,  // Statistical correlation
    pub lift: f64,  // Lift over random chance
    pub segments_affected: Vec<usize>,
}

impl ProductAffinity {
    pub fn new(
        product_a: String,
        product_b: String,
        affinity_score: f64,
        co_purchase_rate: f64,
        correlation: f64,
    ) -> Self {
        // Lift = actual co-purchase / (rate_a * rate_b)
        // Approximation: lift based on correlation
        let lift = (1.0 + correlation).max(0.1);

        Self {
            product_pair: (product_a, product_b),
            affinity_score,
            co_purchase_rate,
            correlation,
            lift,
            segments_affected: Vec::new(),
        }
    }

    pub fn add_segment(&mut self, segment_id: usize) {
        self.segments_affected.push(segment_id);
    }

    pub fn is_strong_affinity(&self) -> bool {
        self.affinity_score > 0.6 && self.lift > 1.5
    }
}

/// Pattern Discovery Engine
pub struct PatternDiscovery;

impl PatternDiscovery {
    /// Detect emerging audiences
    pub fn detect_emerging_audiences(
        segment_data: &[(usize, usize, usize, Vec<usize>)],  // (id, current, previous, history)
    ) -> Vec<EmergingAudience> {
        segment_data
            .iter()
            .map(|(id, current, prev, hist)| {
                EmergingAudience::new(*id, *current, *prev, hist)
            })
            .collect()
    }

    /// Analyze segment trends
    pub fn analyze_trends(
        segment_id: usize,
        metric_history: &[f64],
    ) -> Result<SegmentTrend> {
        SegmentTrend::from_time_series(segment_id, metric_history)
    }

    /// Discover intent clusters from behavioral signals
    pub fn discover_intent_clusters(
        segment_behaviors: &HashMap<usize, Vec<(String, f64)>>,  // segment_id -> signals
    ) -> Vec<IntentCluster> {
        let mut clusters = Vec::new();

        for (segment_id, signals) in segment_behaviors {
            // Infer intent from signals
            let mut churn_risk = 0.0;
            let mut growth_potential = 0.0;

            for (signal, strength) in signals {
                match signal.as_str() {
                    "engagement_declining" => churn_risk += strength,
                    "engagement_increasing" => growth_potential += strength,
                    "high_ltv" => growth_potential += strength * 0.5,
                    "low_recency" => churn_risk += strength,
                    "high_frequency" => growth_potential += strength,
                    _ => {}
                }
            }

            let intent_type = if churn_risk > growth_potential {
                "high_churn_risk".to_string()
            } else if growth_potential > churn_risk {
                "growth_opportunity".to_string()
            } else {
                "stable_engagement".to_string()
            };

            let mut cluster = IntentCluster::new(*segment_id, intent_type, 0);

            for (signal, strength) in signals {
                cluster.add_signal(signal.clone(), *strength);
            }

            cluster.calculate_confidence();
            clusters.push(cluster);
        }

        clusters
    }

    /// Forecast segment growth
    pub fn forecast_growth(
        segment_id: usize,
        current_size: usize,
        historical_sizes: &[usize],
        periods: usize,
    ) -> Result<GrowthForecast> {
        GrowthForecast::from_historical(segment_id, current_size, historical_sizes, periods)
    }

    /// Generate AI personas from segment characteristics
    pub fn generate_personas(
        segment_profiles: &[(usize, Vec<(String, f64)>)],  // (segment_id, characteristics)
    ) -> Vec<AiPersona> {
        let mut personas = Vec::new();

        for (segment_id, characteristics) in segment_profiles {
            // Infer persona name from characteristics
            let persona_name = Self::infer_persona_name(characteristics);

            let mut persona = AiPersona::new(*segment_id, persona_name.clone(), vec![*segment_id], 100);

            // Add characteristics
            for (char_name, score) in characteristics {
                persona.add_characteristic(char_name.clone(), *score);
            }

            // Generate description
            let description = format!(
                "{} customers with {} characteristics. High engagement and targeted approach recommended.",
                persona_name,
                if characteristics.len() > 2 { "diverse" } else { "focused" }
            );
            persona = persona.with_description(description);

            // Add recommended actions
            persona.add_action("Segment for targeted campaigns".to_string());
            persona.add_action("Personalize messaging".to_string());
            persona.add_action("Monitor lifecycle progression".to_string());

            // Estimate business impact
            let avg_score = characteristics.iter().map(|(_, s)| s).sum::<f64>()
                / characteristics.len().max(1) as f64;
            let impact = if avg_score > 0.7 {
                "high".to_string()
            } else if avg_score > 0.4 {
                "medium".to_string()
            } else {
                "low".to_string()
            };
            persona.set_impact(impact);

            personas.push(persona);
        }

        personas
    }

    /// Infer persona name from characteristics
    fn infer_persona_name(characteristics: &[(String, f64)]) -> String {
        if characteristics.is_empty() {
            return "Standard".to_string();
        }

        let mut churn_score = 0.0;
        let mut growth_score = 0.0;
        let mut value_score = 0.0;

        for (char_name, score) in characteristics {
            match char_name.as_str() {
                s if s.contains("churn") || s.contains("risk") => churn_score += score,
                s if s.contains("growth") || s.contains("opportunity") => growth_score += score,
                s if s.contains("value") || s.contains("ltv") || s.contains("premium") => {
                    value_score += score
                }
                _ => {}
            }
        }

        if churn_score > growth_score && churn_score > value_score {
            "At-Risk".to_string()
        } else if growth_score > value_score {
            "Growth-Oriented".to_string()
        } else if value_score > 0.5 {
            "High-Value".to_string()
        } else {
            "Engaged".to_string()
        }
    }

    /// Discover product affinities from co-purchase data
    pub fn discover_product_affinities(
        segment_products: &HashMap<usize, Vec<String>>,  // segment_id -> products
        product_correlations: &HashMap<(String, String), f64>,  // (product_a, product_b) -> correlation
    ) -> Vec<ProductAffinity> {
        let mut affinities = Vec::new();

        for ((prod_a, prod_b), correlation) in product_correlations {
            if correlation.abs() > 0.2 {
                // Calculate co-purchase rate
                let mut co_purchases = 0;
                let mut total = 0;

                for (_segment_id, products) in segment_products {
                    let has_a = products.iter().any(|p| p == prod_a);
                    let has_b = products.iter().any(|p| p == prod_b);

                    if has_a || has_b {
                        total += 1;
                        if has_a && has_b {
                            co_purchases += 1;
                        }
                    }
                }

                let co_purchase_rate = if total > 0 {
                    co_purchases as f64 / total as f64
                } else {
                    0.0
                };

                // Affinity score: combination of correlation and co-purchase
                let affinity_score = (correlation.abs() + co_purchase_rate) / 2.0;

                let mut affinity = ProductAffinity::new(
                    prod_a.clone(),
                    prod_b.clone(),
                    affinity_score,
                    co_purchase_rate,
                    *correlation,
                );

                // Track which segments show this affinity
                for (segment_id, products) in segment_products {
                    let has_both = products.iter().any(|p| p == prod_a)
                        && products.iter().any(|p| p == prod_b);
                    if has_both {
                        affinity.add_segment(*segment_id);
                    }
                }

                affinities.push(affinity);
            }
        }

        // Sort by affinity score
        affinities.sort_by(|a, b| b.affinity_score.partial_cmp(&a.affinity_score).unwrap_or(std::cmp::Ordering::Equal));

        affinities
    }

    /// Extract causal drivers from feature importance + outcomes
    pub fn extract_causal_drivers(
        feature_importances: &[(String, f64)],  // (feature, importance)
        outcome_correlations: &HashMap<String, f64>,  // feature -> correlation with outcome
        segment_outcomes: &[f64],  // Outcome values (churn, revenue, etc.)
    ) -> Vec<CausalDriver> {
        let mut drivers = Vec::new();

        for (feature_name, importance) in feature_importances {
            let correlation = outcome_correlations.get(feature_name).copied().unwrap_or(0.0);

            // Determine driver type and direction
            let (driver_type, direction) = if correlation.abs() > 0.3 {
                if correlation > 0.0 {
                    ("positive_driver".to_string(), "positive".to_string())
                } else {
                    ("negative_driver".to_string(), "negative".to_string())
                }
            } else {
                ("weak_driver".to_string(), "neutral".to_string())
            };

            // Simple significance: importance > 0.1 is considered significant
            let significance = if *importance > 0.1 { 0.01 } else { 0.5 };

            let driver = CausalDriver::new(
                feature_name.clone(),
                driver_type,
                *importance,
                significance,
                direction,
            );

            drivers.push(driver);
        }

        // Sort by effect size (importance)
        drivers.sort_by(|a, b| b.effect_size.partial_cmp(&a.effect_size).unwrap_or(std::cmp::Ordering::Equal));

        drivers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emerging_audience_accelerating() {
        let emerging = EmergingAudience::new(0, 1000, 500, &vec![100, 200, 400, 500]);
        assert!(emerging.growth_rate > 0.1);
        assert_eq!(emerging.emergence_status, "accelerating");
    }

    #[test]
    fn test_emerging_audience_plateauing() {
        let emerging = EmergingAudience::new(0, 510, 500, &vec![100, 200, 300, 400, 500]);
        assert!(emerging.growth_rate < 0.05);
        assert_eq!(emerging.emergence_status, "plateauing");
    }

    #[test]
    fn test_segment_trend_increasing() {
        let trend = SegmentTrend::from_time_series(0, &vec![100.0, 110.0, 120.0, 130.0]).unwrap();
        assert_eq!(trend.trend_direction, "increasing");
        assert!(trend.trend_strength > 0.0);
    }

    #[test]
    fn test_segment_trend_stable() {
        let trend = SegmentTrend::from_time_series(0, &vec![100.0, 100.5, 99.5, 100.2]).unwrap();
        assert_eq!(trend.trend_direction, "stable");
    }

    #[test]
    fn test_segment_trend_decreasing() {
        let trend = SegmentTrend::from_time_series(0, &vec![100.0, 90.0, 80.0, 70.0]).unwrap();
        assert_eq!(trend.trend_direction, "decreasing");
        assert!(trend.trend_strength > 0.0);
    }

    #[test]
    fn test_intent_cluster() {
        let mut cluster = IntentCluster::new(0, "high_churn_risk".to_string(), 500);
        cluster.add_signal("low_recency".to_string(), 0.8);
        cluster.add_signal("engagement_declining".to_string(), 0.7);
        cluster.calculate_confidence();

        assert!(cluster.confidence > 0.0);
        assert_eq!(cluster.primary_signals.len(), 2);
    }

    #[test]
    fn test_growth_forecast_accelerating() {
        // Growth rates: 20%, 30%, 40% - accelerating
        let forecast = GrowthForecast::from_historical(0, 1000, &vec![1000, 1200, 1560, 2184], 3).unwrap();
        assert_eq!(forecast.forecasted_sizes.len(), 3);
        assert!(forecast.forecasted_sizes[0] > 1000);
        assert_eq!(forecast.growth_rate_trend, "accelerating");
    }

    #[test]
    fn test_growth_forecast_decelerating() {
        let forecast = GrowthForecast::from_historical(0, 1000, &vec![1000, 1100, 1150, 1175], 3).unwrap();
        assert_eq!(forecast.growth_rate_trend, "decelerating");
    }

    #[test]
    fn test_causal_driver_churn() {
        let driver = CausalDriver::new(
            "recency".to_string(),
            "churn_driver".to_string(),
            0.8,
            0.01,
            "negative".to_string(),
        );
        assert!(driver.is_significant());
        assert_eq!(driver.effect_size, 0.8);
    }

    #[test]
    fn test_causal_driver_not_significant() {
        let driver = CausalDriver::new(
            "feature_x".to_string(),
            "weak_driver".to_string(),
            0.05,
            0.5,
            "neutral".to_string(),
        );
        assert!(!driver.is_significant());
    }

    #[test]
    fn test_detect_emerging() {
        let data = vec![
            (0, 1000, 500, vec![100, 200, 400, 500]),
            (1, 100, 95, vec![80, 85, 90, 95]),
        ];
        let emerging = PatternDiscovery::detect_emerging_audiences(&data);
        assert_eq!(emerging.len(), 2);
        assert!(emerging[0].emergence_score > emerging[1].emergence_score);
    }

    #[test]
    fn test_discover_intents() {
        let mut behaviors = HashMap::new();
        let signals = vec![
            ("engagement_declining".to_string(), 0.8),
            ("low_recency".to_string(), 0.7),
        ];
        behaviors.insert(0, signals);

        let clusters = PatternDiscovery::discover_intent_clusters(&behaviors);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].intent_type, "high_churn_risk");
    }

    #[test]
    fn test_extract_drivers() {
        let importance = vec![
            ("recency".to_string(), 0.8),
            ("frequency".to_string(), 0.5),
        ];
        let mut correlations = HashMap::new();
        correlations.insert("recency".to_string(), -0.6);
        correlations.insert("frequency".to_string(), 0.4);

        let outcomes = vec![0.1, 0.2, 0.3];

        let drivers = PatternDiscovery::extract_causal_drivers(&importance, &correlations, &outcomes);
        assert_eq!(drivers.len(), 2);
        assert!(drivers[0].is_significant());
        assert_eq!(drivers[0].direction, "negative");
    }

    #[test]
    fn test_forecast_accuracy() {
        let forecast =
            GrowthForecast::from_historical(0, 1000, &vec![800, 900, 1000], 2).unwrap();
        assert!(forecast.confidence_interval.0 > 0.0);
        assert!(forecast.confidence_interval.1 > forecast.confidence_interval.0);
    }

    #[test]
    fn test_ai_persona_creation() {
        let mut persona =
            AiPersona::new(0, "High-Value".to_string(), vec![0, 1, 2], 5000);
        persona.add_characteristic("high_ltv".to_string(), 0.9);
        persona.add_characteristic("frequent_purchase".to_string(), 0.8);

        assert_eq!(persona.name, "High-Value");
        assert_eq!(persona.key_characteristics.len(), 2);
    }

    #[test]
    fn test_persona_with_actions() {
        let mut persona =
            AiPersona::new(1, "At-Risk".to_string(), vec![3, 4], 2000);
        persona.add_action("Send retention offer".to_string());
        persona.add_action("Schedule check-in call".to_string());

        assert_eq!(persona.recommended_actions.len(), 2);
    }

    #[test]
    fn test_generate_personas() {
        let profiles = vec![
            (0, vec![("high_ltv".to_string(), 0.85), ("engagement".to_string(), 0.8)]),
            (1, vec![("churn_risk".to_string(), 0.75), ("low_recency".to_string(), 0.8)]),
        ];

        let personas = PatternDiscovery::generate_personas(&profiles);
        assert_eq!(personas.len(), 2);
        assert!(personas[0].name.len() > 0);
    }

    #[test]
    fn test_product_affinity() {
        let affinity = ProductAffinity::new(
            "product_a".to_string(),
            "product_b".to_string(),
            0.75,
            0.65,
            0.7,
        );

        assert_eq!(affinity.product_pair.0, "product_a");
        assert!(affinity.is_strong_affinity());
    }

    #[test]
    fn test_discover_affinities() {
        let mut segment_products = HashMap::new();
        segment_products.insert(0, vec!["product_a".to_string(), "product_b".to_string()]);
        segment_products.insert(1, vec!["product_a".to_string(), "product_b".to_string()]);
        segment_products.insert(2, vec!["product_a".to_string()]);

        let mut correlations = HashMap::new();
        correlations.insert(("product_a".to_string(), "product_b".to_string()), 0.8);

        let affinities = PatternDiscovery::discover_product_affinities(&segment_products, &correlations);
        assert_eq!(affinities.len(), 1);
        assert!(affinities[0].co_purchase_rate > 0.5);
    }

    #[test]
    fn test_causal_driver_with_mechanism() {
        let driver = CausalDriver::new(
            "recency".to_string(),
            "churn_driver".to_string(),
            0.8,
            0.01,
            "negative".to_string(),
        )
        .with_mechanism("Low recency indicates dormancy and increases churn risk".to_string());

        assert!(!driver.mechanism.is_empty());
    }

    #[test]
    fn test_persona_impact_high() {
        let profiles = vec![(
            0,
            vec![
                ("high_value".to_string(), 0.9),
                ("high_engagement".to_string(), 0.85),
            ],
        )];

        let personas = PatternDiscovery::generate_personas(&profiles);
        assert_eq!(personas[0].business_impact, "high");
    }

    #[test]
    fn test_product_affinity_weak() {
        let affinity = ProductAffinity::new(
            "product_x".to_string(),
            "product_y".to_string(),
            0.3,
            0.2,
            0.1,
        );

        assert!(!affinity.is_strong_affinity());
    }
}
