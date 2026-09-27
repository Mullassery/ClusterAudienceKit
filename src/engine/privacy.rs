//! Privacy-preserving techniques: Differential privacy and K-anonymity

use crate::Result;
use rand::Rng;
use std::collections::HashMap;

/// Differential privacy mechanism
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum DPMechanism {
    Laplace,     // Laplace mechanism for counts/sums
    Gaussian,    // Gaussian mechanism for range queries
    Exponential, // Exponential mechanism for selection
}

impl DPMechanism {
    pub fn as_str(&self) -> &str {
        match self {
            DPMechanism::Laplace => "laplace",
            DPMechanism::Gaussian => "gaussian",
            DPMechanism::Exponential => "exponential",
        }
    }
}

/// Differential privacy budget
#[derive(Clone, Debug)]
pub struct PrivacyBudget {
    pub epsilon: f64,           // Privacy loss parameter
    pub delta: f64,             // Failure probability
    pub remaining_epsilon: f64, // Remaining budget
}

impl PrivacyBudget {
    pub fn new(epsilon: f64, delta: f64) -> Self {
        Self {
            epsilon,
            delta,
            remaining_epsilon: epsilon,
        }
    }

    pub fn consume(&mut self, cost: f64) -> bool {
        if cost <= self.remaining_epsilon {
            self.remaining_epsilon -= cost;
            true
        } else {
            false
        }
    }

    pub fn budget_exhausted(&self) -> bool {
        self.remaining_epsilon <= 0.0
    }

    pub fn budget_percentage(&self) -> f64 {
        ((self.epsilon - self.remaining_epsilon) / self.epsilon) * 100.0
    }
}

/// K-anonymity quasi-identifier
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct QuasiIdentifier {
    pub name: String,
    pub values: Vec<String>,
}

impl QuasiIdentifier {
    pub fn new(name: String) -> Self {
        Self {
            name,
            values: Vec::new(),
        }
    }

    pub fn add_value(mut self, value: String) -> Self {
        self.values.push(value);
        self
    }
}

/// K-anonymity result
#[derive(Clone, Debug)]
pub struct KAnonymityResult {
    pub k_value: usize,
    pub anonymized: bool,
    pub generalized_rows: usize,
    pub suppressed_rows: usize,
    pub information_loss: f64,
}

/// Differential privacy utilities
pub struct DifferentialPrivacy;

impl DifferentialPrivacy {
    /// Generate Laplace noise for counts.
    ///
    /// Samples a real uniform(0,1) draw and maps it through the standard
    /// inverse-CDF transform for Laplace(0, scale): for u < 0.5,
    /// `scale * ln(2u)`; otherwise `-scale * ln(2(1-u))`. This function
    /// previously hardcoded `u = 0.5`, which — because that's exactly the
    /// boundary between the two branches — made the noise term evaluate to
    /// `scale * ln(1.0) == 0.0` on *every* call, so the "Laplace mechanism"
    /// silently returned the caller's data completely unmodified: zero
    /// actual differential-privacy protection despite the API contract.
    /// See `test_laplace_noise_is_actually_random` below.
    pub fn laplace_noise(epsilon: f64, sensitivity: f64) -> f64 {
        let scale = sensitivity / epsilon;
        let mut rng = rand::thread_rng();
        // Avoid exact 0.0 so `ln()` below can't produce -inf.
        let u: f64 = rng.gen_range(f64::EPSILON..1.0);

        if u < 0.5 {
            scale * (2.0 * u).ln()
        } else {
            -scale * (2.0 * (1.0 - u)).ln()
        }
    }

