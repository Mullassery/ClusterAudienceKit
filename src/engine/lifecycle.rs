//! Customer Lifecycle Tracking and Journey Modeling

use crate::Result;
use std::collections::HashMap;

/// Customer lifecycle stage
#[derive(Clone, Debug, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum LifecycleStage {
    Prospect,       // 0: Pre-purchase, awareness stage
    Onboarding,     // 1: New customer, first 30 days
    Active,         // 2: Regular purchaser, healthy engagement
    Mature,         // 3: Long-term customer, stable behavior
    AtRisk,         // 4: Declining engagement, needs attention
    Dormant,        // 5: Inactive for extended period
    Churned,        // 6: Inactive, likely lost
}

impl LifecycleStage {
    pub fn as_str(&self) -> &str {
        match self {
            LifecycleStage::Prospect => "Prospect",
            LifecycleStage::Onboarding => "Onboarding",
            LifecycleStage::Active => "Active",
            LifecycleStage::Mature => "Mature",
            LifecycleStage::AtRisk => "At Risk",
            LifecycleStage::Dormant => "Dormant",
            LifecycleStage::Churned => "Churned",
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            LifecycleStage::Prospect => 0,
            LifecycleStage::Onboarding => 1,
            LifecycleStage::Active => 2,
            LifecycleStage::Mature => 3,
            LifecycleStage::AtRisk => 4,
            LifecycleStage::Dormant => 5,
            LifecycleStage::Churned => 6,
        }
    }
}

/// Customer journey event
#[derive(Clone, Debug)]
pub struct JourneyEvent {
    pub timestamp: i64,           // Unix timestamp
    pub event_type: String,       // "purchase", "login", "signup", "abandoned_cart", etc.
    pub value: f64,               // Transaction amount, if applicable
    pub metadata: HashMap<String, String>, // Custom event data
}

/// Customer lifecycle profile
#[derive(Clone, Debug)]
pub struct LifecycleProfile {
    pub customer_id: String,
    pub current_stage: LifecycleStage,
    pub previous_stage: Option<LifecycleStage>,
    pub stage_confidence: f64,                    // 0-1
    pub days_in_current_stage: i32,
    pub total_purchase_count: usize,
    pub total_value: f64,
    pub avg_order_value: f64,
    pub purchase_frequency: f64,                 // Purchases per year
    pub days_since_first_purchase: i32,
    pub days_since_last_purchase: i32,
    pub retention_score: f64,                    // 0-1
    pub expansion_potential: f64,                // 0-1 likelihood to increase spend
}

/// Stage transition probability
#[derive(Clone, Debug)]
pub struct StageTransition {
    pub from_stage: LifecycleStage,
    pub to_stage: LifecycleStage,
    pub probability: f64,                        // 0-1
    pub avg_time_in_days: i32,
    pub sample_count: usize,
}

/// Lifecycle tracker
pub struct LifecycleTracker;

