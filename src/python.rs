//! Python bindings using PyO3

use pyo3::prelude::*;
use crate::engine::sql_export::{SQLExporter, SQLDialect, ColumnMapping};
use crate::engine::{SegmenterConfig, AudienceSegmenterCore, NormalizationParams};
use crate::engine::rfm::{RFMConfig, DecayFunction, ScoringMethod, RFMScore, Transaction, calculate_rfm};
use crate::engine::clustering::{KMeansResult, kmeans, ClusteringMethod};
use crate::engine::churn_prediction::{ChurnRiskLevel, ChurnPrediction, ChurnModelType};
use crate::engine::clv::{CLVModel, CustomerLTV, CLVCalculator};
use crate::engine::segments::{SegmentType, SegmentProfile};
use std::collections::HashMap;
use ndarray::Array2;

// ============================================================================
// RFM CLASSES & FUNCTIONS
// ============================================================================

#[pyclass]
struct PyDecayFunction {
    inner: DecayFunction,
}

#[pymethods]
impl PyDecayFunction {
    #[staticmethod]
    fn linear() -> Self {
        PyDecayFunction {
            inner: DecayFunction::Linear,
        }
    }

    #[staticmethod]
    fn exponential() -> Self {
        PyDecayFunction {
            inner: DecayFunction::Exponential,
        }
    }

    #[staticmethod]
    fn inverse() -> Self {
        PyDecayFunction {
            inner: DecayFunction::Inverse,
        }
    }

    fn __repr__(&self) -> String {
        format!("DecayFunction({})", self.inner)
    }
}

#[pyclass]
struct PyScoringMethod {
    inner: ScoringMethod,
}

#[pymethods]
impl PyScoringMethod {
    #[staticmethod]
    fn quintile() -> Self {
        PyScoringMethod {
            inner: ScoringMethod::Quintile,
        }
    }

    #[staticmethod]
    fn decile() -> Self {
        PyScoringMethod {
            inner: ScoringMethod::Decile,
        }
    }

    #[staticmethod]
    fn percentile() -> Self {
        PyScoringMethod {
            inner: ScoringMethod::Percentile,
        }
    }
}

#[pyclass]
struct PyRFMConfig {
    inner: RFMConfig,
}

#[pymethods]
impl PyRFMConfig {
    #[new]
    fn new(
        recency_window_days: Option<u32>,
        frequency_threshold: Option<usize>,
        monetary_threshold: Option<f64>,
        decay_function: Option<&PyDecayFunction>,
        decay_half_life_days: Option<u32>,
        scoring_method: Option<&PyScoringMethod>,
    ) -> Self {
        let mut config = RFMConfig::default();
        if let Some(days) = recency_window_days {
            config.recency_window_days = days;
        }
        if let Some(threshold) = frequency_threshold {
            config.frequency_threshold = threshold;
        }
        if let Some(threshold) = monetary_threshold {
            config.monetary_threshold = threshold;
        }
        if let Some(decay) = decay_function {
            config.decay_function = decay.inner;
        }
        if let Some(half_life) = decay_half_life_days {
            config.decay_half_life_days = half_life;
        }
        if let Some(method) = scoring_method {
            config.scoring_method = method.inner;
        }
        PyRFMConfig { inner: config }
    }

    #[getter]
    fn recency_window_days(&self) -> u32 {
        self.inner.recency_window_days
    }

    #[getter]
    fn frequency_threshold(&self) -> usize {
        self.inner.frequency_threshold
    }

    #[getter]
    fn monetary_threshold(&self) -> f64 {
        self.inner.monetary_threshold
    }

    #[getter]
    fn decay_half_life_days(&self) -> u32 {
        self.inner.decay_half_life_days
    }
}

#[pyclass]
struct PyRFMScore {
    inner: RFMScore,
}

#[pymethods]
impl PyRFMScore {
    #[getter]
    fn customer_id(&self) -> String {
        self.inner.customer_id.clone()
    }

    #[getter]
    fn recency(&self) -> f64 {
        self.inner.recency
    }

