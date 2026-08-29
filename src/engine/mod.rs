//! Core segmentation engine

pub mod activation;
pub mod activation_orchestrator;
pub mod algorithms;
pub mod b2b_governance;
pub mod b2b_segmentation;
pub mod behavioral;
pub mod churn_prediction;
pub mod clustering;
pub mod clv;
pub mod cohorts;
pub mod dashboard;
pub mod drift_detection;
pub mod governance;
pub mod heuristic_score_estimator;
pub mod k_estimation;
pub mod lifecycle;
pub mod lookalike;
pub mod metrics;
pub mod neural_networks;
pub mod pattern_discovery;
pub mod platform_adapters;
pub mod plugins;
pub mod price_intelligence;
pub mod privacy;
pub mod profiling;
pub mod quality_metrics;
pub mod revenue_intelligence;
pub mod rfm;
pub mod segment_intelligence;
pub mod segments;
pub mod sql_export;
pub mod streaming;
pub mod temporal_analytics;

use crate::Result;
use ndarray::Array2;

/// Main segmentation engine configuration
#[derive(Clone, Debug)]
pub struct SegmenterConfig {
    pub method: String,
    pub n_clusters: usize,
    pub rfm_config: rfm::RFMConfig,
    pub clustering_method: clustering::ClusteringMethod,
    pub random_state: u64,
    pub n_jobs: i32,
}

/// Core audience segmenter implementation
pub struct AudienceSegmenterCore {
    pub config: SegmenterConfig,
    pub cluster_centers: Option<Array2<f64>>,
    pub cluster_labels: Option<Vec<usize>>,
    pub normalization_params: Option<NormalizationParams>,
}

/// Normalization parameters for RFM features
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NormalizationParams {
    pub recency_min: f64,
    pub recency_max: f64,
    pub frequency_min: f64,
    pub frequency_max: f64,
    pub monetary_min: f64,
    pub monetary_max: f64,
}

impl AudienceSegmenterCore {
    /// Create a new segmenter instance
    pub fn new(config: SegmenterConfig) -> Self {
        Self {
            config,
            cluster_centers: None,
            cluster_labels: None,
            normalization_params: None,
        }
    }

    /// Default max iterations for the underlying Lloyd's-algorithm loop,
    /// matching scikit-learn's KMeans default.
    const DEFAULT_MAX_ITER: usize = 300;

    /// Fit the segmenter on data: runs the configured clustering method and
    /// stores the resulting centers/labels for later use by `predict`.
    pub fn fit(&mut self, data: &Array2<f64>) -> Result<()> {
        match self.config.clustering_method {
            clustering::ClusteringMethod::KMeans | clustering::ClusteringMethod::KMeansOnly => {
                let result = clustering::kmeans(
                    data,
                    self.config.n_clusters,
                    Self::DEFAULT_MAX_ITER,
                    self.config.random_state,
                    self.config.n_jobs,
                )?;
                self.cluster_centers = Some(result.centers);
                self.cluster_labels = Some(result.labels);
            }
            clustering::ClusteringMethod::KPrototypes => {
                // Numeric-only path: AudienceSegmenterCore's fit() signature
                // doesn't currently carry categorical feature data, so this
                // runs K-Prototypes in numeric-only mode (equivalent to
                // KMeans). Categorical support needs a richer input type
                // than a single Array2<f64> — tracked as a follow-up rather
                // than silently ignored.
                let labels = clustering::kprototypes(
                    data,
                    None,
                    self.config.n_clusters,
                    Self::DEFAULT_MAX_ITER,
                    self.config.random_state,
                    self.config.n_jobs,
                )?;
                let centers = {
                    let dim = data.ncols();
                    let mut centers = Array2::zeros((self.config.n_clusters, dim));
                    let mut counts = vec![0usize; self.config.n_clusters];
                    for (i, &label) in labels.iter().enumerate() {
                        counts[label] += 1;
                        let mut row = centers.row_mut(label);
                        row += &data.row(i);
                    }
                    for (c, &count) in counts.iter().enumerate() {
                        if count > 0 {
                            let mut row = centers.row_mut(c);
                            row /= count as f64;
                        }
                    }
                    centers
                };
                self.cluster_centers = Some(centers);
                self.cluster_labels = Some(labels);
            }
        }
        Ok(())
    }

