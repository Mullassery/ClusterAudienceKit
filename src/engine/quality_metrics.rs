//! Clustering quality metrics

use crate::engine::algorithms::DistanceMetric;
use crate::Result;
use ndarray::{s, Array1, Array2};
use std::collections::HashMap;

/// Clustering quality report
#[derive(Clone, Debug)]
pub struct QualityReport {
    pub silhouette_score: f64,
    pub davies_bouldin_score: f64,
    pub calinski_harabasz_score: f64,
    pub inertia: f64,
    pub n_clusters: usize,
    pub overall_score: f64,
}

/// Silhouette analysis
pub struct SilhouetteMetric;

impl SilhouetteMetric {
    /// Calculate silhouette coefficient (per-point, per-cluster, or global)
    pub fn calculate(data: &Array2<f64>, labels: &[usize]) -> Result<f64> {
        if labels.is_empty() {
            return Ok(0.0);
        }

        let mut silhouette_sum = 0.0;
        let n_clusters = *labels.iter().max().unwrap_or(&0) + 1;

        for (i, sample) in data.outer_iter().enumerate() {
            let cluster_id = labels[i];

            // Calculate intra-cluster distance (a)
            let mut intra_sum = 0.0;
            let mut intra_count = 0;

            for (j, other) in data.outer_iter().enumerate() {
                if labels[j] == cluster_id && i != j {
                    let dist =
                        DistanceMetric::Euclidean.distance(&sample.to_owned(), &other.to_owned());
                    intra_sum += dist;
                    intra_count += 1;
                }
            }

            let a = if intra_count > 0 {
                intra_sum / intra_count as f64
            } else {
                0.0
            };

            // Calculate inter-cluster distance (b)
            let mut b = f64::MAX;

            for c in 0..n_clusters {
                if c == cluster_id {
                    continue;
                }

                let mut inter_sum = 0.0;
                let mut inter_count = 0;

                for (j, other) in data.outer_iter().enumerate() {
                    if labels[j] == c {
                        let dist = DistanceMetric::Euclidean
                            .distance(&sample.to_owned(), &other.to_owned());
                        inter_sum += dist;
                        inter_count += 1;
                    }
                }

                if inter_count > 0 {
                    let avg_dist = inter_sum / inter_count as f64;
                    b = b.min(avg_dist);
                }
            }

            let max_ab = a.max(b);
            if max_ab > 1e-10 {
                silhouette_sum += (b - a) / max_ab;
            }
        }

        Ok(silhouette_sum / labels.len() as f64)
    }
}

/// Davies-Bouldin Index
pub struct DaviesBouldinMetric;

impl DaviesBouldinMetric {
    /// Calculate Davies-Bouldin Index (lower is better)
    pub fn calculate(data: &Array2<f64>, labels: &[usize], centers: &Array2<f64>) -> Result<f64> {
        if labels.is_empty() || centers.is_empty() {
            return Ok(0.0);
        }

        let n_clusters = centers.nrows();
        let mut db_index = 0.0;

        for i in 0..n_clusters {
            let center_i = centers.slice(s![i, ..]);

            // Calculate average distance from samples to center i
            let mut avg_dist_i = 0.0;
            let mut count_i = 0;

            for (j, sample) in data.outer_iter().enumerate() {
                if labels[j] == i {
                    let dist = DistanceMetric::Euclidean
                        .distance(&sample.to_owned(), &center_i.to_owned());
                    avg_dist_i += dist;
                    count_i += 1;
                }
            }

            if count_i == 0 {
                continue;
            }

            avg_dist_i /= count_i as f64;

            let mut max_ratio: f64 = 0.0;

            for k in 0..n_clusters {
                if k == i {
                    continue;
                }

                let center_k = centers.slice(s![k, ..]);

                // Calculate average distance from samples to center k
                let mut avg_dist_k = 0.0;
                let mut count_k = 0;

                for (j, sample) in data.outer_iter().enumerate() {
                    if labels[j] == k {
                        let dist = DistanceMetric::Euclidean
                            .distance(&sample.to_owned(), &center_k.to_owned());
                        avg_dist_k += dist;
                        count_k += 1;
                    }
                }

                if count_k == 0 {
                    continue;
                }

                avg_dist_k /= count_k as f64;

                // Distance between centers
                let center_dist =
                    DistanceMetric::Euclidean.distance(&center_i.to_owned(), &center_k.to_owned());

                if center_dist > 1e-10 {
                    let ratio = (avg_dist_i + avg_dist_k) / center_dist;
                    max_ratio = max_ratio.max(ratio);
                }
            }

            db_index += max_ratio;
        }

        Ok(db_index / n_clusters as f64)
    }
}

/// Calinski-Harabasz Index
pub struct CalinskiHarabaszMetric;

