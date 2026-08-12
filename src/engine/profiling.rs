//! Segment profiling and business insights generation

use crate::Result;
use std::collections::HashMap;

/// Statistical summary for a segment
#[derive(Clone, Debug)]
pub struct SegmentStatistics {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub q25: f64,
    pub q75: f64,
}

impl SegmentStatistics {
    pub fn from_values(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                q25: 0.0,
                q75: 0.0,
            };
        }

        // Sort for percentiles
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let median = if sorted.len().is_multiple_of(2) {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let q25_idx = sorted.len() / 4;
        let q75_idx = (3 * sorted.len()) / 4;

        Self {
            mean,
            median,
            std_dev,
            min: sorted[0],
            max: sorted[sorted.len() - 1],
            q25: sorted[q25_idx],
            q75: sorted[q75_idx],
        }
    }
}

/// Feature importance scoring
#[derive(Clone, Debug)]
pub struct FeatureImportance {
    pub name: String,
    pub score: f64,
    pub contribution: f64,
}

/// Segment health indicator
#[derive(Clone, Debug)]
pub struct SegmentHealth {
    pub stability: f64,    // 0-1 measure of cluster stability
    pub cohesion: f64,     // 0-1 within-cluster tightness
    pub separation: f64,   // 0-1 between-cluster distance
    pub health_score: f64, // 0-100 overall health
}

/// Complete segment profile with insights
#[derive(Clone, Debug)]
pub struct SegmentProfile {
    pub segment_id: usize,
    pub size: usize,
    pub purity: f64,
    pub statistics: HashMap<String, SegmentStatistics>,
    pub feature_importance: Vec<FeatureImportance>,
    pub business_description: String,
    pub key_characteristics: Vec<String>,
    pub health: SegmentHealth,
    pub actionability_score: f64,
}

/// Profiling engine
pub struct ProfilingEngine;

impl ProfilingEngine {
    /// Profile a single segment with full statistics
    pub fn profile_segment(
        segment_id: usize,
        members: &[usize],
        features: &HashMap<usize, Vec<f64>>,
        feature_names: Option<&[String]>,
    ) -> Result<SegmentProfile> {
        let size = members.len();
        let mut statistics = HashMap::new();
        let mut feature_importance = Vec::new();

        // Collect feature statistics
        for feature_name in features.keys() {
            let name = feature_names
                .and_then(|names| names.get(*feature_name))
                .cloned()
                .unwrap_or_else(|| format!("feature_{}", feature_name));

            let values: Vec<f64> = members
                .iter()
                .filter_map(|&member_id| {
                    features
                        .get(&member_id)
                        .and_then(|v| v.get(*feature_name).copied())
                })
                .collect();

            if !values.is_empty() {
                let stats = SegmentStatistics::from_values(&values);
                let importance_score = stats.std_dev / (stats.max - stats.min + 1e-10);

                statistics.insert(name.clone(), stats);
                feature_importance.push(FeatureImportance {
                    name,
                    score: importance_score,
                    contribution: importance_score / features.len() as f64,
                });
            }
        }

        // Sort by importance
        feature_importance.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Generate business description
        let description = Self::generate_description(segment_id, size, &statistics);

        // Extract key characteristics
        let characteristics = Self::extract_characteristics(&statistics);

        // Calculate health indicators
        let health = SegmentHealth {
            stability: Self::calculate_stability(size),
            cohesion: Self::estimate_cohesion(&statistics),
            separation: 0.5, // To be calculated with other segments
            health_score: (size as f64).min(100.0), // Placeholder
        };

        let actionability_score = Self::calculate_actionability(&statistics, size);

        Ok(SegmentProfile {
            segment_id,
            size,
            purity: 0.85, // Placeholder
            statistics,
            feature_importance,
            business_description: description,
            key_characteristics: characteristics,
            health,
            actionability_score,
        })
    }

