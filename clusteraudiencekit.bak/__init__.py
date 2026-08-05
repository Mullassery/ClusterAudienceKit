"""ClusterAudienceKit - Production audience intelligence platform"""

from .clusteraudiencekit import (
    __version__,
    __author__,
    PyAudienceSegmenter as AudienceSegmenter,
    PyRFMConfig as RFMConfig,
    PyRFMScore as RFMScore,
    PyDecayFunction as DecayFunction,
    PyScoringMethod as ScoringMethod,
    PyKMeansResult as KMeansResult,
    calculate_rfm_py as calculate_rfm,
    kmeans_py as kmeans,
    export_segment_sql,
    export_all_segments_sql,
    get_supported_sql_dialects,
    get_segment_rfm_patterns,
    info,
)

__all__ = [
    "__version__",
    "__author__",
    "AudienceSegmenter",
    "RFMConfig",
    "RFMScore",
    "DecayFunction",
    "ScoringMethod",
    "KMeansResult",
    "calculate_rfm",
    "kmeans",
    "export_segment_sql",
    "export_all_segments_sql",
    "get_supported_sql_dialects",
    "get_segment_rfm_patterns",
    "info",
]
