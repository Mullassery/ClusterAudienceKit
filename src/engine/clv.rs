//! Customer Lifetime Value (CLV) prediction and forecasting

use crate::Result;
use std::collections::HashMap;

/// CLV calculation model type
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum CLVModel {
    Simple,        // Avg transaction value × purchase frequency × customer lifespan
    Probabilistic, // Accounts for churn probability
    Predictive,    // Time-series based forecasting
}

/// Customer lifetime value metrics
#[derive(Clone, Debug)]
pub struct CustomerLTV {
    pub customer_id: String,
    pub historical_value: f64,  // Total spent to date
    pub annual_value: f64,      // Estimated annual spending
    pub predicted_ltv: f64,     // Predicted lifetime value
    pub predicted_ltv_3yr: f64, // 3-year prediction
    pub predicted_ltv_5yr: f64, // 5-year prediction
    pub churn_probability: f64, // 0-1 probability of churn
    pub confidence_score: f64,  // Model confidence 0-1
    pub model_used: CLVModel,
}

/// Churn prediction result
#[derive(Clone, Debug)]
pub struct ChurnPrediction {
    pub customer_id: String,
    pub churn_probability: f64,        // 0-1 risk of churn
    pub confidence: f64,               // Model confidence
    pub risk_level: String,            // "low", "medium", "high", "critical"
    pub days_until_churn: Option<i32>, // Days until predicted churn
    pub retention_score: f64,          // 0-1 likelihood to retain
}

/// Revenue projection
#[derive(Clone, Debug)]
pub struct RevenueProjection {
    pub period: String,                  // "30d", "90d", "1y", "3y", "5y"
    pub historical_avg: f64,             // Historical average revenue
    pub projected_revenue: f64,          // Predicted total revenue
    pub growth_rate: f64,                // Expected growth % per period
    pub confidence_interval: (f64, f64), // (lower, upper) bounds
    pub risk_adjusted_revenue: f64,      // Accounting for churn risk
}

/// CLV calculator
pub struct CLVCalculator;

impl CLVCalculator {
    /// Calculate customer lifetime value using simple model
    /// LTV = (Avg Order Value × Purchase Frequency) × Customer Lifespan
    pub fn calculate_simple_ltv(
        customer_id: &str,
        total_spent: f64,
        purchase_count: usize,
        days_active: i32,
        avg_customer_lifespan_days: i32,
    ) -> Result<CustomerLTV> {
        if purchase_count == 0 || days_active == 0 {
            return Ok(CustomerLTV {
                customer_id: customer_id.to_string(),
                historical_value: total_spent,
                annual_value: 0.0,
                predicted_ltv: total_spent,
                predicted_ltv_3yr: total_spent,
                predicted_ltv_5yr: total_spent,
                churn_probability: 0.5,
                confidence_score: 0.3,
                model_used: CLVModel::Simple,
            });
        }

        let avg_order_value = total_spent / purchase_count as f64;
        let purchase_frequency = purchase_count as f64 / (days_active as f64 / 365.0);

        // Annualize and project
        let annual_value = avg_order_value * purchase_frequency;
        let lifespan_years = avg_customer_lifespan_days as f64 / 365.0;
        let predicted_ltv = annual_value * lifespan_years;

        // Multi-year projections with decay
        let predicted_ltv_3yr = annual_value * 3.0 * 0.85;
        let predicted_ltv_5yr = annual_value * 5.0 * 0.70;

        Ok(CustomerLTV {
            customer_id: customer_id.to_string(),
            historical_value: total_spent,
            annual_value,
            predicted_ltv,
            predicted_ltv_3yr,
            predicted_ltv_5yr,
            churn_probability: 0.15,
            confidence_score: 0.75,
            model_used: CLVModel::Simple,
        })
    }