    /// Generate human-readable segment description
    fn generate_description(
        segment_id: usize,
        size: usize,
        statistics: &HashMap<String, SegmentStatistics>,
    ) -> String {
        let size_desc = if size < 50 {
            "small but engaged"
        } else if size < 200 {
            "moderate-sized"
        } else if size < 1000 {
            "large"
        } else {
            "very large"
        };

        format!(
            "Segment {} is a {} group ({} members) with {}",
            segment_id,
            size_desc,
            size,
            if statistics.is_empty() {
                "limited feature data".to_string()
            } else {
                format!(
                    "mean score of {:.2}",
                    statistics.values().next().map(|s| s.mean).unwrap_or(0.0)
                )
            }
        )
    }

    /// Extract key characteristics from statistics
    fn extract_characteristics(statistics: &HashMap<String, SegmentStatistics>) -> Vec<String> {
        let mut characteristics = Vec::new();

        for (name, stats) in statistics.iter().take(3) {
            if stats.mean > stats.median * 1.2 {
                characteristics.push(format!("High {} (mean: {:.2})", name, stats.mean));
            } else if stats.mean < stats.median * 0.8 {
                characteristics.push(format!("Low {} (mean: {:.2})", name, stats.mean));
            }

            if stats.std_dev > stats.mean * 0.5 {
                characteristics.push(format!("Variable {} (std: {:.2})", name, stats.std_dev));
            }
        }

        if characteristics.is_empty() {
            characteristics.push("Homogeneous segment".to_string());
        }

        characteristics
    }

    /// Calculate segment stability (0-1)
    fn calculate_stability(size: usize) -> f64 {
        let size_f = size as f64;
        1.0 / (1.0 + (-size_f / 100.0).exp())
    }

