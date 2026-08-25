"""ClusterAudienceKit - Production audience segmentation engine with Python bindings"""

from .clusteraudiencekit import *

# Create friendly aliases for classes (remove Py prefix)
AudienceSegmenter = PyAudienceSegmenter
RFMConfig = PyRFMConfig
RFMScore = PyRFMScore
DecayFunction = PyDecayFunction
ScoringMethod = PyScoringMethod
KMeansResult = PyKMeansResult
ChurnRiskLevel = PyChurnRiskLevel
ChurnPrediction = PyChurnPrediction
CustomerLTV = PyCustomerLTV
SegmentType = PySegmentType
SegmentProfile = PySegmentProfile
MiniBatchKMeans = PyMiniBatchKMeans

# Streaming segmentation + drift-triggered re-clustering (never aliased
# before -- these were only reachable via their raw Py-prefixed names,
# unlike every other class in this module).
StreamingEvent = PyStreamingEvent
StreamingConfig = PyStreamingConfig
StreamingSegmentUpdate = PyStreamingSegmentUpdate
StreamingSegmentationEngine = PyStreamingSegmentationEngine
ReclusterConfig = PyReclusterConfig
ReclusterEvent = PyReclusterEvent
FeatureDrift = PyFeatureDrift
SegmentCompositionChange = PySegmentCompositionChange

# Create friendly aliases for functions
calculate_rfm = calculate_rfm_py
kmeans = kmeans_py

# Re-export friendly names
__all__ = [
    "AudienceSegmenter", "RFMConfig", "RFMScore", "DecayFunction", "ScoringMethod",
    "KMeansResult", "ChurnRiskLevel", "ChurnPrediction", "CustomerLTV",
    "SegmentType", "SegmentProfile", "MiniBatchKMeans",
    "StreamingEvent", "StreamingConfig", "StreamingSegmentUpdate",
    "StreamingSegmentationEngine", "ReclusterConfig", "ReclusterEvent",
    "FeatureDrift", "SegmentCompositionChange",
    "calculate_rfm", "kmeans", "calculate_simple_ltv",
    "export_segment_sql", "export_all_segments_sql",
    "get_supported_sql_dialects", "get_segment_rfm_patterns",
    "kolmogorov_smirnov", "hellinger_distance", "chi_square_drift",
    "detect_feature_drift",
    "__version__", "__author__", "info",
]
