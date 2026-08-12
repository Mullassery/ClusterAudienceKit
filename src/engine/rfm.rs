//! RFM (Recency-Frequency-Monetary) calculation engine

use crate::Result;
#[cfg(test)]
use chrono::Duration;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::collections::HashMap;

/// Decay function for RFM weighting
#[derive(Clone, Debug, Copy)]
pub enum DecayFunction {
    Linear,
    Exponential,
    Inverse,
}

impl std::fmt::Display for DecayFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecayFunction::Linear => write!(f, "linear"),
            DecayFunction::Exponential => write!(f, "exponential"),
            DecayFunction::Inverse => write!(f, "inverse"),
        }
    }
}

/// Scoring method for RFM
#[derive(Clone, Debug, Copy)]
pub enum ScoringMethod {
    Quintile,   // 1-5 scale
    Decile,     // 1-10 scale
    Percentile, // 0-100 scale
}

/// RFM configuration
#[derive(Clone, Debug)]
pub struct RFMConfig {
    pub recency_window_days: u32,
    pub frequency_threshold: usize,
    pub monetary_threshold: f64,
    pub decay_function: DecayFunction,
    pub decay_half_life_days: u32,
    pub scoring_method: ScoringMethod,
}

impl Default for RFMConfig {
    fn default() -> Self {
        Self {
            recency_window_days: 90,
            frequency_threshold: 1,
            monetary_threshold: 0.0,
            decay_function: DecayFunction::Linear,
            decay_half_life_days: 30,
            scoring_method: ScoringMethod::Quintile,
        }
    }
}

/// RFM score for a single customer
#[derive(Clone, Debug)]
pub struct RFMScore {
    pub customer_id: String,
    pub recency: f64,
    pub frequency: f64,
    pub monetary: f64,
    pub recency_score: u32,
    pub frequency_score: u32,
    pub monetary_score: u32,
    pub rfm_segment: String,
}

impl RFMScore {
    pub fn rfm_rank(&self) -> String {
        format!(
            "{}{}{}",
            self.recency_score, self.frequency_score, self.monetary_score
        )
    }
}

/// Calculate RFM scores from transaction data
pub fn calculate_rfm(transactions: Vec<Transaction>, config: &RFMConfig) -> Result<Vec<RFMScore>> {
    if transactions.is_empty() {
        return Ok(vec![]);
    }

    // Group transactions by customer
    let mut customer_data: HashMap<String, Vec<Transaction>> = HashMap::new();
    for tx in transactions {
        customer_data
            .entry(tx.customer_id.clone())
            .or_default()
            .push(tx);
    }

    let reference_date = Utc::now();

    // Per-customer RFM computation is independent across customers, so for
    // datasets with many distinct customers this is a straightforward
    // rayon parallel map/filter over the grouped transaction lists.
    let mut scores: Vec<RFMScore> = customer_data
        .into_par_iter()
        .filter_map(|(customer_id, txs)| {
            calculate_customer_rfm(&customer_id, &txs, &reference_date, config).ok()
        })
        .collect();

    // Apply scoring
    apply_scoring(&mut scores, config.scoring_method)?;

    // Add segment classification
    classify_segments(&mut scores)?;

    Ok(scores)
}

fn calculate_customer_rfm(
    customer_id: &str,
    transactions: &[Transaction],
    reference_date: &DateTime<Utc>,
    config: &RFMConfig,
) -> Result<RFMScore> {
    let mut latest_date: Option<DateTime<Utc>> = None;
    let mut frequency = 0;
    let mut monetary = 0.0;

    for tx in transactions {
        if let Ok(tx_date) = DateTime::parse_from_rfc3339(&tx.date) {
            let tx_datetime = tx_date.with_timezone(&Utc);

            // Update latest date
            if latest_date.is_none() || tx_datetime > latest_date.unwrap() {
                latest_date = Some(tx_datetime);
            }

            // Count frequency and sum monetary
            frequency += 1;
            monetary += tx.amount;
        }
    }

    // Calculate recency in days
    let recency = if let Some(latest) = latest_date {
        let days_since = (*reference_date - latest).num_days() as f64;
        apply_decay_function(
            days_since,
            config.decay_function,
            config.decay_half_life_days,
        )
    } else {
        0.0
    };

    Ok(RFMScore {
        customer_id: customer_id.to_string(),
        recency,
        frequency: frequency as f64,
        monetary,
        recency_score: 0,
        frequency_score: 0,
        monetary_score: 0,
        rfm_segment: String::new(),
    })
}

