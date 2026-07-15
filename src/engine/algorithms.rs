//! Multiple clustering algorithms implementation

use crate::Result;
use ndarray::{Array1, Array2, s};

/// Distance metric types
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum DistanceMetric {
    Euclidean,
    Manhattan,
    Cosine,
}

impl DistanceMetric {
    pub fn distance(&self, a: &Array1<f64>, b: &Array1<f64>) -> f64 {
        match self {
            DistanceMetric::Euclidean => {
                a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| (x - y).powi(2))
                    .sum::<f64>()
                    .sqrt()
            }
            DistanceMetric::Manhattan => {
                a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| (x - y).abs())
                    .sum()
            }
            DistanceMetric::Cosine => {
                let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
                let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    0.0
                } else {
                    1.0 - (dot_product / (norm_a * norm_b))
                }
            }
        }
    }
}

/// K-Means clustering result
#[derive(Clone, Debug)]
pub struct KMeansResult {
    pub labels: Vec<usize>,
    pub centers: Array2<f64>,
    pub inertia: f64,
    pub iterations: usize,
}

/// K-Means clustering algorithm
pub struct KMeans {
    pub n_clusters: usize,
    pub max_iterations: usize,
    pub random_state: u64,
    pub metric: DistanceMetric,
}

impl KMeans {
    pub fn new(n_clusters: usize) -> Self {
        Self {
            n_clusters,
            max_iterations: 300,
            random_state: 42,
            metric: DistanceMetric::Euclidean,
        }
    }

    pub fn fit(&self, data: &Array2<f64>) -> Result<KMeansResult> {
        let (n_samples, n_features) = data.dim();
        if n_samples == 0 || self.n_clusters == 0 || self.n_clusters > n_samples {
            return Err(crate::ClusterClusterAudienceKitError::InvalidConfig(
                "Invalid cluster count".to_string(),
            )
            .into());
        }

        // Initialize centers randomly
        let mut centers = Array2::zeros((self.n_clusters, n_features));
        let step = n_samples / self.n_clusters;
        for i in 0..self.n_clusters {
            centers
                .slice_mut(s![i, ..])
                .assign(&data.slice(s![i * step, ..]));
        }

        let mut labels = vec![0; n_samples];
        let mut prev_inertia = f64::MAX;

        for iter in 0..self.max_iterations {
            // Assign clusters
            for (i, sample) in data.outer_iter().enumerate() {
                let mut min_dist = f64::MAX;
                let mut best_cluster = 0;

                for (j, center) in centers.outer_iter().enumerate() {
                    let dist = self.metric.distance(&sample.to_owned(), &center.to_owned());
                    if dist < min_dist {
                        min_dist = dist;
                        best_cluster = j;
                    }
                }
                labels[i] = best_cluster;
            }

            // Update centers
            let mut new_centers = Array2::zeros((self.n_clusters, n_features));
            let mut counts = vec![0; self.n_clusters];

            for (i, sample) in data.outer_iter().enumerate() {
                let cluster = labels[i];
                let mut center_slice = new_centers.slice_mut(s![cluster, ..]);
                for (j, &val) in sample.iter().enumerate() {
                    center_slice[j] += val;
                }
                counts[cluster] += 1;
            }

            for i in 0..self.n_clusters {
                if counts[i] > 0 {
                    new_centers.slice_mut(s![i, ..]).mapv_inplace(|x| x / counts[i] as f64);
                } else {
                    // Empty cluster: reinitialize with random data point
                    let idx = (i * 7) % n_samples;
                    new_centers
                        .slice_mut(s![i, ..])
                        .assign(&data.slice(s![idx, ..]));
                }
            }

            // Calculate inertia
            let mut inertia = 0.0;
            for (i, sample) in data.outer_iter().enumerate() {
                let center = new_centers.slice(s![labels[i], ..]);
                inertia += self.metric.distance(&sample.to_owned(), &center.to_owned()).powi(2);
            }

            if (prev_inertia - inertia).abs() < 1e-6 {
                centers = new_centers;
                return Ok(KMeansResult {
                    labels,
                    centers,
                    inertia,
                    iterations: iter + 1,
                });
            }

            prev_inertia = inertia;
            centers = new_centers;
        }

        Ok(KMeansResult {
            labels,
            centers,
            inertia: prev_inertia,
            iterations: self.max_iterations,
        })
    }
}

