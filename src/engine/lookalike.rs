//! Lookalike audience generation from seed customers

use crate::Result;
use std::collections::HashMap;

/// Similarity metric
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum SimilarityMetric {
    Cosine,
    Euclidean,
    Manhattan,
    Jaccard,
}

impl SimilarityMetric {
    pub fn as_str(&self) -> &str {
        match self {
            SimilarityMetric::Cosine => "cosine",
            SimilarityMetric::Euclidean => "euclidean",
            SimilarityMetric::Manhattan => "manhattan",
            SimilarityMetric::Jaccard => "jaccard",
        }
    }
}

/// Seed customer (high-value or target cohort)
#[derive(Clone, Debug)]
pub struct SeedCustomer {
    pub customer_id: String,
    pub features: Vec<f64>,
    pub categorical_features: HashMap<String, String>,
    pub ltv: f64,
    pub cohort: String,
}

/// Candidate for lookalike audience
#[derive(Clone, Debug)]
pub struct LookalikeCandidate {
    pub customer_id: String,
    pub similarity_score: f64,
    pub percentile: f64,
    pub features: Vec<f64>,
}

/// Lookalike audience
#[derive(Clone, Debug)]
pub struct LookalikeAudience {
    pub audience_name: String,
    pub seed_count: usize,
    pub lookalike_count: usize,
    pub min_similarity: f64,
    pub max_similarity: f64,
    pub avg_similarity: f64,
    pub predicted_ltv: f64,
    pub metric: SimilarityMetric,
}

/// Lookalike audience generator
pub struct LookalikeGenerator;

impl LookalikeGenerator {
    /// Calculate cosine similarity between two feature vectors
    pub fn cosine_similarity(vec_a: &[f64], vec_b: &[f64]) -> f64 {
        if vec_a.len() != vec_b.len() || vec_a.is_empty() {
            return 0.0;
        }

        let dot_product: f64 = vec_a.iter().zip(vec_b.iter()).map(|(a, b)| a * b).sum();
        let mag_a: f64 = vec_a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = vec_b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }

