//! B2B account-level segmentation and firmographic clustering

use crate::Result;
use std::collections::HashMap;

/// Company size category
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum CompanySize {
    Micro,      // 1-10 employees
    Small,      // 11-50 employees
    MidMarket,  // 51-500 employees
    Enterprise, // 501+ employees
}

impl CompanySize {
    pub fn as_str(&self) -> &str {
        match self {
            CompanySize::Micro => "micro",
            CompanySize::Small => "small",
            CompanySize::MidMarket => "mid_market",
            CompanySize::Enterprise => "enterprise",
        }
    }
}

/// Industry classification
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Industry {
    pub primary: String,   // "SaaS", "Finance", "Healthcare"
    pub secondary: Option<String>,
    pub vertical: Option<String>,
}

/// Geographic region
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Region {
    pub country: String,
    pub region: String,
    pub timezone: String,
}

/// Company firmographic profile
#[derive(Clone, Debug)]
pub struct FirmographicProfile {
    pub account_id: String,
    pub company_name: String,
    pub size: CompanySize,
    pub industry: Industry,
    pub region: Region,
    pub annual_revenue: Option<f64>,
    pub employee_count: usize,
    pub founded_year: Option<u16>,
    pub growth_rate: f64, // YoY percentage
    pub tech_stack: Vec<String>,
}

/// Account engagement metrics
#[derive(Clone, Debug)]
pub struct AccountEngagement {
    pub account_id: String,
    pub monthly_active_users: usize,
    pub feature_adoption_rate: f64,
    pub api_calls_per_day: usize,
    pub support_tickets_per_month: usize,
    pub nps_score: Option<i32>,
    pub usage_trend: f64, // -1 to 1 (declining to growing)
}

/// Account expansion opportunity
#[derive(Clone, Debug)]
pub struct ExpansionOpportunity {
    pub account_id: String,
    pub opportunity_type: String, // "upsell", "cross_sell", "module_adoption"
    pub estimated_mrr: f64,
    pub adoption_gap: f64,
    pub recommended_actions: Vec<String>,
}

/// B2B Account segment
#[derive(Clone, Debug)]
pub struct B2BAccountSegment {
    pub segment_name: String,
    pub description: String,
    pub account_count: usize,
    pub avg_arr: f64,
    pub avg_engagement: f64,
    pub churn_risk: f64,
    pub expansion_potential: f64,
    pub characteristics: HashMap<String, String>,
}

/// Account health score
#[derive(Clone, Debug)]
pub struct AccountHealthScore {
    pub account_id: String,
    pub overall_health: f64, // 0-1
    pub engagement_score: f64,
    pub expansion_readiness: f64,
    pub churn_risk: f64,
    pub growth_trajectory: f64,
}

/// B2B segmentation engine
pub struct B2BSegmentation;

impl B2BSegmentation {
    /// Classify company size
    pub fn classify_company_size(employee_count: usize) -> CompanySize {
        match employee_count {
            0..=10 => CompanySize::Micro,
            11..=50 => CompanySize::Small,
            51..=500 => CompanySize::MidMarket,
            _ => CompanySize::Enterprise,
        }
    }

    /// Calculate account ARR (Annual Recurring Revenue)
    pub fn calculate_arr(monthly_arr: f64) -> f64 {
        monthly_arr * 12.0
    }

    /// Calculate feature adoption rate
    pub fn calculate_adoption_rate(features_enabled: usize, total_features: usize) -> f64 {
        if total_features == 0 {
            return 0.0;
        }
        features_enabled as f64 / total_features as f64
    }

    /// Score account engagement
    pub fn score_engagement(
        monthly_active_users: usize,
        feature_adoption_rate: f64,
        api_calls_per_day: usize,
        support_tickets_monthly: usize,
    ) -> f64 {
        let user_score = (monthly_active_users as f64 / 100.0).min(1.0);
        let adoption_score = feature_adoption_rate;
        let api_score = (api_calls_per_day as f64 / 1000.0).min(1.0);
        let support_score = 1.0 - (support_tickets_monthly as f64 / 50.0).min(1.0);

        (user_score + adoption_score + api_score + support_score) / 4.0
    }

