//! Cluster quality metrics

use crate::{ClusterClusterAudienceKitError, Result};
use ndarray::Array2;
use std::collections::HashMap;

fn euclidean(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

fn group_indices_by_label(labels: &[usize]) -> HashMap<usize, Vec<usize>> {
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, &label) in labels.iter().enumerate() {
        groups.entry(label).or_default().push(i);
    }
    groups
}

/// Silhouette score (Rousseeuw, 1987): for each point i, a(i) is the mean
/// distance to other points in its own cluster, b(i) is the mean distance to
/// points in the nearest *other* cluster, and s(i) = (b-a) / max(a,b).
/// The overall score is the mean of s(i) over all points, in [-1, 1] —
/// higher means better-separated, more cohesive clusters.
pub fn silhouette_score(data: &Array2<f64>, labels: &[usize]) -> Result<f64> {
    let n = data.nrows();
    if n != labels.len() {
        return Err(ClusterClusterAudienceKitError::DataValidation(
            "data row count doesn't match labels length".to_string(),
        ));
    }
    let groups = group_indices_by_label(labels);
    if groups.len() < 2 {
        return Err(ClusterClusterAudienceKitError::ClusteringError(
            "silhouette score requires at least 2 distinct clusters".to_string(),
        ));
    }

    let mut total = 0.0;
    for i in 0..n {
        let own_label = labels[i];
        let own_group = &groups[&own_label];

        let a_i = if own_group.len() <= 1 {
            0.0 // singleton cluster: silhouette contribution is 0 by convention
        } else {
            let sum: f64 = own_group
                .iter()
                .filter(|&&j| j != i)
                .map(|&j| euclidean(data.row(i).as_slice().unwrap(), data.row(j).as_slice().unwrap()))
                .sum();
            sum / (own_group.len() - 1) as f64
        };

        let b_i = groups
            .iter()
            .filter(|(&label, _)| label != own_label)
            .map(|(_, indices)| {
                let sum: f64 = indices
                    .iter()
                    .map(|&j| euclidean(data.row(i).as_slice().unwrap(), data.row(j).as_slice().unwrap()))
                    .sum();
                sum / indices.len() as f64
            })
            .fold(f64::INFINITY, f64::min);

        let s_i = if a_i.max(b_i) == 0.0 { 0.0 } else { (b_i - a_i) / a_i.max(b_i) };
        total += s_i;
    }

    Ok(total / n as f64)
}

/// Davies-Bouldin score (Davies & Bouldin, 1979): for each pair of clusters,
/// compute (spread_i + spread_j) / distance_between_centers_ij, then average
/// the worst (maximum) such ratio for each cluster over all clusters. Lower
/// is better (0 is the best possible score); unlike silhouette this is
/// unbounded above.
pub fn davies_bouldin_score(data: &Array2<f64>, labels: &[usize], centers: &Array2<f64>) -> Result<f64> {
    let n = data.nrows();
    if n != labels.len() {
        return Err(ClusterClusterAudienceKitError::DataValidation(
            "data row count doesn't match labels length".to_string(),
        ));
    }
    let groups = group_indices_by_label(labels);
    let k = centers.nrows();
    if k < 2 {
        return Err(ClusterClusterAudienceKitError::ClusteringError(
            "Davies-Bouldin score requires at least 2 clusters".to_string(),
        ));
    }

    // Average within-cluster distance to center ("spread"/scatter) per cluster.
    let mut spread = vec![0.0; k];
    for c in 0..k {
        if let Some(indices) = groups.get(&c) {
            if !indices.is_empty() {
                let sum: f64 = indices
                    .iter()
                    .map(|&i| euclidean(data.row(i).as_slice().unwrap(), centers.row(c).as_slice().unwrap()))
                    .sum();
                spread[c] = sum / indices.len() as f64;
            }
        }
    }

    let mut total = 0.0;
    for i in 0..k {
        let worst = (0..k)
            .filter(|&j| j != i)
            .map(|j| {
                let center_dist = euclidean(centers.row(i).as_slice().unwrap(), centers.row(j).as_slice().unwrap());
                if center_dist == 0.0 {
                    f64::INFINITY // coincident centers: maximally bad separation
                } else {
                    (spread[i] + spread[j]) / center_dist
                }
            })
            .fold(f64::NEG_INFINITY, f64::max);
        total += worst;
    }

    Ok(total / k as f64)
}