    #[getter]
    fn frequency(&self) -> f64 {
        self.inner.frequency
    }

    #[getter]
    fn monetary(&self) -> f64 {
        self.inner.monetary
    }

    #[getter]
    fn recency_score(&self) -> u32 {
        self.inner.recency_score
    }

    #[getter]
    fn frequency_score(&self) -> u32 {
        self.inner.frequency_score
    }

    #[getter]
    fn monetary_score(&self) -> u32 {
        self.inner.monetary_score
    }

    #[getter]
    fn rfm_segment(&self) -> String {
        self.inner.rfm_segment.clone()
    }

    #[getter]
    fn rfm_rank(&self) -> String {
        self.inner.rfm_rank()
    }

    fn __repr__(&self) -> String {
        format!(
            "RFMScore(customer_id={}, r={}, f={}, m={}, segment={})",
            self.inner.customer_id, self.inner.recency_score,
            self.inner.frequency_score, self.inner.monetary_score,
            self.inner.rfm_segment
        )
    }
}

#[pyfunction]
fn calculate_rfm_py(
    transactions: Vec<(String, String, f64)>,
    config: &PyRFMConfig,
) -> PyResult<Vec<PyRFMScore>> {
    let tx_vec: Vec<Transaction> = transactions
        .into_iter()
        .map(|(customer_id, date, amount)| Transaction {
            customer_id,
            date,
            amount,
        })
        .collect();

    calculate_rfm(tx_vec, &config.inner)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        .map(|scores| {
            scores
                .into_iter()
                .map(|s| PyRFMScore { inner: s })
                .collect()
        })
}

// ============================================================================
// CLUSTERING CLASSES & FUNCTIONS
// ============================================================================

#[pyclass]
struct PyKMeansResult {
    inner: KMeansResult,
}

#[pymethods]
impl PyKMeansResult {
    #[getter]
    fn labels(&self) -> Vec<usize> {
        self.inner.labels.clone()
    }

    #[getter]
    fn inertia(&self) -> f64 {
        self.inner.inertia
    }

    #[getter]
    fn n_iter(&self) -> usize {
        self.inner.n_iter
    }

    #[getter]
    fn centers(&self) -> Vec<Vec<f64>> {
        self.inner
            .centers
            .outer_iter()
            .map(|row| row.to_vec())
            .collect()
    }
}