    /// Detect expansion opportunities
    pub fn identify_expansion_opportunities(
        account: &FirmographicProfile,
        engagement: &AccountEngagement,
        current_arr: f64,
    ) -> Result<Vec<ExpansionOpportunity>> {
        let mut opportunities = Vec::new();

        // Upsell opportunity: Low adoption rate
        if engagement.feature_adoption_rate < 0.5 {
            opportunities.push(ExpansionOpportunity {
                account_id: account.account_id.clone(),
                opportunity_type: "feature_training".to_string(),
                estimated_mrr: current_arr / 12.0 * 0.2,
                adoption_gap: 1.0 - engagement.feature_adoption_rate,
                recommended_actions: vec![
                    "Schedule training webinar".to_string(),
                    "Send feature guides".to_string(),
                ],
            });
        }

        // Cross-sell opportunity: Growing usage
        if engagement.usage_trend > 0.3 {
            opportunities.push(ExpansionOpportunity {
                account_id: account.account_id.clone(),
                opportunity_type: "cross_sell".to_string(),
                estimated_mrr: current_arr / 12.0 * 0.15,
                adoption_gap: 0.2,
                recommended_actions: vec![
                    "Schedule product review".to_string(),
                    "Propose add-on modules".to_string(),
                ],
            });
        }

        // Upsell opportunity: High engagement, low MAU
        if engagement.feature_adoption_rate > 0.8 && engagement.monthly_active_users < 10 {
            opportunities.push(ExpansionOpportunity {
                account_id: account.account_id.clone(),
                opportunity_type: "user_seat_upsell".to_string(),
                estimated_mrr: engagement.monthly_active_users as f64 * 100.0,
                adoption_gap: 0.0,
                recommended_actions: vec![
                    "Propose team expansion".to_string(),
                    "Show ROI calculation".to_string(),
                ],
            });
        }

        Ok(opportunities)
    }

    /// Calculate account health score
    pub fn calculate_account_health(
        engagement: &AccountEngagement,
        arr: f64,
        churn_risk_prediction: f64,
    ) -> Result<AccountHealthScore> {
        let engagement_score = Self::score_engagement(
            engagement.monthly_active_users,
            engagement.feature_adoption_rate,
            engagement.api_calls_per_day,
            engagement.support_tickets_per_month,
        );

        let churn_score = 1.0 - churn_risk_prediction;
        let growth = engagement.usage_trend.max(0.0);

        let overall = (engagement_score * 0.4) + (churn_score * 0.35) + (growth * 0.25);

        Ok(AccountHealthScore {
            account_id: engagement.account_id.clone(),
            overall_health: overall.min(1.0),
            engagement_score,
            expansion_readiness: engagement_score + (engagement.usage_trend / 2.0).max(0.0),
            churn_risk: churn_risk_prediction,
            growth_trajectory: engagement.usage_trend,
        })
    }

    /// Segment accounts into strategic buckets
    pub fn segment_accounts(
        accounts: &[(FirmographicProfile, AccountEngagement, f64)], // (profile, engagement, arr)
    ) -> Result<Vec<B2BAccountSegment>> {
        let mut segments: HashMap<String, (Vec<String>, Vec<f64>, Vec<f64>)> = HashMap::new();

        for (profile, engagement, arr) in accounts {
            let size_str = profile.size.as_str();
            let industry = &profile.industry.primary;
            let key = format!("{}-{}", size_str, industry);

            let engagement_score = Self::score_engagement(
                engagement.monthly_active_users,
                engagement.feature_adoption_rate,
                engagement.api_calls_per_day,
                engagement.support_tickets_per_month,
            );

            segments
                .entry(key.clone())
                .or_insert_with(|| (Vec::new(), Vec::new(), Vec::new()))
                .0
                .push(profile.account_id.clone());

            segments
                .entry(key.clone())
                .or_insert_with(|| (Vec::new(), Vec::new(), Vec::new()))
                .1
                .push(*arr);

            segments
                .entry(key)
                .or_insert_with(|| (Vec::new(), Vec::new(), Vec::new()))
                .2
                .push(engagement_score);
        }

        let mut result = Vec::new();

        for (segment_name, (account_ids, arrs, engagement_scores)) in segments {
            let avg_arr = if !arrs.is_empty() {
                arrs.iter().sum::<f64>() / arrs.len() as f64
            } else {
                0.0
            };

            let avg_engagement = if !engagement_scores.is_empty() {
                engagement_scores.iter().sum::<f64>() / engagement_scores.len() as f64
            } else {
                0.0
            };

            let churn = 1.0 - avg_engagement;
            let expansion = avg_engagement * 0.8;

            result.push(B2BAccountSegment {
                segment_name: segment_name.clone(),
                description: format!("Segment: {}", segment_name),
                account_count: account_ids.len(),
                avg_arr,
                avg_engagement,
                churn_risk: churn,
                expansion_potential: expansion,
                characteristics: {
                    let mut map = HashMap::new();
                    map.insert("segment".to_string(), segment_name);
                    map
                },
            });
        }

        Ok(result)
    }

