//! XGBoost wrapper for advanced predictive modeling

use crate::Result;
use std::collections::HashMap;

/// XGBoost model type
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XGBModelType {
    Classification, // Binary/multi-class
    Regression,     // Continuous prediction
}

impl XGBModelType {
    pub fn as_str(&self) -> &str {
        match self {
            XGBModelType::Classification => "classification",
            XGBModelType::Regression => "regression",
        }
    }
}

/// XGBoost hyperparameters
#[derive(Clone, Debug)]
pub struct XGBParams {
    pub n_estimators: usize,        // Number of boosting rounds
    pub max_depth: usize,           // Max tree depth
    pub learning_rate: f64,         // Shrinkage (eta)
    pub subsample: f64,             // Fraction of samples for each tree
    pub colsample_bytree: f64,      // Fraction of features for each tree
    pub min_child_weight: f64,      // Min sum of weights for child
    pub lambda: f64,                // L2 regularization
    pub alpha: f64,                 // L1 regularization
}

impl Default for XGBParams {
    fn default() -> Self {
        Self {
            n_estimators: 100,
            max_depth: 6,
            learning_rate: 0.1,
            subsample: 0.8,
            colsample_bytree: 0.8,
            min_child_weight: 1.0,
            lambda: 1.0,
            alpha: 0.0,
        }
    }
}

impl XGBParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_n_estimators(mut self, n: usize) -> Self {
        self.n_estimators = n;
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }

    pub fn with_regularization(mut self, lambda: f64, alpha: f64) -> Self {
        self.lambda = lambda;
        self.alpha = alpha;
        self
    }
}

/// Feature importance
#[derive(Clone, Debug)]
pub struct FeatureImportance {
    pub feature_name: String,
    pub importance_score: f64,
    pub rank: usize,
}

/// XGBoost model prediction
#[derive(Clone, Debug)]
pub struct XGBPrediction {
    pub sample_id: String,
    pub prediction: f64,
    pub probability: Option<f64>, // For classification
    pub feature_contributions: HashMap<String, f64>,
}

/// Model training result
#[derive(Clone, Debug)]
pub struct TrainingResult {
    pub model_type: XGBModelType,
    pub n_features: usize,
    pub n_samples: usize,
    pub train_score: f64,
    pub validation_score: f64,
    pub feature_importances: Vec<FeatureImportance>,
}

/// Feature engineering utilities
pub struct FeatureEngineering;

impl FeatureEngineering {
    /// Normalize features to [0, 1]
    pub fn normalize(data: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        let n_features = data[0].len();
        let mut mins = vec![f64::INFINITY; n_features];
        let mut maxs = vec![f64::NEG_INFINITY; n_features];

        // Find min/max for each feature
        for sample in data {
            for (i, val) in sample.iter().enumerate() {
                mins[i] = mins[i].min(*val);
                maxs[i] = maxs[i].max(*val);
            }
        }

        // Normalize
        let mut normalized = Vec::new();
        for sample in data {
            let mut norm_sample = Vec::new();
            for (i, val) in sample.iter().enumerate() {
                let range = maxs[i] - mins[i];
                let norm_val = if range > 0.0 {
                    (val - mins[i]) / range
                } else {
                    0.5
                };
                norm_sample.push(norm_val);
            }
            normalized.push(norm_sample);
        }

        Ok(normalized)
    }

    /// Standardize features (zero mean, unit variance)
    pub fn standardize(data: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        let n_features = data[0].len();
        let n_samples = data.len() as f64;

        // Calculate mean
        let mut means = vec![0.0; n_features];
        for sample in data {
            for (i, val) in sample.iter().enumerate() {
                means[i] += val / n_samples;
            }
        }

        // Calculate std dev
        let mut stds = vec![0.0; n_features];
        for sample in data {
            for (i, val) in sample.iter().enumerate() {
                stds[i] += (val - means[i]).powi(2) / n_samples;
            }
        }
        for std in &mut stds {
            *std = std.sqrt();
        }

        // Standardize
        let mut standardized = Vec::new();
        for sample in data {
            let mut std_sample = Vec::new();
            for (i, val) in sample.iter().enumerate() {
                let std_val = if stds[i] > 0.0 {
                    (val - means[i]) / stds[i]
                } else {
                    0.0
                };
                std_sample.push(std_val);
            }
            standardized.push(std_sample);
        }

        Ok(standardized)
    }

    /// Polynomial feature generation (degree 2)
    pub fn polynomial_features(data: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        let mut poly_data = Vec::new();

        for sample in data {
            let mut poly_sample = sample.clone();

            // Add interaction terms and squared terms
            for i in 0..sample.len() {
                for j in i..sample.len() {
                    poly_sample.push(sample[i] * sample[j]);
                }
            }

            poly_data.push(poly_sample);
        }

        Ok(poly_data)
    }

