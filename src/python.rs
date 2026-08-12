//! Python bindings using PyO3
//!
//! NOTE on `#![allow(clippy::useless_conversion)]` below: pyo3's
//! `#[pyfunction]`/`#[pymethods]`/`#[new]` proc-macros generate wrapper code
//! (to adapt a function returning `PyResult<T>` into the FFI-facing
//! signature) that itself triggers `clippy::useless_conversion` on every
//! single pyo3-exposed function in this file, including ones nobody hand-
//! wrote a conversion in. It's a false positive against macro-generated
//! code, not a real issue in our code — allowing it here (rather than
//! peppering every function with its own `#[allow(...)]`) keeps this file's
//! actual lint signal visible.
#![allow(clippy::useless_conversion)]

use crate::engine::churn_prediction::{ChurnPrediction, ChurnRiskLevel};
use crate::engine::clustering::{kmeans, ClusteringMethod, KMeansResult};
use crate::engine::clv::{CLVCalculator, CustomerLTV};
use crate::engine::rfm::{
    calculate_rfm, DecayFunction, RFMConfig, RFMScore, ScoringMethod, Transaction,
};
use crate::engine::segments::{SegmentProfile, SegmentType};
use crate::engine::sql_export::{ColumnMapping, SQLDialect, SQLExporter};
use crate::engine::{AudienceSegmenterCore, SegmenterConfig};
use ndarray::Array2;
use pyo3::prelude::*;
use std::collections::HashMap;

// Newly wired-up modules (see the corresponding sections below):
use crate::engine::behavioral::{
    BehavioralRule, BehavioralSegment, BehavioralSegmenter, ComparisonOp, Condition, LogicalOp,
    RuleValue,
};
use crate::engine::cohorts::{Cohort, CohortAnalytics, CohortId, CohortPeriod};
use crate::engine::drift_detection::{DriftDetector, DriftMethod};
use crate::engine::k_estimation::{
    CombinedKEstimation, ElbowMethod, GapStatistic, SilhouetteEstimation,
};
use crate::engine::lifecycle::{LifecycleStage, LifecycleTracker};
use crate::engine::lookalike::{LookalikeGenerator, SeedCustomer, SimilarityMetric};
use crate::engine::privacy::{DifferentialPrivacy, KAnonymity, PrivacyBudget};
use crate::engine::profiling::ProfilingEngine;
use crate::engine::quality_metrics::{
    CalinskiHarabaszMetric, DaviesBouldinMetric, QualityAssessment, SilhouetteMetric,
};
use crate::engine::streaming::{
    StreamEventType, StreamingConfig, StreamingEvent, StreamingSegmentationEngine, StreamingWindow,
};

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
    #[pyo3(signature = (recency_window_days=None, frequency_threshold=None, monetary_threshold=None, decay_function=None, decay_half_life_days=None, scoring_method=None))]
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
            self.inner.customer_id,
            self.inner.recency_score,
            self.inner.frequency_score,
            self.inner.monetary_score,
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
#[pyo3(signature = (data, n_clusters, max_iter=None, random_state=None))]
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
    CLVCalculator::calculate_simple_ltv(
        &customer_id,
        total_spent,
        purchase_count,
        days_active,
        avg_customer_lifespan_days,
    )
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
#[pyo3(signature = (segment_name, dialect, table_name, customer_id=None, recency_score=None, frequency_score=None, monetary_score=None))]
fn export_segment_sql(
    segment_name: &str,
    dialect: &str,
    table_name: &str,
    customer_id: Option<String>,
    recency_score: Option<String>,
    frequency_score: Option<String>,
    monetary_score: Option<String>,
) -> PyResult<String> {
    let sql_dialect = SQLDialect::from_str(dialect).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unsupported SQL dialect: {}",
            dialect
        ))
    })?;

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
#[pyo3(signature = (dialect, table_name, customer_id=None, recency_score=None, frequency_score=None, monetary_score=None))]
fn export_all_segments_sql(
    dialect: &str,
    table_name: &str,
    customer_id: Option<String>,
    recency_score: Option<String>,
    frequency_score: Option<String>,
    monetary_score: Option<String>,
) -> PyResult<HashMap<String, String>> {
    let sql_dialect = SQLDialect::from_str(dialect).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unsupported SQL dialect: {}",
            dialect
        ))
    })?;

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
// PRIVACY: DIFFERENTIAL PRIVACY & K-ANONYMITY
// ============================================================================

#[pyclass]
struct PyPrivacyBudget {
    inner: PrivacyBudget,
}

#[pymethods]
impl PyPrivacyBudget {
    #[new]
    fn new(epsilon: f64, delta: f64) -> Self {
        PyPrivacyBudget {
            inner: PrivacyBudget::new(epsilon, delta),
        }
    }

    #[getter]
    fn epsilon(&self) -> f64 {
        self.inner.epsilon
    }

    #[getter]
    fn delta(&self) -> f64 {
        self.inner.delta
    }

    #[getter]
    fn remaining_epsilon(&self) -> f64 {
        self.inner.remaining_epsilon
    }

    /// Consume `cost` epsilon from the budget. Returns False (and leaves the
    /// budget untouched) if the remaining budget is insufficient.
    fn consume(&mut self, cost: f64) -> bool {
        self.inner.consume(cost)
    }

    fn budget_exhausted(&self) -> bool {
        self.inner.budget_exhausted()
    }

    fn budget_percentage(&self) -> f64 {
        self.inner.budget_percentage()
    }

    fn __repr__(&self) -> String {
        format!(
            "PrivacyBudget(epsilon={}, remaining={:.4})",
            self.inner.epsilon, self.inner.remaining_epsilon
        )
    }
}

