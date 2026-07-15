//! Automatic customer segment classification

use crate::Result;
use std::collections::HashMap;

/// Standard customer segment
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum SegmentType {
    Champions,
    LoyalCustomers,
    PotentialLoyalists,
    AtRisk,
    CannotLose,
    VIP,
    NewCustomers,
    NeedAttention,
    AboutToSleep,
    AtRiskSleeping,
    Promising,
    Lost,
    Hibernating,
}

impl SegmentType {
    pub fn as_str(&self) -> &str {
        match self {
            SegmentType::Champions => "Champions",
            SegmentType::LoyalCustomers => "Loyal Customers",
            SegmentType::PotentialLoyalists => "Potential Loyalists",
            SegmentType::AtRisk => "At Risk",
            SegmentType::CannotLose => "Cannot Lose Them",
            SegmentType::VIP => "VIP",
            SegmentType::NewCustomers => "New Customers",
            SegmentType::NeedAttention => "Need Attention",
            SegmentType::AboutToSleep => "About to Sleep",
            SegmentType::AtRiskSleeping => "At Risk - Sleeping",
            SegmentType::Promising => "Promising",
            SegmentType::Lost => "Lost",
            SegmentType::Hibernating => "Hibernating",
        }
    }
}

/// Segment profile with metadata
#[derive(Clone, Debug)]
pub struct SegmentProfile {
    pub segment_type: SegmentType,
    pub description: String,
    pub size: usize,
    pub avg_monetary: f64,
    pub avg_frequency: f64,
    pub avg_recency: f64,
    pub churn_risk: f64,
    pub revenue_contribution: f64,
}

impl SegmentProfile {
    pub fn new(segment_type: SegmentType) -> Self {
        let description = match segment_type {
            SegmentType::Champions => "Best customers with highest value and engagement".to_string(),
            SegmentType::LoyalCustomers => "Consistent, valuable customers with steady engagement".to_string(),
            SegmentType::PotentialLoyalists => "Recent high-value customers with growth potential".to_string(),
            SegmentType::AtRisk => "Once valuable customers showing declining engagement".to_string(),
            SegmentType::CannotLose => "High-value customers at risk of churn - critical retention".to_string(),
            SegmentType::VIP => "Premium customers with exceptional value metrics".to_string(),
            SegmentType::NewCustomers => "Recently acquired customers with high potential".to_string(),
            SegmentType::NeedAttention => "Moderate-value customers requiring engagement".to_string(),
            SegmentType::AboutToSleep => "Low engagement customers at risk of dormancy".to_string(),
            SegmentType::AtRiskSleeping => "Previously inactive customers showing re-engagement".to_string(),
            SegmentType::Promising => "New customers showing early signs of high value".to_string(),
            SegmentType::Lost => "Inactive customers unlikely to return".to_string(),
            SegmentType::Hibernating => "Inactive but potentially recoverable customers".to_string(),
        };

        Self {
            segment_type,
            description,
            size: 0,
            avg_monetary: 0.0,
            avg_frequency: 0.0,
            avg_recency: 0.0,
            churn_risk: 0.0,
            revenue_contribution: 0.0,
        }
    }
}

/// Customer segment with RFM scores
#[derive(Clone, Debug)]
pub struct CustomerSegment {
    pub customer_id: String,
    pub segment: SegmentType,
    pub recency_score: u32,
    pub frequency_score: u32,
    pub monetary_score: u32,
    pub confidence: f64,
}

impl CustomerSegment {
    pub fn rfm_rank(&self) -> String {
        format!(
            "{}{}{}",
            self.recency_score, self.frequency_score, self.monetary_score
        )
    }
}

/// Segment classifier based on RFM scores
pub struct SegmentClassifier;