    /// Predict cluster assignments for new data using the centers learned
    /// by a prior call to `fit`.
    pub fn predict(&self, data: &Array2<f64>) -> Result<Vec<usize>> {
        let centers = self.cluster_centers.as_ref().ok_or_else(|| {
            crate::ClusterClusterAudienceKitError::ClusteringError(
                "predict() called before fit() — no cluster centers available".to_string(),
            )
        })?;
        clustering::assign_to_clusters(data, centers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmenter_creation() {
        let config = SegmenterConfig {
            method: "rfm_kmeans".to_string(),
            n_clusters: 4,
            rfm_config: rfm::RFMConfig::default(),
            clustering_method: clustering::ClusteringMethod::KMeans,
            random_state: 42,
            n_jobs: -1,
        };

        let _segmenter = AudienceSegmenterCore::new(config);
    }

    fn make_config(n_clusters: usize, method: clustering::ClusteringMethod) -> SegmenterConfig {
        SegmenterConfig {
            method: "rfm_kmeans".to_string(),
            n_clusters,
            rfm_config: rfm::RFMConfig::default(),
            clustering_method: method,
            random_state: 42,
            n_jobs: -1,
        }
    }

    #[test]
    fn predict_before_fit_returns_a_clear_error_not_an_empty_result() {
        let segmenter =
            AudienceSegmenterCore::new(make_config(3, clustering::ClusteringMethod::KMeans));
        let data = ndarray::array![[1.0, 1.0]];
        let err = segmenter.predict(&data).unwrap_err();
        assert!(err.to_string().contains("predict() called before fit()"));
    }

    #[test]
    fn fit_then_predict_round_trips_through_real_kmeans() {
        let mut segmenter =
            AudienceSegmenterCore::new(make_config(2, clustering::ClusteringMethod::KMeans));
        let data = ndarray::array![
            [1.0, 1.0],
            [1.1, 0.9],
            [0.9, 1.1],
            [50.0, 50.0],
            [50.1, 49.9],
            [49.9, 50.1],
        ];
        segmenter.fit(&data).unwrap();

        assert!(segmenter.cluster_centers.is_some());
        let labels = segmenter.cluster_labels.as_ref().unwrap();
        assert_eq!(labels.len(), 6);
        assert!(labels[0] == labels[1] && labels[1] == labels[2]);
        assert!(labels[3] == labels[4] && labels[4] == labels[5]);
        assert_ne!(labels[0], labels[3]);

        // predict() on the same data should reproduce the fitted labels.
        let predicted = segmenter.predict(&data).unwrap();
        assert_eq!(predicted, *labels);

        // predict() on a brand-new point near the first blob should land
        // in that blob's cluster.
        let new_point = ndarray::array![[1.05, 0.95]];
        let new_labels = segmenter.predict(&new_point).unwrap();
        assert_eq!(new_labels[0], labels[0]);
    }

    #[test]
    fn fit_with_kprototypes_falls_back_to_numeric_only_and_still_produces_centers() {
        let mut segmenter =
            AudienceSegmenterCore::new(make_config(2, clustering::ClusteringMethod::KPrototypes));
        let data = ndarray::array![[1.0, 1.0], [1.1, 0.9], [20.0, 20.0], [20.1, 19.9]];
        segmenter.fit(&data).unwrap();
        assert!(segmenter.cluster_centers.is_some());
        assert_eq!(segmenter.cluster_labels.as_ref().unwrap().len(), 4);
    }

    /// `n_jobs` is a real, load-bearing config field: it controls the size of
    /// the scoped rayon thread pool `clustering::kmeans`/`kprototypes` build
    /// and run under (see `clustering::build_thread_pool`). It must never
    /// change what `fit`/`predict` compute -- only how many threads do the
    /// computing. Exercise n_jobs=1 (single-threaded) and n_jobs=2 (capped)
    /// end-to-end through the real segmenter and confirm both construct,
    /// fit, and predict without error, and agree with the n_jobs=-1
    /// ("use all cores") baseline on actual results.
    #[test]
    fn segmenter_with_explicit_n_jobs_constructs_fits_and_predicts_correctly() {
        let data = ndarray::array![
            [1.0, 1.0],
            [1.1, 0.9],
            [0.9, 1.1],
            [50.0, 50.0],
            [50.1, 49.9],
            [49.9, 50.1],
        ];

        let mut baseline_config = make_config(2, clustering::ClusteringMethod::KMeans);
        baseline_config.n_jobs = -1;
        let mut baseline = AudienceSegmenterCore::new(baseline_config);
        baseline.fit(&data).unwrap();
        let baseline_labels = baseline.cluster_labels.clone().unwrap();

        for n_jobs in [1, 2] {
            let mut config = make_config(2, clustering::ClusteringMethod::KMeans);
            config.n_jobs = n_jobs;
            let mut segmenter = AudienceSegmenterCore::new(config);

            segmenter.fit(&data).unwrap();
            assert!(segmenter.cluster_centers.is_some());
            let labels = segmenter.cluster_labels.as_ref().unwrap();
            assert_eq!(labels.len(), 6);
            assert!(labels[0] == labels[1] && labels[1] == labels[2]);
            assert!(labels[3] == labels[4] && labels[4] == labels[5]);
            assert_ne!(labels[0], labels[3]);
            assert_eq!(
                labels, &baseline_labels,
                "n_jobs={n_jobs} produced different cluster assignments than n_jobs=-1"
            );

            let predicted = segmenter.predict(&data).unwrap();
            assert_eq!(&predicted, labels);
        }
    }
}