    /// Generate Gaussian noise for range queries via a Box-Muller transform
    /// over two real uniform(0,1) draws.
    ///
    /// This function previously hardcoded `sigma * 0.5` — a fixed,
    /// deterministic shift with no randomness at all (and trivially
    /// reversible by anyone who knows `sigma`), despite being the "Gaussian
    /// mechanism" backing the DP-noise API. See
    /// `test_gaussian_noise_is_actually_random` below.
    pub fn gaussian_noise(epsilon: f64, delta: f64, sensitivity: f64) -> f64 {
        let sigma = sensitivity * (2.0 * (1.25 / delta).ln()).sqrt() / epsilon;
        let mut rng = rand::thread_rng();
        let u1: f64 = rng.gen_range(f64::EPSILON..1.0);
        let u2: f64 = rng.gen_range(0.0..1.0);
        // Box-Muller transform: two independent uniform(0,1) draws -> one
        // standard-normal sample.
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        sigma * z0
    }

    /// Add Laplace mechanism noise to counts
    pub fn add_laplace_noise(data: &[f64], epsilon: f64, sensitivity: f64) -> Result<Vec<f64>> {
        let mut noisy = Vec::new();

        for value in data {
            let noise = Self::laplace_noise(epsilon, sensitivity);
            noisy.push((value + noise).max(0.0)); // Ensure non-negative counts
        }

        Ok(noisy)
    }

    /// Add Gaussian mechanism noise
    pub fn add_gaussian_noise(
        data: &[f64],
        epsilon: f64,
        delta: f64,
        sensitivity: f64,
    ) -> Result<Vec<f64>> {
        let mut noisy = Vec::new();

        for value in data {
            let noise = Self::gaussian_noise(epsilon, delta, sensitivity);
            noisy.push((value + noise).max(0.0));
        }

        Ok(noisy)
    }

    /// Calculate epsilon cost for query
    pub fn query_cost(&self, sensitivity: f64, required_accuracy: f64) -> f64 {
        sensitivity / required_accuracy
    }
}

/// K-anonymity implementation
pub struct KAnonymity;

impl KAnonymity {
    /// Check if dataset is k-anonymous
    pub fn check_k_anonymity(
        data: &[HashMap<String, String>],
        quasi_identifiers: &[String],
        k: usize,
    ) -> Result<KAnonymityResult> {
        if data.is_empty() {
            return Ok(KAnonymityResult {
                k_value: k,
                anonymized: true,
                generalized_rows: 0,
                suppressed_rows: 0,
                information_loss: 0.0,
            });
        }

        // Group rows by quasi-identifier combinations
        let mut groups: HashMap<Vec<String>, usize> = HashMap::new();

        for row in data {
            let mut key = Vec::new();
            for qi in quasi_identifiers {
                key.push(row.get(qi).cloned().unwrap_or_default());
            }
            *groups.entry(key).or_insert(0) += 1;
        }

        // Count groups smaller than k
        let suppressed = groups.values().filter(|count| **count < k).sum::<usize>();
        let anonymized = suppressed == 0;

        Ok(KAnonymityResult {
            k_value: k,
            anonymized,
            generalized_rows: data.len() - suppressed,
            suppressed_rows: suppressed,
            information_loss: if !data.is_empty() {
                (suppressed as f64 / data.len() as f64) * 100.0
            } else {
                0.0
            },
        })
    }

    /// Suppress rows to achieve k-anonymity
    pub fn suppress_to_k_anonymous(
        data: &[HashMap<String, String>],
        quasi_identifiers: &[String],
        k: usize,
    ) -> Result<Vec<HashMap<String, String>>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        // Group rows by quasi-identifier combinations
        let mut groups: HashMap<Vec<String>, Vec<HashMap<String, String>>> = HashMap::new();

        for row in data {
            let mut key = Vec::new();
            for qi in quasi_identifiers {
                key.push(row.get(qi).cloned().unwrap_or_default());
            }
            groups.entry(key).or_default().push(row.clone());
        }

        // Keep only groups with size >= k
        let mut result = Vec::new();
        for group in groups.values() {
            if group.len() >= k {
                result.extend(group.clone());
            }
        }