#[pyfunction]
fn kmeans_py(
    data: Vec<Vec<f64>>,
    n_clusters: usize,
    max_iter: Option<usize>,
    random_state: Option<u64>,
) -> PyResult<PyKMeansResult> {
    let max_iter = max_iter.unwrap_or(300);
    let random_state = random_state.unwrap_or(42);

    let data_array = Array2::from_shape_vec(
        (data.len(), if data.is_empty() { 0 } else { data[0].len() }),
        data.into_iter().flatten().collect(),
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    kmeans(&data_array, n_clusters, max_iter, random_state)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        .map(|result| PyKMeansResult { inner: result })
}

// ============================================================================
// SEGMENTER CORE CLASS
// ============================================================================

#[pyclass]
struct PyAudienceSegmenter {
    inner: AudienceSegmenterCore,
}

#[pymethods]
impl PyAudienceSegmenter {
    #[new]
    fn new(n_clusters: usize) -> Self {
        let config = SegmenterConfig {
            method: "kmeans".to_string(),
            n_clusters,
            rfm_config: RFMConfig::default(),
            clustering_method: ClusteringMethod::KMeans,
            random_state: 42,
            n_jobs: -1,
        };
        PyAudienceSegmenter {
            inner: AudienceSegmenterCore::new(config),
        }
    }

    fn fit(&mut self, data: Vec<Vec<f64>>) -> PyResult<()> {
        let data_array = Array2::from_shape_vec(
            (data.len(), if data.is_empty() { 0 } else { data[0].len() }),
            data.into_iter().flatten().collect(),
        )
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        self.inner
            .fit(&data_array)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn predict(&self, data: Vec<Vec<f64>>) -> PyResult<Vec<usize>> {
        let data_array = Array2::from_shape_vec(
            (data.len(), if data.is_empty() { 0 } else { data[0].len() }),
            data.into_iter().flatten().collect(),
        )
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        self.inner
            .predict(&data_array)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn get_n_clusters(&self) -> usize {
        self.inner.config.n_clusters
    }

    fn __repr__(&self) -> String {
        format!(
            "AudienceSegmenter(n_clusters={}, method={})",
            self.inner.config.n_clusters, self.inner.config.method
        )
    }
}

// ============================================================================
// CHURN PREDICTION CLASSES & FUNCTIONS
// ============================================================================

#[pyclass]
struct PyChurnRiskLevel {
    inner: ChurnRiskLevel,
}

#[pymethods]
impl PyChurnRiskLevel {
    #[staticmethod]
    fn very_low() -> Self {
        PyChurnRiskLevel {
            inner: ChurnRiskLevel::VeryLow,
        }
    }

    #[staticmethod]
    fn low() -> Self {
        PyChurnRiskLevel {
            inner: ChurnRiskLevel::Low,
        }
    }

    #[staticmethod]
    fn medium() -> Self {
        PyChurnRiskLevel {
            inner: ChurnRiskLevel::Medium,
        }
    }

    #[staticmethod]
    fn high() -> Self {
        PyChurnRiskLevel {
            inner: ChurnRiskLevel::High,
        }
    }

    #[staticmethod]
    fn critical() -> Self {
        PyChurnRiskLevel {
            inner: ChurnRiskLevel::Critical,
        }
    }

    fn as_str(&self) -> String {
        self.inner.as_str().to_string()
    }

    fn threshold(&self) -> f64 {
        self.inner.threshold()
    }
}

#[pyclass]
struct PyChurnPrediction {
    inner: ChurnPrediction,
}

#[pymethods]
impl PyChurnPrediction {
    #[getter]
    fn customer_id(&self) -> String {
        self.inner.customer_id.clone()
    }

    #[getter]
    fn churn_probability(&self) -> f64 {
        self.inner.churn_probability
    }

    #[getter]
    fn risk_level(&self) -> String {
        self.inner.risk_level.as_str().to_string()
    }

    #[getter]
    fn confidence(&self) -> f64 {
        self.inner.confidence
    }

    #[getter]
    fn days_until_churn_estimate(&self) -> i32 {
        self.inner.days_until_churn_estimate
    }

    #[getter]
    fn model_type(&self) -> String {
        self.inner.model_type.as_str().to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "ChurnPrediction(customer_id={}, prob={:.4}, risk={}, confidence={:.4})",
            self.inner.customer_id,
            self.inner.churn_probability,
            self.inner.risk_level.as_str(),
            self.inner.confidence
        )
    }
}

// ============================================================================
// CLV CLASSES & FUNCTIONS
// ============================================================================

#[pyclass]
struct PyCustomerLTV {
    inner: CustomerLTV,
}

#[pymethods]
impl PyCustomerLTV {
    #[getter]
    fn customer_id(&self) -> String {
        self.inner.customer_id.clone()
    }

    #[getter]
    fn historical_value(&self) -> f64 {
        self.inner.historical_value
    }

    #[getter]
    fn annual_value(&self) -> f64 {
        self.inner.annual_value
    }

    #[getter]
    fn predicted_ltv(&self) -> f64 {
        self.inner.predicted_ltv
    }

    #[getter]
    fn predicted_ltv_3yr(&self) -> f64 {
        self.inner.predicted_ltv_3yr
    }

    #[getter]
    fn predicted_ltv_5yr(&self) -> f64 {
        self.inner.predicted_ltv_5yr
    }

    #[getter]
    fn churn_probability(&self) -> f64 {
        self.inner.churn_probability
    }

    #[getter]
    fn confidence_score(&self) -> f64 {
        self.inner.confidence_score
    }

    fn __repr__(&self) -> String {
        format!(
            "CustomerLTV(customer_id={}, ltv={:.2}, 3yr={:.2}, 5yr={:.2})",
            self.inner.customer_id,
            self.inner.predicted_ltv,
            self.inner.predicted_ltv_3yr,
            self.inner.predicted_ltv_5yr
        )
    }
}

#[pyfunction]
fn calculate_simple_ltv(
    customer_id: String,
    total_spent: f64,
    purchase_count: usize,
    days_active: i32,
    avg_customer_lifespan_days: i32,
) -> PyResult<PyCustomerLTV> {
    CLVCalculator::calculate_simple_ltv(&customer_id, total_spent, purchase_count, days_active, avg_customer_lifespan_days)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        .map(|ltv| PyCustomerLTV { inner: ltv })
}

// ============================================================================
// SEGMENT CLASSES & FUNCTIONS
// ============================================================================

#[pyclass]
struct PySegmentType {
    inner: SegmentType,
}

#[pymethods]
impl PySegmentType {
    #[staticmethod]
    fn champions() -> Self {
        PySegmentType {
            inner: SegmentType::Champions,
        }
    }

    #[staticmethod]
    fn loyal_customers() -> Self {
        PySegmentType {
            inner: SegmentType::LoyalCustomers,
        }
    }

    #[staticmethod]
    fn potential_loyalists() -> Self {
        PySegmentType {
            inner: SegmentType::PotentialLoyalists,
        }
    }

    #[staticmethod]
    fn at_risk() -> Self {
        PySegmentType {
            inner: SegmentType::AtRisk,
        }
    }

    #[staticmethod]
    fn cannot_lose() -> Self {
        PySegmentType {
            inner: SegmentType::CannotLose,
        }
    }

    #[staticmethod]
    fn vip() -> Self {
        PySegmentType {
            inner: SegmentType::VIP,
        }
    }

    #[staticmethod]
    fn new_customers() -> Self {
        PySegmentType {
            inner: SegmentType::NewCustomers,
        }
    }

    #[staticmethod]
    fn need_attention() -> Self {
        PySegmentType {
            inner: SegmentType::NeedAttention,
        }
    }

    fn as_str(&self) -> String {
        self.inner.as_str().to_string()
    }

    fn __repr__(&self) -> String {
        format!("SegmentType({})", self.inner.as_str())
    }
}

#[pyclass]
struct PySegmentProfile {
    inner: SegmentProfile,
}

#[pymethods]
impl PySegmentProfile {
    #[new]
    fn new(segment_type: &PySegmentType) -> Self {
        PySegmentProfile {
            inner: SegmentProfile::new(segment_type.inner.clone()),
        }
    }

    #[getter]
    fn segment_type(&self) -> String {
        self.inner.segment_type.as_str().to_string()
    }

    #[getter]
    fn description(&self) -> String {
        self.inner.description.clone()
    }

    #[getter]
    fn size(&self) -> usize {
        self.inner.size
    }

    #[getter]
    fn avg_monetary(&self) -> f64 {
        self.inner.avg_monetary
    }

    #[getter]
    fn avg_frequency(&self) -> f64 {
        self.inner.avg_frequency
    }

    #[getter]
    fn avg_recency(&self) -> f64 {
        self.inner.avg_recency
    }

    #[getter]
    fn churn_risk(&self) -> f64 {
        self.inner.churn_risk
    }

    #[getter]
    fn revenue_contribution(&self) -> f64 {
        self.inner.revenue_contribution
    }

    fn __repr__(&self) -> String {
        format!(
            "SegmentProfile(type={}, size={}, avg_monetary={:.2})",
            self.inner.segment_type.as_str(),
            self.inner.size,
            self.inner.avg_monetary
        )
    }
}

// ============================================================================
// SQL EXPORT FUNCTIONS (EXISTING)
// ============================================================================

/// Export a single segment as SQL query
#[pyfunction]
fn export_segment_sql(
    segment_name: &str,
    dialect: &str,
    table_name: &str,
    customer_id: Option<String>,
    recency_score: Option<String>,
    frequency_score: Option<String>,
    monetary_score: Option<String>,
) -> PyResult<String> {
    let sql_dialect = SQLDialect::from_str(dialect)
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Unsupported SQL dialect: {}", dialect)
        ))?;

    let mapping = ColumnMapping {
        customer_id: customer_id.unwrap_or_else(|| "customer_id".to_string()),
        recency_score: recency_score.unwrap_or_else(|| "recency_score".to_string()),
        frequency_score: frequency_score.unwrap_or_else(|| "frequency_score".to_string()),
        monetary_score: monetary_score.unwrap_or_else(|| "monetary_score".to_string()),
    };

    SQLExporter::export_segment(segment_name, sql_dialect, table_name, &mapping)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Export all segments as SQL queries
#[pyfunction]
fn export_all_segments_sql(
    dialect: &str,
    table_name: &str,
    customer_id: Option<String>,
    recency_score: Option<String>,
    frequency_score: Option<String>,
    monetary_score: Option<String>,
) -> PyResult<HashMap<String, String>> {
    let sql_dialect = SQLDialect::from_str(dialect)
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Unsupported SQL dialect: {}", dialect)
        ))?;

    let mapping = ColumnMapping {
        customer_id: customer_id.unwrap_or_else(|| "customer_id".to_string()),
        recency_score: recency_score.unwrap_or_else(|| "recency_score".to_string()),
        frequency_score: frequency_score.unwrap_or_else(|| "frequency_score".to_string()),
        monetary_score: monetary_score.unwrap_or_else(|| "monetary_score".to_string()),
    };

    SQLExporter::export_all_segments(sql_dialect, table_name, &mapping)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Get list of supported SQL dialects
