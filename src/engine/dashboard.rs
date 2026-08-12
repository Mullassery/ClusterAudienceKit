//! Production dashboard metrics and data structures

use crate::Result;
use std::collections::HashMap;

/// Dashboard time range
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum TimeRange {
    Last24Hours,
    Last7Days,
    Last30Days,
    Last90Days,
    LastYear,
}

impl TimeRange {
    pub fn seconds(&self) -> i64 {
        match self {
            TimeRange::Last24Hours => 86400,
            TimeRange::Last7Days => 604800,
            TimeRange::Last30Days => 2592000,
            TimeRange::Last90Days => 7776000,
            TimeRange::LastYear => 31536000,
        }
    }
}

/// Key performance indicator
#[derive(Clone, Debug)]
pub struct KPI {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub previous_value: Option<f64>,
    pub change_percent: Option<f64>,
    pub trend: Trend,
}

/// Trend direction
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum Trend {
    Up,
    Down,
    Stable,
}

impl Trend {
    pub fn as_str(&self) -> &str {
        match self {
            Trend::Up => "up",
            Trend::Down => "down",
            Trend::Stable => "stable",
        }
    }
}

/// Segment dashboard card
#[derive(Clone, Debug)]
pub struct SegmentCard {
    pub segment_name: String,
    pub size: usize,
    pub size_change_percent: f64,
    pub avg_ltv: f64,
    pub retention_rate: f64,
    pub churn_risk: f64,
    pub health_score: f64,
}

/// Historical metrics point
#[derive(Clone, Debug)]
pub struct MetricsPoint {
    pub timestamp: i64,
    pub segment_size: usize,
    pub active_customers: usize,
    pub churn_rate: f64,
    pub revenue: f64,
}

/// Segment comparison
#[derive(Clone, Debug)]
pub struct SegmentComparison {
    pub segment_a: String,
    pub segment_b: String,
    pub size_diff: i32,
    pub ltv_diff: f64,
    pub retention_diff: f64,
    pub better_segment: String,
}

/// Real-time streaming metrics
#[derive(Clone, Debug)]
pub struct StreamingMetrics {
    pub events_per_second: f64,
    pub customers_updated_today: usize,
    pub segment_changes_today: usize,
    pub buffer_size: usize,
    pub latency_ms: f64,
}

/// Drift alert summary
#[derive(Clone, Debug)]
pub struct DriftAlertSummary {
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub high_alerts: usize,
    pub features_with_drift: Vec<String>,
}

/// Dashboard summary
#[derive(Clone, Debug)]
pub struct DashboardSummary {
    pub total_customers: usize,
    pub total_segments: usize,
    pub avg_segment_size: usize,
    pub total_revenue: f64,
    pub avg_customer_ltv: f64,
    pub overall_retention: f64,
    pub overall_churn: f64,
    pub timestamp: i64,
}

/// Segment health summary
#[derive(Clone, Debug)]
pub struct SegmentHealthSummary {
    pub healthy_segments: usize,
    pub at_risk_segments: usize,
    pub declining_segments: usize,
    pub growing_segments: usize,
}

/// Dashboard data container
#[derive(Clone, Debug)]
pub struct DashboardData {
    pub summary: DashboardSummary,
    pub kpis: Vec<KPI>,
    pub segments: Vec<SegmentCard>,
    pub streaming: StreamingMetrics,
    pub drift_alerts: DriftAlertSummary,
    pub health_summary: SegmentHealthSummary,
    pub time_range: TimeRange,
}

/// Dashboard metrics builder
pub struct DashboardMetrics;

impl DashboardMetrics {
    /// Calculate KPI trend
    pub fn calculate_trend(current: f64, previous: f64) -> Trend {
        if (current - previous).abs() < 0.001 {
            Trend::Stable
        } else if current > previous {
            Trend::Up
        } else {
            Trend::Down
        }
    }

