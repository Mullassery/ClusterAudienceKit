"""
ClusterAudienceKit v1.5: Advanced Customer Segmentation Engine

High-performance audience segmentation with RFM analysis, multiple clustering algorithms,
automatic K estimation, segment profiling, and quality metrics.

Features:
- RFM (Recency-Frequency-Monetary) analysis with decay functions
- 4 clustering algorithms: K-Means, DBSCAN, Hierarchical, GMM
- 3 auto K estimation methods: Elbow, Gap Statistic, Silhouette
- 13 business-standard customer segments
- Segment profiling with feature importance & health scoring
- Quality metrics: Silhouette, Davies-Bouldin, Calinski-Harabasz
- Stability tracking with Adjusted Rand Index

Example:
    >>> from clusteraudiencekit import __version__
    >>> print(f"ClusterAudienceKit {__version__}")
"""

__version__ = "1.5.0"
__author__ = "Georgi Mammen Mullassery"
__license__ = "MIT"

# Try to import the Rust extension if available
try:
    from . import clusteraudiencekit as _ext
    __rust_available__ = True
except ImportError:
    __rust_available__ = False

__all__ = [
    "__version__",
    "__author__",
    "__license__",
    "__rust_available__",
]
