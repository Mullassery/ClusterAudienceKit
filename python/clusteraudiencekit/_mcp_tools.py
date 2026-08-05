"""
MCP Tool Definitions and Handlers for ClusterAudienceKit
Provides customer segmentation, CLV, churn prediction, and lookalike generation via MCP
"""

import logging
from typing import Any, Dict, List, Optional

logger = logging.getLogger(__name__)


class ClusterAudienceKitMCPTools:
    """MCP tool definitions for ClusterAudienceKit"""

    @staticmethod
    def get_tools() -> Dict[str, Any]:
        """Return all MCP tools for ClusterAudienceKit"""
        return {
            "segment_customers": {
                "description": "Cluster customers using RFM, K-Means, or other algorithms",
                "params": {
                    "customer_data": "dict - Customer transactions/attributes",
                    "method": "str - Clustering method (rfm_kmeans, auto_k, hierarchical)",
                    "n_clusters": "int - Number of clusters (auto-detect if method=auto_k)",
                },
                "returns": {
                    "segments": "list[int] - Cluster assignment per customer",
                    "segment_profiles": "list[dict] - Segment metadata (size, metrics)",
                    "stability_score": "float - Segment stability (0-1)",
                    "execution_time_ms": "int - Clustering duration",
                },
            },
            "get_segment_profiles": {
                "description": "Get detailed profile for each segment",
                "params": {},
                "returns": {
                    "profiles": "list[dict] - Segment profiles with RFM, CLV, etc.",
                    "total_segments": "int - Count",
                },
            },
            "export_segments_sql": {
                "description": "Generate SQL to define segments in warehouse",
                "params": {
                    "segment_ids": "list[int] - Segments to export",
                    "database_dialect": "str - SQL dialect (postgres, snowflake, etc.)",
                },
                "returns": {
                    "sql_queries": "list[str] - CREATE TABLE / INSERT statements",
                    "total_rows": "int - Rows to be inserted",
                },
            },
            "sync_to_platform": {
                "description": "Send segments to marketing platform (Braze, Klaviyo, etc.)",
                "params": {
                    "segment_id": "int - Segment to sync",
                    "platform": "str - Target platform (braze, klaviyo, segment, etc.)",
                    "push_type": "str - 'rtl' (real-time list) or 'scheduled'",
                },
                "returns": {
                    "status": "str - 'queued' | 'syncing' | 'complete' | 'failed'",
                    "synced_count": "int - Customers pushed",
                    "duration_ms": "int - Sync duration",
                    "error": "str - Error if failed",
                },
            },
            "detect_segment_drift": {
                "description": "Detect if segment characteristics have changed",
                "params": {
                    "segment_id": "int - Segment to check",
                    "comparison_date": "str - ISO date to compare against (default 7d ago)",
                },
                "returns": {
                    "drift_detected": "bool - Segment changed significantly",
                    "size_change_pct": "float - Size change percentage",
                    "churn_rate_change": "float - Member churn rate",
                    "metric_changes": "dict - Changes per RFM dimension",
                    "severity": "str - low | medium | high",
                },
            },
            "estimate_auto_k": {
                "description": "Recommend optimal cluster count using elbow method",
                "params": {
                    "customer_data": "dict - Customer data",
                    "k_range": "list[int] - Range to test (default [2,20])",
                },
                "returns": {
                    "recommended_k": "int - Suggested cluster count",
                    "elbow_point": "int - Statistical elbow",
                    "silhouette_scores": "dict - Score per k",
                    "visualization_url": "str - Elbow curve chart (optional)",
                },
            },
            "calculate_customer_lifetime_value": {
                "description": "Predict CLV for each customer",
                "params": {
                    "customer_data": "dict - Transaction history",
                    "segment_id": "int - Optional: segment-specific CLV",
                },
                "returns": {
                    "clv_by_customer": "dict - Customer ID -> CLV",
                    "avg_clv": "float - Average CLV",
                    "clv_percentiles": "dict - 25th, 50th, 75th, 95th percentiles",
                    "revenue_impact": "float - Total predicted revenue",
                },
            },
            "predict_churn": {
                "description": "Identify at-risk customers likely to churn",
                "params": {
                    "segment_id": "int - Segment to analyze (default: all)",
                    "churn_threshold": "float - Risk threshold 0-1 (default 0.5)",
                },
                "returns": {
                    "at_risk_customers": "list[str] - Customer IDs with churn risk",
                    "churn_risk_scores": "dict - Customer ID -> risk score",
                    "overall_churn_risk_pct": "float - % at risk",
                    "retention_actions": "list[str] - Recommended actions",
                },
            },
            "generate_lookalike_audience": {
                "description": "Find customers similar to high-value segment",
                "params": {
                    "segment_id": "int - Source segment",
                    "lookalike_pct": "float - Size as % of source (default 10)",
                    "similarity_threshold": "float - How similar 0-1 (default 0.7)",
                },
                "returns": {
                    "lookalike_customer_ids": "list[str] - Similar customers",
                    "similarity_scores": "dict - Customer ID -> score",
                    "predicted_value": "float - Est. revenue from lookalikes",
                },
            },
            "cohort_analysis": {
                "description": "Analyze retention by segment and cohort",
                "params": {
                    "segment_id": "int - Segment to analyze",
                    "cohort_period": "str - 'week' | 'month' | 'quarter'",
                },
                "returns": {
                    "cohorts": "list[dict] - Cohort retention curves",
                    "retention_rates": "dict - Retention % by period",
                    "trends": "list[str] - Key findings",
                },
            },
            "get_segment_quality": {
                "description": "Assess data quality and model performance",
                "params": {
                    "segment_id": "int - Segment to assess (default: all)",
                },
                "returns": {
                    "data_quality_score": "float - 0-100",
                    "model_stability": "float - Cluster stability",
                    "anomalies_detected": "list[str] - Data issues",
                    "recommendations": "list[str] - Quality improvements",
                },
            },
        }