impl LifecycleTracker {
    /// Classify customer into lifecycle stage based on behavior
    pub fn classify_stage(
        customer_id: &str,
        days_since_signup: i32,
        purchase_count: usize,
        total_value: f64,
        days_since_last_purchase: i32,
        purchase_frequency: f64,
    ) -> Result<LifecycleProfile> {
        // Determine stage
        let stage = if purchase_count == 0 {
            if days_since_signup < 30 {
                LifecycleStage::Prospect
            } else {
                LifecycleStage::Dormant
            }
        } else if days_since_signup < 30 {
            LifecycleStage::Onboarding
        } else if days_since_last_purchase > 180 {
            LifecycleStage::Dormant
        } else if days_since_last_purchase > 90 {
            LifecycleStage::AtRisk
        } else if days_since_signup > 730 && days_since_last_purchase <= 90 {
            // 2+ years as customer with recent activity
            LifecycleStage::Mature
        } else {
            LifecycleStage::Active
        };

        // Calculate confidence
        let confidence = match stage {
            LifecycleStage::Prospect => 0.9,
            LifecycleStage::Onboarding => 0.95,
            LifecycleStage::Active => 0.85,
            LifecycleStage::Mature => 0.90,
            LifecycleStage::AtRisk => 0.80,
            LifecycleStage::Dormant => 0.75,
            LifecycleStage::Churned => 0.85,
        };

        // Calculate retention score
        let base_retention = 1.0 - ((days_since_last_purchase as f64 / 365.0).min(1.0));
        let frequency_factor = (purchase_frequency / 10.0).min(1.0);
        let retention_score = (base_retention + frequency_factor) / 2.0;

        // Calculate expansion potential
        let expansion_potential = match stage {
            LifecycleStage::Onboarding => 0.85,
            LifecycleStage::Active => 0.70,
            LifecycleStage::Mature => 0.60,
            LifecycleStage::AtRisk => 0.40,
            _ => 0.0,
        };

        let avg_order_value = if purchase_count > 0 {
            total_value / purchase_count as f64
        } else {
            0.0
        };

        Ok(LifecycleProfile {
            customer_id: customer_id.to_string(),
            current_stage: stage,
            previous_stage: None,
            stage_confidence: confidence,
            days_in_current_stage: days_since_last_purchase,
            total_purchase_count: purchase_count,
            total_value,
            avg_order_value,
            purchase_frequency,
            days_since_first_purchase: days_since_signup,
            days_since_last_purchase,
            retention_score,
            expansion_potential,
        })
    }

    /// Build stage transition matrix from historical data
    pub fn build_transition_matrix(
        customer_journeys: &[(String, Vec<JourneyEvent>)],
    ) -> Result<HashMap<(LifecycleStage, LifecycleStage), StageTransition>> {
        let mut transitions: HashMap<(LifecycleStage, LifecycleStage), StageTransition> =
            HashMap::new();
        let mut transition_counts: HashMap<(LifecycleStage, LifecycleStage), usize> =
            HashMap::new();
        let mut transition_times: HashMap<(LifecycleStage, LifecycleStage), i32> =
            HashMap::new();

        for (_customer_id, events) in customer_journeys {
            let mut prev_stage: Option<LifecycleStage> = None;
            let mut prev_time: Option<i64> = None;

            for event in events {
                // Infer stage from event
                let stage = match event.event_type.as_str() {
                    "signup" => Some(LifecycleStage::Prospect),
                    "first_purchase" => Some(LifecycleStage::Onboarding),
                    "purchase" => Some(LifecycleStage::Active),
                    "inactivity" => Some(LifecycleStage::AtRisk),
                    "churn" => Some(LifecycleStage::Churned),
                    _ => prev_stage,
                };

                if let (Some(from), Some(to)) = (prev_stage, stage) {
                    if from != to {
                        *transition_counts.entry((from, to)).or_insert(0) += 1;

                        if let Some(prev_ts) = prev_time {
                            let days_diff = ((event.timestamp - prev_ts) / (86400)) as i32;
                            *transition_times.entry((from, to)).or_insert(0) += days_diff;
                        }
                    }
                }

                prev_stage = stage;
                prev_time = Some(event.timestamp);
            }
        }

        // Calculate probabilities
        for ((from, to), count) in &transition_counts {
            let total: usize = transition_counts
                .iter()
                .filter(|((f, _), _)| f == from)
                .map(|(_, c)| c)
                .sum();

            let probability = *count as f64 / total as f64;
            let avg_time = if *count > 0 {
                *transition_times.get(&(*from, *to)).unwrap_or(&0) / *count as i32
            } else {
                0
            };

            transitions.insert(
                (*from, *to),
                StageTransition {
                    from_stage: *from,
                    to_stage: *to,
                    probability,
                    avg_time_in_days: avg_time,
                    sample_count: *count,
                },
            );
        }

        Ok(transitions)
    }

