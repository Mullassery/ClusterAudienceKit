//! Segment Intelligence: Explainability, confidence, stability, health metrics

use crate::Result;
use std::collections::HashMap;

/// Segment confidence based on cluster membership certainty
#[derive(Clone, Debug)]
pub struct SegmentConfidence {
    pub segment_id: usize,
    pub member_id: String,
    pub confidence_score: f64,  // 0.0-1.0: higher = more certain
    pub distance_to_centroid: f64,
    pub distance_to_nearest_other: f64,
}

impl SegmentConfidence {
    /// Create new confidence score from distances
    /// Formula: 1 / (1 + distance_ratio) where ratio = dist_to_centroid / dist_to_nearest_other
    pub fn new(
        segment_id: usize,
        member_id: String,
        distance_to_centroid: f64,
        distance_to_nearest_other: f64,
    ) -> Self {
        let confidence_score = if distance_to_nearest_other > 0.0 {
            let ratio = distance_to_centroid / distance_to_nearest_other;
            1.0 / (1.0 + ratio)
        } else {
            1.0
        };

        Self {
            segment_id,
            member_id,
            confidence_score,
            distance_to_centroid,
            distance_to_nearest_other,
        }
    }
}

/// Segment entropy (diversity measure)
#[derive(Clone, Debug)]
pub struct SegmentEntropy {
    pub segment_id: usize,
    pub shannon_entropy: f64,      // 0.0 = homogeneous, higher = more diverse
    pub gini_coefficient: f64,     // 0.0 = uniform, 1.0 = concentrated
    pub member_count: usize,
    pub feature_diversity: HashMap<String, f64>,  // Entropy per feature
}

impl SegmentEntropy {
    pub fn new(segment_id: usize, member_count: usize) -> Self {
        Self {
            segment_id,
            shannon_entropy: 0.0,
            gini_coefficient: 0.0,
            member_count,
            feature_diversity: HashMap::new(),
        }
    }