    /// Feature selection by variance threshold
    pub fn select_high_variance_features(
        data: &[Vec<f64>],
        threshold: f64,
    ) -> Result<Vec<usize>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        let n_features = data[0].len();
        let n_samples = data.len() as f64;

        // Calculate variance for each feature
        let mut variances = vec![0.0; n_features];

        // Calculate means
        let mut means = vec![0.0; n_features];
        for sample in data {
            for (i, val) in sample.iter().enumerate() {
                means[i] += val / n_samples;
            }
        }

        // Calculate variances
        for sample in data {
            for (i, val) in sample.iter().enumerate() {
                variances[i] += (val - means[i]).powi(2) / n_samples;
            }
        }

        // Select features with variance > threshold
        let selected: Vec<usize> = variances
            .iter()
            .enumerate()
            .filter(|(_, var)| **var > threshold)
            .map(|(i, _)| i)
            .collect();

        Ok(selected)
    }
}

/// XGBoost model wrapper
pub struct XGBModel {
    pub model_type: XGBModelType,
    pub params: XGBParams,
    pub feature_names: Vec<String>,
    pub trained: bool,
    pub feature_importances: Vec<FeatureImportance>,
}

impl XGBModel {
    pub fn new(model_type: XGBModelType, params: XGBParams) -> Self {
        Self {
            model_type,
            params,
            feature_names: Vec::new(),
            trained: false,
            feature_importances: Vec::new(),
        }
    }

    pub fn with_features(mut self, feature_names: Vec<String>) -> Self {
        self.feature_names = feature_names;
        self
    }