    /// Calculate CLV with churn probability (probabilistic model)
    pub fn calculate_probabilistic_ltv(
        customer_id: &str,
        total_spent: f64,
        purchase_count: usize,
        days_active: i32,
        recency_days: i32,
        rfm_score: f64, // 0-15 scale
        churn_probability: f64,
    ) -> Result<CustomerLTV> {
        if purchase_count == 0 || days_active == 0 {
            return Ok(CustomerLTV {
                customer_id: customer_id.to_string(),
                historical_value: total_spent,
                annual_value: 0.0,
                predicted_ltv: 0.0,
                predicted_ltv_3yr: 0.0,
                predicted_ltv_5yr: 0.0,
                churn_probability,
                confidence_score: 0.4,
                model_used: CLVModel::Probabilistic,
            });
        }

        // Base LTV calculation
        let avg_order_value = total_spent / purchase_count as f64;
        let purchase_frequency = purchase_count as f64 / (days_active as f64 / 365.0);
        let annual_value = avg_order_value * purchase_frequency;

        // Apply RFM quality factor
        let rfm_factor = (rfm_score / 15.0).clamp(0.5, 1.0);

        // Adjust for recency (recent activity lowers churn risk)
        let recency_factor = if recency_days < 30 {
            1.0
        } else if recency_days < 90 {
            0.85
        } else if recency_days < 180 {
            0.65
        } else {
            0.4
        };

        // Calculate retention probability
        let retention_prob = 1.0 - churn_probability;

        // Project LTV accounting for churn
        let base_ltv = annual_value * 3.0 * rfm_factor * recency_factor;
        let predicted_ltv = base_ltv * retention_prob;
        let predicted_ltv_3yr = base_ltv * (retention_prob.powi(3));
        let predicted_ltv_5yr = base_ltv * (retention_prob.powi(5));

        // Confidence based on data quality
        let confidence = (rfm_factor * recency_factor).min(0.95);

        Ok(CustomerLTV {
            customer_id: customer_id.to_string(),
            historical_value: total_spent,
            annual_value,
            predicted_ltv,
            predicted_ltv_3yr,
            predicted_ltv_5yr,
            churn_probability,
            confidence_score: confidence,
            model_used: CLVModel::Probabilistic,
        })
    }

    /// Predict churn probability based on RFM and engagement metrics
    pub fn predict_churn(
        customer_id: &str,
        recency_days: i32,
        purchase_frequency: f64,
        monetary_value: f64,
        days_since_signup: i32,
    ) -> Result<ChurnPrediction> {
        // Churn factors
        let mut churn_score: f64 = 0.0;

        // Recency factor (most important)
        if recency_days < 30 {
            churn_score += 0.05;
        } else if recency_days < 90 {
            churn_score += 0.15;
        } else if recency_days < 180 {
            churn_score += 0.35;
        } else {
            churn_score += 0.65;
        }

        // Frequency factor
        if purchase_frequency < 1.0 {
            churn_score += 0.25;
        } else if purchase_frequency < 3.0 {
            churn_score += 0.15;
        } else if purchase_frequency < 6.0 {
            churn_score += 0.05;
        }

        // Monetary factor
        if monetary_value < 100.0 {
            churn_score += 0.15;
        } else if monetary_value < 500.0 {
            churn_score += 0.08;
        }

        // Tenure factor (newer customers more likely to churn)
        if days_since_signup < 90 {
            churn_score += 0.20;
        } else if days_since_signup < 365 {
            churn_score += 0.10;
        }

        let churn_probability: f64 = (churn_score / 2.0).clamp(0.0, 1.0);

        // Risk level classification
        let risk_level = if churn_probability > 0.65 {
            "critical".to_string()
        } else if churn_probability > 0.45 {
            "high".to_string()
        } else if churn_probability > 0.25 {
            "medium".to_string()
        } else {
            "low".to_string()
        };

        // Estimate days until churn
        let days_until_churn = if churn_probability > 0.3 {
            Some(recency_days + 30)
        } else {
            None
        };

        // Retention score (inverse of churn)
        let retention_score = 1.0 - churn_probability;

        // Confidence based on data completeness
        let confidence = 0.85;

        Ok(ChurnPrediction {
            customer_id: customer_id.to_string(),
            churn_probability,
            confidence,
            risk_level,
            days_until_churn,
            retention_score,
        })
    }

