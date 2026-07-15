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

/// Causal driver discovery (Features 31-40)
#[derive(Clone, Debug)]
pub struct CausalDriver {
    pub feature_name: String,
    pub driver_type: String,  // "revenue_driver", "churn_driver", "retention_driver", etc.
    pub effect_size: f64,     // 0-1: magnitude of causal effect
    pub statistical_significance: f64,  // p-value analog
    pub direction: String,    // "positive" or "negative"
    pub affected_segments: Vec<usize>,
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
        }
    }

    pub fn add_affected_segment(&mut self, segment_id: usize) {
        self.affected_segments.push(segment_id);
    }

    pub fn is_significant(&self) -> bool {
        self.statistical_significance < 0.05
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
}