fn apply_decay_function(days: f64, decay: DecayFunction, half_life: u32) -> f64 {
    match decay {
        DecayFunction::Linear => {
            // Linear decay: higher recency (fewer days) = higher score
            let max_days = (half_life * 3) as f64;
            ((max_days - days) / max_days).clamp(0.0, 1.0)
        }
        DecayFunction::Exponential => {
            // Exponential decay with half-life
            let half_life_f = half_life as f64;
            (2.0_f64).powf(-days / half_life_f).min(1.0)
        }
        DecayFunction::Inverse => {
            // Inverse decay: 1 / (1 + days)
            1.0 / (1.0 + days / (half_life as f64))
        }
    }
}

fn apply_scoring(scores: &mut [RFMScore], method: ScoringMethod) -> Result<()> {
    if scores.is_empty() {
        return Ok(());
    }

    // Sort and score each dimension
    score_dimension(scores, method, |s| s.recency, |s| &mut s.recency_score)?;
    score_dimension(scores, method, |s| s.frequency, |s| &mut s.frequency_score)?;
    score_dimension(scores, method, |s| s.monetary, |s| &mut s.monetary_score)?;

    Ok(())
}

fn score_dimension<F, G>(
    scores: &mut [RFMScore],
    method: ScoringMethod,
    getter: F,
    setter: G,
) -> Result<()>
where
    F: Fn(&RFMScore) -> f64,
    G: Fn(&mut RFMScore) -> &mut u32,
{
    // Sort by dimension value
    scores.sort_by(|a, b| {
        getter(b)
            .partial_cmp(&getter(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total = scores.len() as f64;
    let max_score = match method {
        ScoringMethod::Quintile => 5,
        ScoringMethod::Decile => 10,
        ScoringMethod::Percentile => 100,
    };

    for (idx, score) in scores.iter_mut().enumerate() {
        let percentile = (idx as f64) / total;
        let rank = match method {
            ScoringMethod::Quintile => (percentile * 5.0).ceil() as u32,
            ScoringMethod::Decile => (percentile * 10.0).ceil() as u32,
            ScoringMethod::Percentile => ((1.0 - percentile) * 100.0) as u32,
        }
        .max(1)
        .min(max_score);

        *setter(score) = rank;
    }

    Ok(())
}

fn classify_segments(scores: &mut [RFMScore]) -> Result<()> {
    for score in scores {
        let r = score.recency_score;
        let f = score.frequency_score;
        let m = score.monetary_score;

        score.rfm_segment = match (r >= 4, f >= 4, m >= 4) {
            (true, true, true) => "Champions",
            (true, true, false) => "Loyal Customers",
            (true, false, true) => "Potential Loyalists",
            (true, false, false) => "At Risk",
            (false, true, true) => "VIP",
            (false, true, false) => "Need Attention",
            (false, false, true) => "Promising",
            (false, false, false) => "Lost",
        }
        .to_string();
    }

    Ok(())
}

/// Normalize RFM scores to [0, 1]
pub fn normalize_rfm(scores: &mut [RFMScore]) -> Result<()> {
    if scores.is_empty() {
        return Ok(());
    }

    // Find min/max for each dimension
    let mut min_r = f64::MAX;
    let mut max_r = f64::MIN;
    let mut min_f = f64::MAX;
    let mut max_f = f64::MIN;
    let mut min_m = f64::MAX;
    let mut max_m = f64::MIN;

    for score in scores.iter() {
        min_r = min_r.min(score.recency);
        max_r = max_r.max(score.recency);
        min_f = min_f.min(score.frequency);
        max_f = max_f.max(score.frequency);
        min_m = min_m.min(score.monetary);
        max_m = max_m.max(score.monetary);
    }

    let range_r = max_r - min_r;
    let range_f = max_f - min_f;
    let range_m = max_m - min_m;

    // Normalize each score
    for score in scores.iter_mut() {
        score.recency = if range_r > 0.0 {
            (score.recency - min_r) / range_r
        } else {
            0.5
        };

        score.frequency = if range_f > 0.0 {
            (score.frequency - min_f) / range_f
        } else {
            0.5
        };

        score.monetary = if range_m > 0.0 {
            (score.monetary - min_m) / range_m
        } else {
            0.5
        };
    }

    Ok(())
}

/// Transaction data
#[derive(Clone, Debug)]
pub struct Transaction {
    pub customer_id: String,
    pub date: String, // ISO 8601 date string
    pub amount: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_function_display() {
        assert_eq!(DecayFunction::Linear.to_string(), "linear");
        assert_eq!(DecayFunction::Exponential.to_string(), "exponential");
        assert_eq!(DecayFunction::Inverse.to_string(), "inverse");
    }

    #[test]
    fn test_rfm_config_default() {
        let config = RFMConfig::default();
        assert_eq!(config.recency_window_days, 90);
        assert_eq!(config.frequency_threshold, 1);
        assert_eq!(config.monetary_threshold, 0.0);
    }

    #[test]
    fn test_linear_decay() {
        let score = apply_decay_function(10.0, DecayFunction::Linear, 30);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_exponential_decay() {
        let score = apply_decay_function(10.0, DecayFunction::Exponential, 30);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_inverse_decay() {
        let score = apply_decay_function(10.0, DecayFunction::Inverse, 30);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_calculate_rfm() {
        let transactions = vec![
            Transaction {
                customer_id: "cust1".to_string(),
                date: Utc::now().to_rfc3339(),
                amount: 100.0,
            },
            Transaction {
                customer_id: "cust1".to_string(),
                date: (Utc::now() - Duration::days(10)).to_rfc3339(),
                amount: 50.0,
            },
        ];

        let config = RFMConfig::default();
        let scores = calculate_rfm(transactions, &config).unwrap();

        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].customer_id, "cust1");
        assert_eq!(scores[0].frequency, 2.0);
        assert_eq!(scores[0].monetary, 150.0);
    }

    #[test]
    fn test_rfm_scoring() {
        let mut scores = vec![
            RFMScore {
                customer_id: "c1".to_string(),
                recency: 0.9,
                frequency: 10.0,
                monetary: 1000.0,
                recency_score: 0,
                frequency_score: 0,
                monetary_score: 0,
                rfm_segment: String::new(),
            },
            RFMScore {
                customer_id: "c2".to_string(),
                recency: 0.5,
                frequency: 5.0,
                monetary: 500.0,
                recency_score: 0,
                frequency_score: 0,
                monetary_score: 0,
                rfm_segment: String::new(),
            },
        ];

        apply_scoring(&mut scores, ScoringMethod::Quintile).unwrap();

        assert!(scores[0].recency_score > 0);
        assert!(scores[0].frequency_score > 0);
        assert!(scores[0].monetary_score > 0);
    }

    #[test]
    fn test_segment_classification() {
        let mut scores = vec![RFMScore {
            customer_id: "c1".to_string(),
            recency: 0.9,
            frequency: 10.0,
            monetary: 1000.0,
            recency_score: 5,
            frequency_score: 5,
            monetary_score: 5,
            rfm_segment: String::new(),
        }];

        classify_segments(&mut scores).unwrap();
        assert_eq!(scores[0].rfm_segment, "Champions");
    }

    #[test]
    fn test_normalize_rfm() {
        let mut scores = vec![
            RFMScore {
                customer_id: "c1".to_string(),
                recency: 100.0,
                frequency: 50.0,
                monetary: 5000.0,
                recency_score: 0,
                frequency_score: 0,
                monetary_score: 0,
                rfm_segment: String::new(),
            },
            RFMScore {
                customer_id: "c2".to_string(),
                recency: 50.0,
                frequency: 25.0,
                monetary: 2500.0,
                recency_score: 0,
                frequency_score: 0,
                monetary_score: 0,
                rfm_segment: String::new(),
            },
        ];

        normalize_rfm(&mut scores).unwrap();

        for score in &scores {
            assert!(score.recency >= 0.0 && score.recency <= 1.0);
            assert!(score.frequency >= 0.0 && score.frequency <= 1.0);
            assert!(score.monetary >= 0.0 && score.monetary <= 1.0);
        }
    }
}