    /// Compute Shannon entropy from probability distribution
    /// H(X) = -Σ p(x) * log2(p(x))
    pub fn compute_shannon(frequencies: &[usize]) -> f64 {
        if frequencies.is_empty() || frequencies.iter().sum::<usize>() == 0 {
            return 0.0;
        }

        let total: usize = frequencies.iter().sum();
        let mut entropy = 0.0;

        for &freq in frequencies {
            if freq > 0 {
                let p = freq as f64 / total as f64;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Compute Gini coefficient (concentration metric)
    /// Gini = Σ |p_i - p_j| / 2 (normalized)
    pub fn compute_gini(frequencies: &[usize]) -> f64 {
        if frequencies.is_empty() || frequencies.iter().sum::<usize>() <= 1 {
            return 0.0;
        }

        let total: usize = frequencies.iter().sum();
        let n = frequencies.len() as f64;

        // Sort frequencies
        let mut sorted_freq: Vec<usize> = frequencies.to_vec();
        sorted_freq.sort_unstable();

        let mut gini = 0.0;
        for (i, &freq) in sorted_freq.iter().enumerate() {
            gini += (2.0 * (i as f64 + 1.0) - n - 1.0) * freq as f64;
        }

        gini / ((n - 1.0) * total as f64)
    }
}

/// Segment differentiation (uniqueness score)
#[derive(Clone, Debug)]
pub struct SegmentDifferentiation {
    pub segment_id: usize,
    pub differentiation_score: f64,  // 0.0 = identical to others, 1.0 = unique
    pub similarity_to_nearest: f64,   // Distance to most similar segment
    pub similarity_to_farthest: f64,  // Distance to most different segment
}

impl SegmentDifferentiation {
    /// Compute differentiation as inverse of nearest similarity
    /// Formula: 1.0 - similarity_to_nearest
    pub fn new(
        segment_id: usize,
        similarity_to_nearest: f64,
        similarity_to_farthest: f64,
    ) -> Self {
        let differentiation_score = 1.0 - similarity_to_nearest;

        Self {
            segment_id,
            differentiation_score,
            similarity_to_nearest,
            similarity_to_farthest,
        }
    }
}

/// Segment predictability (stability of assignments)
#[derive(Clone, Debug)]
pub struct SegmentPredictability {
    pub segment_id: usize,
    pub predictability_score: f64,   // 0.0 = unpredictable, 1.0 = highly stable
    pub assignment_variance: f64,    // Variance in member confidence over time
    pub churn_rate: f64,             // Fraction of members changing segments
    pub stability_trend: String,     // "stable", "declining", "improving"
}

impl SegmentPredictability {
    pub fn new(
        segment_id: usize,
        assignment_variance: f64,
        churn_rate: f64,
    ) -> Self {
        // Predictability = 1.0 - variance - churn
        let predictability_score = (1.0 - assignment_variance * 0.5 - churn_rate * 0.5).max(0.0);

        let stability_trend = if churn_rate <= 0.05 {
            "stable".to_string()
        } else if churn_rate > 0.15 {
            "declining".to_string()
        } else {
            "neutral".to_string()
        };

        Self {
            segment_id,
            predictability_score,
            assignment_variance,
            churn_rate,
            stability_trend,
        }
    }
}

/// Segment aging (tenure and lifespan analysis)
#[derive(Clone, Debug)]
pub struct SegmentAging {
    pub segment_id: usize,
    pub avg_member_tenure_days: f64,
    pub median_member_tenure_days: f64,
    pub newest_member_days_ago: f64,
    pub oldest_member_days_ago: f64,
    pub member_churn_rate_30d: f64,
    pub segment_lifespan_stage: String,  // "emerging", "growth", "mature", "declining"
}

impl SegmentAging {
    pub fn new(
        segment_id: usize,
        tenure_days: Vec<f64>,
        member_churn_30d: f64,
    ) -> Result<Self> {
        if tenure_days.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Empty tenure data".to_string(),
            ));
        }

        let avg = tenure_days.iter().sum::<f64>() / tenure_days.len() as f64;

        let mut sorted = tenure_days.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let newest = sorted.iter().copied().fold(f64::INFINITY, f64::min);
        let oldest = sorted.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        // Lifecycle stage based on age and churn
        let lifespan_stage = if avg < 30.0 && member_churn_30d < 0.1 {
            "emerging".to_string()
        } else if avg < 180.0 && member_churn_30d < 0.15 {
            "growth".to_string()
        } else if member_churn_30d < 0.2 {
            "mature".to_string()
        } else {
            "declining".to_string()
        };

        Ok(Self {
            segment_id,
            avg_member_tenure_days: avg,
            median_member_tenure_days: median,
            newest_member_days_ago: newest,
            oldest_member_days_ago: oldest,
            member_churn_rate_30d: member_churn_30d,
            segment_lifespan_stage: lifespan_stage,
        })
    }
}

/// Feature importance explanation for segment membership
#[derive(Clone, Debug)]
pub struct ExplainabilityReport {
    pub segment_id: usize,
    pub member_id: String,
    pub top_contributing_features: Vec<(String, f64)>,  // (feature_name, importance_score)
    pub bottom_contributing_features: Vec<(String, f64)>,
    pub feature_explanations: HashMap<String, String>,
}

impl ExplainabilityReport {
    pub fn new(
        segment_id: usize,
        member_id: String,
        feature_importance: Vec<(String, f64)>,
    ) -> Self {
        let mut sorted = feature_importance.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_count = (sorted.len() / 3).max(1).min(5);
        let top_features = sorted.iter().take(top_count).cloned().collect();
        let bottom_features = sorted.iter().rev().take(top_count).cloned().collect();

        Self {
            segment_id,
            member_id,
            top_contributing_features: top_features,
            bottom_contributing_features: bottom_features,
            feature_explanations: HashMap::new(),
        }
    }

    /// Add natural language explanation for a feature
    pub fn add_explanation(&mut self, feature: String, explanation: String) {
        self.feature_explanations.insert(feature, explanation);
    }
}

/// Segment health score (composite metric)
#[derive(Clone, Debug)]
pub struct SegmentHealth {
    pub segment_id: usize,
    pub health_score: f64,  // 0-100: higher = healthier
    pub confidence: f64,    // 0-100: membership certainty
    pub stability: f64,     // 0-100: churn resistance
    pub differentiation: f64,  // 0-100: uniqueness
    pub size_health: f64,   // 0-100: optimal size?
    pub trend: String,      // "improving", "stable", "declining"
    pub alerts: Vec<String>,
}