#[pyfunction]
fn get_supported_sql_dialects() -> Vec<String> {
    SQLExporter::supported_dialects()
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Get RFM patterns for a segment
#[pyfunction]
fn get_segment_rfm_patterns(segment_name: &str) -> PyResult<Vec<(u32, u32, u32)>> {
    SQLExporter::get_segment_patterns(segment_name)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// ============================================================================
// PYTHON MODULE INITIALIZATION
// ============================================================================

/// Python module initialization
#[pymodule]
fn clusteraudiencekit(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add("__version__", crate::VERSION)?;
    m.add("__author__", "Georgi Mammen Mullassery")?;

    // Add core classes
    m.add_class::<PyAudienceSegmenter>()?;
    m.add_class::<PyRFMConfig>()?;
    m.add_class::<PyRFMScore>()?;
    m.add_class::<PyDecayFunction>()?;
    m.add_class::<PyScoringMethod>()?;
    m.add_class::<PyKMeansResult>()?;
    m.add_class::<PyChurnRiskLevel>()?;
    m.add_class::<PyChurnPrediction>()?;
    m.add_class::<PyCustomerLTV>()?;
    m.add_class::<PySegmentType>()?;
    m.add_class::<PySegmentProfile>()?;

    // Add RFM functions
    m.add_function(wrap_pyfunction!(calculate_rfm_py, m)?)?;

    // Add clustering functions
    m.add_function(wrap_pyfunction!(kmeans_py, m)?)?;

    // Add churn & CLV functions
    m.add_function(wrap_pyfunction!(calculate_simple_ltv, m)?)?;

    // Add SQL export functions
    m.add_function(wrap_pyfunction!(export_segment_sql, m)?)?;
    m.add_function(wrap_pyfunction!(export_all_segments_sql, m)?)?;
    m.add_function(wrap_pyfunction!(get_supported_sql_dialects, m)?)?;
    m.add_function(wrap_pyfunction!(get_segment_rfm_patterns, m)?)?;

    // Add module info
    let info = PyModule::new_bound(py, "info")?;
    info.add("algorithms", vec!["kmeans", "dbscan", "hierarchical", "gmm"])?;
    info.add("metrics", vec!["silhouette", "davies_bouldin", "calinski_harabasz"])?;
    info.add("segments", 13usize)?;
    m.add_submodule(&info)?;

    Ok(())
}