impl CalinskiHarabaszMetric {
    /// Calculate Calinski-Harabasz Index (higher is better)
    pub fn calculate(data: &Array2<f64>, labels: &[usize]) -> Result<f64> {
        if labels.is_empty() {
            return Ok(0.0);
        }

        let n_clusters = *labels.iter().max().unwrap_or(&0) + 1;
        if n_clusters < 2 {
            return Ok(0.0);
        }

        let (n_samples, _) = data.dim();

        // Calculate global centroid
        let mut global_centroid = Array1::zeros(data.ncols());
        for row in data.outer_iter() {
            for (i, &val) in row.iter().enumerate() {
                global_centroid[i] += val;
            }
        }
        global_centroid.mapv_inplace(|x| x / n_samples as f64);

        // Calculate between-cluster variance
        let mut between_variance = 0.0;
        for c in 0..n_clusters {
            let mut cluster_sum = Array1::zeros(data.ncols());
            let mut cluster_count = 0;

            for (i, sample) in data.outer_iter().enumerate() {
                if labels[i] == c {
                    for (j, &val) in sample.iter().enumerate() {
                        cluster_sum[j] += val;
                    }
                    cluster_count += 1;
                }
            }

            if cluster_count > 0 {
                cluster_sum.mapv_inplace(|x| x / cluster_count as f64);

                let dist = DistanceMetric::Euclidean.distance(&cluster_sum, &global_centroid);
                between_variance += cluster_count as f64 * dist * dist;
            }
        }

        // Calculate within-cluster variance
        let mut within_variance = 0.0;
        for c in 0..n_clusters {
            let mut cluster_center = Array1::zeros(data.ncols());
            let mut cluster_count = 0;

            for (i, sample) in data.outer_iter().enumerate() {
                if labels[i] == c {
                    for (j, &val) in sample.iter().enumerate() {
                        cluster_center[j] += val;
                    }
                    cluster_count += 1;
                }
            }

            if cluster_count > 0 {
                cluster_center.mapv_inplace(|x| x / cluster_count as f64);

                for (i, sample) in data.outer_iter().enumerate() {
                    if labels[i] == c {
                        let dist =
                            DistanceMetric::Euclidean.distance(&sample.to_owned(), &cluster_center);
                        within_variance += dist * dist;
                    }
                }
            }
        }

        if within_variance < 1e-10 {
            return Ok(0.0);
        }

        let ch_score = (between_variance / (n_clusters - 1) as f64)
            / (within_variance / (n_samples - n_clusters) as f64);

        Ok(ch_score)
    }
}

/// Stability tracking for consecutive clustering runs
#[derive(Clone, Debug)]
pub struct StabilityAnalysis {
    pub run_number: usize,
    pub label_agreement: f64,        // Adjusted Rand Index
    pub cluster_correspondence: f64, // Stability of cluster assignments
    pub size_consistency: f64,       // Cluster size consistency
}

impl StabilityAnalysis {
    /// Calculate stability between two clusterings
    pub fn calculate(
        previous_labels: &[usize],
        current_labels: &[usize],
    ) -> Result<StabilityAnalysis> {
        if previous_labels.len() != current_labels.len() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Label arrays must have same length".to_string(),
            ));
        }

        // Adjusted Rand Index
        let ari = Self::adjusted_rand_index(previous_labels, current_labels);

        // Cluster correspondence (percentage of samples staying in similar cluster)
        let correspondence = Self::calculate_correspondence(previous_labels, current_labels);

        // Size consistency
        let size_consistency = Self::calculate_size_consistency(previous_labels, current_labels);

        Ok(StabilityAnalysis {
            run_number: 1,
            label_agreement: ari,
            cluster_correspondence: correspondence,
            size_consistency,
        })
    }

    fn adjusted_rand_index(labels1: &[usize], labels2: &[usize]) -> f64 {
        if labels1.len() < 2 {
            return 1.0;
        }

        let n = labels1.len() as f64;

        // Check if labels are identical
        if labels1.iter().zip(labels2.iter()).all(|(a, b)| a == b) {
            return 1.0;
        }

        // Calculate agreement (Rand Index)
        let mut agreement = 0.0;
        for i in 0..labels1.len() {
            for j in (i + 1)..labels1.len() {
                let same1 = labels1[i] == labels1[j];
                let same2 = labels2[i] == labels2[j];
                if same1 == same2 {
                    agreement += 1.0;
                }
            }
        }

        let n_pairs = n * (n - 1.0) / 2.0;
        let ri = agreement / n_pairs;

        // Simplified ARI calculation: directly use RI
        (ri * 2.0 - 1.0).clamp(-1.0, 1.0)
    }

    fn calculate_correspondence(labels1: &[usize], labels2: &[usize]) -> f64 {
        let n = labels1.len();
        let agreement = labels1
            .iter()
            .zip(labels2.iter())
            .filter(|(a, b)| a == b)
            .count();

        agreement as f64 / n as f64
    }

    fn calculate_size_consistency(labels1: &[usize], labels2: &[usize]) -> f64 {
        let mut sizes1: HashMap<usize, usize> = HashMap::new();
        let mut sizes2: HashMap<usize, usize> = HashMap::new();

        for &label in labels1 {
            *sizes1.entry(label).or_insert(0) += 1;
        }

        for &label in labels2 {
            *sizes2.entry(label).or_insert(0) += 1;
        }

        let mut size_diffs = 0.0;
        let max_cluster = std::cmp::max(
            sizes1.keys().max().copied().unwrap_or(0),
            sizes2.keys().max().copied().unwrap_or(0),
        );

        for c in 0..=max_cluster {
            let s1 = sizes1.get(&c).copied().unwrap_or(0) as f64;
            let s2 = sizes2.get(&c).copied().unwrap_or(0) as f64;
            size_diffs += (s1 - s2).abs();
        }

        1.0 - (size_diffs / labels1.len() as f64)
    }
}

