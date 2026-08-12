//! Predictive churn modeling and risk scoring

use crate::Result;
use std::collections::HashMap;

/// Churn risk level
#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum ChurnRiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    Critical,
}

impl ChurnRiskLevel {
    pub fn as_str(&self) -> &str {
        match self {
            ChurnRiskLevel::VeryLow => "very_low",
            ChurnRiskLevel::Low => "low",
            ChurnRiskLevel::Medium => "medium",
            ChurnRiskLevel::High => "high",
            ChurnRiskLevel::Critical => "critical",
        }
    }

    pub fn threshold(&self) -> f64 {
        match self {
            ChurnRiskLevel::VeryLow => 0.1,
            ChurnRiskLevel::Low => 0.25,
            ChurnRiskLevel::Medium => 0.5,
            ChurnRiskLevel::High => 0.75,
            ChurnRiskLevel::Critical => 0.9,
        }
    }
}

/// Feature importance in churn prediction
#[derive(Clone, Debug)]
pub struct ChurnFeatureImportance {
    pub feature_name: String,
    pub importance_score: f64,
    pub coefficient: f64,
    pub direction: String, // "positive" = increases churn, "negative" = decreases churn
}

/// Churn prediction model types
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum ChurnModelType {
    LogisticRegression,
    RandomForest,
    GradientBoosting,
}

impl ChurnModelType {
    pub fn as_str(&self) -> &str {
        match self {
            ChurnModelType::LogisticRegression => "logistic_regression",
            ChurnModelType::RandomForest => "random_forest",
            ChurnModelType::GradientBoosting => "gradient_boosting",
        }
    }
}

/// Churn prediction for a customer
#[derive(Clone, Debug)]
pub struct ChurnPrediction {
    pub customer_id: String,
    pub churn_probability: f64,
    pub risk_level: ChurnRiskLevel,
    pub confidence: f64,
    pub days_until_churn_estimate: i32,
    pub feature_contributions: HashMap<String, f64>,
    pub model_type: ChurnModelType,
    pub timestamp: i64,
}

/// Churn retention action
#[derive(Clone, Debug)]
pub struct RetentionAction {
    pub customer_id: String,
    pub action_type: String, // "discount", "email_campaign", "product_upgrade", "vip_support"
    pub urgency: String,     // "low", "medium", "high", "critical"
    pub recommended_offer: Option<String>,
    pub estimated_save_probability: f64,
}

/// Churn cohort summary
#[derive(Clone, Debug)]
pub struct ChurnCohortSummary {
    pub cohort_name: String,
    pub customer_count: usize,
    pub avg_churn_probability: f64,
    pub critical_churn_count: usize,
    pub high_churn_count: usize,
    pub retention_potential: f64,
    pub estimated_ltv_at_risk: f64,
}

/// Churn model performance metrics
#[derive(Clone, Debug)]
pub struct ChurnModelPerformance {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub auc_roc: f64,
    pub training_samples: usize,
    pub model_type: ChurnModelType,
}

/// Churn predictor
pub struct ChurnPredictor;

impl ChurnPredictor {
    /// Logistic regression-based churn prediction
    /// Uses customer engagement metrics to predict churn probability
    pub fn predict_logistic(
        recency: f64,
        frequency: f64,
        monetary: f64,
        days_since_signup: f64,
        engagement_score: f64,
        support_tickets: f64,
    ) -> Result<f64> {
        // Normalize inputs (0-1)
        let recency_norm = (recency / 90.0).min(1.0);
        let frequency_norm = (frequency / 20.0).min(1.0);
        let monetary_norm = (monetary / 5000.0).min(1.0);
        let days_norm = (days_since_signup / 1000.0).min(1.0);
        let engagement_norm = engagement_score.min(1.0);

        // Logistic coefficients (from training data)
        let coef_recency = 0.35;
        let coef_frequency = -0.4;
        let coef_monetary = -0.3;
        let coef_days = -0.15;
        let coef_engagement = -0.25;
        let coef_support = 0.1;
        let intercept = -0.5;

        let logit = intercept
            + (coef_recency * recency_norm)
            + (coef_frequency * frequency_norm)
            + (coef_monetary * monetary_norm)
            + (coef_days * days_norm)
            + (coef_engagement * engagement_norm)
            + (coef_support * support_tickets);

        // Sigmoid function
        let probability = 1.0 / (1.0 + (-logit).exp());
        Ok(probability.clamp(0.0, 1.0))
    }