        dot_product / (mag_a * mag_b)
    }

    /// Calculate Euclidean distance
    pub fn euclidean_distance(vec_a: &[f64], vec_b: &[f64]) -> f64 {
        if vec_a.len() != vec_b.len() {
            return f64::MAX;
        }

        let sum: f64 = vec_a
            .iter()
            .zip(vec_b.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();

        sum.sqrt()
    }

    /// Convert Euclidean distance to similarity (0-1 scale)
    pub fn euclidean_to_similarity(distance: f64, max_distance: f64) -> f64 {
        if max_distance == 0.0 {
            return 0.0;
        }
        1.0 - (distance / max_distance).min(1.0)
    }

    /// Calculate Manhattan distance
    pub fn manhattan_distance(vec_a: &[f64], vec_b: &[f64]) -> f64 {
        if vec_a.len() != vec_b.len() {
            return f64::MAX;
        }

        vec_a
            .iter()
            .zip(vec_b.iter())
            .map(|(a, b)| (a - b).abs())
            .sum()
    }

    /// Calculate Jaccard similarity for categorical features
    pub fn jaccard_similarity(
        features_a: &HashMap<String, String>,
        features_b: &HashMap<String, String>,
    ) -> f64 {
        if features_a.is_empty() || features_b.is_empty() {
            return 0.0;
        }

        let intersection = features_a
            .iter()
            .filter(|(k, v)| features_b.get(*k) == Some(v))
            .count();

        let union = features_a.len() + features_b.len() - intersection;

        intersection as f64 / union as f64
    }

    /// Generate lookalike audience from seed customers
    pub fn generate_lookalike(
        seed_customers: &[SeedCustomer],
        candidate_customers: &[SeedCustomer],
        metric: SimilarityMetric,
        percentile_threshold: f64,
        max_lookalikes: Option<usize>,
    ) -> Result<LookalikeAudience> {
        if seed_customers.is_empty() || candidate_customers.is_empty() {
            return Ok(LookalikeAudience {
                audience_name: "Empty".to_string(),
                seed_count: 0,
                lookalike_count: 0,
                min_similarity: 0.0,
                max_similarity: 0.0,
                avg_similarity: 0.0,
                predicted_ltv: 0.0,
                metric,
            });
        }

        // Calculate average seed features
        let avg_seed_features = Self::calculate_average_features(seed_customers);

        // Score all candidates
        let mut candidates = Vec::new();
        let mut similarities = Vec::new();

        for candidate in candidate_customers {
            let similarity = match metric {
                SimilarityMetric::Cosine => Self::cosine_similarity(&avg_seed_features, &candidate.features),
                SimilarityMetric::Euclidean => {
                    let max_dist = 100.0; // Normalize
                    Self::euclidean_to_similarity(
                        Self::euclidean_distance(&avg_seed_features, &candidate.features),
                        max_dist,
                    )
                }
                SimilarityMetric::Manhattan => {
                    let max_dist = 200.0; // Normalize
                    Self::euclidean_to_similarity(
                        Self::manhattan_distance(&avg_seed_features, &candidate.features),
                        max_dist,
                    )
                }
                SimilarityMetric::Jaccard => {
                    // Use seed average categorical features
                    let avg_categorical = Self::calculate_average_categorical(seed_customers);
                    Self::jaccard_similarity(&avg_categorical, &candidate.categorical_features)
                }
            };

            similarities.push(similarity);

            if similarity > 0.0 {
                candidates.push((similarity, candidate.clone()));
            }
        }

        // Sort by similarity descending
        candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Apply percentile threshold
        let threshold_score = if similarities.len() > 0 {
            let sorted: Vec<f64> = similarities.iter().copied().collect();
            let idx = ((1.0 - percentile_threshold) * sorted.len() as f64) as usize;
            sorted.get(idx).copied().unwrap_or(0.0)
        } else {
            0.0
        };

        let filtered: Vec<_> = candidates
            .into_iter()
            .filter(|(sim, _)| *sim >= threshold_score)
            .collect();

        let lookalikes = filtered
            .iter()
            .take(max_lookalikes.unwrap_or(usize::MAX))
            .collect::<Vec<_>>();

        // Calculate metrics
        let min_sim = lookalikes.iter().map(|(s, _)| *s).fold(f64::INFINITY, f64::min);
        let max_sim = lookalikes.iter().map(|(s, _)| *s).fold(0.0, f64::max);
        let avg_sim = if !lookalikes.is_empty() {
            lookalikes.iter().map(|(s, _)| s).sum::<f64>() / lookalikes.len() as f64
        } else {
            0.0
        };

        let avg_seed_ltv = seed_customers.iter().map(|c| c.ltv).sum::<f64>() / seed_customers.len() as f64;
        let predicted_ltv = avg_sim * avg_seed_ltv;

        Ok(LookalikeAudience {
            audience_name: format!("Lookalike ({})", metric.as_str()),
            seed_count: seed_customers.len(),
            lookalike_count: lookalikes.len(),
            min_similarity: min_sim,
            max_similarity: max_sim,
            avg_similarity: avg_sim,
            predicted_ltv,
            metric,
        })
    }

    /// Calculate average feature vector from seed customers
    fn calculate_average_features(seeds: &[SeedCustomer]) -> Vec<f64> {
        if seeds.is_empty() {
            return vec![];
        }

        let feature_dim = seeds[0].features.len();
        let mut avg = vec![0.0; feature_dim];

        for seed in seeds {
            for (i, val) in seed.features.iter().enumerate() {
                avg[i] += val;
            }
        }

        for avg_val in &mut avg {
            *avg_val /= seeds.len() as f64;
        }

        avg
    }

    /// Calculate most common categorical features
    fn calculate_average_categorical(seeds: &[SeedCustomer]) -> HashMap<String, String> {
        let mut feature_counts: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for seed in seeds {
            for (key, val) in &seed.categorical_features {
                feature_counts
                    .entry(key.clone())
                    .or_insert_with(HashMap::new)
                    .entry(val.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        }

        let mut result = HashMap::new();
        for (key, val_counts) in feature_counts {
            let most_common = val_counts.into_iter().max_by_key(|(_, count)| *count).map(|(v, _)| v);
            if let Some(val) = most_common {
                result.insert(key, val);
            }
        }

        result
    }

    /// Find top N similar customers
    pub fn find_similar_customers(
        seed: &SeedCustomer,
        candidates: &[SeedCustomer],
        n: usize,
        metric: SimilarityMetric,
    ) -> Result<Vec<LookalikeCandidate>> {
        let mut scored = Vec::new();

        for candidate in candidates {
            let similarity = match metric {
                SimilarityMetric::Cosine => Self::cosine_similarity(&seed.features, &candidate.features),
                SimilarityMetric::Euclidean => {
                    Self::euclidean_to_similarity(
                        Self::euclidean_distance(&seed.features, &candidate.features),
                        100.0,
                    )
                }
                SimilarityMetric::Manhattan => {
                    Self::euclidean_to_similarity(
                        Self::manhattan_distance(&seed.features, &candidate.features),
                        200.0,
                    )
                }
                SimilarityMetric::Jaccard => {
                    Self::jaccard_similarity(&seed.categorical_features, &candidate.categorical_features)
                }
            };

            scored.push((similarity, candidate.clone()));
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let total = scored.len() as f64;
        let mut results = Vec::new();

        for (idx, (sim, candidate)) in scored.iter().take(n).enumerate() {
            let percentile = (idx as f64 / total) * 100.0;

            results.push(LookalikeCandidate {
                customer_id: candidate.customer_id.clone(),
                similarity_score: *sim,
                percentile,
                features: candidate.features.clone(),
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_seed_customer() -> SeedCustomer {
        SeedCustomer {
            customer_id: "seed_1".to_string(),
            features: vec![0.8, 0.9, 0.7],
            categorical_features: {
                let mut m = HashMap::new();
                m.insert("industry".to_string(), "SaaS".to_string());
                m
            },
            ltv: 5000.0,
            cohort: "high_value".to_string(),
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![1.0, 0.0, 0.0];

        let sim = LookalikeGenerator::cosine_similarity(&vec_a, &vec_b);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_euclidean_distance() {
        let vec_a = vec![0.0, 0.0];
        let vec_b = vec![3.0, 4.0];

        let dist = LookalikeGenerator::euclidean_distance(&vec_a, &vec_b);
        assert!((dist - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_manhattan_distance() {
        let vec_a = vec![0.0, 0.0];
        let vec_b = vec![3.0, 4.0];

        let dist = LookalikeGenerator::manhattan_distance(&vec_a, &vec_b);
        assert_eq!(dist, 7.0);
    }

    #[test]
    fn test_jaccard_similarity() {
        let mut set_a = HashMap::new();
        set_a.insert("a".to_string(), "1".to_string());
        set_a.insert("b".to_string(), "2".to_string());

        let mut set_b = HashMap::new();
        set_b.insert("a".to_string(), "1".to_string());

        let sim = LookalikeGenerator::jaccard_similarity(&set_a, &set_b);
        assert!((sim - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_generate_lookalike() {
        let seeds = vec![create_seed_customer()];

        let candidate = SeedCustomer {
            customer_id: "cand_1".to_string(),
            features: vec![0.75, 0.85, 0.72],
            categorical_features: {
                let mut m = HashMap::new();
                m.insert("industry".to_string(), "SaaS".to_string());
                m
            },
            ltv: 4500.0,
            cohort: "similar".to_string(),
        };

        let audience = LookalikeGenerator::generate_lookalike(
            &seeds,
            &[candidate],
            SimilarityMetric::Cosine,
            0.5,
            Some(10),
        )
        .unwrap();

        assert_eq!(audience.seed_count, 1);
        assert!(audience.avg_similarity > 0.0);
    }

    #[test]
    fn test_find_similar_customers() {
        let seed = create_seed_customer();

        let candidate = SeedCustomer {
            customer_id: "cand_1".to_string(),
            features: vec![0.75, 0.85, 0.72],
            categorical_features: HashMap::new(),
            ltv: 4500.0,
            cohort: "similar".to_string(),
        };

        let similar = LookalikeGenerator::find_similar_customers(
            &seed,
            &[candidate],
            5,
            SimilarityMetric::Cosine,
        )
        .unwrap();

        assert_eq!(similar.len(), 1);
        assert!(similar[0].similarity_score > 0.0);
    }

    #[test]
    fn test_similarity_metric_names() {
        assert_eq!(SimilarityMetric::Cosine.as_str(), "cosine");
        assert_eq!(SimilarityMetric::Euclidean.as_str(), "euclidean");
    }

    #[test]
    fn test_lookalike_audience_metrics() {
        let seed = create_seed_customer();

        let candidates = vec![
            SeedCustomer {
                customer_id: "cand_1".to_string(),
                features: vec![0.8, 0.9, 0.7],
                categorical_features: HashMap::new(),
                ltv: 5000.0,
                cohort: "similar".to_string(),
            },
            SeedCustomer {
                customer_id: "cand_2".to_string(),
                features: vec![0.2, 0.3, 0.4],
                categorical_features: HashMap::new(),
                ltv: 1000.0,
                cohort: "dissimilar".to_string(),
            },
        ];

        let audience = LookalikeGenerator::generate_lookalike(
            &[seed],
            &candidates,
            SimilarityMetric::Cosine,
            0.0,
            None,
        )
        .unwrap();

        assert!(audience.min_similarity >= 0.0);
        assert!(audience.max_similarity <= 1.0);
        assert!(audience.avg_similarity > 0.0);
    }

    #[test]
    fn test_percentile_filtering() {
        let seed = create_seed_customer();

        let candidates: Vec<_> = (0..100)
            .map(|i| SeedCustomer {
                customer_id: format!("cand_{}", i),
                features: vec![0.5 + (i as f64 / 200.0), 0.5, 0.5],
                categorical_features: HashMap::new(),
                ltv: 3000.0,
                cohort: "test".to_string(),
            })
            .collect();

        let audience = LookalikeGenerator::generate_lookalike(
            &[seed],
            &candidates,
            SimilarityMetric::Cosine,
            0.9, // Top 10%
            None,
        )
        .unwrap();

        assert!(audience.lookalike_count <= 20); // ~10% of 100
    }
}
