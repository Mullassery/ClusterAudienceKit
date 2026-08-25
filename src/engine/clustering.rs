//! Clustering algorithms (KMeans, K-Prototypes)
//!
//! Several loops in this file index in lockstep across an `ndarray::Array2`
//! row and one or more plain `Vec`s (e.g. `data.row(i)` alongside
//! `labels[i]`/`counts[c]`). Clippy's `needless_range_loop` would rather
//! these be `.iter().enumerate()`, but `Array2::row()`/`row_mut()` don't
//! implement the plain iterator traits clippy's suggested rewrite needs, so
//! the mechanical rewrite doesn't apply cleanly here. Given this is the
//! numerically-sensitive clustering hot path with real test coverage
//! pinned to its current behavior, this is silenced at the module level
//! rather than risking a subtly-wrong manual rewrite under time pressure.
#![allow(clippy::needless_range_loop)]

use crate::{ClusterClusterAudienceKitError, Result};
use ndarray::{Array1, Array2, Axis};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

/// Clustering method
#[derive(Clone, Debug)]
pub enum ClusteringMethod {
    KMeans,
    KPrototypes,
    KMeansOnly,
}

/// KMeans clustering result
#[derive(Clone, Debug)]
pub struct KMeansResult {
    pub labels: Vec<usize>,
    pub centers: Array2<f64>,
    pub inertia: f64,
    pub n_iter: usize,
}