/// Inertia: sum of squared distances from each point to its assigned
/// cluster center. This is KMeans' own objective function — lower is a
/// tighter fit (though trivially minimized by k == n, so it's meaningful
/// for comparing runs at the same k, not for choosing k directly).
pub fn inertia(data: &Array2<f64>, labels: &[usize], centers: &Array2<f64>) -> Result<f64> {
    let n = data.nrows();
    if n != labels.len() {
        return Err(ClusterClusterAudienceKitError::DataValidation(
            "data row count doesn't match labels length".to_string(),
        ));
    }
    let mut total = 0.0;
    for i in 0..n {
        let label = labels[i];
        if label >= centers.nrows() {
            return Err(ClusterClusterAudienceKitError::DataValidation(format!(
                "label {label} at row {i} has no corresponding center (centers has {} rows)",
                centers.nrows()
            )));
        }
        let d: f64 = data
            .row(i)
            .iter()
            .zip(centers.row(label).iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum();
        total += d;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    /// Fixed dataset/labels/centers, independently cross-checked against
    /// sklearn.metrics.silhouette_score / davies_bouldin_score on the exact
    /// same input (see commit message / PR description for the Python
    /// snippet used to generate these reference values).
    fn fixed_case() -> (Array2<f64>, Vec<usize>, Array2<f64>) {
        let data = array![
            [1.0, 1.0], [1.5, 2.0], [1.2, 1.8],
            [8.0, 8.0], [8.5, 8.2], [8.1, 7.9],
            [1.0, 8.0], [1.3, 8.4],
        ];
        let labels = vec![0, 0, 0, 1, 1, 1, 2, 2];
        let centers = array![
            [1.2333333333333334, 1.5999999999999999],
            [8.200000000000001, 8.033333333333333],
            [1.15, 8.2],
        ];
        (data, labels, centers)
    }

    #[test]
    fn silhouette_matches_sklearn_reference_value() {
        let (data, labels, _) = fixed_case();
        let score = silhouette_score(&data, &labels).unwrap();
        assert!((score - 0.9169705098274541).abs() < 1e-9, "got {score}");
    }

    #[test]
    fn davies_bouldin_matches_sklearn_reference_value() {
        let (data, labels, centers) = fixed_case();
        let score = davies_bouldin_score(&data, &labels, &centers).unwrap();
        assert!((score - 0.093838016058385).abs() < 1e-9, "got {score}");
    }

    #[test]
    fn inertia_matches_reference_value() {
        let (data, labels, centers) = fixed_case();
        let score = inertia(&data, &labels, &centers).unwrap();
        assert!((score - 0.9983333333333331).abs() < 1e-9, "got {score}");
    }

    #[test]
    fn silhouette_requires_at_least_two_clusters() {
        let data = array![[1.0, 1.0], [2.0, 2.0]];
        let labels = vec![0, 0];
        assert!(silhouette_score(&data, &labels).is_err());
    }

    #[test]
    fn silhouette_is_near_one_for_extremely_well_separated_clusters() {
        let data = array![[0.0, 0.0], [0.01, 0.01], [100.0, 100.0], [100.01, 100.01]];
        let labels = vec![0, 0, 1, 1];
        let score = silhouette_score(&data, &labels).unwrap();
        assert!(score > 0.99, "expected near-perfect separation, got {score}");
    }

    #[test]
    fn davies_bouldin_is_near_zero_for_extremely_well_separated_tight_clusters() {
        let data = array![[0.0, 0.0], [0.001, 0.001], [100.0, 100.0], [100.001, 100.001]];
        let labels = vec![0, 0, 1, 1];
        let centers = array![[0.0005, 0.0005], [100.0005, 100.0005]];
        let score = davies_bouldin_score(&data, &labels, &centers).unwrap();
        assert!(score < 0.001, "expected near-zero for tight well-separated clusters, got {score}");
    }

    #[test]
    fn inertia_is_zero_when_every_point_sits_exactly_on_its_center() {
        let data = array![[1.0, 1.0], [5.0, 5.0]];
        let labels = vec![0, 1];
        let centers = array![[1.0, 1.0], [5.0, 5.0]];
        assert_eq!(inertia(&data, &labels, &centers).unwrap(), 0.0);
    }
}