impl SegmentHealth {
    pub fn new(
        segment_id: usize,
        avg_confidence: f64,
        predictability: f64,
        differentiation: f64,
        member_count: usize,
    ) -> Self {
        // Size health: optimal is 5%-25% of total
        let size_health = if member_count > 100 && member_count < 10000 {
            80.0
        } else if member_count > 10 {
            60.0
        } else {
            40.0
        };

        // Composite health (normalize size_health from 0-100 to 0-1)
        let health_score = (avg_confidence * 30.0 + predictability * 30.0
                           + differentiation * 20.0 + (size_health / 100.0) * 20.0);

        let trend = if predictability > 0.7 && avg_confidence > 0.7 {
            "improving".to_string()
        } else if predictability < 0.4 || avg_confidence < 0.4 {
            "declining".to_string()
        } else {
            "stable".to_string()
        };

        let mut alerts = Vec::new();
        if avg_confidence < 0.5 {
            alerts.push("Low membership confidence".to_string());
        }
        if predictability < 0.5 {
            alerts.push("High member churn".to_string());
        }
        if differentiation < 0.3 {
            alerts.push("Not differentiated from other segments".to_string());
        }
        if member_count < 10 {
            alerts.push("Very small segment".to_string());
        }

        Self {
            segment_id,
            health_score,
            confidence: avg_confidence * 100.0,
            stability: predictability * 100.0,
            differentiation: differentiation * 100.0,
            size_health,
            trend,
            alerts,
        }
    }
}

/// Feature attribution for segment membership (leverages XGBoost importance)
#[derive(Clone, Debug)]
pub struct FeatureAttribution {
    pub segment_id: usize,
    pub feature_name: String,
    pub importance_score: f64,  // 0-1: normalized importance
    pub direction: String,      // "high" = feature high in segment, "low" = feature low
    pub prevalence: f64,        // % of segment members exhibiting this pattern
}

/// SHAP-like value for model explainability
#[derive(Clone, Debug)]
pub struct ShapValue {
    pub segment_id: usize,
    pub feature_name: String,
    pub shap_value: f64,        // Contribution to segment membership probability
    pub base_value: f64,        // Average prediction across all segments
    pub feature_value: f64,     // This segment's feature value
}

/// Segment stability metric (Feature 3 - complements Drift Detection)
#[derive(Clone, Debug)]
pub struct SegmentStability {
    pub segment_id: usize,
    pub stability_score: f64,   // 0-1: 1.0 = very stable, 0.0 = unstable
    pub member_retention_rate: f64,  // % of members staying in segment
    pub size_change_rate: f64,  // % change in segment size
    pub trend: String,          // "stable", "growing", "shrinking", "volatile"
    pub risk_level: String,     // "low", "medium", "high"
}

impl SegmentStability {
    pub fn new(
        segment_id: usize,
        retention_rate: f64,
        size_change: f64,
    ) -> Self {
        // Stability = retention_rate × (1 - |size_change|)
        let stability_score = retention_rate * (1.0 - size_change.abs()).max(0.0);

        let trend = if size_change > 0.1 {
            "growing".to_string()
        } else if size_change < -0.1 {
            "shrinking".to_string()
        } else if retention_rate > 0.9 {
            "stable".to_string()
        } else {
            "volatile".to_string()
        };

        let risk_level = if stability_score > 0.8 {
            "low".to_string()
        } else if stability_score > 0.5 {
            "medium".to_string()
        } else {
            "high".to_string()
        };

        Self {
            segment_id,
            stability_score,
            member_retention_rate: retention_rate,
            size_change_rate: size_change,
            trend,
            risk_level,
        }
    }
}

/// Segment decay detection (Feature 7)
#[derive(Clone, Debug)]
pub struct SegmentDecay {
    pub segment_id: usize,
    pub decay_rate: f64,        // % loss per period
    pub half_life_periods: f64, // Periods until 50% decay
    pub is_decaying: bool,      // decay_rate > 0.05
    pub decay_status: String,   // "healthy", "slow_decay", "rapid_decay"
    pub estimated_extinction_periods: f64,  // Periods until segment disappears
}