        Ok(result)
    }

    /// Generalize values to intervals
    pub fn generalize_numeric(data: &[f64], intervals: usize) -> Result<Vec<usize>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        let min = data.iter().copied().fold(f64::INFINITY, f64::min);
        let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        let interval_size = if range > 0.0 {
            range / intervals as f64
        } else {
            1.0
        };

        let mut generalized = Vec::new();
        for value in data {
            let interval = if interval_size > 0.0 {
                ((value - min) / interval_size) as usize
            } else {
                0
            };
            generalized.push(interval.min(intervals - 1));
        }

        Ok(generalized)
    }

    /// Calculate information loss from generalization
    pub fn calculate_information_loss(
        original: &[f64],
        generalized: &[usize],
        intervals: usize,
    ) -> f64 {
        if original.is_empty() || original.len() != generalized.len() {
            return 0.0;
        }

        let min = original.iter().copied().fold(f64::INFINITY, f64::min);
        let max = original.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        if range == 0.0 {
            return 0.0;
        }

        let interval_size = range / intervals as f64;
        let mut total_loss = 0.0;

        for (orig, gen) in original.iter().zip(generalized.iter()) {
            let interval_min = min + (*gen as f64) * interval_size;
            let interval_max = interval_min + interval_size;
            let mid = (interval_min + interval_max) / 2.0;
            let loss = ((orig - mid).abs() / range).powi(2);
            total_loss += loss;
        }

        (total_loss / original.len() as f64).sqrt()
    }
}

