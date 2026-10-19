"""ClusterAudienceKit - Production audience segmentation engine with Python bindings"""

from .clusteraudiencekit import *
from .preprocessing import (
    PreprocessingPipeline,
    PreprocessingUnavailable,
    SegmenterWithPreprocessing,
    fit_with_preprocessing,
)

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
SegmentTrend = PySegmentTrend
EmergingAudience = PyEmergingAudience

# Segment intelligence metrics
SegmentConfidence = PySegmentConfidence
SegmentEntropy = PySegmentEntropy
SegmentDifferentiation = PySegmentDifferentiation
SegmentPredictability = PySegmentPredictability
SegmentAging = PySegmentAging

# Temporal analytics
SizeForecast = PySizeForecast
CompositionForecast = PyCompositionForecast
WhatIfScenario = PyWhatIfScenario

# Price intelligence
ElasticityAnalysis = PyElasticityAnalysis
DiscountStrategy = PyDiscountStrategy

# Revenue intelligence
SegmentROI = PySegmentROI
RevenueAttribution = PyRevenueAttribution
RevenueForecast = PyRevenueForecast

# Neural networks
NeuralNetwork = PyNeuralNetwork
TrainingStats = PyTrainingStats

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
    "AudienceSegmenter",
    "ChurnPrediction",
    "ChurnRiskLevel",
    "CompositionForecast",
    "CustomerLTV",
    "DecayFunction",
    "DiscountStrategy",
    "ElasticityAnalysis",
    "EmergingAudience",
    "FeatureDrift",
    "KMeansResult",
    "MiniBatchKMeans",
    "NeuralNetwork",
    "PreprocessingPipeline",
    "PreprocessingUnavailable",
    "RFMConfig",
    "RFMScore",
    "ReclusterConfig",
    "ReclusterEvent",
    "RevenueAttribution",
    "RevenueForecast",
    "ScoringMethod",
    "SegmentAging",
    "SegmentCompositionChange",
    "SegmentConfidence",
    "SegmentDifferentiation",
    "SegmentEntropy",
    "SegmentPredictability",
    "SegmentProfile",
    "SegmentROI",
    "SegmentTrend",
    "SegmentType",
    "SegmenterWithPreprocessing",
    "SizeForecast",
    "StreamingConfig",
    "StreamingEvent",
    "StreamingSegmentUpdate",
    "StreamingSegmentationEngine",
    "TrainingStats",
    "WhatIfScenario",
    "__author__",
    "__version__",
    "analyze_price_elasticity",
    "calculate_revenue_attribution",
    "calculate_rfm",
    "calculate_segment_roi",
    "calculate_simple_ltv",
    "chi_square_drift",
    "detect_feature_drift",
    "export_all_segments_sql",
    "export_segment_sql",
    "fit_with_preprocessing",
    "forecast_composition",
    "forecast_revenue",
    "forecast_segment_sizes",
    "get_segment_rfm_patterns",
    "get_supported_sql_dialects",
    "hellinger_distance",
    "info",
    "kmeans",
    "kolmogorov_smirnov",
    "optimize_discount",
    "simulate_what_if",
]