    /// Identify VIP accounts (high value, high risk)
    pub fn identify_vip_accounts(
        accounts: &[(FirmographicProfile, AccountEngagement, f64)],
    ) -> Result<Vec<String>> {
        let mut vips = Vec::new();

        for (profile, engagement, arr) in accounts {
            // VIP: High ARR + High engagement OR High ARR + High growth potential
            if *arr > 100000.0 {
                let eng_score = Self::score_engagement(
                    engagement.monthly_active_users,
                    engagement.feature_adoption_rate,
                    engagement.api_calls_per_day,
                    engagement.support_tickets_per_month,
                );

                if eng_score > 0.6 || engagement.usage_trend > 0.5 {
                    vips.push(profile.account_id.clone());
                }
            }
        }

        Ok(vips)
    }

    /// Identify at-risk accounts
    pub fn identify_at_risk_accounts(
        accounts: &[(FirmographicProfile, AccountEngagement, f64)],
    ) -> Result<Vec<String>> {
        let mut at_risk = Vec::new();

        for (profile, engagement, _arr) in accounts {
            let eng_score = Self::score_engagement(
                engagement.monthly_active_users,
                engagement.feature_adoption_rate,
                engagement.api_calls_per_day,
                engagement.support_tickets_per_month,
            );

            // At-risk: Low engagement + Declining usage
            if eng_score < 0.4 && engagement.usage_trend < -0.3 {
                at_risk.push(profile.account_id.clone());
            }
        }

        Ok(at_risk)
    }

    /// Calculate Total Addressable Market (TAM) by segment
    pub fn calculate_tam(
        segments: &[B2BAccountSegment],
    ) -> Result<HashMap<String, f64>> {
        let mut tam = HashMap::new();

        for segment in segments {
            let segment_tam = segment.avg_arr * segment.account_count as f64;
            tam.insert(segment.segment_name.clone(), segment_tam);
        }

        Ok(tam)
    }

