//! Cohort analytics and retention tracking

use crate::Result;
use std::collections::HashMap;

/// Cohort grouping period
#[derive(Clone, Debug, Copy, Eq, PartialEq, Hash)]
pub enum CohortPeriod {
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl CohortPeriod {
    pub fn days(&self) -> i32 {
        match self {
            CohortPeriod::Weekly => 7,
            CohortPeriod::Monthly => 30,
            CohortPeriod::Quarterly => 90,
            CohortPeriod::Yearly => 365,
        }
    }
}

/// Cohort identifier (e.g., "2024-01" for January 2024)
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CohortId(pub String);

impl CohortId {
    pub fn new(period: CohortPeriod, date: i64) -> Self {
        let days_since_epoch = date / 86400;
        let cohort_number = match period {
            CohortPeriod::Weekly => days_since_epoch / 7,
            CohortPeriod::Monthly => days_since_epoch / 30,
            CohortPeriod::Quarterly => days_since_epoch / 90,
            CohortPeriod::Yearly => days_since_epoch / 365,
        };
        CohortId(format!("cohort_{}", cohort_number))
    }
}

/// Retention curve point
#[derive(Clone, Debug)]
pub struct RetentionPoint {
    pub age_in_periods: usize, // How many periods since cohort creation
    pub retained_count: usize, // Customers still active
    pub churn_rate: f64,       // 0-1 churn rate at this point
    pub retention_rate: f64,   // 0-1 retention rate (1 - churn_rate)
}

/// Cohort profile with retention analytics
#[derive(Clone, Debug)]
pub struct Cohort {
    pub cohort_id: CohortId,
    pub period: CohortPeriod,
    pub size: usize,     // Initial cohort size
    pub created_at: i64, // Unix timestamp
    pub revenue: f64,    // Total revenue
    pub avg_ltv: f64,    // Average lifetime value
    pub retention_curve: Vec<RetentionPoint>,
    pub churn_rate_total: f64,     // Overall churn rate
    pub retention_rate_total: f64, // Overall retention rate
}

/// Cohort comparison metrics
#[derive(Clone, Debug)]
pub struct CohortComparison {
    pub cohort_a: CohortId,
    pub cohort_b: CohortId,
    pub size_diff: i32,
    pub revenue_diff: f64,
    pub ltv_diff: f64,
    pub retention_rate_diff: f64,
    pub better_performer: CohortId,
}

/// Custom analytics query result
#[derive(Clone, Debug)]
pub struct AnalyticsResult {
    pub metric_name: String,
    pub value: f64,
    pub cohort_id: Option<CohortId>,
    pub period: Option<String>,
}

/// Cohort manager
pub struct CohortAnalytics;

impl CohortAnalytics {
    /// Create a cohort from customer data
    pub fn create_cohort(
        cohort_id: CohortId,
        period: CohortPeriod,
        created_at: i64,
        customers: &[(String, f64, bool)], // (id, ltv, is_retained)
    ) -> Result<Cohort> {
        let size = customers.len();
        let revenue: f64 = customers.iter().map(|(_, ltv, _)| ltv).sum();
        let avg_ltv = if size > 0 { revenue / size as f64 } else { 0.0 };

        let retained = customers.iter().filter(|(_, _, r)| *r).count();
        let retention_rate_total = if size > 0 {
            retained as f64 / size as f64
        } else {
            0.0
        };

        let retention_point = RetentionPoint {
            age_in_periods: 0,
            retained_count: retained,
            churn_rate: 1.0 - retention_rate_total,
            retention_rate: retention_rate_total,
        };

        Ok(Cohort {
            cohort_id,
            period,
            size,
            created_at,
            revenue,
            avg_ltv,
            retention_curve: vec![retention_point],
            churn_rate_total: 1.0 - retention_rate_total,
            retention_rate_total,
        })
    }

    /// Add retention data to a cohort (for time-series tracking)
    pub fn add_retention_point(
        cohort: &mut Cohort,
        age_in_periods: usize,
        retained_count: usize,
    ) -> Result<()> {
        let retention_rate = if cohort.size > 0 {
            retained_count as f64 / cohort.size as f64
        } else {
            0.0
        };

        let point = RetentionPoint {
            age_in_periods,
            retained_count,
            churn_rate: 1.0 - retention_rate,
            retention_rate,
        };

        cohort.retention_curve.push(point);
        Ok(())
    }

    /// Calculate retention curve decay rate
    pub fn retention_decay_rate(cohort: &Cohort) -> Result<f64> {
        if cohort.retention_curve.len() < 2 {
            return Ok(0.0);
        }

        let first = &cohort.retention_curve[0];
        let last = &cohort.retention_curve[cohort.retention_curve.len() - 1];

        let periods_diff = last.age_in_periods as f64 - first.age_in_periods as f64;
        if periods_diff == 0.0 {
            return Ok(0.0);
        }

        let decay = (last.retention_rate - first.retention_rate) / periods_diff;
        Ok(decay)
    }