/// Add Laplace-mechanism noise to a vector of counts/sums for
/// epsilon-differential privacy.
#[pyfunction]
fn add_laplace_noise(data: Vec<f64>, epsilon: f64, sensitivity: f64) -> PyResult<Vec<f64>> {
    DifferentialPrivacy::add_laplace_noise(&data, epsilon, sensitivity)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Add Gaussian-mechanism noise to a vector for (epsilon, delta)-differential
/// privacy on range queries.
#[pyfunction]
fn add_gaussian_noise(
    data: Vec<f64>,
    epsilon: f64,
    delta: f64,
    sensitivity: f64,
) -> PyResult<Vec<f64>> {
    DifferentialPrivacy::add_gaussian_noise(&data, epsilon, delta, sensitivity)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

#[pyclass]
struct PyKAnonymityResult {
    #[pyo3(get)]
    k_value: usize,
    #[pyo3(get)]
    anonymized: bool,
    #[pyo3(get)]
    generalized_rows: usize,
    #[pyo3(get)]
    suppressed_rows: usize,
    #[pyo3(get)]
    information_loss: f64,
}

#[pymethods]
impl PyKAnonymityResult {
    fn __repr__(&self) -> String {
        format!(
            "KAnonymityResult(k={}, anonymized={}, suppressed_rows={})",
            self.k_value, self.anonymized, self.suppressed_rows
        )
    }
}

/// Check whether `data` (a list of row dicts) is k-anonymous with respect to
/// `quasi_identifiers` (the column names that, combined, could re-identify a
/// row) for the given `k`.
#[pyfunction]
fn check_k_anonymity(
    data: Vec<HashMap<String, String>>,
    quasi_identifiers: Vec<String>,
    k: usize,
) -> PyResult<PyKAnonymityResult> {
    let result = KAnonymity::check_k_anonymity(&data, &quasi_identifiers, k)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyKAnonymityResult {
        k_value: result.k_value,
        anonymized: result.anonymized,
        generalized_rows: result.generalized_rows,
        suppressed_rows: result.suppressed_rows,
        information_loss: result.information_loss,
    })
}

/// Drop rows from `data` that fall in quasi-identifier groups smaller than
/// `k`, returning only the rows that are already k-anonymous.
#[pyfunction]
fn suppress_to_k_anonymous(
    data: Vec<HashMap<String, String>>,
    quasi_identifiers: Vec<String>,
    k: usize,
) -> PyResult<Vec<HashMap<String, String>>> {
    KAnonymity::suppress_to_k_anonymous(&data, &quasi_identifiers, k)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Bucket numeric values into `intervals` equal-width bins (returns each
/// value's bin index) — a generalization step for k-anonymity.
#[pyfunction]
fn generalize_numeric(data: Vec<f64>, intervals: usize) -> PyResult<Vec<usize>> {
    KAnonymity::generalize_numeric(&data, intervals)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Root-mean-square normalized distance between original values and the
/// midpoints of the bins they were generalized into.
#[pyfunction]
fn calculate_information_loss(
    original: Vec<f64>,
    generalized: Vec<usize>,
    intervals: usize,
) -> f64 {
    KAnonymity::calculate_information_loss(&original, &generalized, intervals)
}

// ============================================================================
// STREAMING: REAL-TIME SEGMENTATION
// ============================================================================

fn parse_stream_event_type(s: &str) -> PyResult<StreamEventType> {
    match s.to_lowercase().as_str() {
        "purchase" => Ok(StreamEventType::Purchase),
        "engagement" => Ok(StreamEventType::Engagement),
        "pageview" | "page_view" => Ok(StreamEventType::PageView),
        "custom" => Ok(StreamEventType::Custom),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown event type '{}'. Expected one of: purchase, engagement, pageview, custom",
            other
        ))),
    }
}

fn parse_streaming_window(s: &str) -> PyResult<StreamingWindow> {
    match s.to_lowercase().as_str() {
        "minute" => Ok(StreamingWindow::Minute),
        "hour" => Ok(StreamingWindow::Hour),
        "day" => Ok(StreamingWindow::Day),
        "week" => Ok(StreamingWindow::Week),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown window '{}'. Expected one of: minute, hour, day, week",
            other
        ))),
    }
}

#[pyclass]
#[derive(Clone)]
struct PyStreamingEvent {
    inner: StreamingEvent,
}

#[pymethods]
impl PyStreamingEvent {
    #[new]
    #[pyo3(signature = (customer_id, event_type, value, timestamp, metadata=None))]
    fn new(
        customer_id: String,
        event_type: &str,
        value: f64,
        timestamp: i64,
        metadata: Option<HashMap<String, String>>,
    ) -> PyResult<Self> {
        let mut event = StreamingEvent::new(
            customer_id,
            parse_stream_event_type(event_type)?,
            value,
            timestamp,
        );
        if let Some(meta) = metadata {
            for (k, v) in meta {
                event = event.with_metadata(k, v);
            }
        }
        Ok(PyStreamingEvent { inner: event })
    }

    #[getter]
    fn customer_id(&self) -> String {
        self.inner.customer_id.clone()
    }

    #[getter]
    fn value(&self) -> f64 {
        self.inner.value
    }

    #[getter]
    fn timestamp(&self) -> i64 {
        self.inner.timestamp
    }
}

#[pyclass]
struct PyStreamingConfig {
    inner: StreamingConfig,
}

#[pymethods]
impl PyStreamingConfig {
    #[new]
    #[pyo3(signature = (batch_size=100, batch_timeout_ms=5000, window="hour", decay_factor=0.95))]
    fn new(
        batch_size: usize,
        batch_timeout_ms: u64,
        window: &str,
        decay_factor: f64,
    ) -> PyResult<Self> {
        Ok(PyStreamingConfig {
            inner: StreamingConfig {
                batch_size,
                batch_timeout_ms,
                window: parse_streaming_window(window)?,
                decay_factor,
            },
        })
    }
}

#[pyclass]
struct PyStreamingSegmentUpdate {
    #[pyo3(get)]
    customer_id: String,
    #[pyo3(get)]
    previous_segment: Option<String>,
    #[pyo3(get)]
    new_segment: Option<String>,
    #[pyo3(get)]
    segment_changed: bool,
    #[pyo3(get)]
    confidence: f64,
    #[pyo3(get)]
    timestamp: i64,
}

/// Stateful real-time segmentation engine: feed it events one at a time (or
/// in batches) and it maintains an incrementally-updated RFM-like state and
/// segment assignment per customer, without requiring a full batch refit.
#[pyclass]
struct PyStreamingSegmentationEngine {
    inner: StreamingSegmentationEngine,
}

#[pymethods]
impl PyStreamingSegmentationEngine {
    #[new]
    fn new(config: &PyStreamingConfig) -> Self {
        PyStreamingSegmentationEngine {
            inner: StreamingSegmentationEngine::new(config.inner.clone()),
        }
    }

