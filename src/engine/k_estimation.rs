//! Automatic K estimation for optimal cluster count

use crate::engine::algorithms::{DistanceMetric, KMeans};
use crate::Result;
use ndarray::{s, Array1, Array2};

/// K estimation result
#[derive(Clone, Debug)]
pub struct KEstimationResult {
    pub method: String,
    pub k: usize,
    pub scores: Vec<(usize, f64)>,
    pub confidence: f64,
}

/// Elbow method for K estimation
pub struct ElbowMethod;

impl ElbowMethod {
    /// Estimate optimal K using elbow method
    /// Finds the "elbow" in inertia curve
    pub fn estimate(data: &Array2<f64>, k_range: (usize, usize)) -> Result<KEstimationResult> {
        let (k_min, k_max) = k_range;
        let mut scores = Vec::new();

        for k in k_min..=k_max {
            let kmeans = KMeans::new(k);
            let result = kmeans.fit(data)?;
            scores.push((k, result.inertia));
        }

        // Find elbow point using second derivative
        let best_k = if scores.len() > 2 {
            let mut max_curvature = 0.0;
            let mut elbow_k = k_min;

            for i in 1..scores.len() - 1 {
                let (_, inertia1) = scores[i - 1];
                let (_, inertia2) = scores[i];
                let (_, inertia3) = scores[i + 1];

                let d1 = inertia1 - inertia2;
                let d2 = inertia2 - inertia3;
                let d2_d = (d1 - d2).abs();

                if d2_d > max_curvature {
                    max_curvature = d2_d;
                    elbow_k = scores[i].0;
                }
            }

            elbow_k
        } else {
            k_min
        };

        let confidence = 1.0 / (1.0 + (scores.len() as f64).sqrt().powf(0.5));

        Ok(KEstimationResult {
            method: "elbow".to_string(),
            k: best_k,
            scores,
            confidence,
        })
    }
}

/// Gap statistic method for K estimation
pub struct GapStatistic;

impl GapStatistic {
    /// Estimate optimal K using gap statistic
    pub fn estimate(data: &Array2<f64>, k_range: (usize, usize)) -> Result<KEstimationResult> {
        let (k_min, k_max) = k_range;
        let (n_samples, _) = data.dim();

        // Calculate reference distribution statistics
        let reference_stats = Self::calculate_reference_dispersion(data);

        let mut scores = Vec::new();
        let mut gaps = Vec::new();

        for k in k_min..=k_max {
            let kmeans = KMeans::new(k);
            let result = kmeans.fit(data)?;

            // Calculate within-cluster dispersion
            let mut within_cluster_dispersion = 0.0;
            for (i, sample) in data.outer_iter().enumerate() {
                let cluster_center = result.centers.slice(s![result.labels[i], ..]);
                let dist = DistanceMetric::Euclidean
                    .distance(&sample.to_owned(), &cluster_center.to_owned());
                within_cluster_dispersion += dist;
            }

            let log_w_k = (within_cluster_dispersion / n_samples as f64).log2();
            let gap = reference_stats - log_w_k;
            gaps.push(gap);
            scores.push((k, gap));
        }

        // Find K where gap is largest while still reasonable
        let best_k = if !gaps.is_empty() {
            let max_gap_idx = gaps
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);

            scores[max_gap_idx].0
        } else {
            k_min
        };

        let confidence = if !gaps.is_empty() {
            let max_gap = gaps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (max_gap / 10.0).clamp(0.0, 1.0)
        } else {
            0.0
        };

        Ok(KEstimationResult {
            method: "gap_statistic".to_string(),
            k: best_k,
            scores,
            confidence,
        })
    }

    fn calculate_reference_dispersion(data: &Array2<f64>) -> f64 {
        let (n_samples, n_features) = data.dim();

        // Use uniform reference distribution
        let mut min_vals = Array1::from_elem(n_features, f64::MAX);
        let mut max_vals = Array1::from_elem(n_features, f64::MIN);

        for row in data.outer_iter() {
            for (i, &val) in row.iter().enumerate() {
                min_vals[i] = min_vals[i].min(val);
                max_vals[i] = max_vals[i].max(val);
            }
        }

        let mut dispersion = 0.0;
        for i in 0..n_features {
            let range = max_vals[i] - min_vals[i];
            dispersion += range * range / 12.0;
        }

        (dispersion * n_samples as f64).log2()
    }
}

/// Silhouette-based K estimation
pub struct SilhouetteEstimation;

impl SilhouetteEstimation {
    /// Estimate optimal K using silhouette analysis
    pub fn estimate(data: &Array2<f64>, k_range: (usize, usize)) -> Result<KEstimationResult> {
        let (k_min, k_max) = k_range;
        let mut scores = Vec::new();

        for k in k_min..=k_max {
            let kmeans = KMeans::new(k);
            let result = kmeans.fit(data)?;

            // Calculate silhouette score
            let silhouette = Self::calculate_silhouette_score(data, &result.labels)?;
            scores.push((k, silhouette));
        }

        // Find K with maximum silhouette score
        let best_k = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(k, _)| *k)
            .unwrap_or(k_min);

        let max_score = scores
            .iter()
            .map(|(_, s)| *s)
            .fold(f64::NEG_INFINITY, f64::max);
        let confidence = max_score;