impl SegmentClassifier {
    /// Classify customer into segment based on RFM scores
    pub fn classify(
        customer_id: &str,
        recency_score: u32,
        frequency_score: u32,
        monetary_score: u32,
    ) -> Result<CustomerSegment> {
        let r = recency_score;
        let f = frequency_score;
        let m = monetary_score;

        // RFM-based classification logic (ordered to avoid unreachable patterns)
        let segment = match (r, f, m) {
            // Champions: Top performers across all dimensions
            (5, 5, 5) | (5, 5, 4) | (5, 4, 5) | (4, 5, 5) => SegmentType::Champions,

            // VIP: High monetary, very recent activity (catches remaining 5x5 combos)
            (5, _, 5) | (5, 5, _) => SegmentType::VIP,

            // Loyal Customers: Consistent across all dimensions
            (4, 4, 4) | (4, 4, 5) | (4, 5, 4) => SegmentType::LoyalCustomers,

            // Potential Loyalists: High monetary + recent but lower frequency
            (5, 3, 4) | (4, 3, 5) => SegmentType::PotentialLoyalists,

            // Cannot Lose Them: High monetary but declining engagement
            (3, 4, 5) | (3, 5, 5) | (2, 4, 5) | (2, 5, 5) => SegmentType::CannotLose,

            // At Risk: Used to be good but now declining across the board
            (2, 2, 4) | (2, 2, 5) | (2, 3, 4) | (2, 3, 5) | (3, 2, 4) | (3, 2, 5) => {
                SegmentType::AtRisk
            }

            // About to Sleep: Low recency + moderate frequency
            (2, 2, 3) | (2, 2, 2) | (2, 3, 2) | (2, 3, 3) => SegmentType::AboutToSleep,

            // New Customers: High frequency + very recent but lower monetary
            (5, 4, 3) | (5, 4, 2) | (5, 3, 3) | (5, 3, 2) => SegmentType::NewCustomers,

            // Promising: Recent + moderate frequency + moderate monetary
            (5, 2, 3) | (5, 2, 4) => SegmentType::Promising,

            // Need Attention: Low frequency despite recent activity
            (4, 2, 2) | (4, 3, 2) | (4, 3, 3) => SegmentType::NeedAttention,

            // Lost: Low across all dimensions
            (1, 1, 1) | (1, 1, 2) | (1, 2, 1) => SegmentType::Lost,

            // At Risk - Sleeping: Used to purchase but now gone cold
            (1, 3, 4) | (1, 3, 5) | (1, 4, 4) | (1, 4, 5) => SegmentType::AtRiskSleeping,

            // Hibernating: Very low recency but used to be valuable
            (1, 2, 3) | (1, 2, 4) | (1, 2, 5) | (1, 3, 2) | (1, 3, 3) => SegmentType::Hibernating,

            // Default fallback
            _ => SegmentType::NeedAttention,
        };

        // Calculate confidence based on RFM balance
        let scores = vec![r as f64, f as f64, m as f64];
        let avg = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance = scores
            .iter()
            .map(|s| (s - avg).powi(2))
            .sum::<f64>()
            / scores.len() as f64;
        let std_dev = variance.sqrt();
        let cv = if avg > 0.0 { std_dev / avg } else { 0.0 };
        let confidence = 1.0 / (1.0 + cv); // Higher confidence when scores are balanced

        Ok(CustomerSegment {
            customer_id: customer_id.to_string(),
            segment,
            recency_score: r,
            frequency_score: f,
            monetary_score: m,
            confidence: confidence.min(1.0).max(0.0),
        })
    }

    /// Classify batch of customers
    pub fn classify_batch(
        customers: Vec<(String, u32, u32, u32)>,
    ) -> Result<Vec<CustomerSegment>> {
        let mut segments = Vec::new();
        for (customer_id, r, f, m) in customers {
            segments.push(Self::classify(&customer_id, r, f, m)?);
        }
        Ok(segments)
    }

    /// Get segment churn risk (0-1, higher = more at risk)
    pub fn churn_risk(segment: &SegmentType) -> f64 {
        match segment {
            SegmentType::AtRisk => 0.9,
            SegmentType::CannotLose => 0.85,
            SegmentType::AboutToSleep => 0.8,
            SegmentType::AtRiskSleeping => 0.75,
            SegmentType::Lost => 0.95,
            SegmentType::Hibernating => 0.7,
            SegmentType::NeedAttention => 0.5,
            SegmentType::Promising => 0.2,
            SegmentType::NewCustomers => 0.3,
            SegmentType::PotentialLoyalists => 0.15,
            SegmentType::LoyalCustomers => 0.05,
            SegmentType::Champions => 0.02,
            SegmentType::VIP => 0.01,
        }
    }