/// DBSCAN clustering result
#[derive(Clone, Debug)]
pub struct DBSCANResult {
    pub labels: Vec<Option<usize>>,
    pub n_clusters: usize,
    pub n_noise: usize,
}

/// DBSCAN clustering algorithm
pub struct DBSCAN {
    pub eps: f64,
    pub min_samples: usize,
    pub metric: DistanceMetric,
}

impl DBSCAN {
    pub fn new(eps: f64, min_samples: usize) -> Self {
        Self {
            eps,
            min_samples,
            metric: DistanceMetric::Euclidean,
        }
    }

    pub fn fit(&self, data: &Array2<f64>) -> Result<DBSCANResult> {
        let n_samples = data.nrows();
        let mut labels: Vec<Option<usize>> = vec![None; n_samples];
        let mut cluster_id = 0;
        let mut visited = vec![false; n_samples];

        for i in 0..n_samples {
            if visited[i] {
                continue;
            }
            visited[i] = true;

            let neighbors = self.get_neighbors(data, i);

            if neighbors.len() < self.min_samples {
                continue;
            }

            cluster_id += 1;
            self.expand_cluster(data, i, cluster_id, &mut labels, &mut visited, &neighbors);
        }

        let n_clusters = cluster_id;
        let n_noise = labels.iter().filter(|l| l.is_none()).count();

        Ok(DBSCANResult {
            labels,
            n_clusters,
            n_noise,
        })
    }

    fn get_neighbors(&self, data: &Array2<f64>, idx: usize) -> Vec<usize> {
        let sample = data.slice(s![idx, ..]);
        let mut neighbors = Vec::new();

        for (i, other) in data.outer_iter().enumerate() {
            let dist = self.metric.distance(&sample.to_owned(), &other.to_owned());
            if dist <= self.eps {
                neighbors.push(i);
            }
        }

        neighbors
    }

    fn expand_cluster(
        &self,
        data: &Array2<f64>,
        idx: usize,
        cluster_id: usize,
        labels: &mut [Option<usize>],
        visited: &mut [bool],
        neighbors: &[usize],
    ) {
        labels[idx] = Some(cluster_id);
        let mut queue = neighbors.to_vec();
        let mut pos = 0;

        while pos < queue.len() {
            let current = queue[pos];
            pos += 1;

            if !visited[current] {
                visited[current] = true;
                let new_neighbors = self.get_neighbors(data, current);

                if new_neighbors.len() >= self.min_samples {
                    queue.extend(&new_neighbors);
                }
            }

            if labels[current].is_none() {
                labels[current] = Some(cluster_id);
            }
        }
    }
}

/// Hierarchical clustering result
#[derive(Clone, Debug)]
pub struct HierarchicalResult {
    pub labels: Vec<usize>,
    pub n_clusters: usize,
}

/// Hierarchical clustering algorithm
pub struct HierarchicalClustering {
    pub n_clusters: usize,
    pub metric: DistanceMetric,
    pub linkage: String, // "ward", "complete", "average"
}

impl HierarchicalClustering {
    pub fn new(n_clusters: usize) -> Self {
        Self {
            n_clusters,
            metric: DistanceMetric::Euclidean,
            linkage: "ward".to_string(),
        }
    }