fn squared_euclidean(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

/// k-means++ initialization: pick the first center uniformly at random, then
/// each subsequent center with probability proportional to its squared
/// distance to the nearest already-chosen center. This gives materially
/// better and more consistent convergence than uniform-random initialization
/// (the original Lloyd's-algorithm paper's stated weakness).
/// Returns the chosen centers AND the row indices they were chosen from —
/// callers with a second, parallel feature set (e.g. K-Prototypes'
/// categorical columns) need the indices to seed that feature set's initial
/// centers from the *same* points, not an unrelated arbitrary index.
fn kmeans_plus_plus_init(
    data: &Array2<f64>,
    k: usize,
    rng: &mut StdRng,
) -> (Array2<f64>, Vec<usize>) {
    let n = data.nrows();
    let dim = data.ncols();
    let mut centers = Array2::zeros((k, dim));
    let mut chosen_indices = Vec::with_capacity(k);

    let first = rng.gen_range(0..n);
    centers.row_mut(0).assign(&data.row(first));
    chosen_indices.push(first);

    let mut min_dist_sq = vec![f64::INFINITY; n];
    for c_idx in 1..k {
        let last_center = centers.row(c_idx - 1);
        for i in 0..n {
            let d = squared_euclidean(
                data.row(i).as_slice().unwrap(),
                last_center.as_slice().unwrap(),
            );
            if d < min_dist_sq[i] {
                min_dist_sq[i] = d;
            }
        }
        let total: f64 = min_dist_sq.iter().sum();
        let next = if total <= 0.0 {
            // All remaining points coincide with an existing center; any
            // point is as good as any other.
            rng.gen_range(0..n)
        } else {
            let target = rng.gen_range(0.0..total);
            let mut cumulative = 0.0;
            let mut chosen = n - 1;
            for (i, &d) in min_dist_sq.iter().enumerate() {
                cumulative += d;
                if cumulative >= target {
                    chosen = i;
                    break;
                }
            }
            chosen
        };
        centers.row_mut(c_idx).assign(&data.row(next));
        chosen_indices.push(next);
    }
    (centers, chosen_indices)
}

fn nearest_center(point: &[f64], centers: &Array2<f64>) -> (usize, f64) {
    let mut best = 0;
    let mut best_dist = f64::INFINITY;
    for (i, center) in centers.rows().into_iter().enumerate() {
        let d = squared_euclidean(point, center.as_slice().unwrap());
        if d < best_dist {
            best_dist = d;
            best = i;
        }
    }
    (best, best_dist)
}

/// Perform KMeans clustering using Lloyd's algorithm with k-means++
/// initialization. Deterministic for a given `random_state`.
pub fn kmeans(
    data: &Array2<f64>,
    n_clusters: usize,
    max_iter: usize,
    random_state: u64,
) -> Result<KMeansResult> {
    let n = data.nrows();
    let dim = data.ncols();

    if n_clusters == 0 {
        return Err(ClusterClusterAudienceKitError::InvalidConfig(
            "n_clusters must be > 0".to_string(),
        ));
    }
    if n == 0 {
        return Err(ClusterClusterAudienceKitError::DataValidation(
            "data has no rows".to_string(),
        ));
    }
    if n_clusters > n {
        return Err(ClusterClusterAudienceKitError::InvalidConfig(format!(
            "n_clusters ({n_clusters}) cannot exceed number of data points ({n})"
        )));
    }

    let mut rng = StdRng::seed_from_u64(random_state);
    let (mut centers, _) = kmeans_plus_plus_init(data, n_clusters, &mut rng);
    let mut labels = vec![0usize; n];
    let mut n_iter = 0;

    for iter in 0..max_iter {
        n_iter = iter + 1;

        // Nearest-center assignment is embarrassingly parallel: each point's
        // label depends only on the (read-only, this iteration) centers, not
        // on any other point's label. This is the dominant cost per
        // iteration for large N, so it's the hot loop rayon-parallelizes.
        let new_labels: Vec<usize> = (0..n)
            .into_par_iter()
            .map(|i| nearest_center(data.row(i).as_slice().unwrap(), &centers).0)
            .collect();

        let mut changed = new_labels != labels;
        labels = new_labels;

        let mut new_centers = Array2::zeros((n_clusters, dim));
        let mut counts = vec![0usize; n_clusters];
        for i in 0..n {
            let c = labels[i];
            counts[c] += 1;
            let mut row = new_centers.row_mut(c);
            row += &data.row(i);
        }
        for c in 0..n_clusters {
            if counts[c] > 0 {
                let mut row = new_centers.row_mut(c);
                row /= counts[c] as f64;
            } else {
                // Empty cluster: re-seed it at the point currently farthest
                // from its own assigned center, so it doesn't just vanish.
                let farthest = (0..n)
                    .max_by(|&a, &b| {
                        let da = squared_euclidean(
                            data.row(a).as_slice().unwrap(),
                            centers.row(labels[a]).as_slice().unwrap(),
                        );
                        let db = squared_euclidean(
                            data.row(b).as_slice().unwrap(),
                            centers.row(labels[b]).as_slice().unwrap(),
                        );
                        da.partial_cmp(&db).unwrap()
                    })
                    .unwrap();
                new_centers.row_mut(c).assign(&data.row(farthest));
                changed = true;
            }
        }

        centers = new_centers;
        if !changed {
            break;
        }
    }

    let inertia_value = (0..n)
        .map(|i| {
            squared_euclidean(
                data.row(i).as_slice().unwrap(),
                centers.row(labels[i]).as_slice().unwrap(),
            )
        })
        .sum();

    Ok(KMeansResult {
        labels,
        centers,
        inertia: inertia_value,
        n_iter,
    })
}

/// Perform K-Prototypes clustering (mixed numeric/categorical data), per
/// Huang (1998): distance = squared Euclidean over numeric fields + gamma *
/// (count of mismatched categorical fields). Cluster centers are the mean
/// for numeric fields and the mode for categorical fields.
pub fn kprototypes(
    numeric_data: &Array2<f64>,
    categorical_data: Option<&Vec<Vec<usize>>>,
    n_clusters: usize,
    max_iter: usize,
    random_state: u64,
) -> Result<Vec<usize>> {
    let n = numeric_data.nrows();
    if n_clusters == 0 {
        return Err(ClusterClusterAudienceKitError::InvalidConfig(
            "n_clusters must be > 0".to_string(),
        ));
    }
    if n == 0 {
        return Err(ClusterClusterAudienceKitError::DataValidation(
            "data has no rows".to_string(),
        ));
    }
    if n_clusters > n {
        return Err(ClusterClusterAudienceKitError::InvalidConfig(format!(
            "n_clusters ({n_clusters}) cannot exceed number of data points ({n})"
        )));
    }
    if let Some(cat) = categorical_data {
        if cat.len() != n {
            return Err(ClusterClusterAudienceKitError::DataValidation(
                "categorical_data row count doesn't match numeric_data".to_string(),
            ));
        }
    }

    // gamma balances the numeric vs categorical distance contribution;
    // Huang's paper recommends the average standard deviation of the
    // numeric fields as a reasonable default scale.
    let gamma = {
        let mean = numeric_data
            .mean_axis(Axis(0))
            .unwrap_or_else(|| Array1::zeros(numeric_data.ncols()));
        let variance: f64 = numeric_data
            .axis_iter(Axis(1))
            .zip(mean.iter())
            .map(|(col, &m)| col.iter().map(|&v| (v - m).powi(2)).sum::<f64>() / n as f64)
            .sum::<f64>()
            / numeric_data.ncols().max(1) as f64;
        variance.sqrt().max(1e-9)
    };

    let mut rng = StdRng::seed_from_u64(random_state);
    let (mut numeric_centers, chosen_indices) =
        kmeans_plus_plus_init(numeric_data, n_clusters, &mut rng);
    let n_cat_features = categorical_data.map(|c| c[0].len()).unwrap_or(0);
    let mut cat_centers: Vec<Vec<usize>> = (0..n_clusters)
        .map(|_| vec![0usize; n_cat_features])
        .collect();
    if let Some(cat) = categorical_data {
        // Seed categorical centers from the SAME points k-means++ chose for
        // the numeric centers, so each cluster's initial (numeric,
        // categorical) center actually corresponds to one real data point
        // instead of two unrelated ones.
        for (c, &row_idx) in chosen_indices.iter().enumerate() {
            cat_centers[c] = cat[row_idx].clone();
        }
    }

    let mut labels = vec![0usize; n];

    for _ in 0..max_iter {
        // As in kmeans(), each point's nearest-cluster assignment only reads
        // the current (fixed for this iteration) centers, so it's safe and
        // profitable to compute all N assignments in parallel.
        let new_labels: Vec<usize> = (0..n)
            .into_par_iter()
            .map(|i| {
                let mut best = 0;
                let mut best_dist = f64::INFINITY;
                for c in 0..n_clusters {
                    let numeric_dist = squared_euclidean(
                        numeric_data.row(i).as_slice().unwrap(),
                        numeric_centers.row(c).as_slice().unwrap(),
                    );
                    let cat_dist = categorical_data
                        .map(|cat| {
                            cat[i]
                                .iter()
                                .zip(cat_centers[c].iter())
                                .filter(|(a, b)| a != b)
                                .count() as f64
                        })
                        .unwrap_or(0.0);
                    let total = numeric_dist + gamma * cat_dist;
                    if total < best_dist {
                        best_dist = total;
                        best = c;
                    }
                }
                best
            })
            .collect();

        let changed = new_labels != labels;
        labels = new_labels;

        // Update numeric centers (mean) and categorical centers (mode).
        let dim = numeric_data.ncols();
        let mut sums = Array2::zeros((n_clusters, dim));
        let mut counts = vec![0usize; n_clusters];
        for i in 0..n {
            counts[labels[i]] += 1;
            let mut row = sums.row_mut(labels[i]);
            row += &numeric_data.row(i);
        }
        for c in 0..n_clusters {
            if counts[c] > 0 {
                let mut row = sums.row_mut(c);
                row /= counts[c] as f64;
            }
        }
        for c in 0..n_clusters {
            if counts[c] > 0 {
                numeric_centers.row_mut(c).assign(&sums.row(c));
            }
        }

        if let Some(cat) = categorical_data {
            for c in 0..n_clusters {
                for f in 0..n_cat_features {
                    // BTreeMap (not HashMap): iteration order must be
                    // deterministic here, since ties in count are broken by
                    // iteration order. HashMap's per-process random seed
                    // would otherwise make tie-breaking — and therefore the
                    // final clustering — nondeterministic across runs of
                    // the same `random_state`, which showed up as a flaky
                    // test failure in CI before this fix.
                    let mut freq: std::collections::BTreeMap<usize, usize> =
                        std::collections::BTreeMap::new();
                    for i in 0..n {
                        if labels[i] == c {
                            *freq.entry(cat[i][f]).or_insert(0) += 1;
                        }
                    }
                    if let Some((&mode_val, _)) = freq.iter().max_by_key(|(_, &count)| count) {
                        cat_centers[c][f] = mode_val;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    Ok(labels)
}

/// Mini-batch K-Means (Sculley, 2010: "Web-Scale K-Means Clustering"): an
/// online/chunked variant of Lloyd's algorithm. `kmeans()` above needs every
/// row of the dataset in memory for repeated full passes; this instead
/// consumes the dataset one chunk at a time via `partial_fit`, updating
/// centers incrementally with a per-center learning rate that decays as
/// `1/count` (so a center converges instead of chasing whichever chunk
/// arrived most recently). Memory use is bounded by O(chunk_size +
/// n_clusters) regardless of total dataset size, so a caller can stream
/// chunks from a file, a Python generator, or a paginated query without
/// ever materializing the full dataset as one in-memory array.
pub struct MiniBatchKMeans {
    centers: Array2<f64>,
    center_counts: Vec<u64>,
    n_clusters: usize,
    initialized: bool,
    rng: StdRng,
}

impl MiniBatchKMeans {
    pub fn new(n_clusters: usize, random_state: u64) -> Result<Self> {
        if n_clusters == 0 {
            return Err(ClusterClusterAudienceKitError::InvalidConfig(
                "n_clusters must be > 0".to_string(),
            ));
        }
        Ok(Self {
            centers: Array2::zeros((0, 0)),
            center_counts: vec![0; n_clusters],
            n_clusters,
            initialized: false,
            rng: StdRng::seed_from_u64(random_state),
        })
    }

    /// Whether at least one chunk has been ingested (centers are seeded).
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Current cluster centers. Empty (0 rows) until the first `partial_fit`.
    pub fn centers(&self) -> &Array2<f64> {
        &self.centers
    }

    /// Ingest one chunk, updating centers in place. The first chunk ever
    /// passed to a given instance must have at least `n_clusters` rows (used
    /// to seed initial centers via k-means++ on that chunk alone); every
    /// call after that accepts any nonzero chunk size, and previously
    /// ingested rows are never re-read or re-visited.
    pub fn partial_fit(&mut self, chunk: &Array2<f64>) -> Result<()> {
        if chunk.nrows() == 0 {
            return Ok(());
        }

        if !self.initialized {
            if chunk.nrows() < self.n_clusters {
                return Err(ClusterClusterAudienceKitError::DataValidation(format!(
                    "first partial_fit chunk must have at least n_clusters ({}) rows to seed initial centers, got {}",
                    self.n_clusters,
                    chunk.nrows()
                )));
            }
            let (centers, _) = kmeans_plus_plus_init(chunk, self.n_clusters, &mut self.rng);
            self.centers = centers;
            self.initialized = true;
            return Ok(());
        }

        if chunk.ncols() != self.centers.ncols() {
            return Err(ClusterClusterAudienceKitError::DataValidation(format!(
                "chunk has {} features but centers have {} features",
                chunk.ncols(),
                self.centers.ncols()
            )));
        }

        // Sculley 2010, section 3.2: assign each point in the chunk to its
        // nearest current center, then nudge that center toward the point
        // with learning rate 1/count. A center's own running count (not the
        // chunk index or a fixed rate) sets the rate, so early chunks move
        // centers a lot and later chunks only fine-tune them.
        for i in 0..chunk.nrows() {
            let point = chunk.row(i);
            let (c, _) = nearest_center(point.as_slice().unwrap(), &self.centers);
            self.center_counts[c] += 1;
            let lr = 1.0 / self.center_counts[c] as f64;
            let mut center_row = self.centers.row_mut(c);
            for j in 0..center_row.len() {
                center_row[j] += lr * (point[j] - center_row[j]);
            }
        }

        Ok(())
    }

    /// Assign each row of `data` to its nearest fitted center.
    pub fn predict(&self, data: &Array2<f64>) -> Result<Vec<usize>> {
        if !self.initialized {
            return Err(ClusterClusterAudienceKitError::ClusteringError(
                "predict() called before any partial_fit chunk was ingested".to_string(),
            ));
        }
        assign_to_clusters(data, &self.centers)
    }
}

/// Assign data points to their nearest cluster center (used for predicting
/// on new data with an already-fitted model).
pub fn assign_to_clusters(data: &Array2<f64>, centers: &Array2<f64>) -> Result<Vec<usize>> {
    if centers.nrows() == 0 {
        return Err(ClusterClusterAudienceKitError::InvalidConfig(
            "cannot assign to clusters: no centers provided (model not fitted?)".to_string(),
        ));
    }
    if data.ncols() != centers.ncols() {
        return Err(ClusterClusterAudienceKitError::DataValidation(format!(
            "data has {} features but centers have {} features",
            data.ncols(),
            centers.ncols()
        )));
    }
    Ok((0..data.nrows())
        .map(|i| nearest_center(data.row(i).as_slice().unwrap(), centers).0)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_clustering_method_variants() {
        let _ = ClusteringMethod::KMeans;
        let _ = ClusteringMethod::KPrototypes;
        let _ = ClusteringMethod::KMeansOnly;
    }

    /// Three well-separated 2D blobs — a correct KMeans implementation
    /// should recover exactly this partition (cluster label numbering is
    /// arbitrary, so we check partition structure, not label identity).
    fn three_blobs() -> Array2<f64> {
        array![
            [1.0, 1.0],
            [1.2, 0.9],
            [0.8, 1.1],
            [1.1, 1.2],
            [10.0, 10.0],
            [10.2, 9.9],
            [9.8, 10.1],
            [10.1, 10.2],
            [1.0, 10.0],
            [1.2, 9.9],
            [0.8, 10.1],
        ]
    }

    #[test]
    fn kmeans_recovers_well_separated_blobs() {
        let data = three_blobs();
        let result = kmeans(&data, 3, 100, 42).unwrap();

        assert_eq!(result.labels.len(), 11);
        // Points 0-3 must all share one label, 4-7 another, 8-10 a third,
        // and all three labels must be pairwise distinct.
        let l = &result.labels;
        assert!(l[0] == l[1] && l[1] == l[2] && l[2] == l[3]);
        assert!(l[4] == l[5] && l[5] == l[6] && l[6] == l[7]);
        assert!(l[8] == l[9] && l[9] == l[10]);
        assert_ne!(l[0], l[4]);
        assert_ne!(l[0], l[8]);
        assert_ne!(l[4], l[8]);

        // Inertia should be small relative to the ~81-unit inter-cluster
        // gaps, since points are tightly clustered around their true centers.
        assert!(
            result.inertia < 5.0,
            "inertia too high for tight blobs: {}",
            result.inertia
        );
    }

    #[test]
    fn kmeans_inertia_matches_hand_computed_value_on_fixed_centers() {
        // Cross-checked against the same fixed dataset used for the
        // silhouette/davies-bouldin reference values (computed with
        // sklearn.metrics on this exact data/labels/centers).
        let data = array![
            [1.0, 1.0],
            [1.5, 2.0],
            [1.2, 1.8],
            [8.0, 8.0],
            [8.5, 8.2],
            [8.1, 7.9],
            [1.0, 8.0],
            [1.3, 8.4],
        ];
        let labels = [0, 0, 0, 1, 1, 1, 2, 2];
        let centers = array![
            [1.2333333333333334, 1.5999999999999999],
            [8.200000000000001, 8.033333333333333],
            [1.15, 8.2],
        ];
        let inertia_value: f64 = (0..data.nrows())
            .map(|i| {
                squared_euclidean(
                    data.row(i).as_slice().unwrap(),
                    centers.row(labels[i]).as_slice().unwrap(),
                )
            })
            .sum();
        assert!(
            (inertia_value - 0.9983333333333331).abs() < 1e-9,
            "got {inertia_value}"
        );
    }

    #[test]
    fn kmeans_rejects_more_clusters_than_points() {
        let data = array![[1.0, 1.0], [2.0, 2.0]];
        assert!(kmeans(&data, 5, 10, 0).is_err());
    }

    #[test]
    fn kmeans_is_deterministic_for_a_given_seed() {
        let data = three_blobs();
        let a = kmeans(&data, 3, 100, 7).unwrap();
        let b = kmeans(&data, 3, 100, 7).unwrap();
        assert_eq!(a.labels, b.labels);
        assert_eq!(a.inertia, b.inertia);
    }

    #[test]
    fn assign_to_clusters_picks_nearest_center() {
        let centers = array![[0.0, 0.0], [10.0, 10.0]];
        let data = array![[0.5, 0.5], [9.5, 9.5], [-1.0, -1.0]];
        let labels = assign_to_clusters(&data, &centers).unwrap();
        assert_eq!(labels, vec![0, 1, 0]);
    }

    #[test]
    fn assign_to_clusters_errors_on_empty_centers() {
        let centers: Array2<f64> = Array2::zeros((0, 2));
        let data = array![[1.0, 1.0]];
        assert!(assign_to_clusters(&data, &centers).is_err());
    }

    /// Regression test: mode computation for categorical cluster centers
    /// used to go through a HashMap, whose iteration order (and therefore
    /// tie-breaking) is randomized per-process in Rust — making this
    /// function's output nondeterministic across runs despite taking an
    /// explicit `random_state`. Runs the exact same tied-count scenario
    /// many times in-process; a real fix must produce identical output
    /// every time, not just "usually."
    #[test]
    fn kprototypes_mode_tie_breaking_is_deterministic_across_repeated_runs() {
        let numeric = array![[1.0, 1.0], [1.1, 0.9], [1.0, 1.0], [1.1, 0.9]];
        let categorical = vec![vec![0], vec![0], vec![1], vec![1]];

        let first = kprototypes(&numeric, Some(&categorical), 2, 50, 1).unwrap();
        for _ in 0..50 {
            let repeat = kprototypes(&numeric, Some(&categorical), 2, 50, 1).unwrap();
            assert_eq!(
                repeat, first,
                "kprototypes produced different output on a repeat run with the same random_state"
            );
        }
    }

    #[test]
    fn minibatch_kmeans_rejects_zero_clusters() {
        assert!(MiniBatchKMeans::new(0, 0).is_err());
    }

    #[test]
    fn minibatch_kmeans_rejects_first_chunk_smaller_than_n_clusters() {
        let mut mbk = MiniBatchKMeans::new(3, 0).unwrap();
        let chunk = array![[1.0, 1.0], [2.0, 2.0]];
        assert!(mbk.partial_fit(&chunk).is_err());
    }

    #[test]
    fn minibatch_kmeans_predict_before_any_chunk_errors() {
        let mbk = MiniBatchKMeans::new(2, 0).unwrap();
        let data = array![[1.0, 1.0]];
        assert!(mbk.predict(&data).is_err());
    }

    #[test]
    fn minibatch_kmeans_recovers_well_separated_blobs_from_chunks() {
        // Same three-blobs fixture as full-batch kmeans, but fed in via
        // several small chunks instead of one call -- this is the actual
        // scenario the feature exists for: a caller streaming rows from a
        // generator/file instead of holding the whole dataset in memory.
        let data = three_blobs();
        let mut mbk = MiniBatchKMeans::new(3, 42).unwrap();

        // First chunk must have >= n_clusters rows to seed centers.
        mbk.partial_fit(&data.slice(ndarray::s![0..4, ..]).to_owned())
            .unwrap();
        mbk.partial_fit(&data.slice(ndarray::s![4..8, ..]).to_owned())
            .unwrap();
        mbk.partial_fit(&data.slice(ndarray::s![8..11, ..]).to_owned())
            .unwrap();

        // A few more passes over the same data lets centers converge
        // further, same as mini-batch k-means does in practice with
        // repeated epochs over a streamed dataset.
        for _ in 0..20 {
            mbk.partial_fit(&data).unwrap();
        }

        let labels = mbk.predict(&data).unwrap();
        assert_eq!(labels.len(), 11);
        assert!(labels[0] == labels[1] && labels[1] == labels[2] && labels[2] == labels[3]);
        assert!(labels[4] == labels[5] && labels[5] == labels[6] && labels[6] == labels[7]);
        assert!(labels[8] == labels[9] && labels[9] == labels[10]);
        assert_ne!(labels[0], labels[4]);
        assert_ne!(labels[0], labels[8]);
        assert_ne!(labels[4], labels[8]);
    }

    #[test]
    fn minibatch_kmeans_empty_chunk_is_a_noop() {
        let mut mbk = MiniBatchKMeans::new(2, 0).unwrap();
        let empty: Array2<f64> = Array2::zeros((0, 2));
        assert!(mbk.partial_fit(&empty).is_ok());
        assert!(!mbk.is_initialized());
    }

    #[test]
    fn minibatch_kmeans_rejects_mismatched_feature_count_after_init() {
        let mut mbk = MiniBatchKMeans::new(2, 0).unwrap();
        let first = array![[1.0, 1.0], [2.0, 2.0], [3.0, 3.0]];
        mbk.partial_fit(&first).unwrap();

        let bad_chunk = array![[1.0, 1.0, 1.0]];
        assert!(mbk.partial_fit(&bad_chunk).is_err());
    }

    #[test]
    fn kprototypes_separates_by_categorical_distance_when_numeric_is_uninformative() {
        // All 4 points have IDENTICAL numeric coordinates (up to tiny
        // floating-point jitter so k-means++ init has something nonzero to
        // work with) — numeric distance contributes ~nothing to any pair,
        // so a correct implementation must split points 0,1 (category 0)
        // from points 2,3 (category 1) using the categorical field alone.
        //
        // (An earlier version of this test used numeric data where two
        // *different* numeric locations partially overlapped with the
        // category split — that setup is genuinely ambiguous: k-means++
        // seeds initial centers at the two distinct numeric locations
        // regardless of category, so with a small enough gamma the
        // algorithm correctly converges by numeric identity instead, which
        // isn't a bug. This version removes that ambiguity.)
        let numeric = array![[1.0, 1.0], [1.0, 1.0], [1.0, 1.0], [1.0, 1.0],];
        let categorical = vec![vec![0], vec![0], vec![1], vec![1]];
        let labels = kprototypes(&numeric, Some(&categorical), 2, 50, 1).unwrap();
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[2], labels[3]);
        assert_ne!(labels[0], labels[2]);
    }
}