    /// Create KPI
    pub fn create_kpi(name: String, value: f64, unit: String, previous_value: Option<f64>) -> KPI {
        let (change_percent, trend) = match previous_value {
            Some(prev) if prev > 0.0 => {
                let change = ((value - prev) / prev) * 100.0;
                let trend = Self::calculate_trend(value, prev);
                (Some(change), trend)
            }
            _ => (None, Trend::Stable),
        };

        KPI {
            name,
            value,
            unit,
            previous_value,
            change_percent,
            trend,
        }
    }

    /// Build summary from metrics
    pub fn build_summary(
        total_customers: usize,
        segment_counts: &HashMap<String, usize>,
        total_revenue: f64,
        avg_ltv: f64,
        retention: f64,
    ) -> Result<DashboardSummary> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let total_segments = segment_counts.len();
        let avg_segment_size = if total_segments > 0 {
            total_customers / total_segments
        } else {
            0
        };

        Ok(DashboardSummary {
            total_customers,
            total_segments,
            avg_segment_size,
            total_revenue,
            avg_customer_ltv: avg_ltv,
            overall_retention: retention,
            overall_churn: 1.0 - retention,
            timestamp: now,
        })
    }

    /// Create segment card
    pub fn create_segment_card(
        name: String,
        size: usize,
        size_change_percent: f64,
        avg_ltv: f64,
        retention: f64,
        churn_risk: f64,
        health_score: f64,
    ) -> SegmentCard {
        SegmentCard {
            segment_name: name,
            size,
            size_change_percent,
            avg_ltv,
            retention_rate: retention,
            churn_risk,
            health_score,
        }
    }

    /// Calculate metrics point
    pub fn calculate_metrics_point(
        timestamp: i64,
        segment_size: usize,
        active_customers: usize,
        churn_rate: f64,
        revenue: f64,
    ) -> MetricsPoint {
        MetricsPoint {
            timestamp,
            segment_size,
            active_customers,
            churn_rate,
            revenue,
        }
    }

    /// Compare segments
    pub fn compare_segments(
        segment_a: String,
        segment_a_size: usize,
        segment_a_ltv: f64,
        segment_a_retention: f64,
        segment_b: String,
        segment_b_size: usize,
        segment_b_ltv: f64,
        segment_b_retention: f64,
    ) -> SegmentComparison {
        let size_diff = segment_b_size as i32 - segment_a_size as i32;
        let ltv_diff = segment_b_ltv - segment_a_ltv;
        let retention_diff = segment_b_retention - segment_a_retention;

        let better_segment = if retention_diff > 0.0 {
            segment_b.clone()
        } else {
            segment_a.clone()
        };

        SegmentComparison {
            segment_a,
            segment_b,
            size_diff,
            ltv_diff,
            retention_diff,
            better_segment,
        }
    }

    /// Create streaming metrics
    pub fn create_streaming_metrics(
        events_per_second: f64,
        customers_updated_today: usize,
        segment_changes: usize,
        buffer_size: usize,
        latency_ms: f64,
    ) -> StreamingMetrics {
        StreamingMetrics {
            events_per_second,
            customers_updated_today,
            segment_changes_today: segment_changes,
            buffer_size,
            latency_ms,
        }
    }

    /// Classify segment health
    pub fn classify_segment_health(health_score: f64) -> String {
        if health_score >= 0.8 {
            "healthy".to_string()
        } else if health_score >= 0.6 {
            "at_risk".to_string()
        } else {
            "declining".to_string()
        }
    }

    /// Calculate health summary
    pub fn calculate_health_summary(segments: &[SegmentCard]) -> SegmentHealthSummary {
        let mut healthy = 0;
        let mut at_risk = 0;
        let mut declining = 0;
        let mut growing = 0;

        for segment in segments {
            if segment.health_score >= 0.8 {
                healthy += 1;
            } else if segment.health_score >= 0.6 {
                at_risk += 1;
            } else {
                declining += 1;
            }

            if segment.size_change_percent > 0.0 {
                growing += 1;
            }
        }

        SegmentHealthSummary {
            healthy_segments: healthy,
            at_risk_segments: at_risk,
            declining_segments: declining,
            growing_segments: growing,
        }
    }
}

/// Dashboard data provider
pub struct DashboardProvider;

