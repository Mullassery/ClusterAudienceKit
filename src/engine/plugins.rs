//! Extensible plugin system for custom algorithms

use crate::Result;
use std::collections::HashMap;

/// Algorithm type enum
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlgorithmType {
    Clustering,
    Classification,
    Regression,
    RFM,
    Custom,
}

impl AlgorithmType {
    pub fn as_str(&self) -> &str {
        match self {
            AlgorithmType::Clustering => "clustering",
            AlgorithmType::Classification => "classification",
            AlgorithmType::Regression => "regression",
            AlgorithmType::RFM => "rfm",
            AlgorithmType::Custom => "custom",
        }
    }
}

/// Plugin metadata
#[derive(Clone, Debug)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub algorithm_type: AlgorithmType,
    pub min_samples: usize,
    pub max_samples: usize,
    pub supported_features: Vec<String>,
}

impl PluginMetadata {
    pub fn new(name: String, version: String, algorithm_type: AlgorithmType) -> Self {
        Self {
            name,
            version,
            author: "Custom".to_string(),
            description: "".to_string(),
            algorithm_type,
            min_samples: 10,
            max_samples: 10_000_000,
            supported_features: vec![],
        }
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_samples(mut self, min: usize, max: usize) -> Self {
        self.min_samples = min;
        self.max_samples = max;
        self
    }

    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.supported_features = features;
        self
    }
}

/// Plugin execution result
#[derive(Clone, Debug)]
pub struct PluginResult {
    pub plugin_name: String,
    pub success: bool,
    pub outputs: HashMap<String, Vec<f64>>,
    pub metadata: HashMap<String, String>,
    pub execution_time_ms: u64,
}

impl PluginResult {
    pub fn new(plugin_name: String) -> Self {
        Self {
            plugin_name,
            success: true,
            outputs: HashMap::new(),
            metadata: HashMap::new(),
            execution_time_ms: 0,
        }
    }

    pub fn with_output(mut self, key: String, values: Vec<f64>) -> Self {
        self.outputs.insert(key, values);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_timing(mut self, ms: u64) -> Self {
        self.execution_time_ms = ms;
        self
    }
}

/// Plugin parameter
#[derive(Clone, Debug)]
pub struct PluginParameter {
    pub name: String,
    pub param_type: String, // "float", "int", "string", "bool"
    pub default_value: String,
    pub required: bool,
    pub description: String,
}

impl PluginParameter {
    pub fn new(name: String, param_type: String, default_value: String) -> Self {
        Self {
            name,
            param_type,
            default_value,
            required: false,
            description: "".to_string(),
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
}

/// Plugin configuration
#[derive(Clone, Debug)]
pub struct PluginConfig {
    pub parameters: HashMap<String, String>,
}

impl PluginConfig {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    pub fn set_parameter(mut self, key: String, value: String) -> Self {
        self.parameters.insert(key, value);
        self
    }

    pub fn get_parameter(&self, key: &str) -> Option<String> {
        self.parameters.get(key).cloned()
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin interface (trait for implementing custom algorithms)
pub trait Algorithm: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn parameters(&self) -> Vec<PluginParameter>;
    fn execute(&self, data: &[Vec<f64>], config: &PluginConfig) -> Result<PluginResult>;
    fn validate(&self, data: &[Vec<f64>]) -> Result<bool>;
}

/// Plugin registry
pub struct PluginRegistry {
    algorithms: HashMap<String, Box<dyn Algorithm>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            algorithms: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register<A: Algorithm + 'static>(&mut self, algorithm: A) -> Result<()> {
        let name = algorithm.metadata().name.clone();
        self.algorithms.insert(name, Box::new(algorithm));
        Ok(())
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<&dyn Algorithm> {
        self.algorithms.get(name).map(|a| a.as_ref())
    }

    /// Execute a plugin
    pub fn execute(
        &self,
        name: &str,
        data: &[Vec<f64>],
        config: &PluginConfig,
    ) -> Result<PluginResult> {
        let algo = self.get(name).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Plugin {} not found", name),
            )
        })?;

        algo.validate(data)?;
        algo.execute(data, config)
    }