    /// Random forest-like ensemble prediction
    pub fn predict_ensemble(
        recency: f64,
        frequency: f64,
        monetary: f64,
        days_since_signup: f64,
        engagement_score: f64,
        _support_tickets: f64,
    ) -> Result<f64> {
        // Tree 1: Engagement-based
        let tree1 = if engagement_score < 0.3 {
            0.75
        } else if engagement_score < 0.6 {
            0.5
        } else {
            0.2
        };

        // Tree 2: Recency-based
        let tree2 = if recency > 60.0 {
            0.8
        } else if recency > 30.0 {
            0.5
        } else {
            0.15
        };

        // Tree 3: Monetary-based
        let tree3 = if monetary < 100.0 {
            0.7
        } else if monetary < 500.0 {
            0.4
        } else {
            0.15
        };

        // Tree 4: Frequency-based
        let tree4 = if frequency < 2.0 {
            0.65
        } else if frequency < 5.0 {
            0.35
        } else {
            0.1
        };

        // Tree 5: Age-based
        let tree5 = if days_since_signup < 90.0 {
            0.6
        } else if days_since_signup < 365.0 {
            0.3
        } else {
            0.1
        };

        let avg = (tree1 + tree2 + tree3 + tree4 + tree5) / 5.0;
        Ok(avg)
    }

    /// Calculate feature importance scores
    pub fn calculate_feature_importance(
        recency: f64,
        frequency: f64,
        monetary: f64,
        _engagement_score: f64,
    ) -> Result<Vec<ChurnFeatureImportance>> {
        let mut importances = vec![
            ChurnFeatureImportance {
                feature_name: "recency".to_string(),
                importance_score: 0.35,
                coefficient: 0.35,
                direction: "positive".to_string(),
            },
            ChurnFeatureImportance {
                feature_name: "frequency".to_string(),
                importance_score: 0.25,
                coefficient: -0.4,
                direction: "negative".to_string(),
            },
            ChurnFeatureImportance {
                feature_name: "monetary".to_string(),
                importance_score: 0.25,
                coefficient: -0.3,
                direction: "negative".to_string(),
            },
            ChurnFeatureImportance {
                feature_name: "engagement_score".to_string(),
                importance_score: 0.15,
                coefficient: -0.25,
                direction: "negative".to_string(),
            },
        ];

        // Weight by actual values
        if recency > 60.0 {
            importances[0].importance_score = 0.45;
        }
        if frequency < 2.0 {
            importances[1].importance_score = 0.35;
        }
        if monetary < 100.0 {
            importances[2].importance_score = 0.35;
        }

        Ok(importances)
    }

    /// Classify churn risk level
    pub fn classify_risk_level(churn_probability: f64) -> ChurnRiskLevel {
        if churn_probability < 0.1 {
            ChurnRiskLevel::VeryLow
        } else if churn_probability < 0.25 {
            ChurnRiskLevel::Low
        } else if churn_probability < 0.5 {
            ChurnRiskLevel::Medium
        } else if churn_probability < 0.75 {
            ChurnRiskLevel::High
        } else {
            ChurnRiskLevel::Critical
        }
    }

    /// Estimate days until churn
    pub fn estimate_days_until_churn(churn_probability: f64, recency: f64) -> i32 {
        let base_days = 90;
        let adjustment = (churn_probability * 90.0) as i32;
        let recency_factor = (recency / 30.0) as i32;

        (base_days - adjustment + recency_factor).max(1)
    }

    /// Generate retention action recommendation
    pub fn recommend_retention_action(
        churn_probability: f64,
        monetary: f64,
        engagement_score: f64,
    ) -> Result<RetentionAction> {
        let risk_level = Self::classify_risk_level(churn_probability);
        let urgency = risk_level.as_str().to_string();

        let action_type = if engagement_score < 0.3 {
            "email_campaign".to_string()
        } else if monetary < 100.0 {
            "discount".to_string()
        } else {
            "vip_support".to_string()
        };

        let recommended_offer = match risk_level {
            ChurnRiskLevel::Critical => Some("20% discount + priority support".to_string()),
            ChurnRiskLevel::High => Some("15% discount".to_string()),
            ChurnRiskLevel::Medium => Some("10% discount".to_string()),
            _ => None,
        };

        let save_prob = 1.0 - churn_probability;

        Ok(RetentionAction {
            customer_id: "".to_string(),
            action_type,
            urgency,
            recommended_offer,
            estimated_save_probability: save_prob,
        })
    }