    /// Get recommended action for segment
    pub fn recommended_action(segment: &SegmentType) -> &'static str {
        match segment {
            SegmentType::Champions => "Reward loyalty, ask for referrals",
            SegmentType::LoyalCustomers => "Maintain engagement, upsell opportunities",
            SegmentType::PotentialLoyalists => "Personalized engagement, nurture relationship",
            SegmentType::AtRisk => "Win-back campaign, special offers",
            SegmentType::CannotLose => "Priority retention, VIP treatment",
            SegmentType::VIP => "Exclusive experiences, personalization",
            SegmentType::NewCustomers => "Welcome series, onboarding support",
            SegmentType::NeedAttention => "Re-engagement campaign, surveys",
            SegmentType::AboutToSleep => "Targeted win-back, special incentives",
            SegmentType::AtRiskSleeping => "Reactivation campaign, incentives",
            SegmentType::Promising => "Regular engagement, growth nurturing",
            SegmentType::Lost => "Sunset, or aggressive win-back",
            SegmentType::Hibernating => "Careful reactivation, special offer",
        }
    }

    /// Calculate segment metrics
    pub fn calculate_profiles(
        segments: &[CustomerSegment],
        monetary_values: &HashMap<String, f64>,
        frequency_values: &HashMap<String, f64>,
        recency_values: &HashMap<String, f64>,
    ) -> Result<HashMap<SegmentType, SegmentProfile>> {
        let mut profiles: HashMap<SegmentType, SegmentProfile> = HashMap::new();

        // Initialize profiles
        for segment in &[
            SegmentType::Champions,
            SegmentType::LoyalCustomers,
            SegmentType::PotentialLoyalists,
            SegmentType::AtRisk,
            SegmentType::CannotLose,
            SegmentType::VIP,
            SegmentType::NewCustomers,
            SegmentType::NeedAttention,
            SegmentType::AboutToSleep,
            SegmentType::AtRiskSleeping,
            SegmentType::Promising,
            SegmentType::Lost,
            SegmentType::Hibernating,
        ] {
            profiles.insert(segment.clone(), SegmentProfile::new(segment.clone()));
        }

        // Aggregate metrics
        let total_monetary: f64 = monetary_values.values().sum();

        for segment in segments {
            let monetary = monetary_values.get(&segment.customer_id).copied().unwrap_or(0.0);
            let frequency = frequency_values
                .get(&segment.customer_id)
                .copied()
                .unwrap_or(0.0);
            let recency = recency_values
                .get(&segment.customer_id)
                .copied()
                .unwrap_or(0.0);

            if let Some(profile) = profiles.get_mut(&segment.segment) {
                profile.size += 1;
                profile.avg_monetary += monetary;
                profile.avg_frequency += frequency;
                profile.avg_recency += recency;
                profile.churn_risk += Self::churn_risk(&segment.segment);
                profile.revenue_contribution += monetary;
            }
        }

        // Calculate averages
        for profile in profiles.values_mut() {
            if profile.size > 0 {
                let size_f = profile.size as f64;
                profile.avg_monetary /= size_f;
                profile.avg_frequency /= size_f;
                profile.avg_recency /= size_f;
                profile.churn_risk /= size_f;
                if total_monetary > 0.0 {
                    profile.revenue_contribution /= total_monetary;
                }
            }
        }

        Ok(profiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_type_display() {
        assert_eq!(SegmentType::Champions.as_str(), "Champions");
        assert_eq!(SegmentType::LoyalCustomers.as_str(), "Loyal Customers");
    }

    #[test]
    fn test_classify_champions() {
        let segment = SegmentClassifier::classify("c1", 5, 5, 5).unwrap();
        assert_eq!(segment.segment, SegmentType::Champions);
        assert!(segment.confidence > 0.9);
    }

    #[test]
    fn test_classify_at_risk() {
        let segment = SegmentClassifier::classify("c2", 2, 2, 5).unwrap();
        assert_eq!(segment.segment, SegmentType::AtRisk);
    }

    #[test]
    fn test_classify_new_customers() {
        let segment = SegmentClassifier::classify("c3", 5, 3, 2).unwrap();
        assert_eq!(segment.segment, SegmentType::NewCustomers);
    }

    #[test]
    fn test_churn_risk() {
        assert!(SegmentClassifier::churn_risk(&SegmentType::Lost) > 0.9);
        assert!(SegmentClassifier::churn_risk(&SegmentType::Champions) < 0.05);
    }

    #[test]
    fn test_recommended_action() {
        let action = SegmentClassifier::recommended_action(&SegmentType::Champions);
        assert!(!action.is_empty());
    }

    #[test]
    fn test_classify_batch() {
        let customers = vec![
            ("c1".to_string(), 5, 5, 5),
            ("c2".to_string(), 2, 2, 5),
            ("c3".to_string(), 1, 1, 1),
        ];
        let segments = SegmentClassifier::classify_batch(customers).unwrap();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].segment, SegmentType::Champions);
        assert_eq!(segments[1].segment, SegmentType::AtRisk);
        assert_eq!(segments[2].segment, SegmentType::Lost);
    }

    #[test]
    fn test_segment_profile_creation() {
        let profile = SegmentProfile::new(SegmentType::Champions);
        assert_eq!(profile.segment_type, SegmentType::Champions);
        assert!(!profile.description.is_empty());
    }

    #[test]
    fn test_confidence_calculation() {
        // High confidence when scores are balanced
        let balanced = SegmentClassifier::classify("c1", 5, 5, 5).unwrap();
        // Lower confidence when scores are imbalanced
        let imbalanced = SegmentClassifier::classify("c2", 5, 1, 1).unwrap();
        assert!(balanced.confidence > imbalanced.confidence);
    }

    #[test]
    fn test_calculate_profiles() {
        let segments = vec![
            CustomerSegment {
                customer_id: "c1".to_string(),
                segment: SegmentType::Champions,
                recency_score: 5,
                frequency_score: 5,
                monetary_score: 5,
                confidence: 0.95,
            },
            CustomerSegment {
                customer_id: "c2".to_string(),
                segment: SegmentType::Lost,
                recency_score: 1,
                frequency_score: 1,
                monetary_score: 1,
                confidence: 0.9,
            },
        ];

        let mut monetary = HashMap::new();
        monetary.insert("c1".to_string(), 1000.0);
        monetary.insert("c2".to_string(), 100.0);

        let mut frequency = HashMap::new();
        frequency.insert("c1".to_string(), 10.0);
        frequency.insert("c2".to_string(), 1.0);

        let mut recency = HashMap::new();
        recency.insert("c1".to_string(), 0.9);
        recency.insert("c2".to_string(), 0.1);

        let profiles =
            SegmentClassifier::calculate_profiles(&segments, &monetary, &frequency, &recency)
                .unwrap();

        assert_eq!(profiles.len(), 13);
        assert!(profiles.get(&SegmentType::Champions).unwrap().size > 0);
    }
}