    fn process_event(
        &mut self,
        event: &PyStreamingEvent,
    ) -> PyResult<Option<PyStreamingSegmentUpdate>> {
        let update = self
            .inner
            .process_event(event.inner.clone())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(update.map(|u| PyStreamingSegmentUpdate {
            customer_id: u.customer_id,
            previous_segment: u.previous_segment,
            new_segment: u.new_segment,
            segment_changed: u.segment_changed,
            confidence: u.confidence,
            timestamp: u.timestamp,
        }))
    }

    fn process_batch(
        &mut self,
        events: Vec<PyStreamingEvent>,
    ) -> PyResult<Vec<PyStreamingSegmentUpdate>> {
        let raw_events: Vec<StreamingEvent> = events.into_iter().map(|e| e.inner).collect();
        let updates = self
            .inner
            .process_batch(raw_events)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(updates
            .into_iter()
            .map(|u| PyStreamingSegmentUpdate {
                customer_id: u.customer_id,
                previous_segment: u.previous_segment,
                new_segment: u.new_segment,
                segment_changed: u.segment_changed,
                confidence: u.confidence,
                timestamp: u.timestamp,
            })
            .collect())
    }

    fn get_segment(&self, customer_id: &str) -> Option<String> {
        self.inner.get_segment(customer_id)
    }

    fn get_all_segments(&self) -> HashMap<String, String> {
        self.inner.get_all_segments()
    }

    fn customer_count(&self) -> usize {
        self.inner.customer_count()
    }

    fn buffer_size(&self) -> usize {
        self.inner.buffer_size()
    }

    fn segment_distribution(&self) -> PyResult<HashMap<String, usize>> {
        self.inner
            .segment_distribution()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

// ============================================================================
// DRIFT DETECTION: SEGMENT & FEATURE DRIFT MONITORING
// ============================================================================

fn parse_drift_method(s: &str) -> PyResult<DriftMethod> {
    match s.to_lowercase().as_str() {
        "ks" | "kolmogorov_smirnov" => Ok(DriftMethod::KolmogorovSmirnov),
        "kl" | "kullback_leibler" => Ok(DriftMethod::KullbackLeibler),
        "hellinger" | "hellinger_distance" => Ok(DriftMethod::HellingerDistance),
        "chi_square" | "chi2" => Ok(DriftMethod::ChiSquare),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown drift method '{}'. Expected one of: ks, kl, hellinger, chi_square",
            other
        ))),
    }
}

/// Kolmogorov-Smirnov statistic (max CDF gap) between two samples.
#[pyfunction]
fn kolmogorov_smirnov(baseline: Vec<f64>, current: Vec<f64>) -> PyResult<f64> {
    DriftDetector::kolmogorov_smirnov(&baseline, &current)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Hellinger distance between two samples' fitted distributions.
#[pyfunction]
fn hellinger_distance(baseline: Vec<f64>, current: Vec<f64>) -> PyResult<f64> {
    DriftDetector::hellinger_distance(&baseline, &current)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Chi-square statistic for drift between two categorical count distributions.
#[pyfunction]
fn chi_square_drift(
    baseline_counts: HashMap<String, usize>,
    current_counts: HashMap<String, usize>,
) -> PyResult<f64> {
    DriftDetector::chi_square(&baseline_counts, &current_counts)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

#[pyclass]
struct PyFeatureDrift {
    #[pyo3(get)]
    feature_name: String,
    #[pyo3(get)]
    drift_score: f64,
    #[pyo3(get)]
    severity: String,
    #[pyo3(get)]
    timestamp: i64,
}

#[pymethods]
impl PyFeatureDrift {
    fn __repr__(&self) -> String {
        format!(
            "FeatureDrift(feature={}, score={:.4}, severity={})",
            self.feature_name, self.drift_score, self.severity
        )
    }
}

/// Detect drift in a single numeric feature between a `baseline` and
/// `current` sample, classified into a severity level ("none"/"low"/
/// "medium"/"high"/"critical").
#[pyfunction]
fn detect_feature_drift(
    feature_name: String,
    baseline: Vec<f64>,
    current: Vec<f64>,
    method: &str,
) -> PyResult<PyFeatureDrift> {
    let method = parse_drift_method(method)?;
    let drift = DriftDetector::detect_feature_drift(feature_name, &baseline, &current, method)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyFeatureDrift {
        feature_name: drift.feature_name,
        drift_score: drift.drift_score,
        severity: drift.severity.as_str().to_string(),
        timestamp: drift.timestamp,
    })
}

#[pyclass]
struct PySegmentCompositionChange {
    #[pyo3(get)]
    segment_name: String,
    #[pyo3(get)]
    previous_size: usize,
    #[pyo3(get)]
    current_size: usize,
    #[pyo3(get)]
    size_change_percent: f64,
    #[pyo3(get)]
    churn_rate: f64,
    #[pyo3(get)]
    growth_rate: f64,
}

/// Compare segment membership counts between two points in time and report
/// size/churn/growth changes per segment.
#[pyfunction]
fn detect_segment_composition_change(
    previous_segments: HashMap<String, usize>,
    current_segments: HashMap<String, usize>,
) -> PyResult<Vec<PySegmentCompositionChange>> {
    let changes =
        DriftDetector::detect_segment_composition_change(&previous_segments, &current_segments)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(changes
        .into_iter()
        .map(|c| PySegmentCompositionChange {
            segment_name: c.segment_name,
            previous_size: c.previous_size,
            current_size: c.current_size,
            size_change_percent: c.size_change_percent,
            churn_rate: c.churn_rate,
            growth_rate: c.growth_rate,
        })
        .collect())
}

// ============================================================================
// LOOKALIKE AUDIENCE MODELING
// ============================================================================

fn parse_similarity_metric(s: &str) -> PyResult<SimilarityMetric> {
    match s.to_lowercase().as_str() {
        "cosine" => Ok(SimilarityMetric::Cosine),
        "euclidean" => Ok(SimilarityMetric::Euclidean),
        "manhattan" => Ok(SimilarityMetric::Manhattan),
        "jaccard" => Ok(SimilarityMetric::Jaccard),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown similarity metric '{}'. Expected one of: cosine, euclidean, manhattan, jaccard", other
        ))),
    }
}