    /// Predict churn for multiple customers
    pub fn predict_batch(
        customers: &[(String, f64, f64, f64, f64, f64)], // id, recency, frequency, monetary, engagement, days
    ) -> Result<Vec<ChurnPrediction>> {
        let mut predictions = Vec::new();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        for (customer_id, recency, frequency, monetary, engagement, days) in customers {
            let churn_prob =
                Self::predict_logistic(*recency, *frequency, *monetary, *days, *engagement, 0.0)?;

            let risk_level = Self::classify_risk_level(churn_prob);
            let days_until = Self::estimate_days_until_churn(churn_prob, *recency);

            let importances =
                Self::calculate_feature_importance(*recency, *frequency, *monetary, *engagement)?;
            let mut feature_map = HashMap::new();
            for imp in importances {
                feature_map.insert(imp.feature_name, imp.importance_score);
            }

            predictions.push(ChurnPrediction {
                customer_id: customer_id.clone(),
                churn_probability: churn_prob,
                risk_level,
                confidence: 0.85,
                days_until_churn_estimate: days_until,
                feature_contributions: feature_map,
                model_type: ChurnModelType::LogisticRegression,
                timestamp: now,
            });
        }

        Ok(predictions)
    }

    /// Summarize churn by cohort
    pub fn cohort_summary(
        predictions: &[ChurnPrediction],
        cohort_name: String,
    ) -> Result<ChurnCohortSummary> {
        if predictions.is_empty() {
            return Ok(ChurnCohortSummary {
                cohort_name,
                customer_count: 0,
                avg_churn_probability: 0.0,
                critical_churn_count: 0,
                high_churn_count: 0,
                retention_potential: 0.0,
                estimated_ltv_at_risk: 0.0,
            });
        }

        let avg_churn =
            predictions.iter().map(|p| p.churn_probability).sum::<f64>() / predictions.len() as f64;
        let critical_count = predictions
            .iter()
            .filter(|p| p.risk_level == ChurnRiskLevel::Critical)
            .count();
        let high_count = predictions
            .iter()
            .filter(|p| p.risk_level == ChurnRiskLevel::High)
            .count();

        Ok(ChurnCohortSummary {
            cohort_name,
            customer_count: predictions.len(),
            avg_churn_probability: avg_churn,
            critical_churn_count: critical_count,
            high_churn_count: high_count,
            retention_potential: 1.0 - avg_churn,
            estimated_ltv_at_risk: (critical_count + high_count) as f64 * 500.0,
        })
    }

    /// Calculate model performance metrics
    pub fn evaluate_model_performance(
        actual_churn: &[bool],
        predicted_probability: &[f64],
    ) -> Result<ChurnModelPerformance> {
        if actual_churn.len() != predicted_probability.len() || actual_churn.is_empty() {
            return Ok(ChurnModelPerformance {
                accuracy: 0.0,
                precision: 0.0,
                recall: 0.0,
                f1_score: 0.0,
                auc_roc: 0.0,
                training_samples: 0,
                model_type: ChurnModelType::LogisticRegression,
            });
        }

        let mut tp = 0;
        let mut tn = 0;
        let mut fp = 0;
        let mut fn_count = 0;

        for (actual, predicted) in actual_churn.iter().zip(predicted_probability.iter()) {
            let pred_churn = *predicted > 0.5;

            match (*actual, pred_churn) {
                (true, true) => tp += 1,
                (false, false) => tn += 1,
                (false, true) => fp += 1,
                (true, false) => fn_count += 1,
            }
        }

        let total = tp + tn + fp + fn_count;
        let accuracy = (tp + tn) as f64 / total as f64;
        let precision = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };
        let recall = if tp + fn_count > 0 {
            tp as f64 / (tp + fn_count) as f64
        } else {
            0.0
        };
        let f1 = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        Ok(ChurnModelPerformance {
            accuracy,
            precision,
            recall,
            f1_score: f1,
            auc_roc: compute_auc_roc(actual_churn, predicted_probability),
            training_samples: total,
            model_type: ChurnModelType::LogisticRegression,
        })
    }
}