    /// Score account-industry fit
    pub fn calculate_industry_fit(
        industry: &Industry,
        company_size: CompanySize,
        growth_rate: f64,
    ) -> f64 {
        let size_score = match company_size {
            CompanySize::Micro => 0.3,
            CompanySize::Small => 0.6,
            CompanySize::MidMarket => 0.8,
            CompanySize::Enterprise => 0.95,
        };

        let growth_bonus = (growth_rate.max(-1.0) / 3.0).min(0.2);

        (size_score + growth_bonus).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_company_size() {
        assert_eq!(B2BSegmentation::classify_company_size(5), CompanySize::Micro);
        assert_eq!(B2BSegmentation::classify_company_size(25), CompanySize::Small);
        assert_eq!(B2BSegmentation::classify_company_size(200), CompanySize::MidMarket);
        assert_eq!(B2BSegmentation::classify_company_size(2000), CompanySize::Enterprise);
    }

    #[test]
    fn test_calculate_arr() {
        assert_eq!(B2BSegmentation::calculate_arr(10000.0), 120000.0);
    }

    #[test]
    fn test_calculate_adoption_rate() {
        assert_eq!(B2BSegmentation::calculate_adoption_rate(5, 10), 0.5);
        assert_eq!(B2BSegmentation::calculate_adoption_rate(0, 10), 0.0);
    }

    #[test]
    fn test_score_engagement() {
        let score = B2BSegmentation::score_engagement(50, 0.8, 500, 2);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_expansion_opportunities() {
        let profile = FirmographicProfile {
            account_id: "acc_123".to_string(),
            company_name: "TechCorp".to_string(),
            size: CompanySize::MidMarket,
            industry: Industry {
                primary: "SaaS".to_string(),
                secondary: None,
                vertical: None,
            },
            region: Region {
                country: "USA".to_string(),
                region: "West".to_string(),
                timezone: "PST".to_string(),
            },
            annual_revenue: Some(5000000.0),
            employee_count: 100,
            founded_year: Some(2018),
            growth_rate: 0.25,
            tech_stack: vec!["AWS".to_string()],
        };

        let engagement = AccountEngagement {
            account_id: "acc_123".to_string(),
            monthly_active_users: 20,
            feature_adoption_rate: 0.4,
            api_calls_per_day: 500,
            support_tickets_per_month: 3,
            nps_score: Some(45),
            usage_trend: 0.5,
        };

        let opportunities =
            B2BSegmentation::identify_expansion_opportunities(&profile, &engagement, 50000.0)
                .unwrap();

        assert!(!opportunities.is_empty());
    }

    #[test]
    fn test_account_health_score() {
        let engagement = AccountEngagement {
            account_id: "acc_123".to_string(),
            monthly_active_users: 50,
            feature_adoption_rate: 0.9,
            api_calls_per_day: 1000,
            support_tickets_per_month: 1,
            nps_score: Some(60),
            usage_trend: 0.3,
        };

        let health = B2BSegmentation::calculate_account_health(&engagement, 120000.0, 0.1).unwrap();

        assert!(health.overall_health > 0.0);
        assert!(health.engagement_score > 0.5);
    }

    #[test]
    fn test_industry_fit() {
        let industry = Industry {
            primary: "SaaS".to_string(),
            secondary: None,
            vertical: None,
        };

        let fit = B2BSegmentation::calculate_industry_fit(&industry, CompanySize::MidMarket, 0.25);
        assert!(fit > 0.0 && fit <= 1.0);
    }

    #[test]
    fn test_identify_vip_accounts() {
        let profile = FirmographicProfile {
            account_id: "acc_123".to_string(),
            company_name: "TechCorp".to_string(),
            size: CompanySize::Enterprise,
            industry: Industry {
                primary: "SaaS".to_string(),
                secondary: None,
                vertical: None,
            },
            region: Region {
                country: "USA".to_string(),
                region: "East".to_string(),
                timezone: "EST".to_string(),
            },
            annual_revenue: Some(50000000.0),
            employee_count: 500,
            founded_year: Some(2015),
            growth_rate: 0.15,
            tech_stack: vec!["AWS".to_string()],
        };

        let engagement = AccountEngagement {
            account_id: "acc_123".to_string(),
            monthly_active_users: 100,
            feature_adoption_rate: 0.95,
            api_calls_per_day: 5000,
            support_tickets_per_month: 2,
            nps_score: Some(70),
            usage_trend: 0.4,
        };

        let vips = B2BSegmentation::identify_vip_accounts(&[(profile, engagement, 300000.0)]).unwrap();
        assert_eq!(vips.len(), 1);
    }

    #[test]
    fn test_at_risk_accounts() {
        let profile = FirmographicProfile {
            account_id: "acc_456".to_string(),
            company_name: "StartupXYZ".to_string(),
            size: CompanySize::Small,
            industry: Industry {
                primary: "Finance".to_string(),
                secondary: None,
                vertical: None,
            },
            region: Region {
                country: "USA".to_string(),
                region: "Midwest".to_string(),
                timezone: "CST".to_string(),
            },
            annual_revenue: Some(500000.0),
            employee_count: 15,
            founded_year: Some(2022),
            growth_rate: -0.1,
            tech_stack: vec![],
        };

        let engagement = AccountEngagement {
            account_id: "acc_456".to_string(),
            monthly_active_users: 3,
            feature_adoption_rate: 0.2,
            api_calls_per_day: 50,
            support_tickets_per_month: 8,
            nps_score: Some(20),
            usage_trend: -0.5,
        };

        let at_risk = B2BSegmentation::identify_at_risk_accounts(&[(profile, engagement, 10000.0)]).unwrap();
        assert_eq!(at_risk.len(), 1);
    }

    #[test]
    fn test_company_size_classification() {
        assert_eq!(CompanySize::Micro.as_str(), "micro");
        assert_eq!(CompanySize::Enterprise.as_str(), "enterprise");
    }
}