impl SegmentDecay {
    pub fn new(
        segment_id: usize,
        current_size: usize,
        historical_sizes: &[usize],  // Sizes over time periods
    ) -> Result<Self> {
        if historical_sizes.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "No historical data".to_string(),
            ));
        }

        // Calculate decay rate via exponential fit
        let first_size = historical_sizes[0] as f64;
        let last_size = historical_sizes[historical_sizes.len() - 1] as f64;
        let periods = (historical_sizes.len() - 1) as f64;

        let decay_rate = if first_size > 0.0 && periods > 0.0 {
            1.0 - (last_size / first_size).powf(1.0 / periods)
        } else {
            0.0
        };

        // Half-life: time until 50% remains
        let half_life = if decay_rate > 0.0 && decay_rate < 1.0 {
            0.693_147 / decay_rate.ln().abs()
        } else {
            f64::INFINITY
        };

        let is_decaying = decay_rate > 0.05;

        let decay_status = if decay_rate < 0.02 {
            "healthy".to_string()
        } else if decay_rate < 0.1 {
            "slow_decay".to_string()
        } else {
            "rapid_decay".to_string()
        };

        // Estimate extinction time
        let extinction_time = if decay_rate > 0.0 && current_size > 0 {
            (current_size as f64).log(1.0 - decay_rate)
        } else {
            f64::INFINITY
        };

        Ok(Self {
            segment_id,
            decay_rate,
            half_life_periods: half_life,
            is_decaying,
            decay_status,
            estimated_extinction_periods: extinction_time,
        })
    }
}

/// Segment Explainability Report (wired to XGBoost)
#[derive(Clone, Debug)]
pub struct SegmentExplainability {
    pub segment_id: usize,
    pub primary_drivers: Vec<FeatureAttribution>,
    pub secondary_features: Vec<FeatureAttribution>,
    pub shap_values: Vec<ShapValue>,
    pub summary: String,  // Natural language summary
}

impl SegmentExplainability {
    pub fn new(segment_id: usize) -> Self {
        Self {
            segment_id,
            primary_drivers: Vec::new(),
            secondary_features: Vec::new(),
            shap_values: Vec::new(),
            summary: String::new(),
        }
    }

    /// Add primary driver features
    pub fn add_driver(&mut self, feature_name: String, importance: f64, direction: String, prevalence: f64) {
        self.primary_drivers.push(FeatureAttribution {
            segment_id: self.segment_id,
            feature_name,
            importance_score: importance,
            direction,
            prevalence,
        });
    }

    /// Add secondary features
    pub fn add_secondary(&mut self, feature_name: String, importance: f64, direction: String, prevalence: f64) {
        self.secondary_features.push(FeatureAttribution {
            segment_id: self.segment_id,
            feature_name,
            importance_score: importance,
            direction,
            prevalence,
        });
    }

    /// Add SHAP value for feature contribution
    pub fn add_shap(&mut self, feature_name: String, shap_value: f64, base_value: f64, feature_value: f64) {
        self.shap_values.push(ShapValue {
            segment_id: self.segment_id,
            feature_name,
            shap_value,
            base_value,
            feature_value,
        });
    }

    /// Generate human-readable summary
    pub fn generate_summary(&mut self) -> String {
        if self.primary_drivers.is_empty() {
            self.summary = format!("Segment {} has no primary drivers identified.", self.segment_id);
            return self.summary.clone();
        }

        let top_driver = &self.primary_drivers[0];
        let direction_text = if top_driver.direction == "high" {
            "elevated"
        } else {
            "depressed"
        };

        self.summary = format!(
            "Segment {} is primarily characterized by {} {} (importance: {:.0}%, {} of members affected).",
            self.segment_id,
            direction_text,
            top_driver.feature_name,
            top_driver.importance_score * 100.0,
            (top_driver.prevalence * 100.0) as u32
        );

        self.summary.clone()
    }
}

/// Segment Intelligence Engine
pub struct SegmentIntelligence;