impl DashboardProvider {
    /// Generate full dashboard data
    pub fn generate_dashboard(
        summary: DashboardSummary,
        segments: Vec<SegmentCard>,
        kpis: Vec<KPI>,
        streaming: StreamingMetrics,
        drift_alerts: DriftAlertSummary,
        time_range: TimeRange,
    ) -> Result<DashboardData> {
        let health_summary = DashboardMetrics::calculate_health_summary(&segments);

        Ok(DashboardData {
            summary,
            kpis,
            segments,
            streaming,
            drift_alerts,
            health_summary,
            time_range,
        })
    }

    /// Export dashboard data as JSON-friendly format
    pub fn export_summary(dashboard: &DashboardData) -> Result<HashMap<String, String>> {
        let mut export = HashMap::new();

        export.insert(
            "total_customers".to_string(),
            dashboard.summary.total_customers.to_string(),
        );
        export.insert(
            "total_segments".to_string(),
            dashboard.summary.total_segments.to_string(),
        );
        export.insert(
            "total_revenue".to_string(),
            format!("{:.2}", dashboard.summary.total_revenue),
        );
        export.insert(
            "avg_ltv".to_string(),
            format!("{:.2}", dashboard.summary.avg_customer_ltv),
        );
        export.insert(
            "retention_rate".to_string(),
            format!("{:.2}", dashboard.summary.overall_retention),
        );
        export.insert(
            "churn_rate".to_string(),
            format!("{:.2}", dashboard.summary.overall_churn),
        );
        export.insert(
            "streaming_latency_ms".to_string(),
            format!("{:.1}", dashboard.streaming.latency_ms),
        );
        export.insert(
            "critical_alerts".to_string(),
            dashboard.drift_alerts.critical_alerts.to_string(),
        );
        export.insert(
            "healthy_segments".to_string(),
            dashboard.health_summary.healthy_segments.to_string(),
        );

        Ok(export)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range() {
        assert_eq!(TimeRange::Last24Hours.seconds(), 86400);
        assert_eq!(TimeRange::Last7Days.seconds(), 604800);
        assert_eq!(TimeRange::Last30Days.seconds(), 2592000);
    }

    #[test]
    fn test_trend_classification() {
        assert_eq!(DashboardMetrics::calculate_trend(10.0, 9.0), Trend::Up);
        assert_eq!(DashboardMetrics::calculate_trend(9.0, 10.0), Trend::Down);
        assert_eq!(DashboardMetrics::calculate_trend(10.0, 10.0), Trend::Stable);
    }

    #[test]
    fn test_kpi_creation() {
        let kpi = DashboardMetrics::create_kpi(
            "Revenue".to_string(),
            1000.0,
            "USD".to_string(),
            Some(900.0),
        );

        assert_eq!(kpi.name, "Revenue");
        assert!(kpi.change_percent.is_some());
        assert_eq!(kpi.trend, Trend::Up);
    }

    #[test]
    fn test_summary_creation() {
        let mut segments = HashMap::new();
        segments.insert("Champions".to_string(), 100);
        segments.insert("Loyal".to_string(), 200);

        let summary =
            DashboardMetrics::build_summary(300, &segments, 50000.0, 166.67, 0.85).unwrap();

        assert_eq!(summary.total_customers, 300);
        assert_eq!(summary.total_segments, 2);
        assert_eq!(summary.avg_segment_size, 150);
    }

    #[test]
    fn test_segment_card_creation() {
        let card = DashboardMetrics::create_segment_card(
            "Champions".to_string(),
            100,
            5.0,
            500.0,
            0.95,
            0.05,
            0.95,
        );

        assert_eq!(card.segment_name, "Champions");
        assert_eq!(card.size, 100);
        assert_eq!(card.health_score, 0.95);
    }

    #[test]
    fn test_segment_comparison() {
        let comparison = DashboardMetrics::compare_segments(
            "Champions".to_string(),
            100,
            500.0,
            0.95,
            "Loyal".to_string(),
            200,
            400.0,
            0.85,
        );

        assert_eq!(comparison.size_diff, 100);
        assert_eq!(comparison.better_segment, "Champions");
    }

    #[test]
    fn test_streaming_metrics() {
        let metrics = DashboardMetrics::create_streaming_metrics(1000.5, 5000, 100, 500, 45.3);

        assert_eq!(metrics.events_per_second, 1000.5);
        assert_eq!(metrics.customers_updated_today, 5000);
    }

    #[test]
    fn test_segment_health_classification() {
        assert_eq!(DashboardMetrics::classify_segment_health(0.9), "healthy");
        assert_eq!(DashboardMetrics::classify_segment_health(0.7), "at_risk");
        assert_eq!(DashboardMetrics::classify_segment_health(0.3), "declining");
    }

    #[test]
    fn test_health_summary() {
        let segments = vec![
            DashboardMetrics::create_segment_card(
                "Seg1".to_string(),
                100,
                5.0,
                500.0,
                0.9,
                0.05,
                0.9,
            ),
            DashboardMetrics::create_segment_card(
                "Seg2".to_string(),
                200,
                -3.0,
                400.0,
                0.7,
                0.2,
                0.7,
            ),
        ];

        let health = DashboardMetrics::calculate_health_summary(&segments);

        assert_eq!(health.healthy_segments, 1);
        assert_eq!(health.at_risk_segments, 1);
    }

    #[test]
    fn test_dashboard_generation() {
        let summary = DashboardSummary {
            total_customers: 1000,
            total_segments: 5,
            avg_segment_size: 200,
            total_revenue: 100000.0,
            avg_customer_ltv: 100.0,
            overall_retention: 0.85,
            overall_churn: 0.15,
            timestamp: 0,
        };

        let segments = vec![];
        let kpis = vec![];
        let streaming = DashboardMetrics::create_streaming_metrics(500.0, 1000, 50, 100, 25.0);
        let drift_alerts = DriftAlertSummary {
            total_alerts: 0,
            critical_alerts: 0,
            high_alerts: 0,
            features_with_drift: vec![],
        };

        let dashboard = DashboardProvider::generate_dashboard(
            summary,
            segments,
            kpis,
            streaming,
            drift_alerts,
            TimeRange::Last7Days,
        )
        .unwrap();

        assert_eq!(dashboard.summary.total_customers, 1000);
    }

    #[test]
    fn test_export_summary() {
        let summary = DashboardSummary {
            total_customers: 1000,
            total_segments: 5,
            avg_segment_size: 200,
            total_revenue: 100000.0,
            avg_customer_ltv: 100.0,
            overall_retention: 0.85,
            overall_churn: 0.15,
            timestamp: 0,
        };

        let segments = vec![];
        let kpis = vec![];
        let streaming = DashboardMetrics::create_streaming_metrics(500.0, 1000, 50, 100, 25.0);
        let drift_alerts = DriftAlertSummary {
            total_alerts: 0,
            critical_alerts: 0,
            high_alerts: 0,
            features_with_drift: vec![],
        };

        let dashboard = DashboardProvider::generate_dashboard(
            summary,
            segments,
            kpis,
            streaming,
            drift_alerts,
            TimeRange::Last7Days,
        )
        .unwrap();

        let export = DashboardProvider::export_summary(&dashboard).unwrap();

        assert!(export.contains_key("total_customers"));
        assert!(export.contains_key("retention_rate"));
        assert!(export.contains_key("critical_alerts"));
    }

    #[test]
    fn test_metrics_point() {
        let point = DashboardMetrics::calculate_metrics_point(1704067200, 1000, 900, 0.1, 50000.0);

        assert_eq!(point.segment_size, 1000);
        assert_eq!(point.active_customers, 900);
        assert_eq!(point.churn_rate, 0.1);
    }

    #[test]
    fn test_drift_alert_summary() {
        let summary = DriftAlertSummary {
            total_alerts: 5,
            critical_alerts: 1,
            high_alerts: 2,
            features_with_drift: vec!["recency".to_string(), "frequency".to_string()],
        };

        assert_eq!(summary.total_alerts, 5);
        assert_eq!(summary.features_with_drift.len(), 2);
    }
}
