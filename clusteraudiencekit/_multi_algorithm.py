"""
Multi-algorithm clustering support for customer segmentation.

Supports multiple clustering algorithms beyond K-means:
- K-means (existing, fast)
- DBSCAN (density-based, finds variable-sized clusters)
- Gaussian Mixture Models (probabilistic, soft assignments)

This CRITICAL feature unblocks data scientists with specific clustering needs.
"""

from typing import List, Dict, Optional, Tuple, Union
from enum import Enum
from dataclasses import dataclass
import numpy as np
from sklearn.cluster import KMeans, DBSCAN
from sklearn.mixture import GaussianMixture
from sklearn.preprocessing import StandardScaler
from sklearn.metrics import silhouette_score, davies_bouldin_score


class ClusteringAlgorithm(Enum):
    """Available clustering algorithms."""
    KMEANS = "kmeans"
    DBSCAN = "dbscan"
    GMM = "gmm"  # Gaussian Mixture Models


@dataclass
class DistanceMetric:
    """Distance metric configuration."""
    metric: str  # euclidean, manhattan, cosine
    scale_features: bool = True  # Standardize before clustering


@dataclass
class ClusterResult:
    """Result of clustering analysis."""
    algorithm: ClusteringAlgorithm
    n_clusters: int
    labels: np.ndarray
    silhouette_score: Optional[float] = None
    davies_bouldin_score: Optional[float] = None
    inertia: Optional[float] = None  # For K-means
    bic: Optional[float] = None  # For GMM
    aic: Optional[float] = None  # For GMM
    model_details: Dict = None