/// Privacy audit result
#[derive(Clone, Debug)]
pub struct PrivacyAuditResult {
    pub compliant: bool,
    pub k_anonymous: bool,
    pub dp_budget_sufficient: bool,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

impl Default for PrivacyAuditResult {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivacyAuditResult {
    pub fn new() -> Self {
        Self {
            compliant: true,
            k_anonymous: true,
            dp_budget_sufficient: true,
            issues: Vec::new(),
            recommendations: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_budget() {
        let mut budget = PrivacyBudget::new(1.0, 0.01);

        assert!(budget.consume(0.3));
        assert!(budget.consume(0.3));
        assert!(budget.consume(0.39));
        assert!(!budget.consume(0.01));
        assert!(!budget.budget_exhausted()); // budget is at 0.01, not 0

        // Test exhaustion
        let mut budget2 = PrivacyBudget::new(0.5, 0.01);
        assert!(budget2.consume(0.5));
        assert!(budget2.budget_exhausted());
    }

    #[test]
    fn test_laplace_noise() {
        let noise = DifferentialPrivacy::laplace_noise(1.0, 1.0);
        assert!(noise.is_finite());
    }

    #[test]
    fn test_gaussian_noise() {
        let noise = DifferentialPrivacy::gaussian_noise(1.0, 0.01, 1.0);
        assert!(noise.is_finite());
    }

    #[test]
    fn test_add_laplace_noise() {
        let data = vec![10.0, 20.0, 30.0];
        let noisy = DifferentialPrivacy::add_laplace_noise(&data, 1.0, 1.0).unwrap();

        assert_eq!(noisy.len(), 3);
        for value in noisy {
            assert!(value >= 0.0);
        }
    }

    #[test]
    fn test_k_anonymity_check() {
        let mut data = Vec::new();

        // Create 4 rows: 2 with (20-30, M), 2 with (30-40, F)
        for _ in 0..2 {
            let mut row = HashMap::new();
            row.insert("age_group".to_string(), "20-30".to_string());
            row.insert("gender".to_string(), "M".to_string());
            data.push(row);
        }

        for _ in 0..2 {
            let mut row = HashMap::new();
            row.insert("age_group".to_string(), "30-40".to_string());
            row.insert("gender".to_string(), "F".to_string());
            data.push(row);
        }

        let quasi_ids = vec!["age_group".to_string(), "gender".to_string()];
        let result = KAnonymity::check_k_anonymity(&data, &quasi_ids, 2).unwrap();

        assert!(result.anonymized);
        assert_eq!(result.k_value, 2);
        assert_eq!(result.suppressed_rows, 0);
    }

    #[test]
    fn test_suppress_to_k_anonymous() {
        let mut row1 = HashMap::new();
        row1.insert("id".to_string(), "1".to_string());
        row1.insert("age".to_string(), "25".to_string());

        let mut row2 = HashMap::new();
        row2.insert("id".to_string(), "2".to_string());
        row2.insert("age".to_string(), "30".to_string());

        let data = vec![row1, row2];
        let quasi_ids = vec!["age".to_string()];

        let result = KAnonymity::suppress_to_k_anonymous(&data, &quasi_ids, 2).unwrap();

        assert_eq!(result.len(), 0); // Each value appears once, can't achieve k=2
    }

    #[test]
    fn test_generalize_numeric() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let generalized = KAnonymity::generalize_numeric(&data, 5).unwrap();

        assert_eq!(generalized.len(), 5);
        for value in generalized {
            assert!(value < 5);
        }
    }

    #[test]
    fn test_information_loss() {
        let original = vec![10.0, 20.0, 30.0];
        let generalized = vec![0, 1, 2];

        let loss = KAnonymity::calculate_information_loss(&original, &generalized, 3);
        assert!((0.0..=1.0).contains(&loss));
    }

    #[test]
    fn test_privacy_audit() {
        let audit = PrivacyAuditResult::new();
        assert!(audit.compliant);
        assert!(audit.k_anonymous);
    }

    #[test]
    fn test_quasi_identifier() {
        let qi = QuasiIdentifier::new("age_group".to_string())
            .add_value("20-30".to_string())
            .add_value("30-40".to_string());

        assert_eq!(qi.name, "age_group");
        assert_eq!(qi.values.len(), 2);
    }

    #[test]
    fn test_differential_privacy_mechanism() {
        assert_eq!(DPMechanism::Laplace.as_str(), "laplace");
        assert_eq!(DPMechanism::Gaussian.as_str(), "gaussian");
    }

    #[test]
    fn test_budget_percentage() {
        let mut budget = PrivacyBudget::new(1.0, 0.01);
        budget.consume(0.5);

        assert_eq!(budget.budget_percentage(), 50.0);
    }

    #[test]
    fn test_gaussian_noise_range() {
        for _ in 0..100 {
            let noise = DifferentialPrivacy::gaussian_noise(0.5, 0.01, 1.0);
            assert!(noise.is_finite());
        }
    }

    /// Regression test for a real, security-relevant bug: `laplace_noise`
    /// hardcoded `u = 0.5` instead of sampling uniform(0,1), which made the
    /// noise term evaluate to exactly 0.0 on every call -- the "Laplace
    /// mechanism" silently returned data completely unmodified, providing
    /// zero actual differential-privacy protection. Assert real variation
    /// (nonzero spread) across repeated calls with the same parameters.
    #[test]
    fn test_laplace_noise_is_actually_random() {
        let samples: Vec<f64> = (0..200)
            .map(|_| DifferentialPrivacy::laplace_noise(1.0, 1.0))
            .collect();

        let first = samples[0];
        assert!(
            samples.iter().any(|&s| (s - first).abs() > 1e-9),
            "laplace_noise() produced identical output across {} calls -- noise is not random",
            samples.len()
        );

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let variance =
            samples.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / samples.len() as f64;
        assert!(
            variance > 1e-6,
            "laplace_noise() output has near-zero variance ({variance}) across samples -- \
             noise is effectively constant"
        );
    }

    /// Regression test for a real, security-relevant bug: `gaussian_noise`
    /// hardcoded `sigma * 0.5` -- a fixed, deterministic (and trivially
    /// reversible) shift with no randomness at all. Assert real variation
    /// across repeated calls with the same parameters.
    #[test]
    fn test_gaussian_noise_is_actually_random() {
        let samples: Vec<f64> = (0..200)
            .map(|_| DifferentialPrivacy::gaussian_noise(1.0, 0.01, 1.0))
            .collect();

        let first = samples[0];
        assert!(
            samples.iter().any(|&s| (s - first).abs() > 1e-9),
            "gaussian_noise() produced identical output across {} calls -- noise is not random",
            samples.len()
        );

        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let variance =
            samples.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / samples.len() as f64;
        assert!(
            variance > 1e-6,
            "gaussian_noise() output has near-zero variance ({variance}) across samples -- \
             noise is effectively constant"
        );
    }
}