    pub fn fit(&self, data: &Array2<f64>) -> Result<HierarchicalResult> {
        let n_samples = data.nrows();
        if n_samples == 0 || self.n_clusters == 0 || self.n_clusters > n_samples {
            return Err(crate::ClusterClusterAudienceKitError::InvalidConfig(
                "Invalid cluster count".to_string(),
            )
            .into());
        }

        // Simple agglomerative clustering
        let mut clusters: Vec<Vec<usize>> = (0..n_samples).map(|i| vec![i]).collect();
        let mut cluster_data: Vec<Array1<f64>> =
            (0..n_samples).map(|i| data.slice(s![i, ..]).to_owned()).collect();

        while clusters.len() > self.n_clusters {
            let mut min_dist = f64::MAX;
            let mut merge_i = 0;
            let mut merge_j = 1;

            // Find closest pair of clusters
            for i in 0..clusters.len() {
                for j in (i + 1)..clusters.len() {
                    let dist = self.metric.distance(&cluster_data[i], &cluster_data[j]);
                    if dist < min_dist {
                        min_dist = dist;
                        merge_i = i;
                        merge_j = j;
                    }
                }
            }

            // Merge closest clusters
            let mut new_cluster = clusters[merge_i].clone();
            new_cluster.extend(&clusters[merge_j]);

            // Calculate new center
            let new_center = Array1::from_vec(
                (0..cluster_data[0].len())
                    .map(|d| {
                        (cluster_data[merge_i][d] * clusters[merge_i].len() as f64
                            + cluster_data[merge_j][d] * clusters[merge_j].len() as f64)
                            / new_cluster.len() as f64
                    })
                    .collect(),
            );

            clusters.remove(merge_j);
            clusters.remove(merge_i);
            clusters.push(new_cluster);

            cluster_data.remove(merge_j);
            cluster_data.remove(merge_i);
            cluster_data.push(new_center);
        }

        // Assign labels
        let mut labels = vec![0; n_samples];
        for (cluster_id, cluster) in clusters.iter().enumerate() {
            for &sample_id in cluster {
                labels[sample_id] = cluster_id;
            }
        }

        Ok(HierarchicalResult {
            labels,
            n_clusters: clusters.len(),
        })
    }
}

/// Gaussian Mixture Model result
#[derive(Clone, Debug)]
pub struct GMMResult {
    pub labels: Vec<usize>,
    pub probabilities: Vec<Vec<f64>>,
    pub n_clusters: usize,
}

/// Gaussian Mixture Model clustering
pub struct GaussianMixture {
    pub n_components: usize,
    pub max_iterations: usize,
    pub random_state: u64,
}

impl GaussianMixture {
    pub fn new(n_components: usize) -> Self {
        Self {
            n_components,
            max_iterations: 100,
            random_state: 42,
        }
    }