class MultiAlgorithmClusterer:
    """
    Flexible clustering engine supporting multiple algorithms.

    Allows data scientists to choose the right algorithm for their data:
    - K-means: Fast, spherical clusters
    - DBSCAN: Density-based, handles outliers, variable cluster sizes
    - GMM: Probabilistic, soft assignments, works well with overlapping clusters
    """

    def __init__(self, algorithm: Union[str, ClusteringAlgorithm] = ClusteringAlgorithm.KMEANS,
                 distance_metric: Optional[DistanceMetric] = None,
                 random_state: int = 42):
        """
        Args:
            algorithm: Clustering algorithm to use
            distance_metric: How to measure distances
            random_state: For reproducibility
        """
        if isinstance(algorithm, str):
            algorithm = ClusteringAlgorithm(algorithm)

        self.algorithm = algorithm
        self.distance_metric = distance_metric or DistanceMetric("euclidean")
        self.random_state = random_state
        self.scaler = StandardScaler() if self.distance_metric.scale_features else None
        self.model = None
        self.labels_ = None

    def fit_predict(self, X: np.ndarray, **algorithm_params) -> ClusterResult:
        """
        Fit clustering model and return cluster assignments.

        Args:
            X: Feature matrix (n_samples, n_features)
            **algorithm_params: Algorithm-specific parameters

        Returns:
            ClusterResult with clustering details
        """
        # Standardize if requested
        if self.scaler is not None:
            X_scaled = self.scaler.fit_transform(X)
        else:
            X_scaled = X

        if self.algorithm == ClusteringAlgorithm.KMEANS:
            result = self._fit_kmeans(X_scaled, **algorithm_params)
        elif self.algorithm == ClusteringAlgorithm.DBSCAN:
            result = self._fit_dbscan(X_scaled, **algorithm_params)
        elif self.algorithm == ClusteringAlgorithm.GMM:
            result = self._fit_gmm(X_scaled, **algorithm_params)
        else:
            raise ValueError(f"Unknown algorithm: {self.algorithm}")

        self.labels_ = result.labels
        return result

    def _fit_kmeans(self, X: np.ndarray, n_clusters: int = 3, **kwargs) -> ClusterResult:
        """Fit K-means clustering."""
        model = KMeans(n_clusters=n_clusters, random_state=self.random_state, **kwargs)
        labels = model.fit_predict(X)

        sil_score = silhouette_score(X, labels) if len(set(labels)) > 1 else 0
        db_score = davies_bouldin_score(X, labels) if len(set(labels)) > 1 else 0

        return ClusterResult(
            algorithm=ClusteringAlgorithm.KMEANS,
            n_clusters=n_clusters,
            labels=labels,
            silhouette_score=sil_score,
            davies_bouldin_score=db_score,
            inertia=model.inertia_,
            model_details={
                "centers": model.cluster_centers_.tolist(),
                "iterations": model.n_iter_,
            }
        )

    def _fit_dbscan(self, X: np.ndarray, eps: float = 0.5, min_samples: int = 5, **kwargs) -> ClusterResult:
        """
        Fit DBSCAN clustering.

        DBSCAN finds density-connected regions, automatically determining cluster count.
        Good for:
        - Outlier detection (label -1)
        - Variable-size clusters
        - Non-spherical shapes
        """
        model = DBSCAN(eps=eps, min_samples=min_samples, metric=self.distance_metric.metric, **kwargs)
        labels = model.fit_predict(X)

        n_clusters = len(set(labels)) - (1 if -1 in labels else 0)
        n_outliers = list(labels).count(-1)

        # Silhouette score (ignore noise points)
        if len(set(labels)) > 1 and n_clusters > 1:
            mask = labels != -1
            if mask.sum() > 0:
                sil_score = silhouette_score(X[mask], labels[mask])
                db_score = davies_bouldin_score(X[mask], labels[mask])
            else:
                sil_score = 0
                db_score = 0
        else:
            sil_score = 0
            db_score = 0

        return ClusterResult(
            algorithm=ClusteringAlgorithm.DBSCAN,
            n_clusters=n_clusters,
            labels=labels,
            silhouette_score=sil_score,
            davies_bouldin_score=db_score,
            model_details={
                "n_outliers": n_outliers,
                "outlier_ratio": n_outliers / len(labels),
                "eps": eps,
                "min_samples": min_samples,
            }
        )

    def _fit_gmm(self, X: np.ndarray, n_components: int = 3, **kwargs) -> ClusterResult:
        """
        Fit Gaussian Mixture Model clustering.

        GMM provides:
        - Probabilistic assignments (soft vs hard)
        - Confidence for each assignment
        - Better for overlapping clusters
        """
        model = GaussianMixture(n_components=n_components, random_state=self.random_state, **kwargs)
        labels = model.fit_predict(X)

        sil_score = silhouette_score(X, labels) if n_components > 1 else 0
        db_score = davies_bouldin_score(X, labels) if n_components > 1 else 0

        # Get soft assignments (probabilities)
        probabilities = model.predict_proba(X)

        return ClusterResult(
            algorithm=ClusteringAlgorithm.GMM,
            n_clusters=n_components,
            labels=labels,
            silhouette_score=sil_score,
            davies_bouldin_score=db_score,
            bic=model.bic(X),
            aic=model.aic(X),
            model_details={
                "weights": model.weights_.tolist(),
                "converged": model.converged_,
                "n_iter": model.n_iter_,
                "probabilities": probabilities.tolist(),
            }
        )

    def compare_algorithms(self, X: np.ndarray) -> Dict[str, ClusterResult]:
        """
        Compare all algorithms on the same data.

        Helpful for choosing the best algorithm for your dataset.

        Returns:
            Dict mapping algorithm names to their results
        """
        results = {}

        # K-means with 3 clusters
        clusterer_km = MultiAlgorithmClusterer(ClusteringAlgorithm.KMEANS)
        results['kmeans_3'] = clusterer_km.fit_predict(X, n_clusters=3)

        # DBSCAN with default parameters
        clusterer_db = MultiAlgorithmClusterer(ClusteringAlgorithm.DBSCAN)
        results['dbscan'] = clusterer_db.fit_predict(X, eps=0.5, min_samples=5)

        # GMM with 3 components
        clusterer_gmm = MultiAlgorithmClusterer(ClusteringAlgorithm.GMM)
        results['gmm_3'] = clusterer_gmm.fit_predict(X, n_components=3)

        return results

    def find_optimal_clusters(self, X: np.ndarray, max_clusters: int = 10) -> Dict:
        """
        Find optimal number of clusters using elbow method + silhouette score.

        Works best with K-means and GMM.
        """
        if self.algorithm == ClusteringAlgorithm.DBSCAN:
            return {"note": "DBSCAN determines clusters automatically"}

        results = []
        for n in range(2, min(max_clusters + 1, len(X))):
            if self.algorithm == ClusteringAlgorithm.KMEANS:
                result = self.fit_predict(X, n_clusters=n)
            else:  # GMM
                result = self.fit_predict(X, n_components=n)

            results.append({
                "n_clusters": n,
                "silhouette": result.silhouette_score or 0,
                "davies_bouldin": result.davies_bouldin_score or 0,
                "inertia": result.inertia,
                "bic": result.bic,
            })

        # Find optimal (highest silhouette)
        optimal_idx = max(range(len(results)), key=lambda i: results[i]["silhouette"])
        optimal_n = results[optimal_idx]["n_clusters"]

        return {
            "optimal_n_clusters": optimal_n,
            "results": results,
            "recommendation": f"Use {optimal_n} clusters (silhouette score: {results[optimal_idx]['silhouette']:.3f})"
        }


class CustomDistanceMetric:
    """Support for custom distance functions."""

    @staticmethod
    def manhattan(X):
        """Manhattan/L1 distance."""
        return "manhattan"

    @staticmethod
    def cosine(X):
        """Cosine distance (useful for text/sparse data)."""
        return "cosine"

    @staticmethod
    def euclidean(X):
        """Euclidean/L2 distance (default)."""
        return "euclidean"


def recommend_algorithm(X: np.ndarray, n_samples: int = None, n_features: int = None) -> str:
    """
    Recommend best algorithm based on data characteristics.

    Returns:
        Recommended algorithm name (kmeans, dbscan, or gmm)
    """
    if n_samples is None:
        n_samples = X.shape[0]
    if n_features is None:
        n_features = X.shape[1]

    # Heuristics for algorithm selection
    if n_samples < 1000:
        return "gmm"  # GMM good for small datasets
    elif n_features > 50:
        return "kmeans"  # K-means fast for high dimensions
    else:
        # Check for density variations (suggests DBSCAN)
        return "dbscan"  # Default to DBSCAN for medium datasets