#[pyclass]
#[derive(Clone)]
struct PySeedCustomer {
    inner: SeedCustomer,
}

#[pymethods]
impl PySeedCustomer {
    #[new]
    #[pyo3(signature = (customer_id, features, ltv=0.0, cohort=String::new(), categorical_features=None))]
    fn new(
        customer_id: String,
        features: Vec<f64>,
        ltv: f64,
        cohort: String,
        categorical_features: Option<HashMap<String, String>>,
    ) -> Self {
        PySeedCustomer {
            inner: SeedCustomer {
                customer_id,
                features,
                categorical_features: categorical_features.unwrap_or_default(),
                ltv,
                cohort,
            },
        }
    }

    #[getter]
    fn customer_id(&self) -> String {
        self.inner.customer_id.clone()
    }
}

#[pyclass]
struct PyLookalikeCandidate {
    #[pyo3(get)]
    customer_id: String,
    #[pyo3(get)]
    similarity_score: f64,
    #[pyo3(get)]
    percentile: f64,
}

#[pyclass]
struct PyLookalikeAudience {
    #[pyo3(get)]
    audience_name: String,
    #[pyo3(get)]
    seed_count: usize,
    #[pyo3(get)]
    lookalike_count: usize,
    #[pyo3(get)]
    min_similarity: f64,
    #[pyo3(get)]
    max_similarity: f64,
    #[pyo3(get)]
    avg_similarity: f64,
    #[pyo3(get)]
    predicted_ltv: f64,
}

#[pymethods]
impl PyLookalikeAudience {
    fn __repr__(&self) -> String {
        format!(
            "LookalikeAudience(name={}, seeds={}, lookalikes={}, avg_similarity={:.4})",
            self.audience_name, self.seed_count, self.lookalike_count, self.avg_similarity
        )
    }
}

/// Build a lookalike audience: score every candidate against the average
/// feature profile of the seed customers, keep the top `percentile_threshold`
/// fraction (e.g. 0.9 = top 10%), capped at `max_lookalikes`.
#[pyfunction]
#[pyo3(signature = (seed_customers, candidate_customers, metric="cosine", percentile_threshold=0.9, max_lookalikes=None))]
fn generate_lookalike(
    seed_customers: Vec<PySeedCustomer>,
    candidate_customers: Vec<PySeedCustomer>,
    metric: &str,
    percentile_threshold: f64,
    max_lookalikes: Option<usize>,
) -> PyResult<PyLookalikeAudience> {
    let metric = parse_similarity_metric(metric)?;
    let seeds: Vec<SeedCustomer> = seed_customers.into_iter().map(|s| s.inner).collect();
    let candidates: Vec<SeedCustomer> = candidate_customers.into_iter().map(|c| c.inner).collect();

    let audience = LookalikeGenerator::generate_lookalike(
        &seeds,
        &candidates,
        metric,
        percentile_threshold,
        max_lookalikes,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    Ok(PyLookalikeAudience {
        audience_name: audience.audience_name,
        seed_count: audience.seed_count,
        lookalike_count: audience.lookalike_count,
        min_similarity: audience.min_similarity,
        max_similarity: audience.max_similarity,
        avg_similarity: audience.avg_similarity,
        predicted_ltv: audience.predicted_ltv,
    })
}

/// Find the top `n` candidates most similar to a single seed customer.
#[pyfunction]
#[pyo3(signature = (seed, candidates, n, metric="cosine"))]
fn find_similar_customers(
    seed: &PySeedCustomer,
    candidates: Vec<PySeedCustomer>,
    n: usize,
    metric: &str,
) -> PyResult<Vec<PyLookalikeCandidate>> {
    let metric = parse_similarity_metric(metric)?;
    let candidates: Vec<SeedCustomer> = candidates.into_iter().map(|c| c.inner).collect();
    let results = LookalikeGenerator::find_similar_customers(&seed.inner, &candidates, n, metric)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(results
        .into_iter()
        .map(|c| PyLookalikeCandidate {
            customer_id: c.customer_id,
            similarity_score: c.similarity_score,
            percentile: c.percentile,
        })
        .collect())
}

/// Cosine similarity between two equal-length numeric feature vectors.
#[pyfunction]
fn cosine_similarity(vec_a: Vec<f64>, vec_b: Vec<f64>) -> f64 {
    LookalikeGenerator::cosine_similarity(&vec_a, &vec_b)
}

// ============================================================================
// COHORT ANALYTICS
// ============================================================================

fn parse_cohort_period(s: &str) -> PyResult<CohortPeriod> {
    match s.to_lowercase().as_str() {
        "weekly" => Ok(CohortPeriod::Weekly),
        "monthly" => Ok(CohortPeriod::Monthly),
        "quarterly" => Ok(CohortPeriod::Quarterly),
        "yearly" => Ok(CohortPeriod::Yearly),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown cohort period '{}'. Expected one of: weekly, monthly, quarterly, yearly",
            other
        ))),
    }
}

#[pyclass]
#[derive(Clone)]
struct PyCohort {
    inner: Cohort,
}

#[pymethods]
impl PyCohort {
    #[getter]
    fn cohort_id(&self) -> String {
        self.inner.cohort_id.0.clone()
    }

    #[getter]
    fn size(&self) -> usize {
        self.inner.size
    }

    #[getter]
    fn created_at(&self) -> i64 {
        self.inner.created_at
    }

    #[getter]
    fn revenue(&self) -> f64 {
        self.inner.revenue
    }

    #[getter]
    fn avg_ltv(&self) -> f64 {
        self.inner.avg_ltv
    }

    #[getter]
    fn churn_rate_total(&self) -> f64 {
        self.inner.churn_rate_total
    }

    #[getter]
    fn retention_rate_total(&self) -> f64 {
        self.inner.retention_rate_total
    }

    /// Retention curve as a list of (age_in_periods, retained_count,
    /// churn_rate, retention_rate) tuples.
    #[getter]
    fn retention_curve(&self) -> Vec<(usize, usize, f64, f64)> {
        self.inner
            .retention_curve
            .iter()
            .map(|p| {
                (
                    p.age_in_periods,
                    p.retained_count,
                    p.churn_rate,
                    p.retention_rate,
                )
            })
            .collect()
    }

