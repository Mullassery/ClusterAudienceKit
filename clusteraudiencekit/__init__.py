"""ClusterAudienceKit: High-performance audience segmentation library.

A pure Rust library with PyO3 bindings for customer segmentation using RFM analysis
and advanced clustering algorithms (K-Means, K-Prototypes). Optimized for performance
on large datasets with O(n) silhouette computation.

Example:
    >>> from clusteraudiencekit import AudienceSegmenter
    >>> segmenter = AudienceSegmenter()
    >>> segments = segmenter.segment(transactions_df, num_segments=5)
    >>> profiles = segmenter.get_segment_profiles()

Attributes:
    __version__ (str): Package version
    __author__ (str): Primary author
    __email__ (str): Author email
    __license__ (str): License type (MIT)
"""

from typing import Final

import sys
import os
import importlib.util

# Import the native Rust module
# The .so file has the same name as the package, so we need special handling
_pkg_dir = os.path.dirname(__file__)
_so_files = [f for f in os.listdir(_pkg_dir) if f.startswith("clusteraudiencekit") and f.endswith(".so")]

if _so_files:
    try:
        _so_path = os.path.join(_pkg_dir, _so_files[0])
        # Load using the actual module name (which is "clusteraudiencekit")
        _spec = importlib.util.spec_from_file_location("clusteraudiencekit._native", _so_path)
        if _spec and _spec.loader:
            _core = importlib.util.module_from_spec(_spec)
            sys.modules["clusteraudiencekit._native"] = _core
            _spec.loader.exec_module(_core)
            AudienceSegmenter = _core.PyAudienceSegmenter
        else:
            raise ImportError("Could not create module spec for .so file")
    except Exception as e:
        raise ImportError(f"Failed to load Rust bindings: {e}") from e
else:
    raise ImportError("ClusterAudienceKit native extension not found. Please reinstall.")

# CRITICAL: Multi-algorithm clustering support (unblocks data scientists)
from ._multi_algorithm import (
    MultiAlgorithmClusterer,
    ClusteringAlgorithm,
    DistanceMetric,
    ClusterResult,
    CustomDistanceMetric,
    recommend_algorithm,
)

__version__: Final[str] = "1.0.0"
__author__: Final[str] = "Georgi Mammen Mullassery"
__email__: Final[str] = "mullassery@gmail.com"
__license__: Final[str] = "MIT"

__all__: Final[list[str]] = [
    "AudienceSegmenter",
    # Multi-algorithm clustering (v1.1.0+)
    "MultiAlgorithmClusterer",
    "ClusteringAlgorithm",
    "DistanceMetric",
    "ClusterResult",
    "CustomDistanceMetric",
    "recommend_algorithm",
    "__version__",
    "__author__",
    "__email__",
]