    /// Simulate training (in production, would call actual XGBoost library)
    pub fn train(
        &mut self,
        X: &[Vec<f64>],
        y: &[f64],
        validation_split: f64,
    ) -> Result<TrainingResult> {
        if X.is_empty() || y.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Empty training data",
            )
            .into());
        }

        let n_samples = X.len();
        let n_features = X[0].len();
        let n_validation = (n_samples as f64 * validation_split) as usize;
        let n_train = n_samples - n_validation;

        // Simulate feature importance (in real XGBoost, computed from tree splits)
        let mut importances = vec![0.0; n_features];
        for sample in X.iter().take(n_train) {
            for (i, val) in sample.iter().enumerate() {
                importances[i] += val.abs();
            }
        }

        // Normalize importances
        let total: f64 = importances.iter().sum();
        if total > 0.0 {
            for imp in &mut importances {
                *imp /= total;
            }
        }

        // Create feature importance list
        let mut feature_imps: Vec<FeatureImportance> = importances
            .iter()
            .enumerate()
            .map(|(i, score)| FeatureImportance {
                feature_name: self
                    .feature_names
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("feature_{}", i)),
                importance_score: *score,
                rank: 0,
            })
            .collect();

        // Sort by importance and assign ranks
        feature_imps.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap());
        for (rank, imp) in feature_imps.iter_mut().enumerate() {
            imp.rank = rank + 1;
        }

        // Simulate scores
        let train_score = 0.85 + (self.params.learning_rate * 10.0).min(0.1);
        let validation_score = 0.82 + (self.params.learning_rate * 10.0).min(0.08);

        self.trained = true;
        self.feature_importances = feature_imps.clone();

        Ok(TrainingResult {
            model_type: self.model_type.clone(),
            n_features,
            n_samples,
            train_score,
            validation_score,
            feature_importances: feature_imps,
        })
    }

    /// Simulate prediction
    pub fn predict(&self, X: &[Vec<f64>]) -> Result<Vec<XGBPrediction>> {
        if !self.trained {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Model not trained",
            )
            .into());
        }

        let mut predictions = Vec::new();

        for (idx, sample) in X.iter().enumerate() {
            let mut pred_value = 0.5; // Base prediction

            // Simulate prediction using features
            for (i, val) in sample.iter().enumerate() {
                if let Some(imp) = self.feature_importances.get(i) {
                    pred_value += imp.importance_score * val * 0.1;
                }
            }

            pred_value = pred_value.max(0.0).min(1.0);

            let probability = match self.model_type {
                XGBModelType::Classification => Some(pred_value),
                XGBModelType::Regression => None,
            };

            let mut contrib = HashMap::new();
            for (i, val) in sample.iter().enumerate() {
                if let Some(imp) = self.feature_importances.get(i) {
                    contrib.insert(
                        imp.feature_name.clone(),
                        imp.importance_score * val * 0.1,
                    );
                }
            }

            predictions.push(XGBPrediction {
                sample_id: format!("sample_{}", idx),
                prediction: pred_value,
                probability,
                feature_contributions: contrib,
            });
        }

        Ok(predictions)
    }

    /// Get feature importance
    pub fn get_feature_importance(&self) -> Vec<FeatureImportance> {
        self.feature_importances.clone()
    }

    /// Get top N features
    pub fn get_top_features(&self, n: usize) -> Vec<FeatureImportance> {
        self.feature_importances.iter().take(n).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xgb_params() {
        let params = XGBParams::default()
            .with_n_estimators(200)
            .with_max_depth(8)
            .with_learning_rate(0.05);

        assert_eq!(params.n_estimators, 200);
        assert_eq!(params.max_depth, 8);
        assert_eq!(params.learning_rate, 0.05);
    }

    #[test]
    fn test_normalize() {
        let data = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
        let normalized = FeatureEngineering::normalize(&data).unwrap();

        assert_eq!(normalized.len(), 3);
        for sample in normalized {
            for val in sample {
                assert!(val >= 0.0 && val <= 1.0);
            }
        }
    }

    #[test]
    fn test_standardize() {
        let data = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
        let standardized = FeatureEngineering::standardize(&data).unwrap();

        assert_eq!(standardized.len(), 3);
        // Mean should be close to 0
        let means: Vec<f64> = (0..standardized[0].len())
            .map(|i| {
                standardized
                    .iter()
                    .map(|s| s[i])
                    .sum::<f64>()
                    / standardized.len() as f64
            })
            .collect();

        for mean in means {
            assert!(mean.abs() < 1e-10);
        }
    }

    #[test]
    fn test_polynomial_features() {
        let data = vec![vec![1.0, 2.0]];
        let poly = FeatureEngineering::polynomial_features(&data).unwrap();

        assert_eq!(poly.len(), 1);
        // Should have original + interaction + squared terms
        assert!(poly[0].len() > data[0].len());
    }

    #[test]
    fn test_high_variance_features() {
        let data = vec![
            vec![1.0, 0.0, 10.0],
            vec![1.0, 0.0, 20.0],
            vec![1.0, 0.0, 30.0],
        ];

        let selected = FeatureEngineering::select_high_variance_features(&data, 1.0).unwrap();

        assert!(!selected.is_empty());
        // Third feature should have high variance
        assert!(selected.contains(&2));
    }

    #[test]
    fn test_xgb_model_creation() {
        let model = XGBModel::new(XGBModelType::Classification, XGBParams::default());

        assert_eq!(model.model_type, XGBModelType::Classification);
        assert!(!model.trained);
    }

    #[test]
    fn test_xgb_training() {
        let mut model = XGBModel::new(
            XGBModelType::Classification,
            XGBParams::default().with_n_estimators(50),
        );

        model = model.with_features(vec![
            "recency".to_string(),
            "frequency".to_string(),
            "monetary".to_string(),
        ]);

        let X = vec![vec![0.5, 0.5, 0.5]; 10];
        let y = vec![0.0; 10];

        let result = model.train(&X, &y, 0.2).unwrap();

        assert_eq!(result.n_features, 3);
        assert_eq!(result.n_samples, 10);
        assert!(result.train_score > 0.0);
        assert!(!result.feature_importances.is_empty());
    }

    #[test]
    fn test_xgb_prediction() {
        let mut model = XGBModel::new(XGBModelType::Classification, XGBParams::default());

        model = model.with_features(vec!["f1".to_string(), "f2".to_string()]);

        let X = vec![vec![0.5, 0.5]; 10];
        let y = vec![1.0; 10];

        model.train(&X, &y, 0.2).unwrap();

        let predictions = model.predict(&X).unwrap();

        assert_eq!(predictions.len(), 10);
        for pred in predictions {
            assert!(pred.prediction >= 0.0 && pred.prediction <= 1.0);
            assert!(pred.probability.is_some());
        }
    }

    #[test]
    fn test_feature_importance() {
        let mut model = XGBModel::new(XGBModelType::Regression, XGBParams::default());

        model = model.with_features(vec!["f1".to_string(), "f2".to_string(), "f3".to_string()]);

        let X = vec![vec![1.0, 2.0, 3.0]; 5];
        let y = vec![1.0; 5];

        model.train(&X, &y, 0.2).unwrap();

        let importances = model.get_feature_importance();
        assert_eq!(importances.len(), 3);

        let top = model.get_top_features(2);
        assert_eq!(top.len(), 2);
        assert!(top[0].rank < top[1].rank);
    }

    #[test]
    fn test_xgb_model_type() {
        assert_eq!(XGBModelType::Classification.as_str(), "classification");
        assert_eq!(XGBModelType::Regression.as_str(), "regression");
    }

    #[test]
    fn test_untrained_predict() {
        let model = XGBModel::new(XGBModelType::Classification, XGBParams::default());

        let X = vec![vec![0.5, 0.5]];
        let result = model.predict(&X);

        assert!(result.is_err());
    }
}
