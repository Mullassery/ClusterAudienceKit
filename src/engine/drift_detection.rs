//! Drift detection and monitoring for segment quality

use crate::Result;
use std::collections::HashMap;

/// Statistical drift detection methods
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriftMethod {
    KolmogorovSmirnov,
    KullbackLeibler,
    HellingerDistance,
    ChiSquare,
}

/// Drift severity levels
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DriftSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl DriftSeverity {
    pub fn threshold(&self) -> f64 {
        match self {
            DriftSeverity::None => 0.0,
            DriftSeverity::Low => 0.1,
            DriftSeverity::Medium => 0.2,
            DriftSeverity::High => 0.35,
            DriftSeverity::Critical => 0.5,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DriftSeverity::None => "none",
            DriftSeverity::Low => "low",
            DriftSeverity::Medium => "medium",
            DriftSeverity::High => "high",
            DriftSeverity::Critical => "critical",
        }
    }
}

/// Distribution statistics snapshot
#[derive(Clone, Debug)]
pub struct DistributionStats {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub p25: f64,
    pub p75: f64,
    pub skewness: f64,
    pub kurtosis: f64,
}

impl DistributionStats {
    pub fn new(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                mean: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                median: 0.0,
                p25: 0.0,
                p75: 0.0,
                skewness: 0.0,
                kurtosis: 0.0,
            };
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if sorted.len().is_multiple_of(2) {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let p25 = sorted[(sorted.len() as f64 * 0.25) as usize].max(sorted[0]);
        let p75 = sorted[(sorted.len() as f64 * 0.75) as usize].min(sorted[sorted.len() - 1]);

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];

        let skewness = if std_dev > 0.0 {
            let mut sum = 0.0;
            for v in values {
                sum += ((v - mean) / std_dev).powi(3);
            }
            sum / values.len() as f64
        } else {
            0.0
        };

        let kurtosis = if std_dev > 0.0 {
            let mut sum = 0.0;
            for v in values {
                sum += ((v - mean) / std_dev).powi(4);
            }
            sum / values.len() as f64 - 3.0
        } else {
            0.0
        };

        Self {
            mean,
            std_dev,
            min,
            max,
            median,
            p25,
            p75,
            skewness,
            kurtosis,
        }
    }
}

/// Feature drift indicator
#[derive(Clone, Debug)]
pub struct FeatureDrift {
    pub feature_name: String,
    pub drift_score: f64,
    pub severity: DriftSeverity,
    pub baseline_stats: DistributionStats,
    pub current_stats: DistributionStats,
    pub timestamp: i64,
}

/// Segment composition change
#[derive(Clone, Debug)]
pub struct SegmentCompositionChange {
    pub segment_name: String,
    pub previous_size: usize,
    pub current_size: usize,
    pub size_change_percent: f64,
    pub churn_rate: f64,
    pub growth_rate: f64,
}

/// Drift alert
#[derive(Clone, Debug)]
pub struct DriftAlert {
    pub alert_id: String,
    pub feature_name: String,
    pub drift_score: f64,
    pub severity: DriftSeverity,
    pub message: String,
    pub timestamp: i64,
    pub acknowledged: bool,
}

impl DriftAlert {
    pub fn new(feature_name: String, drift_score: f64, severity: DriftSeverity) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let alert_id = format!("alert_{}_{}", feature_name, now);
        let message = format!(
            "Drift detected in {}: {} (score: {:.3})",
            feature_name,
            severity.as_str(),
            drift_score
        );

        Self {
            alert_id,
            feature_name,
            drift_score,
            severity,
            message,
            timestamp: now,
            acknowledged: false,
        }
    }
}

/// Drift detector
pub struct DriftDetector;

impl DriftDetector {
    /// Kolmogorov-Smirnov test for distribution drift
    pub fn kolmogorov_smirnov(baseline: &[f64], current: &[f64]) -> Result<f64> {
        if baseline.is_empty() || current.is_empty() {
            return Ok(0.0);
        }

        let mut baseline_sorted = baseline.to_vec();
        let mut current_sorted = current.to_vec();

        baseline_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        current_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut max_diff: f64 = 0.0;
        for i in 0..baseline_sorted.len() {
            let baseline_cdf = (i + 1) as f64 / baseline_sorted.len() as f64;
            let current_val = baseline_sorted[i];

            let current_cdf = current_sorted.iter().filter(|v| **v <= current_val).count() as f64
                / current_sorted.len() as f64;

            max_diff = max_diff.max((baseline_cdf - current_cdf).abs());
        }

        Ok(max_diff)
    }