    /// Compare two cohorts
    pub fn compare_cohorts(cohort_a: &Cohort, cohort_b: &Cohort) -> Result<CohortComparison> {
        let size_diff = cohort_b.size as i32 - cohort_a.size as i32;
        let revenue_diff = cohort_b.revenue - cohort_a.revenue;
        let ltv_diff = cohort_b.avg_ltv - cohort_a.avg_ltv;
        let retention_diff = cohort_b.retention_rate_total - cohort_a.retention_rate_total;

        let better_performer = if retention_diff > 0.0 {
            cohort_b.cohort_id.clone()
        } else {
            cohort_a.cohort_id.clone()
        };

        Ok(CohortComparison {
            cohort_a: cohort_a.cohort_id.clone(),
            cohort_b: cohort_b.cohort_id.clone(),
            size_diff,
            revenue_diff,
            ltv_diff,
            retention_rate_diff: retention_diff,
            better_performer,
        })
    }

    /// Calculate revenue per retained customer
    pub fn revenue_per_retained(cohort: &Cohort) -> Result<f64> {
        if cohort.retention_curve.is_empty() {
            return Ok(0.0);
        }

        let last_point = &cohort.retention_curve[cohort.retention_curve.len() - 1];
        if last_point.retained_count == 0 {
            return Ok(0.0);
        }

        Ok(cohort.revenue / last_point.retained_count as f64)
    }

    /// Get cohort statistics summary
    pub fn cohort_summary(cohort: &Cohort) -> Result<HashMap<String, f64>> {
        let mut summary = HashMap::new();

        summary.insert("size".to_string(), cohort.size as f64);
        summary.insert("revenue".to_string(), cohort.revenue);
        summary.insert("avg_ltv".to_string(), cohort.avg_ltv);
        summary.insert("retention_rate".to_string(), cohort.retention_rate_total);
        summary.insert("churn_rate".to_string(), cohort.churn_rate_total);
        summary.insert(
            "decay_rate".to_string(),
            Self::retention_decay_rate(cohort)?,
        );
        summary.insert(
            "revenue_per_retained".to_string(),
            Self::revenue_per_retained(cohort)?,
        );

        Ok(summary)
    }

    /// Group cohorts by period and calculate aggregate metrics
    pub fn aggregate_by_period(
        cohorts: &[Cohort],
    ) -> Result<HashMap<String, HashMap<String, f64>>> {
        let mut aggregates: HashMap<String, HashMap<String, f64>> = HashMap::new();

        for cohort in cohorts {
            let period_key = cohort.cohort_id.0.clone();

            let entry = aggregates.entry(period_key).or_default();

            *entry.entry("total_size".to_string()).or_insert(0.0) += cohort.size as f64;
            *entry.entry("total_revenue".to_string()).or_insert(0.0) += cohort.revenue;
            *entry.entry("avg_retention".to_string()).or_insert(0.0) += cohort.retention_rate_total;
            *entry.entry("cohort_count".to_string()).or_insert(0.0) += 1.0;
        }

        // Calculate averages
        for metrics in aggregates.values_mut() {
            if let Some(count) = metrics.get("cohort_count") {
                if *count > 0.0 {
                    let avg_retention = metrics["avg_retention"] / count;
                    metrics.insert("avg_retention".to_string(), avg_retention);
                }
            }
        }

        Ok(aggregates)
    }

    /// Calculate retention cohort table
    pub fn retention_table(cohorts: &[Cohort]) -> Result<Vec<Vec<f64>>> {
        if cohorts.is_empty() {
            return Ok(vec![]);
        }

        let max_age = cohorts
            .iter()
            .flat_map(|c| c.retention_curve.iter().map(|p| p.age_in_periods))
            .max()
            .unwrap_or(0);

        let mut table: Vec<Vec<f64>> = vec![vec![0.0; max_age + 1]; cohorts.len()];

        for (i, cohort) in cohorts.iter().enumerate() {
            for point in &cohort.retention_curve {
                if point.age_in_periods <= max_age {
                    table[i][point.age_in_periods] = point.retention_rate;
                }
            }
        }

        Ok(table)
    }