impl SegmentIntelligence {
    /// Calculate confidence scores for all members
    pub fn calculate_confidence(
        assignments: &[usize],
        distances_to_centroid: &[f64],
        all_cluster_distances: &[Vec<f64>],
    ) -> Result<Vec<SegmentConfidence>> {
        if assignments.len() != distances_to_centroid.len() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Assignment/distance length mismatch".to_string(),
            ));
        }

        let mut confidences = Vec::new();

        for (i, &segment_id) in assignments.iter().enumerate() {
            let dist_to_centroid = distances_to_centroid[i];
            let mut distances = all_cluster_distances[i].clone();

            // Remove own segment distance to find nearest other
            if segment_id < distances.len() {
                distances[segment_id] = f64::INFINITY;
            }
            let dist_to_nearest_other = distances
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);

            confidences.push(SegmentConfidence::new(
                segment_id,
                format!("member_{}", i),
                dist_to_centroid,
                dist_to_nearest_other,
            ));
        }

        Ok(confidences)
    }

    /// Calculate entropy for segments
    pub fn calculate_entropy(
        segment_members: &HashMap<usize, Vec<usize>>,
        feature_counts: &HashMap<usize, Vec<usize>>,  // segment_id -> category frequencies
    ) -> HashMap<usize, SegmentEntropy> {
        let mut entropies = HashMap::new();

        for (&segment_id, members) in segment_members {
            let mut entropy = SegmentEntropy::new(segment_id, members.len());

            // Calculate Shannon entropy from feature distribution
            if let Some(counts) = feature_counts.get(&segment_id) {
                entropy.shannon_entropy = SegmentEntropy::compute_shannon(counts);
                entropy.gini_coefficient = SegmentEntropy::compute_gini(counts);
            }

            entropies.insert(segment_id, entropy);
        }

        entropies
    }

    /// Calculate segment differentiation scores
    pub fn calculate_differentiation(
        centroids: &[Vec<f64>],
    ) -> Result<Vec<SegmentDifferentiation>> {
        let mut differentiations = Vec::new();

        for (i, _centroid) in centroids.iter().enumerate() {
            let mut min_similarity: f64 = 1.0;
            let mut max_similarity: f64 = 0.0;

            // Compute cosine similarity to all other centroids
            for (j, other) in centroids.iter().enumerate() {
                if i != j {
                    let similarity = Self::cosine_similarity(_centroid, other)?;
                    min_similarity = min_similarity.min(similarity);
                    max_similarity = max_similarity.max(similarity);
                }
            }

            differentiations.push(SegmentDifferentiation::new(
                i,
                min_similarity,
                max_similarity,
            ));
        }

        Ok(differentiations)
    }

    /// Build explainability report from feature importance scores
    /// Maps global feature importance to segment-specific context
    pub fn build_explainability(
        segment_id: usize,
        feature_importances: &[(String, f64)],  // (feature_name, importance_score)
        segment_features: &[f64],                // This segment's feature values
        global_feature_means: &[f64],            // Population mean for each feature
    ) -> Result<SegmentExplainability> {
        if feature_importances.len() != segment_features.len()
            || segment_features.len() != global_feature_means.len() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Feature array length mismatch".to_string(),
            ));
        }

        let mut explainability = SegmentExplainability::new(segment_id);

        // Sort by importance
        let mut sorted_importance = feature_importances.to_vec();
        sorted_importance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Normalize importances
        let max_importance = sorted_importance[0].1.max(0.001);

        // Add top 3 as primary drivers
        for (i, (feature_name, importance)) in sorted_importance.iter().take(3).enumerate() {
            let normalized_importance = importance / max_importance;
            let segment_value = segment_features.get(i).copied().unwrap_or(0.0);
            let global_mean = global_feature_means.get(i).copied().unwrap_or(0.0);

            let direction = if segment_value > global_mean { "high" } else { "low" };
            let prevalence = (normalized_importance).min(1.0);

            explainability.add_driver(
                feature_name.clone(),
                normalized_importance,
                direction.to_string(),
                prevalence,
            );
        }

        // Add next 3 as secondary
        for (i, (feature_name, importance)) in sorted_importance.iter().skip(3).take(3).enumerate() {
            let normalized_importance = importance / max_importance;
            let segment_value = segment_features.get(i + 3).copied().unwrap_or(0.0);
            let global_mean = global_feature_means.get(i + 3).copied().unwrap_or(0.0);

            let direction = if segment_value > global_mean { "high" } else { "low" };

            explainability.add_secondary(
                feature_name.clone(),
                normalized_importance,
                direction.to_string(),
                (normalized_importance * 0.7).min(1.0),
            );
        }

        // Generate summary
        explainability.generate_summary();

        Ok(explainability)
    }

    /// Calculate SHAP-like values for segment membership
    /// Approximates SHAP values without full model internals
    pub fn calculate_shap_approximation(
        segment_id: usize,
        segment_features: &[f64],
        feature_names: &[String],
        base_value: f64,
    ) -> Result<Vec<ShapValue>> {
        if segment_features.len() != feature_names.len() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Feature/name length mismatch".to_string(),
            ));
        }

        let mut shap_values = Vec::new();

        for (i, feature_name) in feature_names.iter().enumerate() {
            let feature_value = segment_features.get(i).copied().unwrap_or(0.0);

            // Approximate SHAP as: deviation from mean × feature importance
            let deviation = feature_value - base_value;
            let shap_value = deviation * 0.1; // Simplified; real SHAP requires model internals

            shap_values.push(ShapValue {
                segment_id,
                feature_name: feature_name.clone(),
                shap_value,
                base_value,
                feature_value,
            });
        }

        Ok(shap_values)
    }

    /// Cosine similarity between two vectors
    fn cosine_similarity(a: &[f64], b: &[f64]) -> Result<f64> {
        if a.len() != b.len() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Vector length mismatch".to_string(),
            ));
        }

        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for (ai, bi) in a.iter().zip(b.iter()) {
            dot_product += ai * bi;
            norm_a += ai * ai;
            norm_b += bi * bi;
        }

        let denominator = norm_a.sqrt() * norm_b.sqrt();
        if denominator == 0.0 {
            Ok(0.0)
        } else {
            Ok((dot_product / denominator).max(-1.0).min(1.0))
        }
    }

    /// Calculate segment stability
    pub fn calculate_stability(
        segment_id: usize,
        retention_rate: f64,
        size_change_rate: f64,
    ) -> SegmentStability {
        SegmentStability::new(segment_id, retention_rate, size_change_rate)
    }

    /// Detect segment decay from historical size data
    pub fn detect_decay(
        segment_id: usize,
        current_size: usize,
        historical_sizes: &[usize],
    ) -> Result<SegmentDecay> {
        SegmentDecay::new(segment_id, current_size, historical_sizes)
    }

    /// Calculate segment health composite score
    pub fn calculate_health(
        segment_id: usize,
        confidences: &[SegmentConfidence],
        predictability: &SegmentPredictability,
        differentiation: &SegmentDifferentiation,
        member_count: usize,
    ) -> SegmentHealth {
        let avg_confidence = if !confidences.is_empty() {
            confidences.iter().map(|c| c.confidence_score).sum::<f64>()
                / confidences.len() as f64
        } else {
            0.5
        };

        SegmentHealth::new(
            segment_id,
            avg_confidence,
            predictability.predictability_score,
            differentiation.differentiation_score,
            member_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_score() {
        let conf = SegmentConfidence::new(0, "m1".to_string(), 1.0, 3.0);
        assert!(conf.confidence_score > 0.5);
        assert!(conf.confidence_score < 1.0);
    }

    #[test]
    fn test_shannon_entropy() {
        let frequencies = vec![100, 0, 0];
        let entropy = SegmentEntropy::compute_shannon(&frequencies);
        assert_eq!(entropy, 0.0); // Homogeneous
    }

    #[test]
    fn test_shannon_entropy_uniform() {
        let frequencies = vec![50, 50];
        let entropy = SegmentEntropy::compute_shannon(&frequencies);
        assert_eq!(entropy, 1.0); // Maximum for 2 categories
    }

    #[test]
    fn test_gini_coefficient() {
        let frequencies = vec![100, 0];
        let gini = SegmentEntropy::compute_gini(&frequencies);
        assert!(gini > 0.9); // Highly concentrated
    }

    #[test]
    fn test_gini_uniform() {
        let frequencies = vec![50, 50];
        let gini = SegmentEntropy::compute_gini(&frequencies);
        assert!(gini < 0.1); // Uniform distribution
    }

    #[test]
    fn test_differentiation_score() {
        let diff = SegmentDifferentiation::new(0, 0.3, 0.9);
        assert!((diff.differentiation_score - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_predictability_score() {
        let pred = SegmentPredictability::new(0, 0.1, 0.05);
        assert!(pred.predictability_score > 0.8);
        assert_eq!(pred.stability_trend, "stable");
    }

    #[test]
    fn test_segment_aging() {
        let tenure = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let aging = SegmentAging::new(0, tenure, 0.05).unwrap();
        assert_eq!(aging.avg_member_tenure_days, 30.0);
        assert_eq!(aging.median_member_tenure_days, 30.0);
        assert_eq!(aging.newest_member_days_ago, 10.0);
        assert_eq!(aging.oldest_member_days_ago, 50.0);
    }

    #[test]
    fn test_segment_aging_stage() {
        let tenure = vec![10.0, 15.0, 20.0];
        let aging = SegmentAging::new(0, tenure, 0.08).unwrap();
        assert_eq!(aging.segment_lifespan_stage, "emerging");
    }

    #[test]
    fn test_explainability_report() {
        let features = vec![
            ("feature_1".to_string(), 0.8),
            ("feature_2".to_string(), 0.5),
            ("feature_3".to_string(), 0.2),
        ];
        let report = ExplainabilityReport::new(0, "m1".to_string(), features);
        assert_eq!(report.top_contributing_features.len(), 1);
    }

    #[test]
    fn test_segment_health() {
        let confidences = vec![
            SegmentConfidence::new(0, "m1".to_string(), 0.5, 2.0),
            SegmentConfidence::new(0, "m2".to_string(), 0.7, 2.5),
        ];
        let pred = SegmentPredictability::new(0, 0.1, 0.05);
        let diff = SegmentDifferentiation::new(0, 0.3, 0.9);

        let health = SegmentIntelligence::calculate_health(0, &confidences, &pred, &diff, 1000);
        assert!(health.health_score > 50.0);
        assert_eq!(health.trend, "improving");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = SegmentIntelligence::cosine_similarity(&a, &b).unwrap();
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = SegmentIntelligence::cosine_similarity(&a, &b).unwrap();
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_calculate_confidence() {
        let assignments = vec![0, 1, 0];
        let distances_to_centroid = vec![0.5, 0.3, 0.8];
        let all_distances = vec![
            vec![0.5, 2.0],
            vec![3.0, 0.3],
            vec![1.0, 1.5],
        ];

        let confidences =
            SegmentIntelligence::calculate_confidence(&assignments, &distances_to_centroid, &all_distances).unwrap();
        assert_eq!(confidences.len(), 3);
        assert!(confidences[0].confidence_score > 0.0);
    }

    #[test]
    fn test_calculate_entropy() {
        let mut segment_members = HashMap::new();
        segment_members.insert(0, vec![1, 2, 3]);

        let mut feature_counts = HashMap::new();
        feature_counts.insert(0, vec![3, 0]); // Homogeneous

        let entropies = SegmentIntelligence::calculate_entropy(&segment_members, &feature_counts);
        let entropy = entropies.get(&0).unwrap();
        assert_eq!(entropy.shannon_entropy, 0.0);
    }

    #[test]
    fn test_calculate_differentiation() {
        let centroids = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        let diffs = SegmentIntelligence::calculate_differentiation(&centroids).unwrap();
        assert_eq!(diffs.len(), 3);
        // Orthogonal centroids should have low similarity
        for diff in &diffs {
            assert!(diff.similarity_to_nearest < 0.1);
        }
    }

    #[test]
    fn test_segment_explainability() {
        let mut explain = SegmentExplainability::new(0);
        explain.add_driver("recency".to_string(), 0.8, "high".to_string(), 0.85);
        explain.add_driver("frequency".to_string(), 0.6, "low".to_string(), 0.60);

        assert_eq!(explain.primary_drivers.len(), 2);
        assert_eq!(explain.primary_drivers[0].feature_name, "recency");
    }

    #[test]
    fn test_explainability_summary() {
        let mut explain = SegmentExplainability::new(0);
        explain.add_driver("recency".to_string(), 0.8, "high".to_string(), 0.85);
        explain.generate_summary();

        assert!(explain.summary.contains("Segment 0"));
        assert!(explain.summary.contains("recency"));
        assert!(explain.summary.contains("elevated"));
    }

    #[test]
    fn test_build_explainability() {
        let importance = vec![
            ("recency".to_string(), 0.8),
            ("frequency".to_string(), 0.6),
            ("monetary".to_string(), 0.4),
        ];
        let segment_features = vec![15.0, 8.0, 500.0];
        let global_means = vec![30.0, 4.0, 250.0];

        let explain = SegmentIntelligence::build_explainability(0, &importance, &segment_features, &global_means).unwrap();

        assert_eq!(explain.segment_id, 0);
        assert!(!explain.primary_drivers.is_empty());
        assert!(explain.primary_drivers[0].importance_score > 0.0);
    }

    #[test]
    fn test_shap_approximation() {
        let segment_features = vec![0.5, 0.3, 0.7];
        let feature_names = vec!["f1".to_string(), "f2".to_string(), "f3".to_string()];
        let base_value = 0.5;

        let shap = SegmentIntelligence::calculate_shap_approximation(0, &segment_features, &feature_names, base_value).unwrap();

        assert_eq!(shap.len(), 3);
        assert!(shap[0].shap_value.is_finite());
    }

    #[test]
    fn test_feature_attribution() {
        let attr = FeatureAttribution {
            segment_id: 0,
            feature_name: "recency".to_string(),
            importance_score: 0.8,
            direction: "high".to_string(),
            prevalence: 0.85,
        };

        assert_eq!(attr.feature_name, "recency");
        assert_eq!(attr.importance_score, 0.8);
    }

    #[test]
    fn test_shap_value() {
        let shap = ShapValue {
            segment_id: 0,
            feature_name: "recency".to_string(),
            shap_value: 0.15,
            base_value: 0.5,
            feature_value: 0.65,
        };

        assert_eq!(shap.feature_name, "recency");
        assert_eq!(shap.shap_value, 0.15);
    }

    #[test]
    fn test_segment_stability() {
        let stability = SegmentStability::new(0, 0.95, 0.05);
        assert!(stability.stability_score > 0.8);
        assert_eq!(stability.trend, "stable");
        assert_eq!(stability.risk_level, "low");
    }

    #[test]
    fn test_segment_stability_declining() {
        let stability = SegmentStability::new(0, 0.6, -0.2);
        assert!(stability.stability_score < 0.6);
        assert_eq!(stability.trend, "shrinking");
        assert_eq!(stability.risk_level, "high");
    }

    #[test]
    fn test_segment_decay_healthy() {
        let historical = vec![1000, 990, 980, 971];  // ~1% decay per period
        let decay = SegmentDecay::new(0, 971, &historical).unwrap();
        assert!(decay.decay_rate < 0.015);
        assert_eq!(decay.decay_status, "healthy");
        assert!(!decay.is_decaying);
    }

    #[test]
    fn test_segment_decay_rapid() {
        let historical = vec![1000, 800, 640, 512];  // 20% decay per period
        let decay = SegmentDecay::new(0, 512, &historical).unwrap();
        assert!(decay.decay_rate > 0.1);
        assert_eq!(decay.decay_status, "rapid_decay");
        assert!(decay.is_decaying);
    }

    #[test]
    fn test_segment_decay_half_life() {
        let historical = vec![1000, 500, 250];  // 50% decay per period
        let decay = SegmentDecay::new(0, 250, &historical).unwrap();
        assert!((decay.half_life_periods - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_stability() {
        let stability = SegmentIntelligence::calculate_stability(0, 0.92, 0.03);
        assert!(stability.stability_score > 0.8);
    }

    #[test]
    fn test_detect_decay() {
        let historical = vec![1000, 950, 900, 850];
        let decay = SegmentIntelligence::detect_decay(0, 850, &historical).unwrap();
        assert!(decay.decay_rate > 0.0);
    }
}