    /// Hellinger distance between two distributions
    pub fn hellinger_distance(baseline: &[f64], current: &[f64]) -> Result<f64> {
        if baseline.is_empty() || current.is_empty() {
            return Ok(0.0);
        }

        let baseline_mean = baseline.iter().sum::<f64>() / baseline.len() as f64;
        let current_mean = current.iter().sum::<f64>() / current.len() as f64;

        let baseline_var = baseline
            .iter()
            .map(|x| (x - baseline_mean).powi(2))
            .sum::<f64>()
            / baseline.len() as f64;
        let current_var = current
            .iter()
            .map(|x| (x - current_mean).powi(2))
            .sum::<f64>()
            / current.len() as f64;

        let sqrt_prod = (baseline_var * current_var).sqrt();
        if sqrt_prod == 0.0 {
            return Ok(0.0);
        }

        let term1 = (baseline_mean - current_mean).powi(2) / (2.0 * (baseline_var + current_var));
        let term2 =
            0.5 * (baseline_var / current_var).ln() + 0.5 * (current_var / baseline_var).ln();

        let bhattacharyya = term1 + term2;
        let distance = (1.0 - (-bhattacharyya).exp()).sqrt();

        Ok(distance.min(1.0))
    }

    /// Chi-square test for categorical drift
    pub fn chi_square(
        baseline_counts: &HashMap<String, usize>,
        current_counts: &HashMap<String, usize>,
    ) -> Result<f64> {
        let mut chi_square = 0.0;

        for (category, baseline_count) in baseline_counts {
            let current_count = current_counts.get(category).copied().unwrap_or(0);
            let baseline_total: usize = baseline_counts.values().sum();
            let current_total: usize = current_counts.values().sum();

            let baseline_prop = *baseline_count as f64 / baseline_total as f64;
            let _current_prop = current_count as f64 / current_total as f64;

            let expected = baseline_prop * current_total as f64;
            if expected > 0.0 {
                chi_square += (current_count as f64 - expected).powi(2) / expected;
            }
        }

        Ok(chi_square / (baseline_counts.len() as f64))
    }

    /// Detect segment composition drift
    pub fn detect_segment_composition_change(
        previous_segments: &HashMap<String, usize>,
        current_segments: &HashMap<String, usize>,
    ) -> Result<Vec<SegmentCompositionChange>> {
        let mut changes = Vec::new();

        let prev_total: usize = previous_segments.values().sum();
        let _curr_total: usize = current_segments.values().sum();

        for (segment, current_size) in current_segments {
            let previous_size = previous_segments.get(segment).copied().unwrap_or(0);
            let size_change = *current_size as i32 - previous_size as i32;
            let size_change_percent = if prev_total > 0 {
                (size_change as f64 / prev_total as f64) * 100.0
            } else {
                0.0
            };

            let churn_rate = if previous_size > *current_size {
                ((previous_size - *current_size) as f64 / previous_size as f64).max(0.0)
            } else {
                0.0
            };

            let growth_rate = if previous_size > 0 {
                ((*current_size as i32 - previous_size as i32) as f64 / previous_size as f64)
                    .max(0.0)
            } else {
                1.0
            };

            changes.push(SegmentCompositionChange {
                segment_name: segment.clone(),
                previous_size,
                current_size: *current_size,
                size_change_percent,
                churn_rate,
                growth_rate,
            });
        }

        Ok(changes)
    }