        Ok(KEstimationResult {
            method: "silhouette".to_string(),
            k: best_k,
            scores,
            confidence,
        })
    }

    /// Calculate silhouette score for clustering
    pub fn calculate_silhouette_score(data: &Array2<f64>, labels: &[usize]) -> Result<f64> {
        if labels.is_empty() {
            return Ok(0.0);
        }

        let mut silhouette_sum = 0.0;

        for (i, sample) in data.outer_iter().enumerate() {
            let cluster_id = labels[i];

            // Calculate average intra-cluster distance (a)
            let mut intra_sum = 0.0;
            let mut intra_count = 0;

            for (j, other_sample) in data.outer_iter().enumerate() {
                if labels[j] == cluster_id && i != j {
                    let dist = DistanceMetric::Euclidean
                        .distance(&sample.to_owned(), &other_sample.to_owned());
                    intra_sum += dist;
                    intra_count += 1;
                }
            }

            let a = if intra_count > 0 {
                intra_sum / intra_count as f64
            } else {
                0.0
            };

            // Calculate average inter-cluster distance (b)
            let mut min_inter_dist = f64::MAX;
            let n_clusters = labels.iter().max().map(|&x| x + 1).unwrap_or(1);

            for c in 0..n_clusters {
                if c == cluster_id {
                    continue;
                }

                let mut inter_sum = 0.0;
                let mut inter_count = 0;

                for (j, other_sample) in data.outer_iter().enumerate() {
                    if labels[j] == c {
                        let dist = DistanceMetric::Euclidean
                            .distance(&sample.to_owned(), &other_sample.to_owned());
                        inter_sum += dist;
                        inter_count += 1;
                    }
                }

                if inter_count > 0 {
                    let avg_dist = inter_sum / inter_count as f64;
                    min_inter_dist = min_inter_dist.min(avg_dist);
                }
            }

            let b = if min_inter_dist == f64::MAX {
                0.0
            } else {
                min_inter_dist
            };

            // Calculate silhouette coefficient
            let max_ab = a.max(b);
            let silhouette = if max_ab > 1e-10 {
                (b - a) / max_ab
            } else {
                0.0
            };

            silhouette_sum += silhouette;
        }

        Ok(silhouette_sum / labels.len() as f64)
    }
}

/// Combined K estimation using multiple methods
pub struct CombinedKEstimation;

impl CombinedKEstimation {
    /// Estimate K using ensemble of methods
    pub fn estimate(data: &Array2<f64>, k_range: (usize, usize)) -> Result<KEstimationResult> {
        let elbow = ElbowMethod::estimate(data, k_range)?;
        let gap = GapStatistic::estimate(data, k_range)?;
        let silhouette = SilhouetteEstimation::estimate(data, k_range)?;

        // Simple voting scheme
        let mut votes: std::collections::HashMap<usize, i32> = std::collections::HashMap::new();
        *votes.entry(elbow.k).or_insert(0) += 1;
        *votes.entry(gap.k).or_insert(0) += 1;
        *votes.entry(silhouette.k).or_insert(0) += 1;

        let best_k = votes
            .iter()
            .max_by_key(|(_k, count)| *count)
            .map(|(k, _)| *k)
            .unwrap_or(k_range.0);

        let confidence = (votes.get(&best_k).copied().unwrap_or(0) as f64) / 3.0;

        Ok(KEstimationResult {
            method: "ensemble".to_string(),
            k: best_k,
            scores: vec![],
            confidence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elbow_method() {
        let data = Array2::from_shape_vec(
            (10, 2),
            vec![
                0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0, 10.0, 10.0, 10.5, 10.5,
                11.0, 11.0, 11.5, 11.5,
            ],
        )
        .unwrap();

        let result = ElbowMethod::estimate(&data, (2, 5)).unwrap();
        assert!(result.k >= 2 && result.k <= 5);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_gap_statistic() {
        let data = Array2::from_shape_vec(
            (10, 2),
            vec![
                0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0, 10.0, 10.0, 10.5, 10.5,
                11.0, 11.0, 11.5, 11.5,
            ],
        )
        .unwrap();

        let result = GapStatistic::estimate(&data, (2, 5)).unwrap();
        assert!(result.k >= 2 && result.k <= 5);
        assert!(!result.scores.is_empty());
    }

    #[test]
    fn test_silhouette_estimation() {
        let data = Array2::from_shape_vec(
            (10, 2),
            vec![
                0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0, 10.0, 10.0, 10.5, 10.5,
                11.0, 11.0, 11.5, 11.5,
            ],
        )
        .unwrap();

        let result = SilhouetteEstimation::estimate(&data, (2, 5)).unwrap();
        assert!(result.k >= 2 && result.k <= 5);
    }

    #[test]
    fn test_silhouette_score() {
        let data = Array2::from_shape_vec(
            (6, 2),
            vec![0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0],
        )
        .unwrap();
        let labels = vec![0, 0, 0, 1, 1, 1];

        let score = SilhouetteEstimation::calculate_silhouette_score(&data, &labels).unwrap();
        assert!((-1.0..=1.0).contains(&score));
    }

    #[test]
    fn test_combined_estimation() {
        let data = Array2::from_shape_vec(
            (10, 2),
            vec![
                0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0, 10.0, 10.0, 10.5, 10.5,
                11.0, 11.0, 11.5, 11.5,
            ],
        )
        .unwrap();

        let result = CombinedKEstimation::estimate(&data, (2, 5)).unwrap();
        assert!(result.k >= 2 && result.k <= 5);
        assert_eq!(result.method, "ensemble");
    }

    #[test]
    fn test_k_range_validation() {
        let data =
            Array2::from_shape_vec((4, 2), vec![0.0, 0.0, 1.0, 1.0, 5.0, 5.0, 6.0, 6.0]).unwrap();

        let result = ElbowMethod::estimate(&data, (1, 3)).unwrap();
        assert!(result.k >= 1 && result.k <= 3);
    }
}