    /// Predict next lifecycle stage
    pub fn predict_next_stage(
        current_stage: LifecycleStage,
        transitions: &HashMap<(LifecycleStage, LifecycleStage), StageTransition>,
    ) -> Option<LifecycleStage> {
        let relevant_transitions: Vec<_> = transitions
            .iter()
            .filter(|((from, _), _)| *from == current_stage)
            .collect();

        if relevant_transitions.is_empty() {
            return None;
        }

        let max_transition = relevant_transitions
            .iter()
            .max_by(|a, b| a.1.probability.partial_cmp(&b.1.probability).unwrap())
            .map(|(_, t)| t);

        max_transition.map(|t| t.to_stage)
    }

    /// Calculate time to next stage
    pub fn time_to_next_stage(
        current_stage: LifecycleStage,
        transitions: &HashMap<(LifecycleStage, LifecycleStage), StageTransition>,
    ) -> Option<i32> {
        let relevant = transitions
            .iter()
            .filter(|((from, _), _)| *from == current_stage)
            .max_by(|a, b| a.1.probability.partial_cmp(&b.1.probability).unwrap());

        relevant.map(|(_, t)| t.avg_time_in_days)
    }

    /// Recommend retention actions based on lifecycle stage
    pub fn retention_actions(stage: LifecycleStage) -> Vec<&'static str> {
        match stage {
            LifecycleStage::Prospect => vec![
                "Send welcome email with product overview",
                "Offer first-time purchase discount",
                "Create onboarding sequence",
            ],
            LifecycleStage::Onboarding => vec![
                "Guide feature discovery",
                "Celebrate first purchase",
                "Send value-building tips",
            ],
            LifecycleStage::Active => vec![
                "Recommend complementary products",
                "Send personalized offers",
                "Request feedback and reviews",
            ],
            LifecycleStage::Mature => vec![
                "VIP rewards program",
                "Exclusive early access",
                "Loyalty recognition",
            ],
            LifecycleStage::AtRisk => vec![
                "Targeted win-back campaign",
                "Special re-engagement offer",
                "Personalized product recommendations",
            ],
            LifecycleStage::Dormant => vec![
                "Reactivation campaign",
                "Incentivized return offer",
                "Survey for feedback",
            ],
            LifecycleStage::Churned => vec![
                "Final win-back attempt",
                "Exit survey",
                "Sunset workflow",
            ],
        }
    }

    /// Calculate segment composition by lifecycle stage
    pub fn stage_distribution(
        profiles: &[LifecycleProfile],
    ) -> Result<HashMap<String, f64>> {
        let mut distribution = HashMap::new();

        if profiles.is_empty() {
            return Ok(distribution);
        }

        let mut stage_counts: HashMap<LifecycleStage, usize> = HashMap::new();

        for profile in profiles {
            *stage_counts.entry(profile.current_stage).or_insert(0) += 1;
        }

        for (stage, count) in stage_counts {
            let percentage = (count as f64 / profiles.len() as f64) * 100.0;
            distribution.insert(stage.as_str().to_string(), percentage);
        }

        Ok(distribution)
    }

    /// Identify high-value churn risk (VIP at risk)
    pub fn identify_vip_at_risk(
        profiles: &[LifecycleProfile],
        high_value_threshold: f64,
    ) -> Vec<LifecycleProfile> {
        profiles
            .iter()
            .filter(|p| {
                p.total_value > high_value_threshold
                    && (p.current_stage == LifecycleStage::AtRisk
                        || p.current_stage == LifecycleStage::Dormant)
                    && p.retention_score < 0.5
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_prospect() {
        let result = LifecycleTracker::classify_stage("c1", 10, 0, 0.0, 10, 0.0).unwrap();
        assert_eq!(result.current_stage, LifecycleStage::Prospect);
        assert!(result.stage_confidence > 0.8);
    }

    #[test]
    fn test_classify_onboarding() {
        let result = LifecycleTracker::classify_stage("c1", 15, 1, 100.0, 5, 20.0).unwrap();
        assert_eq!(result.current_stage, LifecycleStage::Onboarding);
    }

    #[test]
    fn test_classify_active() {
        let result = LifecycleTracker::classify_stage("c1", 90, 10, 1000.0, 20, 40.0).unwrap();
        assert_eq!(result.current_stage, LifecycleStage::Active);
    }

    #[test]
    fn test_classify_mature() {
        let result = LifecycleTracker::classify_stage("c1", 731, 30, 5000.0, 30, 15.0).unwrap();
        assert_eq!(result.current_stage, LifecycleStage::Mature);
    }

    #[test]
    fn test_classify_at_risk() {
        let result = LifecycleTracker::classify_stage("c1", 180, 5, 500.0, 120, 10.0).unwrap();
        assert_eq!(result.current_stage, LifecycleStage::AtRisk);
    }

    #[test]
    fn test_classify_dormant() {
        let result = LifecycleTracker::classify_stage("c1", 200, 3, 300.0, 200, 5.0).unwrap();
        assert_eq!(result.current_stage, LifecycleStage::Dormant);
    }

    #[test]
    fn test_retention_actions_variety() {
        let actions_prospect = LifecycleTracker::retention_actions(LifecycleStage::Prospect);
        let actions_mature = LifecycleTracker::retention_actions(LifecycleStage::Mature);
        let actions_at_risk = LifecycleTracker::retention_actions(LifecycleStage::AtRisk);

        assert!(!actions_prospect.is_empty());
        assert!(!actions_mature.is_empty());
        assert!(!actions_at_risk.is_empty());
        assert_ne!(actions_prospect, actions_mature);
    }

    #[test]
    fn test_stage_distribution() {
        let profiles = vec![
            LifecycleProfile {
                customer_id: "c1".to_string(),
                current_stage: LifecycleStage::Active,
                previous_stage: None,
                stage_confidence: 0.9,
                days_in_current_stage: 30,
                total_purchase_count: 5,
                total_value: 500.0,
                avg_order_value: 100.0,
                purchase_frequency: 20.0,
                days_since_first_purchase: 180,
                days_since_last_purchase: 20,
                retention_score: 0.8,
                expansion_potential: 0.7,
            },
            LifecycleProfile {
                customer_id: "c2".to_string(),
                current_stage: LifecycleStage::Mature,
                previous_stage: None,
                stage_confidence: 0.9,
                days_in_current_stage: 60,
                total_purchase_count: 20,
                total_value: 2000.0,
                avg_order_value: 100.0,
                purchase_frequency: 12.0,
                days_since_first_purchase: 730,
                days_since_last_purchase: 30,
                retention_score: 0.9,
                expansion_potential: 0.6,
            },
        ];

        let dist = LifecycleTracker::stage_distribution(&profiles).unwrap();
        assert_eq!(dist.len(), 2);
        assert_eq!(dist.get("Active"), Some(&50.0));
        assert_eq!(dist.get("Mature"), Some(&50.0));
    }

    #[test]
    fn test_vip_at_risk() {
        let profiles = vec![
            LifecycleProfile {
                customer_id: "c1".to_string(),
                current_stage: LifecycleStage::AtRisk,
                previous_stage: None,
                stage_confidence: 0.8,
                days_in_current_stage: 100,
                total_purchase_count: 50,
                total_value: 5000.0,
                avg_order_value: 100.0,
                purchase_frequency: 30.0,
                days_since_first_purchase: 365,
                days_since_last_purchase: 100,
                retention_score: 0.3,
                expansion_potential: 0.1,
            },
        ];

        let vip_at_risk = LifecycleTracker::identify_vip_at_risk(&profiles, 1000.0);
        assert_eq!(vip_at_risk.len(), 1);
        assert_eq!(vip_at_risk[0].customer_id, "c1");
    }

    #[test]
    fn test_lifecycle_stage_index() {
        assert_eq!(LifecycleStage::Prospect.to_index(), 0);
        assert_eq!(LifecycleStage::Onboarding.to_index(), 1);
        assert_eq!(LifecycleStage::Churned.to_index(), 6);
    }
}