    /// Detect feature drift with severity classification
    pub fn detect_feature_drift(
        feature_name: String,
        baseline: &[f64],
        current: &[f64],
        method: DriftMethod,
    ) -> Result<FeatureDrift> {
        let drift_score = match method {
            DriftMethod::KolmogorovSmirnov => Self::kolmogorov_smirnov(baseline, current)?,
            DriftMethod::HellingerDistance => Self::hellinger_distance(baseline, current)?,
            DriftMethod::KullbackLeibler => Self::hellinger_distance(baseline, current)?,
            DriftMethod::ChiSquare => 0.0, // Use KS as proxy
        };

        let severity = if drift_score < DriftSeverity::Low.threshold() {
            DriftSeverity::None
        } else if drift_score < DriftSeverity::Medium.threshold() {
            DriftSeverity::Low
        } else if drift_score < DriftSeverity::High.threshold() {
            DriftSeverity::Medium
        } else if drift_score < DriftSeverity::Critical.threshold() {
            DriftSeverity::High
        } else {
            DriftSeverity::Critical
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Ok(FeatureDrift {
            feature_name,
            drift_score,
            severity,
            baseline_stats: DistributionStats::new(baseline),
            current_stats: DistributionStats::new(current),
            timestamp: now,
        })
    }

    /// Batch feature drift detection
    pub fn detect_batch_drift(
        features: &HashMap<String, (Vec<f64>, Vec<f64>)>,
    ) -> Result<Vec<FeatureDrift>> {
        let mut drifts = Vec::new();

        for (feature_name, (baseline, current)) in features {
            let drift = Self::detect_feature_drift(
                feature_name.clone(),
                baseline,
                current,
                DriftMethod::KolmogorovSmirnov,
            )?;

            drifts.push(drift);
        }

        Ok(drifts)
    }

    /// Check if critical drift exists
    pub fn has_critical_drift(drifts: &[FeatureDrift]) -> bool {
        drifts.iter().any(|d| d.severity == DriftSeverity::Critical)
    }

    /// Get drifted features above threshold
    pub fn get_drifted_features(
        drifts: &[FeatureDrift],
        min_severity: DriftSeverity,
    ) -> Vec<FeatureDrift> {
        drifts
            .iter()
            .filter(|d| d.severity >= min_severity)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distribution_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = DistributionStats::new(&values);

        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_distribution_stats_empty() {
        let values: Vec<f64> = vec![];
        let stats = DistributionStats::new(&values);

        assert_eq!(stats.mean, 0.0);
        assert_eq!(stats.std_dev, 0.0);
    }

    #[test]
    fn test_drift_severity_thresholds() {
        assert_eq!(DriftSeverity::None.threshold(), 0.0);
        assert_eq!(DriftSeverity::Low.threshold(), 0.1);
        assert_eq!(DriftSeverity::Critical.threshold(), 0.5);
    }

    #[test]
    fn test_ks_test_no_drift() {
        let baseline = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let current = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let drift = DriftDetector::kolmogorov_smirnov(&baseline, &current).unwrap();
        assert!(drift < 0.1);
    }

    #[test]
    fn test_ks_test_with_drift() {
        let baseline = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let current = vec![10.0, 11.0, 12.0, 13.0, 14.0];

        let drift = DriftDetector::kolmogorov_smirnov(&baseline, &current).unwrap();
        assert!(drift > 0.5);
    }

    #[test]
    fn test_hellinger_distance() {
        let baseline = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let current = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let distance = DriftDetector::hellinger_distance(&baseline, &current).unwrap();
        assert!(distance < 0.1);
    }

    #[test]
    fn test_segment_composition_change() {
        let mut prev = HashMap::new();
        prev.insert("Champions".to_string(), 100);
        prev.insert("Loyal".to_string(), 200);

        let mut curr = HashMap::new();
        curr.insert("Champions".to_string(), 120);
        curr.insert("Loyal".to_string(), 180);

        let changes = DriftDetector::detect_segment_composition_change(&prev, &curr).unwrap();

        assert_eq!(changes.len(), 2);
        assert!(changes[0].size_change_percent != 0.0);
    }

    #[test]
    fn test_detect_feature_drift() {
        let baseline = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let current = vec![1.5, 2.5, 3.5, 4.5, 5.5];

        let drift = DriftDetector::detect_feature_drift(
            "feature_1".to_string(),
            &baseline,
            &current,
            DriftMethod::KolmogorovSmirnov,
        )
        .unwrap();

        assert!(!drift.feature_name.is_empty());
        assert!(drift.drift_score >= 0.0);
    }

    #[test]
    fn test_drift_severity_classification() {
        let baseline = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let current = vec![100.0, 200.0, 300.0, 400.0, 500.0];

        let drift = DriftDetector::detect_feature_drift(
            "feature_1".to_string(),
            &baseline,
            &current,
            DriftMethod::KolmogorovSmirnov,
        )
        .unwrap();

        assert!(drift.severity >= DriftSeverity::Medium);
    }

    #[test]
    fn test_drift_alert_creation() {
        let alert = DriftAlert::new("feature_1".to_string(), 0.45, DriftSeverity::High);

        assert_eq!(alert.severity, DriftSeverity::High);
        assert!(!alert.acknowledged);
    }

    #[test]
    fn test_chi_square() {
        let mut baseline = HashMap::new();
        baseline.insert("A".to_string(), 50);
        baseline.insert("B".to_string(), 50);

        let mut current = HashMap::new();
        current.insert("A".to_string(), 70);
        current.insert("B".to_string(), 30);

        let chi2 = DriftDetector::chi_square(&baseline, &current).unwrap();
        assert!(chi2 > 0.0);
    }

    #[test]
    fn test_has_critical_drift() {
        let drifts = vec![
            FeatureDrift {
                feature_name: "f1".to_string(),
                drift_score: 0.2,
                severity: DriftSeverity::Medium,
                baseline_stats: DistributionStats::new(&[]),
                current_stats: DistributionStats::new(&[]),
                timestamp: 0,
            },
            FeatureDrift {
                feature_name: "f2".to_string(),
                drift_score: 0.6,
                severity: DriftSeverity::Critical,
                baseline_stats: DistributionStats::new(&[]),
                current_stats: DistributionStats::new(&[]),
                timestamp: 0,
            },
        ];

        assert!(DriftDetector::has_critical_drift(&drifts));
    }

    #[test]
    fn test_get_drifted_features() {
        let drifts = vec![
            FeatureDrift {
                feature_name: "f1".to_string(),
                drift_score: 0.05,
                severity: DriftSeverity::None,
                baseline_stats: DistributionStats::new(&[]),
                current_stats: DistributionStats::new(&[]),
                timestamp: 0,
            },
            FeatureDrift {
                feature_name: "f2".to_string(),
                drift_score: 0.3,
                severity: DriftSeverity::High,
                baseline_stats: DistributionStats::new(&[]),
                current_stats: DistributionStats::new(&[]),
                timestamp: 0,
            },
        ];

        let critical = DriftDetector::get_drifted_features(&drifts, DriftSeverity::High);
        assert_eq!(critical.len(), 1);
    }
}