    /// Project revenue for a customer over specified period
    pub fn project_revenue(
        _customer_id: &str,
        historical_avg_monthly: f64,
        purchase_trend: f64, // -1 to 1, trend direction
        churn_risk: f64,     // 0-1
        period_days: i32,
    ) -> Result<RevenueProjection> {
        let months = period_days as f64 / 30.0;

        // Growth/decline based on trend
        let growth_rate = purchase_trend * 0.15; // Up to 15% monthly change

        // Project revenue with compound growth
        let projected_revenue = if growth_rate.abs() < 0.01 {
            historical_avg_monthly * months
        } else {
            historical_avg_monthly * (((1.0 + growth_rate).powf(months)) - 1.0) / growth_rate
        };

        // Adjust for churn risk
        let survival_rate = (1.0 - churn_risk).powf(months / 12.0);
        let risk_adjusted_revenue = projected_revenue * survival_rate;

        // Confidence interval
        let lower_bound = risk_adjusted_revenue * 0.75;
        let upper_bound = risk_adjusted_revenue * 1.25;

        // Period label
        let period = if period_days <= 30 {
            "30d".to_string()
        } else if period_days <= 90 {
            "90d".to_string()
        } else if period_days <= 365 {
            "1y".to_string()
        } else if period_days <= 1095 {
            "3y".to_string()
        } else {
            "5y".to_string()
        };

        Ok(RevenueProjection {
            period,
            historical_avg: historical_avg_monthly,
            projected_revenue,
            growth_rate,
            confidence_interval: (lower_bound, upper_bound),
            risk_adjusted_revenue,
        })
    }

    /// Calculate segment-level CLV metrics
    pub fn calculate_segment_clv(
        segment_customers: &[(String, f64, usize, i32)], // (id, spent, count, days_active)
    ) -> Result<HashMap<String, f64>> {
        let mut metrics = HashMap::new();

        if segment_customers.is_empty() {
            return Ok(metrics);
        }

        // Calculate aggregate metrics
        let total_revenue: f64 = segment_customers.iter().map(|(_, v, _, _)| v).sum();
        let avg_customer_value = total_revenue / segment_customers.len() as f64;
        let total_transactions: usize = segment_customers.iter().map(|(_, _, c, _)| c).sum();
        let avg_order_value = total_revenue / total_transactions as f64;

        // Calculate LTV for segment
        let segment_ltv: f64 = segment_customers
            .iter()
            .map(|(_, spent, count, days)| {
                if *count > 0 && *days > 0 {
                    (*spent / *count as f64) * (*count as f64 / (*days as f64 / 365.0))
                } else {
                    *spent
                }
            })
            .sum::<f64>()
            / segment_customers.len() as f64;

        metrics.insert("total_revenue".to_string(), total_revenue);
        metrics.insert("avg_customer_ltv".to_string(), avg_customer_value);
        metrics.insert("avg_order_value".to_string(), avg_order_value);
        metrics.insert("segment_ltv".to_string(), segment_ltv);
        metrics.insert("customer_count".to_string(), segment_customers.len() as f64);

        Ok(metrics)
    }
}

/// Time-series forecasting for revenue trends
pub struct RevenueForecaster;

impl RevenueForecaster {
    /// Simple exponential smoothing for revenue forecast
    pub fn forecast_exponential_smoothing(
        historical_revenue: &[f64],
        alpha: f64, // Smoothing factor 0-1
        forecast_periods: usize,
    ) -> Result<Vec<f64>> {
        if historical_revenue.is_empty() {
            return Ok(vec![]);
        }

        let mut forecast = vec![];
        let mut s = historical_revenue[0]; // Initial level

        // Apply exponential smoothing to historical data
        for &value in &historical_revenue[1..] {
            s = alpha * value + (1.0 - alpha) * s;
        }

        // Generate forecast
        for _ in 0..forecast_periods {
            forecast.push(s);
        }

        Ok(forecast)
    }

    /// Calculate moving average trend
    pub fn moving_average(values: &[f64], window_size: usize) -> Result<Vec<f64>> {
        if values.len() < window_size {
            return Ok(vec![]);
        }

        let mut averages = vec![];

        for i in 0..=(values.len() - window_size) {
            let window_sum: f64 = values[i..i + window_size].iter().sum();
            let avg = window_sum / window_size as f64;
            averages.push(avg);
        }

        Ok(averages)
    }