/// Complete quality assessment
pub struct QualityAssessment;

impl QualityAssessment {
    /// Generate comprehensive quality report
    pub fn assess(
        data: &Array2<f64>,
        labels: &[usize],
        centers: &Array2<f64>,
    ) -> Result<QualityReport> {
        let silhouette = SilhouetteMetric::calculate(data, labels)?;
        let davies_bouldin = DaviesBouldinMetric::calculate(data, labels, centers)?;
        let calinski_harabasz = CalinskiHarabaszMetric::calculate(data, labels)?;

        let n_clusters = *labels.iter().max().unwrap_or(&0) + 1;

        // Calculate inertia
        let mut inertia = 0.0;
        for (i, sample) in data.outer_iter().enumerate() {
            let center = centers.slice(s![labels[i], ..]);
            let dist = DistanceMetric::Euclidean.distance(&sample.to_owned(), &center.to_owned());
            inertia += dist * dist;
        }

        // Normalize scores to 0-100
        let silhouette_normalized = (silhouette + 1.0) / 2.0 * 100.0;
        let db_normalized = (1.0 - (davies_bouldin / 10.0).min(1.0)) * 100.0;
        let ch_normalized = (calinski_harabasz / 100.0).min(100.0);

        let overall_score = (silhouette_normalized + db_normalized + ch_normalized) / 3.0;

        Ok(QualityReport {
            silhouette_score: silhouette,
            davies_bouldin_score: davies_bouldin,
            calinski_harabasz_score: calinski_harabasz,
            inertia,
            n_clusters,
            overall_score,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silhouette_metric() {
        let data = Array2::from_shape_vec(
            (6, 2),
            vec![0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0],
        )
        .unwrap();
        let labels = vec![0, 0, 0, 1, 1, 1];

        let score = SilhouetteMetric::calculate(&data, &labels).unwrap();
        assert!((-1.0..=1.0).contains(&score));
    }

    #[test]
    fn test_davies_bouldin_metric() {
        let data = Array2::from_shape_vec(
            (6, 2),
            vec![0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0],
        )
        .unwrap();
        let labels = vec![0, 0, 0, 1, 1, 1];
        let centers = Array2::from_shape_vec((2, 2), vec![0.5, 0.5, 5.5, 5.5]).unwrap();

        let score = DaviesBouldinMetric::calculate(&data, &labels, &centers).unwrap();
        assert!(score >= 0.0);
    }

    #[test]
    fn test_calinski_harabasz_metric() {
        let data = Array2::from_shape_vec(
            (6, 2),
            vec![0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0],
        )
        .unwrap();
        let labels = vec![0, 0, 0, 1, 1, 1];

        let score = CalinskiHarabaszMetric::calculate(&data, &labels).unwrap();
        assert!(score > 0.0);
    }

    #[test]
    fn test_stability_analysis() {
        let prev = vec![0, 0, 0, 1, 1, 1];
        let curr = vec![0, 0, 0, 1, 1, 1];

        let stability = StabilityAnalysis::calculate(&prev, &curr).unwrap();
        assert_eq!(stability.label_agreement, 1.0);
        assert_eq!(stability.cluster_correspondence, 1.0);
    }

    #[test]
    fn test_quality_assessment() {
        let data = Array2::from_shape_vec(
            (6, 2),
            vec![0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0],
        )
        .unwrap();
        let labels = vec![0, 0, 0, 1, 1, 1];
        let centers = Array2::from_shape_vec((2, 2), vec![0.5, 0.5, 5.5, 5.5]).unwrap();

        let report = QualityAssessment::assess(&data, &labels, &centers).unwrap();
        assert!(report.overall_score >= 0.0 && report.overall_score <= 100.0);
        assert_eq!(report.n_clusters, 2);
    }
}