    /// List all plugins
    pub fn list(&self) -> Vec<String> {
        self.algorithms.keys().cloned().collect()
    }

    /// Get plugin count
    pub fn count(&self) -> usize {
        self.algorithms.len()
    }

    /// Get plugin info
    pub fn info(&self, name: &str) -> Option<PluginMetadata> {
        self.get(name).map(|a| a.metadata())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple example plugin: Mean-based classifier
pub struct MeanBasedClassifier;

impl Algorithm for MeanBasedClassifier {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "mean_classifier".to_string(),
            version: "1.0.0".to_string(),
            author: "ClusterAudienceKit".to_string(),
            description: "Simple mean-based classification".to_string(),
            algorithm_type: AlgorithmType::Classification,
            min_samples: 10,
            max_samples: 1_000_000,
            supported_features: vec!["numeric".to_string()],
        }
    }

    fn parameters(&self) -> Vec<PluginParameter> {
        vec![PluginParameter::new(
            "threshold".to_string(),
            "float".to_string(),
            "0.5".to_string(),
        )
        .with_description("Classification threshold".to_string())]
    }

    fn execute(&self, data: &[Vec<f64>], config: &PluginConfig) -> Result<PluginResult> {
        if data.is_empty() {
            return Ok(PluginResult::new("mean_classifier".to_string()));
        }

        let threshold: f64 = config
            .get_parameter("threshold")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.5);

        let mean = Self::calculate_mean(data);
        let mut classifications = Vec::new();

        for row in data {
            let row_mean: f64 = row.iter().sum::<f64>() / row.len() as f64;
            classifications.push(if row_mean > threshold { 1.0 } else { 0.0 });
        }

        Ok(PluginResult::new("mean_classifier".to_string())
            .with_output("classifications".to_string(), classifications)
            .with_metadata("threshold".to_string(), threshold.to_string())
            .with_metadata("global_mean".to_string(), mean.to_string()))
    }

