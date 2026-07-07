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

try:
    from ._core import PyAudienceSegmenter as AudienceSegmenter
except ImportError as e:
    raise ImportError(
        "ClusterAudienceKit native extension not found. "
        "Please ensure the package is installed correctly."
    ) from e

__version__: Final[str] = "0.1.0"
__author__: Final[str] = "Georgi Mammen Mullassery"
__email__: Final[str] = "mullassery@gmail.com"
__license__: Final[str] = "MIT"

__all__: Final[list[str]] = [
    "AudienceSegmenter",
    "__version__",
    "__author__",
    "__email__",
]