    /// Detect trend direction from historical data
    pub fn detect_trend(historical_revenue: &[f64]) -> Result<f64> {
        if historical_revenue.len() < 2 {
            return Ok(0.0);
        }

        let first_half_avg: f64 = historical_revenue[..historical_revenue.len() / 2]
            .iter()
            .sum::<f64>()
            / (historical_revenue.len() / 2) as f64;
        let second_half_avg: f64 = historical_revenue[historical_revenue.len() / 2..]
            .iter()
            .sum::<f64>()
            / (historical_revenue.len() - historical_revenue.len() / 2) as f64;

        let trend = (second_half_avg - first_half_avg) / first_half_avg.max(0.01);

        Ok(trend.clamp(-1.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_ltv_calculation() {
        let result = CLVCalculator::calculate_simple_ltv("c1", 1000.0, 10, 365, 1095).unwrap();

        assert_eq!(result.customer_id, "c1");
        assert_eq!(result.historical_value, 1000.0);
        assert!(result.predicted_ltv > 0.0);
        assert_eq!(result.model_used, CLVModel::Simple);
    }

    #[test]
    fn test_probabilistic_ltv() {
        let result =
            CLVCalculator::calculate_probabilistic_ltv("c1", 1000.0, 10, 365, 10, 12.0, 0.2)
                .unwrap();

        assert!(result.churn_probability > 0.0 && result.churn_probability <= 1.0);
        assert!(result.predicted_ltv >= 0.0);
        assert!(result.confidence_score > 0.0 && result.confidence_score <= 1.0);
    }

    #[test]
    fn test_churn_prediction() {
        let result = CLVCalculator::predict_churn("c1", 45, 2.5, 500.0, 180).unwrap();

        assert!(result.churn_probability >= 0.0 && result.churn_probability <= 1.0);
        assert!(!result.risk_level.is_empty());
        assert!(result.retention_score + result.churn_probability > 0.99);
    }

    #[test]
    fn test_revenue_projection() {
        let result = CLVCalculator::project_revenue("c1", 100.0, 0.1, 0.2, 365).unwrap();

        assert!(result.projected_revenue > 0.0);
        assert!(result.risk_adjusted_revenue > 0.0);
        assert!(result.risk_adjusted_revenue <= result.projected_revenue);
        assert_eq!(result.period, "1y");
    }

    #[test]
    fn test_segment_ltv() {
        let customers = vec![
            ("c1".to_string(), 1000.0, 10, 365),
            ("c2".to_string(), 2000.0, 20, 365),
            ("c3".to_string(), 500.0, 5, 365),
        ];

        let result = CLVCalculator::calculate_segment_clv(&customers).unwrap();

        assert_eq!(result.get("customer_count"), Some(&3.0));
        assert!(*result.get("total_revenue").unwrap() > 0.0);
        assert!(*result.get("segment_ltv").unwrap() > 0.0);
    }

    #[test]
    fn test_exponential_smoothing() {
        let data = vec![100.0, 110.0, 105.0, 120.0, 115.0];
        let forecast = RevenueForecaster::forecast_exponential_smoothing(&data, 0.3, 3).unwrap();

        assert_eq!(forecast.len(), 3);
        assert!(forecast.iter().all(|&x| x > 0.0));
    }

    #[test]
    fn test_moving_average() {
        let data = vec![100.0, 110.0, 105.0, 120.0, 115.0, 130.0];
        let avg = RevenueForecaster::moving_average(&data, 3).unwrap();

        assert_eq!(avg.len(), 4);
        assert!(avg[0] > 100.0);
    }

    #[test]
    fn test_trend_detection() {
        let data = vec![100.0, 105.0, 110.0, 115.0, 120.0]; // Uptrend
        let trend = RevenueForecaster::detect_trend(&data).unwrap();

        assert!(trend > 0.0);
    }

    #[test]
    fn test_churn_risk_levels() {
        // High/Critical risk
        let high_risk = CLVCalculator::predict_churn("c1", 180, 0.5, 50.0, 100).unwrap();
        assert!(high_risk.risk_level == "high" || high_risk.risk_level == "critical");

        // Low risk
        let low = CLVCalculator::predict_churn("c2", 5, 5.0, 1000.0, 365).unwrap();
        assert_eq!(low.risk_level, "low");
    }

    #[test]
    fn test_ltv_edge_cases() {
        // Zero transactions
        let result = CLVCalculator::calculate_simple_ltv("c1", 0.0, 0, 0, 365).unwrap();
        assert_eq!(result.historical_value, 0.0);

        // Single large purchase
        let result = CLVCalculator::calculate_simple_ltv("c2", 5000.0, 1, 30, 365).unwrap();
        assert!(result.annual_value > 0.0);
    }
}