    /// Identify best and worst performing cohorts
    pub fn performance_ranking(cohorts: &[Cohort]) -> Result<(Option<CohortId>, Option<CohortId>)> {
        if cohorts.is_empty() {
            return Ok((None, None));
        }

        let best = cohorts
            .iter()
            .max_by(|a, b| {
                a.retention_rate_total
                    .partial_cmp(&b.retention_rate_total)
                    .unwrap()
            })
            .map(|c| c.cohort_id.clone());

        let worst = cohorts
            .iter()
            .min_by(|a, b| {
                a.retention_rate_total
                    .partial_cmp(&b.retention_rate_total)
                    .unwrap()
            })
            .map(|c| c.cohort_id.clone());

        Ok((best, worst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cohort_creation() {
        let customers = vec![
            ("c1".to_string(), 100.0, true),
            ("c2".to_string(), 200.0, true),
            ("c3".to_string(), 150.0, false),
        ];

        let cohort = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers,
        )
        .unwrap();

        assert_eq!(cohort.size, 3);
        assert_eq!(cohort.revenue, 450.0);
        assert_eq!(cohort.avg_ltv, 150.0);
        assert!(cohort.retention_rate_total > 0.6);
    }

    #[test]
    fn test_retention_curve() {
        let customers = vec![
            ("c1".to_string(), 100.0, true),
            ("c2".to_string(), 100.0, true),
            ("c3".to_string(), 100.0, true),
            ("c4".to_string(), 100.0, false),
        ];

        let mut cohort = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers,
        )
        .unwrap();

        CohortAnalytics::add_retention_point(&mut cohort, 1, 3).unwrap();
        CohortAnalytics::add_retention_point(&mut cohort, 2, 2).unwrap();

        assert_eq!(cohort.retention_curve.len(), 3);
        assert!(
            cohort.retention_curve[2].retention_rate < cohort.retention_curve[1].retention_rate
        );
    }

    #[test]
    fn test_decay_rate() {
        let customers = vec![("c1".to_string(), 100.0, true); 10];
        let mut cohort = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers,
        )
        .unwrap();

        CohortAnalytics::add_retention_point(&mut cohort, 1, 9).unwrap();
        CohortAnalytics::add_retention_point(&mut cohort, 2, 7).unwrap();
        CohortAnalytics::add_retention_point(&mut cohort, 3, 5).unwrap();

        let decay = CohortAnalytics::retention_decay_rate(&cohort).unwrap();
        assert!(decay < 0.0); // Should be negative (declining retention)
    }

    #[test]
    fn test_cohort_comparison() {
        let customers_a = vec![("c1".to_string(), 100.0, true); 100];
        let customers_b = vec![("c1".to_string(), 120.0, true); 100];

        let cohort_a = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers_a,
        )
        .unwrap();

        let cohort_b = CohortAnalytics::create_cohort(
            CohortId("2024-02".to_string()),
            CohortPeriod::Monthly,
            1706745600,
            &customers_b,
        )
        .unwrap();

        let comparison = CohortAnalytics::compare_cohorts(&cohort_a, &cohort_b).unwrap();
        assert_eq!(comparison.size_diff, 0);
        assert!(comparison.revenue_diff > 0.0);
    }

    #[test]
    fn test_revenue_per_retained() {
        let customers = vec![("c1".to_string(), 100.0, true); 10];
        let mut cohort = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers,
        )
        .unwrap();

        CohortAnalytics::add_retention_point(&mut cohort, 1, 5).unwrap();

        let rev_per_retained = CohortAnalytics::revenue_per_retained(&cohort).unwrap();
        assert!(rev_per_retained > 0.0);
    }

    #[test]
    fn test_cohort_summary() {
        let customers = vec![("c1".to_string(), 100.0, true); 50];
        let cohort = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers,
        )
        .unwrap();

        let summary = CohortAnalytics::cohort_summary(&cohort).unwrap();

        assert_eq!(summary.get("size"), Some(&50.0));
        assert_eq!(summary.get("revenue"), Some(&5000.0));
        assert!(summary.contains_key("retention_rate"));
    }

    #[test]
    fn test_performance_ranking() {
        let customers_good = vec![("c1".to_string(), 100.0, true); 100];
        let customers_poor = vec![("c1".to_string(), 100.0, false); 100];

        let cohort_good = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers_good,
        )
        .unwrap();

        let cohort_poor = CohortAnalytics::create_cohort(
            CohortId("2024-02".to_string()),
            CohortPeriod::Monthly,
            1706745600,
            &customers_poor,
        )
        .unwrap();

        let (best, worst) =
            CohortAnalytics::performance_ranking(&[cohort_good, cohort_poor]).unwrap();

        assert!(best.is_some());
        assert!(worst.is_some());
    }

    #[test]
    fn test_retention_table() {
        let customers = vec![("c1".to_string(), 100.0, true); 10];
        let mut cohort = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers,
        )
        .unwrap();

        CohortAnalytics::add_retention_point(&mut cohort, 1, 9).unwrap();
        CohortAnalytics::add_retention_point(&mut cohort, 2, 7).unwrap();

        let table = CohortAnalytics::retention_table(&[cohort]).unwrap();

        assert_eq!(table.len(), 1);
        assert!(table[0].len() >= 3);
    }

    #[test]
    fn test_aggregate_by_period() {
        let customers = vec![("c1".to_string(), 100.0, true); 50];

        let cohort1 = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers,
        )
        .unwrap();

        let cohort2 = CohortAnalytics::create_cohort(
            CohortId("2024-01".to_string()),
            CohortPeriod::Monthly,
            1704067200,
            &customers,
        )
        .unwrap();

        let aggregates = CohortAnalytics::aggregate_by_period(&[cohort1, cohort2]).unwrap();

        assert!(!aggregates.is_empty());
    }
}