    pub fn fit(&self, data: &Array2<f64>) -> Result<GMMResult> {
        let n_samples = data.nrows();
        let n_features = data.ncols();

        if n_samples == 0 || self.n_components == 0 || self.n_components > n_samples {
            return Err(crate::ClusterClusterAudienceKitError::InvalidConfig(
                "Invalid component count".to_string(),
            )
            .into());
        }

        // Simple implementation: soft k-means
        let mut centers = Array2::zeros((self.n_components, n_features));
        let step = n_samples / self.n_components;
        for i in 0..self.n_components {
            centers
                .slice_mut(s![i, ..])
                .assign(&data.slice(s![i * step, ..]));
        }

        let mut probabilities = vec![vec![0.0; self.n_components]; n_samples];

        for _ in 0..self.max_iterations {
            // E-step: assign soft labels
            for (i, sample) in data.outer_iter().enumerate() {
                let mut distances = Vec::new();
                for center in centers.outer_iter() {
                    let dist = sample
                        .iter()
                        .zip(center.iter())
                        .map(|(x, y)| (x - y).powi(2))
                        .sum::<f64>()
                        .sqrt();
                    distances.push(dist);
                }

                let sum: f64 = distances
                    .iter()
                    .map(|d| (-(d * d)).exp())
                    .sum();

                for (j, prob) in probabilities[i].iter_mut().enumerate() {
                    *prob = if sum > 0.0 {
                        (-(distances[j] * distances[j])).exp() / sum
                    } else {
                        1.0 / self.n_components as f64
                    };
                }
            }

            // M-step: update centers
            for j in 0..self.n_components {
                let total_prob: f64 = probabilities.iter().map(|p| p[j]).sum();
                if total_prob > 1e-10 {
                    let mut new_center = Array1::zeros(n_features);
                    for (i, sample) in data.outer_iter().enumerate() {
                        for (k, &val) in sample.iter().enumerate() {
                            new_center[k] += val * probabilities[i][j];
                        }
                    }
                    new_center.mapv_inplace(|x| x / total_prob);
                    centers.slice_mut(s![j, ..]).assign(&new_center);
                }
            }
        }

        // Assign hard labels from soft probabilities
        let labels = probabilities
            .iter()
            .map(|p| {
                p.iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            })
            .collect();

        Ok(GMMResult {
            labels,
            probabilities,
            n_clusters: self.n_components,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let a = Array1::from_vec(vec![0.0, 0.0]);
        let b = Array1::from_vec(vec![3.0, 4.0]);
        let dist = DistanceMetric::Euclidean.distance(&a, &b);
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = Array1::from_vec(vec![0.0, 0.0]);
        let b = Array1::from_vec(vec![3.0, 4.0]);
        let dist = DistanceMetric::Manhattan.distance(&a, &b);
        assert!((dist - 7.0).abs() < 1e-6);
    }

    #[test]
    fn test_kmeans_clustering() {
        let data = Array2::from_shape_vec(
            (4, 2),
            vec![0.0, 0.0, 1.0, 1.0, 5.0, 5.0, 6.0, 6.0],
        )
        .unwrap();

        let kmeans = KMeans::new(2);
        let result = kmeans.fit(&data).unwrap();

        assert_eq!(result.labels.len(), 4);
        assert_eq!(result.centers.dim(), (2, 2));
        assert!(result.inertia >= 0.0);
    }

    #[test]
    fn test_dbscan_clustering() {
        let data = Array2::from_shape_vec(
            (5, 2),
            vec![0.0, 0.0, 0.5, 0.5, 5.0, 5.0, 5.5, 5.5, 10.0, 10.0],
        )
        .unwrap();

        let dbscan = DBSCAN::new(1.0, 2);
        let result = dbscan.fit(&data).unwrap();

        assert_eq!(result.labels.len(), 5);
        assert!(result.n_clusters > 0 || result.n_noise > 0);
    }

    #[test]
    fn test_hierarchical_clustering() {
        let data = Array2::from_shape_vec(
            (4, 2),
            vec![0.0, 0.0, 1.0, 1.0, 5.0, 5.0, 6.0, 6.0],
        )
        .unwrap();

        let hc = HierarchicalClustering::new(2);
        let result = hc.fit(&data).unwrap();

        assert_eq!(result.labels.len(), 4);
        assert_eq!(result.n_clusters, 2);
    }

    #[test]
    fn test_gmm_clustering() {
        let data = Array2::from_shape_vec(
            (6, 2),
            vec![
                0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 5.0, 5.0, 5.5, 5.5, 6.0, 6.0,
            ],
        )
        .unwrap();

        let gmm = GaussianMixture::new(2);
        let result = gmm.fit(&data).unwrap();

        assert_eq!(result.labels.len(), 6);
        assert_eq!(result.probabilities.len(), 6);
        assert_eq!(result.n_clusters, 2);
    }

    #[test]
    fn test_distance_metrics() {
        let a = Array1::from_vec(vec![1.0, 2.0, 3.0]);
        let b = Array1::from_vec(vec![4.0, 5.0, 6.0]);

        let euc = DistanceMetric::Euclidean.distance(&a, &b);
        let man = DistanceMetric::Manhattan.distance(&a, &b);

        assert!(euc > 0.0);
        assert!(man > 0.0);
        assert!(man > euc); // Manhattan should be >= Euclidean
    }
}