    fn validate(&self, data: &[Vec<f64>]) -> Result<bool> {
        if data.is_empty() {
            return Ok(false);
        }

        let first_len = data[0].len();
        if first_len == 0 {
            return Ok(false);
        }

        for row in data {
            if row.len() != first_len {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl MeanBasedClassifier {
    fn calculate_mean(data: &[Vec<f64>]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let sum: f64 = data.iter().flat_map(|row| row.iter()).sum();
        let count = data.len() as f64 * data[0].len() as f64;

        sum / count
    }
}

/// Standard deviation calculator plugin
pub struct StdDevCalculator;

impl Algorithm for StdDevCalculator {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "stddev_calculator".to_string(),
            version: "1.0.0".to_string(),
            author: "ClusterAudienceKit".to_string(),
            description: "Calculate standard deviation".to_string(),
            algorithm_type: AlgorithmType::RFM,
            min_samples: 5,
            max_samples: 1_000_000,
            supported_features: vec!["numeric".to_string()],
        }
    }

    fn parameters(&self) -> Vec<PluginParameter> {
        vec![]
    }

    fn execute(&self, data: &[Vec<f64>], _config: &PluginConfig) -> Result<PluginResult> {
        if data.is_empty() {
            return Ok(PluginResult::new("stddev_calculator".to_string()));
        }

        let mut stddevs = Vec::new();

        for col_idx in 0..data[0].len() {
            let col_values: Vec<f64> = data.iter().map(|row| row[col_idx]).collect();
            let mean = col_values.iter().sum::<f64>() / col_values.len() as f64;
            let variance: f64 = col_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                / col_values.len() as f64;
            stddevs.push(variance.sqrt());
        }

        Ok(PluginResult::new("stddev_calculator".to_string())
            .with_output("stddev".to_string(), stddevs))
    }

    fn validate(&self, data: &[Vec<f64>]) -> Result<bool> {
        Ok(!data.is_empty() && !data[0].is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let meta = PluginMetadata::new(
            "test_plugin".to_string(),
            "1.0.0".to_string(),
            AlgorithmType::Clustering,
        )
        .with_author("Test Author".to_string())
        .with_description("Test Description".to_string());

        assert_eq!(meta.name, "test_plugin");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.author, "Test Author");
    }

    #[test]
    fn test_plugin_parameter() {
        let param = PluginParameter::new(
            "threshold".to_string(),
            "float".to_string(),
            "0.5".to_string(),
        )
        .required(true);

        assert_eq!(param.name, "threshold");
        assert!(param.required);
    }

    #[test]
    fn test_plugin_config() {
        let config = PluginConfig::new()
            .set_parameter("threshold".to_string(), "0.75".to_string())
            .set_parameter("max_iter".to_string(), "100".to_string());

        assert_eq!(config.get_parameter("threshold"), Some("0.75".to_string()));
        assert_eq!(config.get_parameter("max_iter"), Some("100".to_string()));
    }

    #[test]
    fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();
        registry.register(MeanBasedClassifier).unwrap();
        registry.register(StdDevCalculator).unwrap();

        assert_eq!(registry.count(), 2);
        assert!(registry.get("mean_classifier").is_some());
        assert!(registry.get("stddev_calculator").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_mean_classifier() {
        let classifier = MeanBasedClassifier;
        let data = vec![vec![0.2, 0.3], vec![0.8, 0.9], vec![0.5, 0.6]];

        assert!(classifier.validate(&data).unwrap());

        let config = PluginConfig::new().set_parameter("threshold".to_string(), "0.5".to_string());
        let result = classifier.execute(&data, &config).unwrap();

        assert!(result.success);
        assert!(result.outputs.contains_key("classifications"));
    }

    #[test]
    fn test_stddev_calculator() {
        let calculator = StdDevCalculator;
        let data = vec![vec![1.0, 2.0], vec![2.0, 3.0], vec![3.0, 4.0]];

        assert!(calculator.validate(&data).unwrap());

        let result = calculator.execute(&data, &PluginConfig::new()).unwrap();

        assert!(result.success);
        assert!(result.outputs.contains_key("stddev"));
    }

    #[test]
    fn test_plugin_registry_execute() {
        let mut registry = PluginRegistry::new();
        registry.register(MeanBasedClassifier).unwrap();

        let data = vec![vec![0.2, 0.3], vec![0.8, 0.9]];
        let config = PluginConfig::new();

        let result = registry.execute("mean_classifier", &data, &config).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_plugin_not_found() {
        let registry = PluginRegistry::new();
        let data = vec![vec![0.2, 0.3]];
        let config = PluginConfig::new();

        let result = registry.execute("nonexistent", &data, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_algorithm_type_string() {
        assert_eq!(AlgorithmType::Clustering.as_str(), "clustering");
        assert_eq!(AlgorithmType::Custom.as_str(), "custom");
    }

    #[test]
    fn test_plugin_result() {
        let result = PluginResult::new("test".to_string())
            .with_output("output".to_string(), vec![1.0, 2.0])
            .with_metadata("key".to_string(), "value".to_string())
            .with_timing(100);

        assert_eq!(result.plugin_name, "test");
        assert!(result.success);
        assert_eq!(result.execution_time_ms, 100);
        assert!(result.outputs.contains_key("output"));
        assert!(result.metadata.contains_key("key"));
    }

    #[test]
    fn test_plugin_list() {
        let mut registry = PluginRegistry::new();
        registry.register(MeanBasedClassifier).unwrap();
        registry.register(StdDevCalculator).unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"mean_classifier".to_string()));
    }

    #[test]
    fn test_empty_data_validation() {
        let classifier = MeanBasedClassifier;
        assert!(!classifier.validate(&[]).unwrap());
        assert!(!classifier.validate(&[vec![]]).unwrap());
    }
}