class ClusterAudienceKitMCPHandler:
    """MCP tool handler for ClusterAudienceKit"""

    def __init__(self, segmenter: "AudienceSegmenter"):
        """
        Args:
            segmenter: ClusterAudienceKit AudienceSegmenter instance
        """
        self.segmenter = segmenter

    async def segment_customers(
        self,
        customer_data: Dict[str, Any],
        method: str = "rfm_kmeans",
        n_clusters: int = 5,
    ) -> Dict[str, Any]:
        """Segment customers"""
        try:
            import time

            start = time.time()
            result = self.segmenter.segment(
                data=customer_data, method=method, n_clusters=n_clusters
            )
            elapsed_ms = int((time.time() - start) * 1000)

            return {
                "segments": result.get("assignments", []),
                "segment_profiles": result.get("profiles", []),
                "stability_score": result.get("stability", 0.8),
                "execution_time_ms": elapsed_ms,
            }

        except Exception as e:
            return {"error": str(e)}

    async def get_segment_profiles(self) -> Dict[str, Any]:
        """Get segment profiles"""
        try:
            profiles = self.segmenter.get_profiles()

            return {
                "profiles": profiles,
                "total_segments": len(profiles),
            }

        except Exception as e:
            return {"profiles": [], "error": str(e)}

    async def export_segments_sql(
        self, segment_ids: List[int], database_dialect: str = "postgresql"
    ) -> Dict[str, Any]:
        """Export segments as SQL"""
        try:
            queries = self.segmenter.export_sql(
                segments=segment_ids, dialect=database_dialect
            )

            return {
                "sql_queries": queries.get("statements", []),
                "total_rows": queries.get("row_count", 0),
            }

        except Exception as e:
            return {"sql_queries": [], "error": str(e)}

    async def sync_to_platform(
        self,
        segment_id: int,
        platform: str,
        push_type: str = "rtl",
    ) -> Dict[str, Any]:
        """Sync to marketing platform"""
        try:
            result = self.segmenter.sync_platform(
                segment_id=segment_id, platform=platform, push_type=push_type
            )

            return {
                "status": "complete",
                "synced_count": result.get("count", 0),
                "duration_ms": result.get("duration_ms", 0),
            }

        except Exception as e:
            return {
                "status": "failed",
                "error": str(e),
            }

    async def detect_segment_drift(
        self, segment_id: int, comparison_date: Optional[str] = None
    ) -> Dict[str, Any]:
        """Detect segment drift"""
        try:
            drift = self.segmenter.detect_drift(
                segment_id=segment_id, comparison_date=comparison_date
            )

            return {
                "drift_detected": drift.get("detected", False),
                "size_change_pct": drift.get("size_change", 0.0),
                "churn_rate_change": drift.get("churn_change", 0.0),
                "metric_changes": drift.get("metrics", {}),
                "severity": drift.get("severity", "low"),
            }

        except Exception as e:
            return {"error": str(e)}

    async def estimate_auto_k(
        self,
        customer_data: Dict[str, Any],
        k_range: Optional[List[int]] = None,
    ) -> Dict[str, Any]:
        """Estimate optimal k"""
        try:
            if k_range is None:
                k_range = [2, 20]

            result = self.segmenter.estimate_k(data=customer_data, k_range=k_range)

            return {
                "recommended_k": result.get("optimal_k", 5),
                "elbow_point": result.get("elbow", 5),
                "silhouette_scores": result.get("scores", {}),
                "visualization_url": result.get("chart_url", ""),
            }

        except Exception as e:
            return {"error": str(e)}

    async def calculate_customer_lifetime_value(
        self,
        customer_data: Dict[str, Any],
        segment_id: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Calculate CLV"""
        try:
            clv = self.segmenter.calculate_clv(
                data=customer_data, segment_id=segment_id
            )

            return {
                "clv_by_customer": clv.get("by_customer", {}),
                "avg_clv": clv.get("average", 0.0),
                "clv_percentiles": clv.get("percentiles", {}),
                "revenue_impact": clv.get("total_revenue", 0.0),
            }

        except Exception as e:
            return {"error": str(e)}

    async def predict_churn(
        self, segment_id: Optional[int] = None, churn_threshold: float = 0.5
    ) -> Dict[str, Any]:
        """Predict churn"""
        try:
            churn = self.segmenter.predict_churn(
                segment_id=segment_id, threshold=churn_threshold
            )

            return {
                "at_risk_customers": churn.get("customers", []),
                "churn_risk_scores": churn.get("scores", {}),
                "overall_churn_risk_pct": churn.get("risk_pct", 0.0),
                "retention_actions": churn.get("actions", []),
            }

        except Exception as e:
            return {"error": str(e)}

    async def generate_lookalike_audience(
        self,
        segment_id: int,
        lookalike_pct: float = 10,
        similarity_threshold: float = 0.7,
    ) -> Dict[str, Any]:
        """Generate lookalikes"""
        try:
            lookalikes = self.segmenter.generate_lookalikes(
                segment_id=segment_id,
                lookalike_pct=lookalike_pct,
                similarity=similarity_threshold,
            )

            return {
                "lookalike_customer_ids": lookalikes.get("customers", []),
                "similarity_scores": lookalikes.get("scores", {}),
                "predicted_value": lookalikes.get("predicted_revenue", 0.0),
            }

        except Exception as e:
            return {"error": str(e)}

    async def cohort_analysis(
        self, segment_id: int, cohort_period: str = "month"
    ) -> Dict[str, Any]:
        """Cohort analysis"""
        try:
            cohorts = self.segmenter.analyze_cohorts(
                segment_id=segment_id, period=cohort_period
            )

            return {
                "cohorts": cohorts.get("cohorts", []),
                "retention_rates": cohorts.get("rates", {}),
                "trends": cohorts.get("trends", []),
            }

        except Exception as e:
            return {"error": str(e)}

    async def get_segment_quality(
        self, segment_id: Optional[int] = None
    ) -> Dict[str, Any]:
        """Assess segment quality"""
        try:
            quality = self.segmenter.assess_quality(segment_id=segment_id)

            return {
                "data_quality_score": quality.get("quality_score", 0),
                "model_stability": quality.get("stability", 0),
                "anomalies_detected": quality.get("anomalies", []),
                "recommendations": quality.get("recommendations", []),
            }

        except Exception as e:
            return {"error": str(e)}