    /// Record how many of this cohort's original customers were still
    /// active `age_in_periods` periods after cohort creation.
    fn add_retention_point(
        &mut self,
        age_in_periods: usize,
        retained_count: usize,
    ) -> PyResult<()> {
        CohortAnalytics::add_retention_point(&mut self.inner, age_in_periods, retained_count)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Average retention-rate change per period across the recorded curve.
    fn retention_decay_rate(&self) -> PyResult<f64> {
        CohortAnalytics::retention_decay_rate(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn revenue_per_retained(&self) -> PyResult<f64> {
        CohortAnalytics::revenue_per_retained(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn summary(&self) -> PyResult<HashMap<String, f64>> {
        CohortAnalytics::cohort_summary(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "Cohort(id={}, size={}, retention={:.2}%)",
            self.inner.cohort_id.0,
            self.inner.size,
            self.inner.retention_rate_total * 100.0
        )
    }
}

/// Derive a deterministic cohort id string from a period type and a Unix
/// timestamp (e.g. all "monthly" signups in the same month get the same id).
#[pyfunction]
fn cohort_id_for(period: &str, date_unix: i64) -> PyResult<String> {
    let period = parse_cohort_period(period)?;
    Ok(CohortId::new(period, date_unix).0)
}

/// Build a cohort from a list of (customer_id, ltv, is_retained) tuples.
#[pyfunction]
fn create_cohort(
    cohort_id: String,
    period: &str,
    created_at: i64,
    customers: Vec<(String, f64, bool)>,
) -> PyResult<PyCohort> {
    let period = parse_cohort_period(period)?;
    let cohort =
        CohortAnalytics::create_cohort(CohortId(cohort_id), period, created_at, &customers)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyCohort { inner: cohort })
}

/// Compare two cohorts; returns (better_performing_cohort_id, size_diff,
/// revenue_diff, avg_ltv_diff, retention_rate_diff).
#[pyfunction]
fn compare_cohorts(
    cohort_a: &PyCohort,
    cohort_b: &PyCohort,
) -> PyResult<(String, i32, f64, f64, f64)> {
    let cmp = CohortAnalytics::compare_cohorts(&cohort_a.inner, &cohort_b.inner)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok((
        cmp.better_performer.0,
        cmp.size_diff,
        cmp.revenue_diff,
        cmp.ltv_diff,
        cmp.retention_rate_diff,
    ))
}

/// Group cohorts by their id and compute per-group total size, total
/// revenue, average retention, and cohort count.
#[pyfunction]
fn aggregate_cohorts_by_period(
    cohorts: Vec<PyCohort>,
) -> PyResult<HashMap<String, HashMap<String, f64>>> {
    let raw: Vec<Cohort> = cohorts.into_iter().map(|c| c.inner).collect();
    CohortAnalytics::aggregate_by_period(&raw)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Build a cohort x period-age retention matrix (each row is one cohort's
/// retention rate at each recorded age).
#[pyfunction]
fn cohort_retention_table(cohorts: Vec<PyCohort>) -> PyResult<Vec<Vec<f64>>> {
    let raw: Vec<Cohort> = cohorts.into_iter().map(|c| c.inner).collect();
    CohortAnalytics::retention_table(&raw)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Identify the best- and worst-performing cohort ids by overall retention
/// rate; returns (best_id, worst_id), either of which may be None if
/// `cohorts` is empty.
#[pyfunction]
fn cohort_performance_ranking(
    cohorts: Vec<PyCohort>,
) -> PyResult<(Option<String>, Option<String>)> {
    let raw: Vec<Cohort> = cohorts.into_iter().map(|c| c.inner).collect();
    let (best, worst) = CohortAnalytics::performance_ranking(&raw)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok((best.map(|c| c.0), worst.map(|c| c.0)))
}

// ============================================================================
// LIFECYCLE TRACKING
// ============================================================================

fn lifecycle_stage_to_str(stage: LifecycleStage) -> &'static str {
    match stage {
        LifecycleStage::Prospect => "prospect",
        LifecycleStage::Onboarding => "onboarding",
        LifecycleStage::Active => "active",
        LifecycleStage::Mature => "mature",
        LifecycleStage::AtRisk => "at_risk",
        LifecycleStage::Dormant => "dormant",
        LifecycleStage::Churned => "churned",
    }
}

#[pyclass]
struct PyLifecycleProfile {
    #[pyo3(get)]
    customer_id: String,
    #[pyo3(get)]
    current_stage: String,
    #[pyo3(get)]
    stage_confidence: f64,
    #[pyo3(get)]
    total_purchase_count: usize,
    #[pyo3(get)]
    total_value: f64,
    #[pyo3(get)]
    avg_order_value: f64,
    #[pyo3(get)]
    retention_score: f64,
    #[pyo3(get)]
    expansion_potential: f64,
}

#[pymethods]
impl PyLifecycleProfile {
    fn __repr__(&self) -> String {
        format!(
            "LifecycleProfile(customer_id={}, stage={}, retention_score={:.2})",
            self.customer_id, self.current_stage, self.retention_score
        )
    }
}

/// Classify a customer into a lifecycle stage (prospect / onboarding /
/// active / mature / at_risk / dormant / churned) from behavioral signals.
#[pyfunction]
fn classify_lifecycle_stage(
    customer_id: &str,
    days_since_signup: i32,
    purchase_count: usize,
    total_value: f64,
    days_since_last_purchase: i32,
    purchase_frequency: f64,
) -> PyResult<PyLifecycleProfile> {
    let profile = LifecycleTracker::classify_stage(
        customer_id,
        days_since_signup,
        purchase_count,
        total_value,
        days_since_last_purchase,
        purchase_frequency,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    Ok(PyLifecycleProfile {
        customer_id: profile.customer_id,
        current_stage: lifecycle_stage_to_str(profile.current_stage).to_string(),
        stage_confidence: profile.stage_confidence,
        total_purchase_count: profile.total_purchase_count,
        total_value: profile.total_value,
        avg_order_value: profile.avg_order_value,
        retention_score: profile.retention_score,
        expansion_potential: profile.expansion_potential,
    })
}

/// Recommended retention actions for a given lifecycle stage name.
#[pyfunction]
fn lifecycle_retention_actions(stage: &str) -> PyResult<Vec<String>> {
    let stage = parse_lifecycle_stage(stage)?;
    Ok(LifecycleTracker::retention_actions(stage)
        .into_iter()
        .map(|s| s.to_string())
        .collect())
}

fn parse_lifecycle_stage(s: &str) -> PyResult<LifecycleStage> {
    match s.to_lowercase().as_str() {
        "prospect" => Ok(LifecycleStage::Prospect),
        "onboarding" => Ok(LifecycleStage::Onboarding),
        "active" => Ok(LifecycleStage::Active),
        "mature" => Ok(LifecycleStage::Mature),
        "at_risk" | "atrisk" => Ok(LifecycleStage::AtRisk),
        "dormant" => Ok(LifecycleStage::Dormant),
        "churned" => Ok(LifecycleStage::Churned),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown lifecycle stage '{}'. Expected one of: prospect, onboarding, active, mature, at_risk, dormant, churned", other
        ))),
    }
}

/// Percentage breakdown of customers across lifecycle stages.
#[pyfunction]
fn lifecycle_stage_distribution(
    profiles: Vec<PyRef<PyLifecycleProfile>>,
) -> PyResult<HashMap<String, f64>> {
    // Re-derive engine profiles is unnecessary here since stage_distribution
    // only needs `current_stage`; build minimal profiles isn't worth it —
    // compute the distribution directly from the already-classified stage
    // strings to avoid a redundant round trip through the engine type.
    let mut counts: HashMap<String, usize> = HashMap::new();
    for p in &profiles {
        *counts.entry(p.current_stage.clone()).or_insert(0) += 1;
    }
    let total = profiles.len() as f64;
    if total == 0.0 {
        return Ok(HashMap::new());
    }
    Ok(counts
        .into_iter()
        .map(|(k, v)| (k, (v as f64 / total) * 100.0))
        .collect())
}

// ============================================================================
// CLUSTER QUALITY METRICS
// ============================================================================

fn to_array2(data: Vec<Vec<f64>>) -> PyResult<Array2<f64>> {
    let ncols = data.first().map(|r| r.len()).unwrap_or(0);
    Array2::from_shape_vec((data.len(), ncols), data.into_iter().flatten().collect())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Silhouette coefficient for a clustering (-1 to 1; higher is better).
#[pyfunction]
fn silhouette_score(data: Vec<Vec<f64>>, labels: Vec<usize>) -> PyResult<f64> {
    let arr = to_array2(data)?;
    SilhouetteMetric::calculate(&arr, &labels)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Davies-Bouldin index for a clustering (>= 0; lower is better).
#[pyfunction]
fn davies_bouldin_score(
    data: Vec<Vec<f64>>,
    labels: Vec<usize>,
    centers: Vec<Vec<f64>>,
) -> PyResult<f64> {
    let arr = to_array2(data)?;
    let centers_arr = to_array2(centers)?;
    DaviesBouldinMetric::calculate(&arr, &labels, &centers_arr)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Calinski-Harabasz index for a clustering (higher is better).
#[pyfunction]
fn calinski_harabasz_score(data: Vec<Vec<f64>>, labels: Vec<usize>) -> PyResult<f64> {
    let arr = to_array2(data)?;
    CalinskiHarabaszMetric::calculate(&arr, &labels)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

#[pyclass]
struct PyQualityReport {
    #[pyo3(get)]
    silhouette_score: f64,
    #[pyo3(get)]
    davies_bouldin_score: f64,
    #[pyo3(get)]
    calinski_harabasz_score: f64,
    #[pyo3(get)]
    inertia: f64,
    #[pyo3(get)]
    n_clusters: usize,
    #[pyo3(get)]
    overall_score: f64,
}

#[pymethods]
impl PyQualityReport {
    fn __repr__(&self) -> String {
        format!(
            "QualityReport(silhouette={:.4}, davies_bouldin={:.4}, calinski_harabasz={:.2}, overall={:.2})",
            self.silhouette_score, self.davies_bouldin_score, self.calinski_harabasz_score, self.overall_score
        )
    }
}

/// Comprehensive cluster quality report combining silhouette,
/// Davies-Bouldin, Calinski-Harabasz, and inertia into a single 0-100
/// overall score.
#[pyfunction]
fn assess_cluster_quality(
    data: Vec<Vec<f64>>,
    labels: Vec<usize>,
    centers: Vec<Vec<f64>>,
) -> PyResult<PyQualityReport> {
    let arr = to_array2(data)?;
    let centers_arr = to_array2(centers)?;
    let report = QualityAssessment::assess(&arr, &labels, &centers_arr)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyQualityReport {
        silhouette_score: report.silhouette_score,
        davies_bouldin_score: report.davies_bouldin_score,
        calinski_harabasz_score: report.calinski_harabasz_score,
        inertia: report.inertia,
        n_clusters: report.n_clusters,
        overall_score: report.overall_score,
    })
}

// ============================================================================
// K ESTIMATION (choosing the number of clusters)
// ============================================================================

#[pyclass]
struct PyKEstimationResult {
    #[pyo3(get)]
    method: String,
    #[pyo3(get)]
    k: usize,
    #[pyo3(get)]
    scores: Vec<(usize, f64)>,
    #[pyo3(get)]
    confidence: f64,
}

#[pymethods]
impl PyKEstimationResult {
    fn __repr__(&self) -> String {
        format!(
            "KEstimationResult(method={}, k={}, confidence={:.2})",
            self.method, self.k, self.confidence
        )
    }
}

fn wrap_k_result(r: crate::engine::k_estimation::KEstimationResult) -> PyKEstimationResult {
    PyKEstimationResult {
        method: r.method,
        k: r.k,
        scores: r.scores,
        confidence: r.confidence,
    }
}

/// Estimate the optimal number of clusters via the elbow method (inertia
/// curvature) over `k_range = (k_min, k_max)`.
#[pyfunction]
fn estimate_k_elbow(data: Vec<Vec<f64>>, k_range: (usize, usize)) -> PyResult<PyKEstimationResult> {
    let arr = to_array2(data)?;
    ElbowMethod::estimate(&arr, k_range)
        .map(wrap_k_result)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Estimate the optimal number of clusters via the gap statistic.
#[pyfunction]
fn estimate_k_gap_statistic(
    data: Vec<Vec<f64>>,
    k_range: (usize, usize),
) -> PyResult<PyKEstimationResult> {
    let arr = to_array2(data)?;
    GapStatistic::estimate(&arr, k_range)
        .map(wrap_k_result)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Estimate the optimal number of clusters via silhouette analysis.
#[pyfunction]
fn estimate_k_silhouette(
    data: Vec<Vec<f64>>,
    k_range: (usize, usize),
) -> PyResult<PyKEstimationResult> {
    let arr = to_array2(data)?;
    SilhouetteEstimation::estimate(&arr, k_range)
        .map(wrap_k_result)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Estimate the optimal number of clusters via a majority vote across the
/// elbow, gap-statistic, and silhouette methods.
#[pyfunction]
fn estimate_k_combined(
    data: Vec<Vec<f64>>,
    k_range: (usize, usize),
) -> PyResult<PyKEstimationResult> {
    let arr = to_array2(data)?;
    CombinedKEstimation::estimate(&arr, k_range)
        .map(wrap_k_result)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

// ============================================================================
// BEHAVIORAL RULE-BASED SEGMENTATION
// ============================================================================

fn parse_comparison_op(s: &str) -> PyResult<ComparisonOp> {
    match s {
        ">" => Ok(ComparisonOp::GreaterThan),
        ">=" => Ok(ComparisonOp::GreaterOrEqual),
        "<" => Ok(ComparisonOp::LessThan),
        "<=" => Ok(ComparisonOp::LessOrEqual),
        "==" | "=" => Ok(ComparisonOp::Equal),
        "!=" | "<>" => Ok(ComparisonOp::NotEqual),
        "in" | "IN" => Ok(ComparisonOp::In),
        "contains" | "CONTAINS" => Ok(ComparisonOp::Contains),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown comparison operator '{}'. Expected one of: >, >=, <, <=, ==, !=, in, contains",
            other
        ))),
    }
}

fn parse_logical_op(s: &str) -> PyResult<LogicalOp> {
    match s.to_uppercase().as_str() {
        "AND" => Ok(LogicalOp::And),
        "OR" => Ok(LogicalOp::Or),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown logical operator '{}'. Expected AND or OR",
            other
        ))),
    }
}

#[pyclass]
#[derive(Clone)]
struct PyCondition {
    inner: Condition,
}

#[pymethods]
impl PyCondition {
    #[new]
    fn new(field: &str, operator: &str, value: f64) -> PyResult<Self> {
        Ok(PyCondition {
            inner: Condition::new(
                field,
                parse_comparison_op(operator)?,
                RuleValue::Number(value),
            ),
        })
    }
}

#[pyclass]
#[derive(Clone)]
struct PyBehavioralRule {
    inner: BehavioralRule,
}

#[pymethods]
impl PyBehavioralRule {
    #[new]
    #[pyo3(signature = (name, description, conditions, logic="AND"))]
    fn new(
        name: &str,
        description: &str,
        conditions: Vec<PyCondition>,
        logic: &str,
    ) -> PyResult<Self> {
        let mut rule = BehavioralRule::new(name, description).with_logic(parse_logical_op(logic)?);
        for c in conditions {
            rule = rule.add_condition(c.inner);
        }
        Ok(PyBehavioralRule { inner: rule })
    }

    /// Render this rule as a SQL WHERE-clause fragment.
    fn to_sql(&self) -> String {
        self.inner.to_sql()
    }
}

#[pyclass]
#[derive(Clone)]
struct PyBehavioralSegment {
    inner: BehavioralSegment,
}

#[pymethods]
impl PyBehavioralSegment {
    #[new]
    #[pyo3(signature = (name, description, rules, priority=0))]
    fn new(name: &str, description: &str, rules: Vec<PyBehavioralRule>, priority: u32) -> Self {
        let mut segment = BehavioralSegment::new(name, description).with_priority(priority);
        for r in rules {
            segment = segment.add_rule(r.inner);
        }
        PyBehavioralSegment { inner: segment }
    }

    fn to_sql(&self) -> String {
        self.inner.to_sql()
    }

    /// Does this customer's data (field name -> numeric value) satisfy the
    /// segment's rule(s)?
    fn matches(&self, customer_data: HashMap<String, f64>) -> bool {
        self.inner.matches(&customer_data)
    }
}

/// Rule-based (deterministic, explainable) customer segmentation — an
/// alternative to statistical clustering when segment boundaries need to be
/// human-defined business rules (e.g. "high value = monetary > 1000").
#[pyclass]
struct PyBehavioralSegmenter {
    inner: BehavioralSegmenter,
}

#[pymethods]
impl PyBehavioralSegmenter {
    #[new]
    fn new(segments: Vec<PyBehavioralSegment>) -> Self {
        let mut segmenter = BehavioralSegmenter::new();
        for s in segments {
            segmenter = segmenter.add_segment(s.inner);
        }
        PyBehavioralSegmenter { inner: segmenter }
    }

    /// All segment names this customer's data matches (there may be more
    /// than one — segments are not mutually exclusive by default).
    fn classify(&self, customer_data: HashMap<String, f64>) -> PyResult<Vec<String>> {
        self.inner
            .classify(&customer_data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// The single highest-priority matching segment name, if any.
    fn classify_primary(&self, customer_data: HashMap<String, f64>) -> PyResult<Option<String>> {
        self.inner
            .classify_primary(&customer_data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Combined SQL export of every configured segment's rule(s).
    fn export_sql(&self) -> PyResult<String> {
        self.inner
            .export_sql()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

// ============================================================================
// SEGMENT PROFILING
// ============================================================================

#[pyclass]
struct PySegmentHealth {
    #[pyo3(get)]
    stability: f64,
    #[pyo3(get)]
    cohesion: f64,
    #[pyo3(get)]
    separation: f64,
    #[pyo3(get)]
    health_score: f64,
}

#[pyclass]
struct PyProfiledSegment {
    #[pyo3(get)]
    segment_id: usize,
    #[pyo3(get)]
    size: usize,
    #[pyo3(get)]
    purity: f64,
    #[pyo3(get)]
    business_description: String,
    #[pyo3(get)]
    key_characteristics: Vec<String>,
    #[pyo3(get)]
    health: Py<PySegmentHealth>,
    #[pyo3(get)]
    actionability_score: f64,
}

#[pymethods]
impl PyProfiledSegment {
    fn __repr__(&self) -> String {
        format!(
            "ProfiledSegment(id={}, size={}, purity={:.2})",
            self.segment_id, self.size, self.purity
        )
    }
}

/// Profile one cluster/segment: per-feature summary statistics, a plain-
/// English business description, key characteristics, and a stability/
/// cohesion/separation health score.
///
/// `members` are row indices belonging to this segment; `features` maps a
/// feature index to that feature's full column of values (all rows, not
/// just this segment's — used to compute cross-segment separation);
/// `feature_names` optionally labels each feature index.
#[pyfunction]
#[pyo3(signature = (segment_id, members, features, feature_names=None))]
fn profile_segment(
    py: Python<'_>,
    segment_id: usize,
    members: Vec<usize>,
    features: HashMap<usize, Vec<f64>>,
    feature_names: Option<Vec<String>>,
) -> PyResult<PyProfiledSegment> {
    let profile =
        ProfilingEngine::profile_segment(segment_id, &members, &features, feature_names.as_deref())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let health = Py::new(
        py,
        PySegmentHealth {
            stability: profile.health.stability,
            cohesion: profile.health.cohesion,
            separation: profile.health.separation,
            health_score: profile.health.health_score,
        },
    )?;

    Ok(PyProfiledSegment {
        segment_id: profile.segment_id,
        size: profile.size,
        purity: profile.purity,
        business_description: profile.business_description,
        key_characteristics: profile.key_characteristics,
        health,
        actionability_score: profile.actionability_score,
    })
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

    // Privacy: differential privacy & k-anonymity
    m.add_class::<PyPrivacyBudget>()?;
    m.add_class::<PyKAnonymityResult>()?;
    m.add_function(wrap_pyfunction!(add_laplace_noise, m)?)?;
    m.add_function(wrap_pyfunction!(add_gaussian_noise, m)?)?;
    m.add_function(wrap_pyfunction!(check_k_anonymity, m)?)?;
    m.add_function(wrap_pyfunction!(suppress_to_k_anonymous, m)?)?;
    m.add_function(wrap_pyfunction!(generalize_numeric, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_information_loss, m)?)?;

    // Streaming: real-time segmentation
    m.add_class::<PyStreamingEvent>()?;
    m.add_class::<PyStreamingConfig>()?;
    m.add_class::<PyStreamingSegmentUpdate>()?;
    m.add_class::<PyStreamingSegmentationEngine>()?;

    // Drift detection
    m.add_class::<PyFeatureDrift>()?;
    m.add_class::<PySegmentCompositionChange>()?;
    m.add_function(wrap_pyfunction!(kolmogorov_smirnov, m)?)?;
    m.add_function(wrap_pyfunction!(hellinger_distance, m)?)?;
    m.add_function(wrap_pyfunction!(chi_square_drift, m)?)?;
    m.add_function(wrap_pyfunction!(detect_feature_drift, m)?)?;
    m.add_function(wrap_pyfunction!(detect_segment_composition_change, m)?)?;

    // Lookalike audiences
    m.add_class::<PySeedCustomer>()?;
    m.add_class::<PyLookalikeCandidate>()?;
    m.add_class::<PyLookalikeAudience>()?;
    m.add_function(wrap_pyfunction!(generate_lookalike, m)?)?;
    m.add_function(wrap_pyfunction!(find_similar_customers, m)?)?;
    m.add_function(wrap_pyfunction!(cosine_similarity, m)?)?;

    // Cohort analytics
    m.add_class::<PyCohort>()?;
    m.add_function(wrap_pyfunction!(cohort_id_for, m)?)?;
    m.add_function(wrap_pyfunction!(create_cohort, m)?)?;
    m.add_function(wrap_pyfunction!(compare_cohorts, m)?)?;
    m.add_function(wrap_pyfunction!(aggregate_cohorts_by_period, m)?)?;
    m.add_function(wrap_pyfunction!(cohort_retention_table, m)?)?;
    m.add_function(wrap_pyfunction!(cohort_performance_ranking, m)?)?;

    // Lifecycle tracking
    m.add_class::<PyLifecycleProfile>()?;
    m.add_function(wrap_pyfunction!(classify_lifecycle_stage, m)?)?;
    m.add_function(wrap_pyfunction!(lifecycle_retention_actions, m)?)?;
    m.add_function(wrap_pyfunction!(lifecycle_stage_distribution, m)?)?;

    // Cluster quality metrics
    m.add_class::<PyQualityReport>()?;
    m.add_function(wrap_pyfunction!(silhouette_score, m)?)?;
    m.add_function(wrap_pyfunction!(davies_bouldin_score, m)?)?;
    m.add_function(wrap_pyfunction!(calinski_harabasz_score, m)?)?;
    m.add_function(wrap_pyfunction!(assess_cluster_quality, m)?)?;

    // K estimation
    m.add_class::<PyKEstimationResult>()?;
    m.add_function(wrap_pyfunction!(estimate_k_elbow, m)?)?;
    m.add_function(wrap_pyfunction!(estimate_k_gap_statistic, m)?)?;
    m.add_function(wrap_pyfunction!(estimate_k_silhouette, m)?)?;
    m.add_function(wrap_pyfunction!(estimate_k_combined, m)?)?;

    // Behavioral rule-based segmentation
    m.add_class::<PyCondition>()?;
    m.add_class::<PyBehavioralRule>()?;
    m.add_class::<PyBehavioralSegment>()?;
    m.add_class::<PyBehavioralSegmenter>()?;

    // Segment profiling
    m.add_class::<PySegmentHealth>()?;
    m.add_class::<PyProfiledSegment>()?;
    m.add_function(wrap_pyfunction!(profile_segment, m)?)?;

    // Add module info. Only lists algorithms that are actually implemented
    // and callable above — a previous version of this listed "dbscan",
    // "hierarchical", and "gmm", none of which exist anywhere in this
    // codebase.
    let info = PyModule::new_bound(py, "info")?;
    info.add("algorithms", vec!["kmeans", "kprototypes"])?;
    info.add(
        "metrics",
        vec!["silhouette", "davies_bouldin", "calinski_harabasz"],
    )?;
    info.add("segments", 13usize)?;
    m.add_submodule(&info)?;

    Ok(())
}