    /// Estimate cohesion from statistics
    fn estimate_cohesion(statistics: &HashMap<String, SegmentStatistics>) -> f64 {
        if statistics.is_empty() {
            return 0.5;
        }

        let avg_cv: f64 = statistics
            .values()
            .map(|s| {
                if s.mean > 1e-10 {
                    s.std_dev / s.mean
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / statistics.len() as f64;

        1.0 - (avg_cv / 2.0).min(1.0)
    }

    /// Calculate actionability score
    fn calculate_actionability(
        statistics: &HashMap<String, SegmentStatistics>,
        size: usize,
    ) -> f64 {
        let size_factor = (size as f64).min(1000.0) / 1000.0;

        let variance_factor = statistics
            .values()
            .map(|s| s.std_dev / (s.max - s.min + 1e-10))
            .sum::<f64>()
            / statistics.len().max(1) as f64;

        ((size_factor + variance_factor) / 2.0 * 100.0).min(100.0)
    }

    /// Calculate inter-segment separation
    pub fn calculate_separation(
        profiles1: &SegmentStatistics,
        profiles2: &SegmentStatistics,
    ) -> f64 {
        let mean_diff = (profiles1.mean - profiles2.mean).abs();
        let combined_std = (profiles1.std_dev + profiles2.std_dev) / 2.0;

        if combined_std < 1e-10 {
            0.0
        } else {
            (mean_diff / combined_std).min(2.0) / 2.0
        }
    }
}

/// Business metrics derived from profiles
#[derive(Clone, Debug)]
pub struct BusinessMetrics {
    pub segment_size_distribution: HashMap<usize, f64>, // segment_id -> percentage
    pub feature_importance_across_segments: HashMap<String, f64>, // feature -> avg importance
    pub segment_diversity: f64,                         // 0-1 measure of segment heterogeneity
    pub overall_health: f64,                            // 0-100 health score
}

impl BusinessMetrics {
    /// Calculate business metrics from profiles
    pub fn from_profiles(profiles: &[SegmentProfile]) -> Self {
        let total_size: usize = profiles.iter().map(|p| p.size).sum();

        let mut segment_size_distribution = HashMap::new();
        for profile in profiles {
            let percentage = (profile.size as f64 / total_size as f64) * 100.0;
            segment_size_distribution.insert(profile.segment_id, percentage);
        }

        let mut feature_importance_across_segments: HashMap<String, f64> = HashMap::new();
        for profile in profiles {
            for feat_imp in &profile.feature_importance {
                feature_importance_across_segments
                    .entry(feat_imp.name.clone())
                    .and_modify(|e| *e += feat_imp.score)
                    .or_insert(feat_imp.score);
            }
        }

        for importance in feature_importance_across_segments.values_mut() {
            *importance /= profiles.len() as f64;
        }

        let segment_diversity =
            profiles.iter().map(|p| p.actionability_score).sum::<f64>() / profiles.len() as f64;

        let overall_health =
            profiles.iter().map(|p| p.health.health_score).sum::<f64>() / profiles.len() as f64;

        Self {
            segment_size_distribution,
            feature_importance_across_segments,
            segment_diversity,
            overall_health,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = SegmentStatistics::from_values(&values);

        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert!(stats.std_dev > 0.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_profile_segment() {
        let members = vec![0, 1, 2, 3];
        let mut features = HashMap::new();
        features.insert(0, vec![1.0, 2.0, 3.0, 4.0]);

        let profile = ProfilingEngine::profile_segment(0, &members, &features, None).unwrap();
        assert_eq!(profile.segment_id, 0);
        assert_eq!(profile.size, 4);
        assert!(!profile.business_description.is_empty());
    }

    #[test]
    fn test_stability_calculation() {
        let stability_small = ProfilingEngine::calculate_stability(10);
        let stability_large = ProfilingEngine::calculate_stability(1000);

        assert!(stability_large > stability_small);
    }

    #[test]
    fn test_cohesion_estimation() {
        let mut statistics = HashMap::new();
        let stats = SegmentStatistics {
            mean: 10.0,
            median: 9.5,
            std_dev: 1.0,
            min: 7.0,
            max: 13.0,
            q25: 8.5,
            q75: 11.5,
        };
        statistics.insert("test".to_string(), stats);

        let cohesion = ProfilingEngine::estimate_cohesion(&statistics);
        assert!(cohesion > 0.0 && cohesion < 1.0);
    }

    #[test]
    fn test_actionability_score() {
        let mut statistics = HashMap::new();
        let stats = SegmentStatistics {
            mean: 50.0,
            median: 48.0,
            std_dev: 10.0,
            min: 30.0,
            max: 70.0,
            q25: 40.0,
            q75: 60.0,
        };
        statistics.insert("test".to_string(), stats);

        let score = ProfilingEngine::calculate_actionability(&statistics, 500);
        assert!(score > 0.0 && score <= 100.0);
    }

    #[test]
    fn test_business_metrics() {
        let profiles = vec![
            SegmentProfile {
                segment_id: 0,
                size: 100,
                purity: 0.9,
                statistics: HashMap::new(),
                feature_importance: Vec::new(),
                business_description: "Segment 0".to_string(),
                key_characteristics: vec!["High value".to_string()],
                health: SegmentHealth {
                    stability: 0.8,
                    cohesion: 0.85,
                    separation: 0.7,
                    health_score: 80.0,
                },
                actionability_score: 85.0,
            },
            SegmentProfile {
                segment_id: 1,
                size: 200,
                purity: 0.88,
                statistics: HashMap::new(),
                feature_importance: Vec::new(),
                business_description: "Segment 1".to_string(),
                key_characteristics: vec!["Medium value".to_string()],
                health: SegmentHealth {
                    stability: 0.75,
                    cohesion: 0.8,
                    separation: 0.7,
                    health_score: 75.0,
                },
                actionability_score: 70.0,
            },
        ];

        let metrics = BusinessMetrics::from_profiles(&profiles);
        assert!(metrics.segment_size_distribution.len() == 2);
        assert!(metrics.overall_health > 0.0);
    }
}