/// Compute AUC-ROC via the trapezoidal rule over the ROC curve traced out by
/// sweeping the decision threshold across every distinct predicted score
/// (descending). This replaces a previously hardcoded `auc_roc: 0.82`
/// placeholder with a real calculation over the same (actual, predicted)
/// pairs already used for the confusion-matrix metrics above.
fn compute_auc_roc(actual_churn: &[bool], predicted_probability: &[f64]) -> f64 {
    let n_pos = actual_churn.iter().filter(|&&a| a).count();
    let n_neg = actual_churn.len() - n_pos;

    // AUC is undefined without at least one example of each class.
    if n_pos == 0 || n_neg == 0 {
        return 0.5;
    }

    // Sort (score, label) pairs by score descending; sweep the threshold
    // down through every distinct score, tracing out (FPR, TPR) points.
    let mut pairs: Vec<(f64, bool)> = predicted_probability
        .iter()
        .copied()
        .zip(actual_churn.iter().copied())
        .collect();
    pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut prev_score = f64::INFINITY;
    let mut prev_fpr = 0.0;
    let mut prev_tpr = 0.0;
    let mut auc = 0.0;

    for (score, is_positive) in pairs {
        if score != prev_score {
            let fpr = fp as f64 / n_neg as f64;
            let tpr = tp as f64 / n_pos as f64;
            // Trapezoidal area under this segment of the ROC curve.
            auc += (fpr - prev_fpr) * (tpr + prev_tpr) / 2.0;
            prev_fpr = fpr;
            prev_tpr = tpr;
            prev_score = score;
        }
        if is_positive {
            tp += 1;
        } else {
            fp += 1;
        }
    }

    // Close out the curve to (1, 1).
    let fpr = fp as f64 / n_neg as f64;
    let tpr = tp as f64 / n_pos as f64;
    auc += (fpr - prev_fpr) * (tpr + prev_tpr) / 2.0;

    auc.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logistic_prediction() {
        let prob = ChurnPredictor::predict_logistic(45.0, 5.0, 500.0, 365.0, 0.7, 1.0).unwrap();
        assert!((0.0..=1.0).contains(&prob));
    }

    #[test]
    fn test_ensemble_prediction() {
        let prob = ChurnPredictor::predict_ensemble(45.0, 5.0, 500.0, 365.0, 0.7, 1.0).unwrap();
        assert!((0.0..=1.0).contains(&prob));
    }

    #[test]
    fn test_risk_level_classification() {
        assert_eq!(
            ChurnPredictor::classify_risk_level(0.05),
            ChurnRiskLevel::VeryLow
        );
        assert_eq!(
            ChurnPredictor::classify_risk_level(0.3),
            ChurnRiskLevel::Medium
        );
        assert_eq!(
            ChurnPredictor::classify_risk_level(0.8),
            ChurnRiskLevel::Critical
        );
    }

    #[test]
    fn test_days_until_churn() {
        let days = ChurnPredictor::estimate_days_until_churn(0.8, 60.0);
        assert!(days > 0);
    }

    #[test]
    fn test_feature_importance() {
        let importances =
            ChurnPredictor::calculate_feature_importance(60.0, 2.0, 100.0, 0.2).unwrap();
        assert_eq!(importances.len(), 4);
        assert!(importances[0].importance_score > 0.0);
    }

    #[test]
    fn test_retention_action() {
        let action = ChurnPredictor::recommend_retention_action(0.85, 50.0, 0.2).unwrap();
        assert!(!action.action_type.is_empty());
        assert!(action.recommended_offer.is_some());
    }

    #[test]
    fn test_batch_prediction() {
        let customers = vec![
            ("cust_1".to_string(), 45.0, 5.0, 500.0, 0.7, 365.0),
            ("cust_2".to_string(), 75.0, 2.0, 100.0, 0.2, 100.0),
        ];

        let predictions = ChurnPredictor::predict_batch(&customers).unwrap();
        assert_eq!(predictions.len(), 2);
    }

    #[test]
    fn test_cohort_summary() {
        let predictions = vec![
            ChurnPrediction {
                customer_id: "cust_1".to_string(),
                churn_probability: 0.8,
                risk_level: ChurnRiskLevel::High,
                confidence: 0.85,
                days_until_churn_estimate: 30,
                feature_contributions: HashMap::new(),
                model_type: ChurnModelType::LogisticRegression,
                timestamp: 0,
            },
            ChurnPrediction {
                customer_id: "cust_2".to_string(),
                churn_probability: 0.2,
                risk_level: ChurnRiskLevel::Low,
                confidence: 0.85,
                days_until_churn_estimate: 90,
                feature_contributions: HashMap::new(),
                model_type: ChurnModelType::LogisticRegression,
                timestamp: 0,
            },
        ];

        let summary =
            ChurnPredictor::cohort_summary(&predictions, "High Value".to_string()).unwrap();
        assert_eq!(summary.customer_count, 2);
        assert!(summary.avg_churn_probability > 0.0);
    }

    #[test]
    fn test_model_performance() {
        let actual = vec![true, false, true, false, true];
        let predicted = vec![0.8, 0.2, 0.7, 0.3, 0.9];

        let perf = ChurnPredictor::evaluate_model_performance(&actual, &predicted).unwrap();
        assert!(perf.accuracy >= 0.0 && perf.accuracy <= 1.0);
        assert!(perf.f1_score >= 0.0);
        // Every positive score here strictly exceeds every negative score,
        // so a real AUC-ROC calculation must yield perfect separation.
        assert_eq!(perf.auc_roc, 1.0);
    }

    #[test]
    fn test_auc_roc_perfect_separation() {
        let actual = vec![true, true, false, false];
        let predicted = vec![0.9, 0.8, 0.2, 0.1];
        let perf = ChurnPredictor::evaluate_model_performance(&actual, &predicted).unwrap();
        assert_eq!(perf.auc_roc, 1.0);
    }

    #[test]
    fn test_auc_roc_perfectly_wrong() {
        // Every negative scored higher than every positive: AUC must be 0.
        let actual = vec![true, true, false, false];
        let predicted = vec![0.1, 0.2, 0.8, 0.9];
        let perf = ChurnPredictor::evaluate_model_performance(&actual, &predicted).unwrap();
        assert_eq!(perf.auc_roc, 0.0);
    }

    #[test]
    fn test_auc_roc_no_discrimination() {
        // Identical scores for positives and negatives: coin-flip AUC.
        let actual = vec![true, true, false, false];
        let predicted = vec![0.5, 0.5, 0.5, 0.5];
        let perf = ChurnPredictor::evaluate_model_performance(&actual, &predicted).unwrap();
        assert_eq!(perf.auc_roc, 0.5);
    }

    #[test]
    fn test_auc_roc_matches_hand_computed_value() {
        // Hand-computable case: positives score 0.9 and 0.3; negatives
        // score 0.35 and 0.2. Of the 4 (positive, negative) score pairs,
        // exactly 3 are correctly ranked (positive > negative) and 1 is
        // not (0.3 < 0.35) -> AUC = 3/4 = 0.75.
        let actual = vec![true, false, true, false];
        let predicted = vec![0.9, 0.35, 0.3, 0.2];
        let perf = ChurnPredictor::evaluate_model_performance(&actual, &predicted).unwrap();
        assert!((perf.auc_roc - 0.75).abs() < 1e-9, "got {}", perf.auc_roc);
    }

    #[test]
    fn test_churn_risk_level_thresholds() {
        assert_eq!(ChurnRiskLevel::VeryLow.threshold(), 0.1);
        assert_eq!(ChurnRiskLevel::Critical.threshold(), 0.9);
    }

    #[test]
    fn test_churn_prediction_complete() {
        let prob = ChurnPredictor::predict_logistic(60.0, 2.0, 100.0, 100.0, 0.2, 0.0).unwrap();
        let risk = ChurnPredictor::classify_risk_level(prob);
        let days = ChurnPredictor::estimate_days_until_churn(prob, 60.0);

        assert!(days > 0);
        assert!(risk >= ChurnRiskLevel::VeryLow && risk <= ChurnRiskLevel::Critical);
    }
}
