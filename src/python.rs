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
use crate::engine::clustering::{kmeans, ClusteringMethod, KMeansResult, MiniBatchKMeans};
use crate::engine::clv::{CLVCalculator, CustomerLTV};
use crate::engine::rfm::{
    calculate_rfm, DecayFunction, RFMConfig, RFMScore, ScoringMethod, Transaction,
};
use crate::engine::segments::{SegmentProfile, SegmentType};
use crate::engine::sql_export::{validate_identifier, ColumnMapping, SQLDialect, SQLExporter};
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
use crate::engine::drift_detection::{DriftDetector, DriftMethod, DriftSeverity, FeatureDrift};
use crate::engine::k_estimation::{
    CombinedKEstimation, ElbowMethod, GapStatistic, SilhouetteEstimation,
};
use crate::engine::lifecycle::{LifecycleStage, LifecycleTracker};
use crate::engine::lookalike::{LookalikeGenerator, SeedCustomer, SimilarityMetric};
use crate::engine::neural_networks::{NNConfig, NeuralNetwork};
use crate::engine::pattern_discovery::{EmergingAudience, SegmentTrend};
use crate::engine::price_intelligence::{
    AffiniCalculator, DiscountOptimizer, ElasticityCalculator, TierMigrationPredictor,
};
use crate::engine::privacy::{DifferentialPrivacy, KAnonymity, PrivacyBudget};
use crate::engine::profiling::ProfilingEngine;
use crate::engine::quality_metrics::{
    CalinskiHarabaszMetric, DaviesBouldinMetric, QualityAssessment, SilhouetteMetric,
};
use crate::engine::revenue_intelligence::{
    AttributionEngine, CACCalculator, ROICalculator, RevenueForecaster,
};
use crate::engine::segment_intelligence::{
    SegmentAging, SegmentConfidence, SegmentDifferentiation, SegmentEntropy, SegmentPredictability,
};
use crate::engine::streaming::{
    ReclusterConfig, ReclusterEvent, StreamEventType, StreamingConfig, StreamingEvent,
    StreamingSegmentationEngine, StreamingWindow,
};
use crate::engine::temporal_analytics::{
    CompositionForecaster, HistoricalReconstruction, MomentumAnalyzer, SegmentSizeForecaster,
    WhatIfSimulator,
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
#[pyo3(signature = (transactions, config, n_jobs=None))]
fn calculate_rfm_py(
    transactions: Vec<(String, String, f64)>,
    config: &PyRFMConfig,
    n_jobs: Option<i32>,
) -> PyResult<Vec<PyRFMScore>> {
    let tx_vec: Vec<Transaction> = transactions
        .into_iter()
        .map(|(customer_id, date, amount)| Transaction {
            customer_id,
            date,
            amount,
        })
        .collect();

    // -1 (use all cores) preserves the pre-n_jobs behavior when the caller
    // doesn't pass n_jobs explicitly.
    calculate_rfm(tx_vec, &config.inner, n_jobs.unwrap_or(-1))
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
#[pyo3(signature = (data, n_clusters, max_iter=None, random_state=None, n_jobs=None))]
fn kmeans_py(
    data: Vec<Vec<f64>>,
    n_clusters: usize,
    max_iter: Option<usize>,
    random_state: Option<u64>,
    n_jobs: Option<i32>,
) -> PyResult<PyKMeansResult> {
    let max_iter = max_iter.unwrap_or(300);
    let random_state = random_state.unwrap_or(42);
    // -1 (use all cores) preserves the pre-n_jobs behavior when the caller
    // doesn't pass n_jobs explicitly.
    let n_jobs = n_jobs.unwrap_or(-1);

    let data_array = Array2::from_shape_vec(
        (data.len(), if data.is_empty() { 0 } else { data[0].len() }),
        data.into_iter().flatten().collect(),
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    kmeans(&data_array, n_clusters, max_iter, random_state, n_jobs)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        .map(|result| PyKMeansResult { inner: result })
}

fn vec_to_array2(data: Vec<Vec<f64>>) -> PyResult<Array2<f64>> {
    let n_rows = data.len();
    let n_cols = if data.is_empty() { 0 } else { data[0].len() };
    Array2::from_shape_vec((n_rows, n_cols), data.into_iter().flatten().collect())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Mini-batch K-Means: chunked/streaming ingestion for batch clustering.
/// Unlike `kmeans()`/`AudienceSegmenter.fit()`, which require the entire
/// dataset as one in-memory `Array2` for repeated full passes,
/// `partial_fit` consumes one chunk at a time and updates centers
/// incrementally -- a caller can stream chunks from a file, database
/// cursor, or Python generator without ever materializing the full
/// dataset, bounding memory to O(chunk_size + n_clusters) regardless of
/// total dataset size.
#[pyclass]
struct PyMiniBatchKMeans {
    inner: MiniBatchKMeans,
}

#[pymethods]
impl PyMiniBatchKMeans {
    #[new]
    #[pyo3(signature = (n_clusters, random_state=42))]
    fn new(n_clusters: usize, random_state: u64) -> PyResult<Self> {
        MiniBatchKMeans::new(n_clusters, random_state)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
            .map(|inner| PyMiniBatchKMeans { inner })
    }

    /// Ingest one chunk of rows, updating centers in place. The first
    /// chunk ever passed must have at least `n_clusters` rows.
    fn partial_fit(&mut self, chunk: Vec<Vec<f64>>) -> PyResult<()> {
        let chunk = vec_to_array2(chunk)?;
        self.inner
            .partial_fit(&chunk)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Assign each row of `data` to its nearest fitted center.
    fn predict(&self, data: Vec<Vec<f64>>) -> PyResult<Vec<usize>> {
        let data = vec_to_array2(data)?;
        self.inner
            .predict(&data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn is_initialized(&self) -> bool {
        self.inner.is_initialized()
    }

    #[getter]
    fn centers(&self) -> Vec<Vec<f64>> {
        self.inner
            .centers()
            .outer_iter()
            .map(|row| row.to_vec())
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "MiniBatchKMeans(initialized={})",
            self.inner.is_initialized()
        )
    }
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
    /// `n_jobs` follows the scikit-learn convention: `-1` (the default)
    /// uses all available CPU cores, `1` forces single-threaded execution
    /// (useful for debugging/reproducing issues without thread scheduling
    /// noise), and any other positive `N` caps parallel work at `N`
    /// threads. It's implemented as a scoped rayon thread pool built fresh
    /// for each `fit()` call (see `clustering::build_thread_pool`), not a
    /// mutation of the process-global rayon pool, so constructing multiple
    /// segmenters with different `n_jobs` values is safe.
    #[new]
    #[pyo3(signature = (n_clusters, n_jobs=None, method=None))]
    fn new(n_clusters: usize, n_jobs: Option<i32>, method: Option<&str>) -> PyResult<Self> {
        let clustering_method = match method.unwrap_or("kmeans") {
            "kmeans" => ClusteringMethod::KMeans,
            "kprototypes" => ClusteringMethod::KPrototypes,
            other => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "unsupported clustering method '{other}'; expected 'kmeans' or 'kprototypes'"
                )))
            }
        };
        let config = SegmenterConfig {
            method: method.unwrap_or("kmeans").to_string(),
            n_clusters,
            rfm_config: RFMConfig::default(),
            clustering_method,
            random_state: 42,
            n_jobs: n_jobs.unwrap_or(-1),
        };
        Ok(PyAudienceSegmenter {
            inner: AudienceSegmenterCore::new(config),
        })
    }

    #[getter]
    fn get_n_jobs(&self) -> i32 {
        self.inner.config.n_jobs
    }

    #[pyo3(signature = (data, categorical_data=None))]
    fn fit(
        &mut self,
        data: Vec<Vec<f64>>,
        categorical_data: Option<Vec<Vec<usize>>>,
    ) -> PyResult<()> {
        let data_array = Array2::from_shape_vec(
            (data.len(), if data.is_empty() { 0 } else { data[0].len() }),
            data.into_iter().flatten().collect(),
        )
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        self.inner
            .fit_with_categorical(&data_array, categorical_data.as_ref())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[pyo3(signature = (data, categorical_data=None))]
    fn predict(
        &self,
        data: Vec<Vec<f64>>,
        categorical_data: Option<Vec<Vec<usize>>>,
    ) -> PyResult<Vec<usize>> {
        let data_array = Array2::from_shape_vec(
            (data.len(), if data.is_empty() { 0 } else { data[0].len() }),
            data.into_iter().flatten().collect(),
        )
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        self.inner
            .predict_with_categorical(&data_array, categorical_data.as_ref())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[getter]
    fn categorical_centers(&self) -> Option<Vec<Vec<usize>>> {
        self.inner.categorical_centers.clone()
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

#[pyclass(from_py_object)]
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

/// Configuration for drift-triggered re-clustering: how severe drift in the
/// tracked population's RFM distribution must be (`min_severity`), how many
/// customers must be tracked before drift is even evaluated
/// (`min_customers`), and how many clusters to re-fit with
/// (`n_clusters`) when it fires.
#[pyclass]
struct PyReclusterConfig {
    inner: ReclusterConfig,
}

#[pymethods]
impl PyReclusterConfig {
    #[new]
    #[pyo3(signature = (min_severity="high", min_customers=30, n_clusters=4))]
    fn new(min_severity: &str, min_customers: usize, n_clusters: usize) -> PyResult<Self> {
        Ok(PyReclusterConfig {
            inner: ReclusterConfig {
                min_severity: parse_drift_severity(min_severity)?,
                min_customers,
                n_clusters,
            },
        })
    }
}

/// Record of a completed drift-triggered re-cluster: which features
/// drifted (and by how much), how many customers were re-fit, and into how
/// many clusters.
#[pyclass]
struct PyReclusterEvent {
    #[pyo3(get)]
    timestamp: i64,
    #[pyo3(get)]
    drifts: Vec<PyFeatureDrift>,
    #[pyo3(get)]
    customers_reclustered: usize,
    #[pyo3(get)]
    n_clusters: usize,
}

fn recluster_event_to_py(e: ReclusterEvent) -> PyReclusterEvent {
    PyReclusterEvent {
        timestamp: e.timestamp,
        drifts: e.drifts.into_iter().map(feature_drift_to_py).collect(),
        customers_reclustered: e.customers_reclustered,
        n_clusters: e.n_clusters,
    }
}

/// Stateful real-time segmentation engine: feed it events one at a time (or
/// in batches) and it maintains an incrementally-updated RFM-like state and
/// segment assignment per customer, without requiring a full batch refit.
///
/// `process_batch` is also the drift-triggered re-clustering checkpoint:
/// once enough customers are tracked, the first batch arms a drift
/// baseline, and every batch after that checks the current RFM
/// distribution against it via `DriftDetector`. If drift crosses
/// `recluster_config`'s threshold, a real k-means re-fit runs over every
/// tracked customer's (recency, frequency, monetary) vector and segment
/// assignments are overwritten with the fresh cluster labels.
#[pyclass]
struct PyStreamingSegmentationEngine {
    inner: StreamingSegmentationEngine,
}

#[pymethods]
impl PyStreamingSegmentationEngine {
    #[new]
    #[pyo3(signature = (config, recluster_config=None))]
    fn new(config: &PyStreamingConfig, recluster_config: Option<&PyReclusterConfig>) -> Self {
        let mut inner = StreamingSegmentationEngine::new(config.inner.clone());
        if let Some(rc) = recluster_config {
            inner = inner.with_recluster_config(rc.inner.clone());
        }
        PyStreamingSegmentationEngine { inner }
    }

    /// (Re-)arm drift detection against the current RFM distribution,
    /// without performing a re-cluster.
    fn set_recluster_baseline(&mut self) {
        self.inner.set_recluster_baseline();
    }

    /// Compare the current RFM distribution against the last-captured
    /// baseline. Returns `None` if no baseline is armed yet or there
    /// aren't enough tracked customers for the comparison to be
    /// meaningful.
    fn check_drift(&self) -> PyResult<Option<Vec<PyFeatureDrift>>> {
        let drifts = self
            .inner
            .check_drift()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(drifts.map(|ds| ds.into_iter().map(feature_drift_to_py).collect()))
    }

    /// Check for drift and, if it crosses the configured threshold, re-fit
    /// real k-means clusters over every tracked customer and overwrite
    /// their segment assignments. Returns `None` if nothing was triggered.
    fn maybe_recluster(&mut self) -> PyResult<Option<PyReclusterEvent>> {
        let event = self
            .inner
            .maybe_recluster()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(event.map(recluster_event_to_py))
    }

    /// History of drift-triggered re-clusters this engine has performed.
    fn recluster_history(&self) -> Vec<PyReclusterEvent> {
        self.inner
            .recluster_history()
            .iter()
            .cloned()
            .map(recluster_event_to_py)
            .collect()
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

fn parse_drift_severity(s: &str) -> PyResult<DriftSeverity> {
    match s.to_lowercase().as_str() {
        "none" => Ok(DriftSeverity::None),
        "low" => Ok(DriftSeverity::Low),
        "medium" => Ok(DriftSeverity::Medium),
        "high" => Ok(DriftSeverity::High),
        "critical" => Ok(DriftSeverity::Critical),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown drift severity '{}'. Expected one of: none, low, medium, high, critical",
            other
        ))),
    }
}

fn feature_drift_to_py(d: FeatureDrift) -> PyFeatureDrift {
    PyFeatureDrift {
        feature_name: d.feature_name,
        drift_score: d.drift_score,
        severity: d.severity.as_str().to_string(),
        timestamp: d.timestamp,
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

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
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

#[pyclass(from_py_object)]
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

#[pyclass(from_py_object)]
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

#[pyclass(from_py_object)]
#[derive(Clone)]
struct PyCondition {
    inner: Condition,
}

#[pymethods]
impl PyCondition {
    #[new]
    fn new(field: &str, operator: &str, value: f64) -> PyResult<Self> {
        // `field` is interpolated verbatim into the SQL this condition's
        // `to_sql()`/`export_sql()` produce (see `Condition::to_sql` in
        // engine::behavioral), with no other validation anywhere on this
        // path. Without this check, e.g. `PyCondition("x; DROP TABLE t; --",
        // ">", 1.0)` renders that payload straight into generated SQL —
        // the same class of SQL-injection vector `validate_identifier` was
        // introduced to close in `sql_export.rs`, just reachable through a
        // different binding it didn't cover.
        validate_identifier(field)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(PyCondition {
            inner: Condition::new(
                field,
                parse_comparison_op(operator)?,
                RuleValue::Number(value),
            ),
        })
    }
}

#[pyclass(from_py_object)]
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

#[pyclass(from_py_object)]
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
///
/// `labels`, if given, is the ground-truth class label per row (e.g. a known
/// customer type), used to compute a true clustering *purity*. Without it,
/// `purity` is a documented label-free homogeneity proxy (see
/// `ProfilingEngine::calculate_purity`), not supervised purity.
#[pyfunction]
#[pyo3(signature = (segment_id, members, features, feature_names=None, labels=None))]
fn profile_segment(
    py: Python<'_>,
    segment_id: usize,
    members: Vec<usize>,
    features: HashMap<usize, Vec<f64>>,
    feature_names: Option<Vec<String>>,
    labels: Option<Vec<usize>>,
) -> PyResult<PyProfiledSegment> {
    let profile = ProfilingEngine::profile_segment_with_labels(
        segment_id,
        &members,
        &features,
        feature_names.as_deref(),
        labels.as_deref(),
    )
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

// ---------------------------------------------------------------------------
// Pattern discovery (exposed slice)
// ---------------------------------------------------------------------------

#[pyclass]
pub struct PySegmentTrend {
    #[pyo3(get)]
    pub segment_id: usize,
    #[pyo3(get)]
    pub trend_direction: String,
    #[pyo3(get)]
    pub trend_strength: f64,
    #[pyo3(get)]
    pub moving_average: f64,
    #[pyo3(get)]
    pub volatility: f64,
    #[pyo3(get)]
    pub forecast_accuracy: f64,
}

#[pymethods]
impl PySegmentTrend {
    #[staticmethod]
    fn from_time_series(segment_id: usize, values: Vec<f64>) -> PyResult<Self> {
        let trend = SegmentTrend::from_time_series(segment_id, &values)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            segment_id: trend.segment_id,
            trend_direction: trend.trend_direction,
            trend_strength: trend.trend_strength,
            moving_average: trend.moving_average,
            volatility: trend.volatility,
            forecast_accuracy: trend.forecast_accuracy,
        })
    }
}

#[pyclass]
pub struct PyEmergingAudience {
    #[pyo3(get)]
    pub segment_id: usize,
    #[pyo3(get)]
    pub emergence_score: f64,
    #[pyo3(get)]
    pub growth_rate: f64,
    #[pyo3(get)]
    pub periods_since_emergence: usize,
    #[pyo3(get)]
    pub current_size: usize,
    #[pyo3(get)]
    pub projected_size_next_period: usize,
    #[pyo3(get)]
    pub emergence_status: String,
}

#[pymethods]
impl PyEmergingAudience {
    #[new]
    fn new(
        segment_id: usize,
        current_size: usize,
        previous_size: usize,
        historical_sizes: Vec<usize>,
    ) -> Self {
        let audience =
            EmergingAudience::new(segment_id, current_size, previous_size, &historical_sizes);
        Self {
            segment_id: audience.segment_id,
            emergence_score: audience.emergence_score,
            growth_rate: audience.growth_rate,
            periods_since_emergence: audience.periods_since_emergence,
            current_size: audience.current_size,
            projected_size_next_period: audience.projected_size_next_period,
            emergence_status: audience.emergence_status,
        }
    }
}

// ---------------------------------------------------------------------------
// Segment intelligence (exposed slice)
// ---------------------------------------------------------------------------

/// Membership certainty for one member, derived from the distance to its own
/// centroid versus the distance to the nearest other centroid.
#[pyclass]
pub struct PySegmentConfidence {
    #[pyo3(get)]
    pub segment_id: usize,
    #[pyo3(get)]
    pub member_id: String,
    #[pyo3(get)]
    pub confidence_score: f64,
    #[pyo3(get)]
    pub distance_to_centroid: f64,
    #[pyo3(get)]
    pub distance_to_nearest_other: f64,
}

#[pymethods]
impl PySegmentConfidence {
    #[new]
    fn new(
        segment_id: usize,
        member_id: String,
        distance_to_centroid: f64,
        distance_to_nearest_other: f64,
    ) -> Self {
        let conf = SegmentConfidence::new(
            segment_id,
            member_id,
            distance_to_centroid,
            distance_to_nearest_other,
        );
        Self {
            segment_id: conf.segment_id,
            member_id: conf.member_id,
            confidence_score: conf.confidence_score,
            distance_to_centroid: conf.distance_to_centroid,
            distance_to_nearest_other: conf.distance_to_nearest_other,
        }
    }
}

/// Diversity of a segment, computed from category frequencies. `compute_shannon`
/// returns Shannon entropy in bits (`-Σ p log2 p`); `compute_gini` returns the
/// normalized Gini concentration coefficient.
#[pyclass]
pub struct PySegmentEntropy {
    #[pyo3(get)]
    pub segment_id: usize,
    #[pyo3(get)]
    pub shannon_entropy: f64,
    #[pyo3(get)]
    pub gini_coefficient: f64,
    #[pyo3(get)]
    pub member_count: usize,
}

#[pymethods]
impl PySegmentEntropy {
    #[new]
    fn new(segment_id: usize, member_count: usize) -> Self {
        let entropy = SegmentEntropy::new(segment_id, member_count);
        Self {
            segment_id: entropy.segment_id,
            shannon_entropy: entropy.shannon_entropy,
            gini_coefficient: entropy.gini_coefficient,
            member_count: entropy.member_count,
        }
    }

    #[staticmethod]
    fn compute_shannon(frequencies: Vec<usize>) -> f64 {
        SegmentEntropy::compute_shannon(&frequencies)
    }

    #[staticmethod]
    fn compute_gini(frequencies: Vec<usize>) -> f64 {
        SegmentEntropy::compute_gini(&frequencies)
    }

    fn __repr__(&self) -> String {
        format!(
            "SegmentEntropy(id={}, shannon={:.4}, gini={:.4}, members={})",
            self.segment_id, self.shannon_entropy, self.gini_coefficient, self.member_count
        )
    }
}

/// Uniqueness of a segment: `1.0 - similarity_to_nearest`.
#[pyclass]
pub struct PySegmentDifferentiation {
    #[pyo3(get)]
    pub segment_id: usize,
    #[pyo3(get)]
    pub differentiation_score: f64,
    #[pyo3(get)]
    pub similarity_to_nearest: f64,
    #[pyo3(get)]
    pub similarity_to_farthest: f64,
}

#[pymethods]
impl PySegmentDifferentiation {
    #[new]
    fn new(segment_id: usize, similarity_to_nearest: f64, similarity_to_farthest: f64) -> Self {
        let diff =
            SegmentDifferentiation::new(segment_id, similarity_to_nearest, similarity_to_farthest);
        Self {
            segment_id: diff.segment_id,
            differentiation_score: diff.differentiation_score,
            similarity_to_nearest: diff.similarity_to_nearest,
            similarity_to_farthest: diff.similarity_to_farthest,
        }
    }
}

/// Stability of segment assignments from assignment variance and churn rate.
#[pyclass]
pub struct PySegmentPredictability {
    #[pyo3(get)]
    pub segment_id: usize,
    #[pyo3(get)]
    pub predictability_score: f64,
    #[pyo3(get)]
    pub assignment_variance: f64,
    #[pyo3(get)]
    pub churn_rate: f64,
    #[pyo3(get)]
    pub stability_trend: String,
}

#[pymethods]
impl PySegmentPredictability {
    #[new]
    fn new(segment_id: usize, assignment_variance: f64, churn_rate: f64) -> Self {
        let pred = SegmentPredictability::new(segment_id, assignment_variance, churn_rate);
        Self {
            segment_id: pred.segment_id,
            predictability_score: pred.predictability_score,
            assignment_variance: pred.assignment_variance,
            churn_rate: pred.churn_rate,
            stability_trend: pred.stability_trend,
        }
    }
}

/// Tenure statistics and lifecycle stage for a segment. Raises `ValueError`
/// when `tenure_days` is empty.
#[pyclass]
pub struct PySegmentAging {
    #[pyo3(get)]
    pub segment_id: usize,
    #[pyo3(get)]
    pub avg_member_tenure_days: f64,
    #[pyo3(get)]
    pub median_member_tenure_days: f64,
    #[pyo3(get)]
    pub newest_member_days_ago: f64,
    #[pyo3(get)]
    pub oldest_member_days_ago: f64,
    #[pyo3(get)]
    pub member_churn_rate_30d: f64,
    #[pyo3(get)]
    pub segment_lifespan_stage: String,
}

#[pymethods]
impl PySegmentAging {
    #[new]
    fn new(segment_id: usize, tenure_days: Vec<f64>, member_churn_30d: f64) -> PyResult<Self> {
        let aging = SegmentAging::new(segment_id, tenure_days, member_churn_30d)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            segment_id: aging.segment_id,
            avg_member_tenure_days: aging.avg_member_tenure_days,
            median_member_tenure_days: aging.median_member_tenure_days,
            newest_member_days_ago: aging.newest_member_days_ago,
            oldest_member_days_ago: aging.oldest_member_days_ago,
            member_churn_rate_30d: aging.member_churn_rate_30d,
            segment_lifespan_stage: aging.segment_lifespan_stage,
        })
    }
}

// ---------------------------------------------------------------------------
// Temporal analytics (exposed slice)
// ---------------------------------------------------------------------------

/// Linear size forecast for one segment. `predictions` holds
/// `(periods_ahead, predicted_size)` pairs and the confidence bounds are
/// the engine's ±10% band, floored at zero.
#[pyclass]
pub struct PySizeForecast {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub forecast_days: u32,
    #[pyo3(get)]
    pub predictions: Vec<(u32, f64)>,
    #[pyo3(get)]
    pub confidence_interval_lower: Vec<f64>,
    #[pyo3(get)]
    pub confidence_interval_upper: Vec<f64>,
    #[pyo3(get)]
    pub trend_type: String,
    #[pyo3(get)]
    pub r_squared: f64,
}

#[pymethods]
impl PySizeForecast {
    fn __repr__(&self) -> String {
        format!(
            "SizeForecast(days={}, trend={}, r2={:.4})",
            self.forecast_days, self.trend_type, self.r_squared
        )
    }
}

/// Forecast future segment sizes from `(day, size)` history using the same
/// least-squares linear fit as the Rust engine. Needs at least two points.
#[pyfunction]
fn forecast_segment_sizes(
    historical_sizes: Vec<(i32, usize)>,
    forecast_periods: u32,
) -> PyResult<PySizeForecast> {
    let forecast = SegmentSizeForecaster::forecast_sizes(&historical_sizes, forecast_periods)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(PySizeForecast {
        segment_id: forecast.segment_id,
        forecast_days: forecast.forecast_days,
        predictions: forecast.predictions,
        confidence_interval_lower: forecast.confidence_interval_lower,
        confidence_interval_upper: forecast.confidence_interval_upper,
        trend_type: forecast.trend_type,
        r_squared: forecast.r_squared,
    })
}

fn validate_ratio(name: &str, value: f64) -> PyResult<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "{name} must be a finite ratio in [0, 1]"
        )));
    }
    Ok(())
}

/// Projected segment composition (high-value, churn-risk, and new-member
/// ratios) from a current snapshot and historical ratio triples.
#[pyclass]
pub struct PyCompositionForecast {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub forecast_periods: u32,
    #[pyo3(get)]
    pub high_value_ratio_forecast: Vec<f64>,
    #[pyo3(get)]
    pub churn_risk_forecast: Vec<f64>,
    #[pyo3(get)]
    pub new_member_forecast: Vec<f64>,
    #[pyo3(get)]
    pub composition_stability: f64,
}

#[pymethods]
impl PyCompositionForecast {
    fn __repr__(&self) -> String {
        format!(
            "CompositionForecast(periods={}, stability={:.4})",
            self.forecast_periods, self.composition_stability
        )
    }
}

/// Forecast composition ratios using the engine's per-ratio linear trend.
/// Current and historical ratios must be finite values in `[0, 1]`.
#[pyfunction]
fn forecast_composition(
    current_high_value_ratio: f64,
    current_churn_risk_ratio: f64,
    current_new_member_ratio: f64,
    historical_ratios: Vec<(f64, f64, f64)>,
    forecast_periods: u32,
) -> PyResult<PyCompositionForecast> {
    if forecast_periods == 0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "forecast_periods must be greater than zero",
        ));
    }
    validate_ratio("current_high_value_ratio", current_high_value_ratio)?;
    validate_ratio("current_churn_risk_ratio", current_churn_risk_ratio)?;
    validate_ratio("current_new_member_ratio", current_new_member_ratio)?;
    for (index, (high_value, churn_risk, new_member)) in historical_ratios.iter().enumerate() {
        validate_ratio(
            &format!("historical_ratios[{index}].high_value_ratio"),
            *high_value,
        )?;
        validate_ratio(
            &format!("historical_ratios[{index}].churn_risk_ratio"),
            *churn_risk,
        )?;
        validate_ratio(
            &format!("historical_ratios[{index}].new_member_ratio"),
            *new_member,
        )?;
    }

    let forecast = CompositionForecaster::forecast_composition(
        current_high_value_ratio,
        current_churn_risk_ratio,
        current_new_member_ratio,
        &historical_ratios,
        forecast_periods,
    )
    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyCompositionForecast {
        segment_id: forecast.segment_id,
        forecast_periods: forecast.forecast_periods,
        high_value_ratio_forecast: forecast.high_value_ratio_forecast,
        churn_risk_forecast: forecast.churn_risk_forecast,
        new_member_forecast: forecast.new_member_forecast,
        composition_stability: forecast.composition_stability,
    })
}

/// Projected outcome of changing segmentation parameters.
#[pyclass]
pub struct PyWhatIfScenario {
    #[pyo3(get)]
    pub scenario_name: String,
    #[pyo3(get)]
    pub description: String,
    #[pyo3(get)]
    pub parameters_changed: HashMap<String, (String, String)>,
    #[pyo3(get)]
    pub projected_segment_sizes: Vec<(String, usize)>,
    #[pyo3(get)]
    pub projected_revenue_impact: f64,
    #[pyo3(get)]
    pub projected_churn_impact: f64,
    #[pyo3(get)]
    pub feasibility_score: f64,
}

#[pymethods]
impl PyWhatIfScenario {
    fn __repr__(&self) -> String {
        format!(
            "WhatIfScenario(name={}, revenue_impact={:.2}%, feasibility={:.2})",
            self.scenario_name, self.projected_revenue_impact, self.feasibility_score
        )
    }
}

/// Simulate a what-if parameter change against baseline `(segment, size)`
/// pairs. Supported parameters are `rfm_threshold`, `n_clusters`, and
/// `churn_threshold`; unknown keys are retained but do not alter sizes.
#[pyfunction]
fn simulate_what_if(
    scenario_name: &str,
    baseline_sizes: Vec<(String, usize)>,
    parameter_changes: HashMap<String, f64>,
) -> PyResult<PyWhatIfScenario> {
    if scenario_name.trim().is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "scenario_name must not be empty",
        ));
    }
    if baseline_sizes.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "baseline_sizes must not be empty",
        ));
    }
    for (name, value) in &parameter_changes {
        if !value.is_finite() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "parameter {name} must be finite"
            )));
        }
    }
    if let Some(n_clusters) = parameter_changes.get("n_clusters") {
        if *n_clusters < 1.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "n_clusters must be at least 1",
            ));
        }
    }

    let scenario = WhatIfSimulator::simulate_parameter_change(
        scenario_name,
        &baseline_sizes,
        &parameter_changes,
    )
    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyWhatIfScenario {
        scenario_name: scenario.scenario_name,
        description: scenario.description,
        parameters_changed: scenario.parameters_changed,
        projected_segment_sizes: scenario.projected_segment_sizes,
        projected_revenue_impact: scenario.projected_revenue_impact,
        projected_churn_impact: scenario.projected_churn_impact,
        feasibility_score: scenario.feasibility_score,
    })
}

// ---------------------------------------------------------------------------
// Price intelligence (exposed slice)
// ---------------------------------------------------------------------------

/// Average price elasticity across consecutive `(price, quantity)` points.
#[pyclass]
pub struct PyElasticityAnalysis {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub price_elasticity: f64,
    #[pyo3(get)]
    pub elasticity_classification: String,
    #[pyo3(get)]
    pub revenue_sensitivity: f64,
    #[pyo3(get)]
    pub demand_curve: Vec<(f64, f64)>,
    #[pyo3(get)]
    pub optimal_price: f64,
}

#[pymethods]
impl PyElasticityAnalysis {
    fn __repr__(&self) -> String {
        format!(
            "ElasticityAnalysis(segment={}, elasticity={:.4}, class={})",
            self.segment_id, self.price_elasticity, self.elasticity_classification
        )
    }
}

/// Analyze price elasticity from observed `(price, quantity)` pairs.
/// Needs at least two points; zero price changes contribute zero elasticity.
#[pyfunction]
fn analyze_price_elasticity(
    segment_id: &str,
    price_changes: Vec<(f64, f64)>,
) -> PyResult<PyElasticityAnalysis> {
    let analysis = ElasticityCalculator::analyze_elasticity(segment_id, &price_changes)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(PyElasticityAnalysis {
        segment_id: analysis.segment_id,
        price_elasticity: analysis.price_elasticity,
        elasticity_classification: analysis.elasticity_classification,
        revenue_sensitivity: analysis.revenue_sensitivity,
        demand_curve: analysis.demand_curve,
        optimal_price: analysis.optimal_price,
    })
}

/// Discount recommendation and its projected revenue/acquisition impact.
#[pyclass]
pub struct PyDiscountStrategy {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub optimal_discount: f64,
    #[pyo3(get)]
    pub expected_conversion_lift: f64,
    #[pyo3(get)]
    pub revenue_impact: f64,
    #[pyo3(get)]
    pub customer_acquisition_benefit: f64,
    #[pyo3(get)]
    pub recommendation: String,
}

#[pymethods]
impl PyDiscountStrategy {
    fn __repr__(&self) -> String {
        format!(
            "DiscountStrategy(segment={}, discount={:.2}%, lift={:.4})",
            self.segment_id,
            self.optimal_discount * 100.0,
            self.expected_conversion_lift
        )
    }
}

/// Optimize a segment discount from its price sensitivity and customer LTV.
/// `baseline_conversion` is accepted for parity with the engine API but the
/// current formula does not use it.
#[pyfunction]
fn optimize_discount(
    segment_id: &str,
    baseline_conversion: f64,
    price_sensitivity: f64,
    customer_ltv: f64,
) -> PyResult<PyDiscountStrategy> {
    if segment_id.trim().is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "segment_id must not be empty",
        ));
    }
    if !baseline_conversion.is_finite() || !(0.0..=1.0).contains(&baseline_conversion) {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "baseline_conversion must be a finite ratio in [0, 1]",
        ));
    }
    if !price_sensitivity.is_finite() || price_sensitivity < 0.0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "price_sensitivity must be a finite non-negative number",
        ));
    }
    if !customer_ltv.is_finite() || customer_ltv < 0.0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "customer_ltv must be a finite non-negative number",
        ));
    }

    let strategy = DiscountOptimizer::optimize_discount(
        segment_id,
        baseline_conversion,
        price_sensitivity,
        customer_ltv,
    )
    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyDiscountStrategy {
        segment_id: strategy.segment_id,
        optimal_discount: strategy.optimal_discount,
        expected_conversion_lift: strategy.expected_conversion_lift,
        revenue_impact: strategy.revenue_impact,
        customer_acquisition_benefit: strategy.customer_acquisition_benefit,
        recommendation: strategy.recommendation,
    })
}

// ---------------------------------------------------------------------------
// Revenue intelligence (exposed slice)
// ---------------------------------------------------------------------------

/// Return on investment for one segment.
#[pyclass]
pub struct PySegmentROI {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub revenue: f64,
    #[pyo3(get)]
    pub investment: f64,
    #[pyo3(get)]
    pub roi_percentage: f64,
    #[pyo3(get)]
    pub payback_period_days: f64,
    #[pyo3(get)]
    pub profitability_index: f64,
    #[pyo3(get)]
    pub risk_level: String,
}

#[pymethods]
impl PySegmentROI {
    fn __repr__(&self) -> String {
        format!(
            "SegmentROI(segment={}, roi={:.2}%, risk={})",
            self.segment_id, self.roi_percentage, self.risk_level
        )
    }
}

/// Calculate ROI (`(revenue - investment) / investment * 100`) and payback
/// for a segment. `clv` is accepted for parity with the engine API but the
/// current ROI formula does not use it.
#[pyfunction]
fn calculate_segment_roi(
    segment_id: &str,
    revenue: f64,
    investment: f64,
    clv: f64,
) -> PyResult<PySegmentROI> {
    let roi = ROICalculator::calculate_roi(segment_id, revenue, investment, clv)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(PySegmentROI {
        segment_id: roi.segment_id,
        revenue: roi.revenue,
        investment: roi.investment,
        roi_percentage: roi.roi_percentage,
        payback_period_days: roi.payback_period_days,
        profitability_index: roi.profitability_index,
        risk_level: roi.risk_level,
    })
}

/// Revenue attributed to channels, products, and cohorts for one segment.
#[pyclass]
pub struct PyRevenueAttribution {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub channel_attributions: HashMap<String, f64>,
    #[pyo3(get)]
    pub product_attributions: HashMap<String, f64>,
    #[pyo3(get)]
    pub cohort_attributions: HashMap<String, f64>,
    #[pyo3(get)]
    pub first_touch_revenue: f64,
    #[pyo3(get)]
    pub last_touch_revenue: f64,
    #[pyo3(get)]
    pub multi_touch_revenue: f64,
    #[pyo3(get)]
    pub attribution_model: String,
}

#[pymethods]
impl PyRevenueAttribution {
    fn __repr__(&self) -> String {
        format!(
            "RevenueAttribution(segment={}, model={}, multi_touch={:.2})",
            self.segment_id, self.attribution_model, self.multi_touch_revenue
        )
    }
}

/// Break revenue down by channel, product, and cohort. Revenue values must
/// be finite and non-negative.
#[pyfunction]
fn calculate_revenue_attribution(
    segment_id: &str,
    channel_revenue: Vec<(String, f64)>,
    product_revenue: Vec<(String, f64)>,
    cohort_revenue: Vec<(String, f64)>,
) -> PyResult<PyRevenueAttribution> {
    if segment_id.trim().is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "segment_id must not be empty",
        ));
    }
    for (label, rows) in [
        ("channel_revenue", &channel_revenue),
        ("product_revenue", &product_revenue),
        ("cohort_revenue", &cohort_revenue),
    ] {
        for (index, (name, revenue)) in rows.iter().enumerate() {
            if name.trim().is_empty() {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "{label}[{index}] name must not be empty"
                )));
            }
            if !revenue.is_finite() || *revenue < 0.0 {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "{label}[{index}] revenue must be finite and non-negative"
                )));
            }
        }
    }

    let attribution = AttributionEngine::calculate_attribution(
        segment_id,
        &channel_revenue,
        &product_revenue,
        &cohort_revenue,
    )
    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyRevenueAttribution {
        segment_id: attribution.segment_id,
        channel_attributions: attribution.channel_attributions,
        product_attributions: attribution.product_attributions,
        cohort_attributions: attribution.cohort_attributions,
        first_touch_revenue: attribution.first_touch_revenue,
        last_touch_revenue: attribution.last_touch_revenue,
        multi_touch_revenue: attribution.multi_touch_revenue,
        attribution_model: attribution.attribution_model,
    })
}

/// Linear revenue forecast with confidence bounds and an R-squared accuracy.
#[pyclass]
pub struct PyRevenueForecast {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub forecast_periods: u32,
    #[pyo3(get)]
    pub predicted_revenue: Vec<f64>,
    #[pyo3(get)]
    pub confidence_intervals_lower: Vec<f64>,
    #[pyo3(get)]
    pub confidence_intervals_upper: Vec<f64>,
    #[pyo3(get)]
    pub growth_rate: f64,
    #[pyo3(get)]
    pub forecast_accuracy: f64,
}

#[pymethods]
impl PyRevenueForecast {
    fn __repr__(&self) -> String {
        format!(
            "RevenueForecast(periods={}, growth={:.2}%, accuracy={:.4})",
            self.forecast_periods, self.growth_rate, self.forecast_accuracy
        )
    }
}

/// Forecast future revenue from a historical series. Needs at least three
/// finite, non-negative points and a non-zero first point (for the growth
/// rate).
#[pyfunction]
fn forecast_revenue(
    historical_revenue: Vec<f64>,
    forecast_periods: u32,
) -> PyResult<PyRevenueForecast> {
    if historical_revenue
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "historical_revenue must contain finite non-negative values",
        ));
    }
    if historical_revenue.first() == Some(&0.0) {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "first historical_revenue value must be non-zero for a defined growth rate",
        ));
    }

    let forecast = RevenueForecaster::forecast_revenue(&historical_revenue, forecast_periods)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyRevenueForecast {
        segment_id: forecast.segment_id,
        forecast_periods: forecast.forecast_periods,
        predicted_revenue: forecast.predicted_revenue,
        confidence_intervals_lower: forecast.confidence_intervals_lower,
        confidence_intervals_upper: forecast.confidence_intervals_upper,
        growth_rate: forecast.growth_rate,
        forecast_accuracy: forecast.forecast_accuracy,
    })
}

// ---------------------------------------------------------------------------
// Neural networks (exposed slice)
// ---------------------------------------------------------------------------

/// Result of one `NeuralNetwork.train()` call.
#[pyclass]
pub struct PyTrainingStats {
    #[pyo3(get)]
    pub epochs: usize,
    #[pyo3(get)]
    pub final_loss: f64,
    #[pyo3(get)]
    pub samples_trained: usize,
}

#[pymethods]
impl PyTrainingStats {
    fn __repr__(&self) -> String {
        format!(
            "TrainingStats(epochs={}, final_loss={:.6}, samples={})",
            self.epochs, self.final_loss, self.samples_trained
        )
    }
}

/// Small fully-connected regressor: hidden layers with the configured
/// activation, then a linear output layer.
#[pyclass]
pub struct PyNeuralNetwork {
    inner: NeuralNetwork,
}

#[pymethods]
impl PyNeuralNetwork {
    #[new]
    #[pyo3(signature = (input_size, output_size, hidden_layers=None, learning_rate=0.01, epochs=100))]
    fn new(
        input_size: usize,
        output_size: usize,
        hidden_layers: Option<Vec<usize>>,
        learning_rate: f64,
        epochs: usize,
    ) -> PyResult<Self> {
        if input_size == 0 || output_size == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "input_size and output_size must be greater than zero",
            ));
        }
        if hidden_layers
            .as_ref()
            .map(|layers| layers.contains(&0))
            .unwrap_or(false)
        {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "hidden layer sizes must be greater than zero",
            ));
        }
        if !learning_rate.is_finite() || learning_rate <= 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "learning_rate must be a positive finite number",
            ));
        }

        let config = NNConfig {
            hidden_layers: hidden_layers.unwrap_or_else(|| vec![8, 4]),
            learning_rate,
            epochs,
            ..NNConfig::default()
        };
        Ok(Self {
            inner: NeuralNetwork::new(input_size, output_size, config),
        })
    }

    #[getter]
    fn input_size(&self) -> usize {
        self.inner.input_size
    }

    #[getter]
    fn output_size(&self) -> usize {
        self.inner.output_size
    }

    /// Run a forward pass for one sample; raises `ValueError` on a size
    /// mismatch.
    fn predict(&self, input: Vec<f64>) -> PyResult<Vec<f64>> {
        self.inner
            .predict(&input)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Train with mini-batch SGD. `inputs` and `y` must be non-empty and of
    /// equal length; each row must match `input_size`/`output_size`.
    fn train(&mut self, inputs: Vec<Vec<f64>>, y: Vec<Vec<f64>>) -> PyResult<PyTrainingStats> {
        for (i, row) in inputs.iter().enumerate() {
            if row.len() != self.inner.input_size {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "input row {} has {} values, expected {}",
                    i,
                    row.len(),
                    self.inner.input_size
                )));
            }
        }
        for (i, row) in y.iter().enumerate() {
            if row.len() != self.inner.output_size {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "target row {} has {} values, expected {}",
                    i,
                    row.len(),
                    self.inner.output_size
                )));
            }
        }
        let stats = self
            .inner
            .train(&inputs, &y)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(PyTrainingStats {
            epochs: stats.epochs,
            final_loss: stats.final_loss,
            samples_trained: stats.samples_trained,
        })
    }

    fn layer_info(&self) -> Vec<String> {
        self.inner.layer_info()
    }
}

// ---------------------------------------------------------------------------
// Temporal analytics: historical reconstruction and trend momentum
// ---------------------------------------------------------------------------

/// Reconstructed membership of one segment at a past date, replaying
/// `(event_date, event_type, member_id)` events up to (and including) that
/// date. `event_type` is `"added"` or `"removed"`; events on other dates and
/// unknown event types are rejected.
#[pyclass]
pub struct PyHistoricalSegmentState {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub date: String,
    #[pyo3(get)]
    pub members: Vec<String>,
    #[pyo3(get)]
    pub size_trend: Vec<(i32, usize)>,
    #[pyo3(get)]
    pub composition_changes: Vec<(i32, String, usize, String)>,
}

#[pymethods]
impl PyHistoricalSegmentState {
    fn __repr__(&self) -> String {
        format!(
            "HistoricalSegmentState(segment={}, date={}, members={})",
            self.segment_id,
            self.date,
            self.members.len()
        )
    }
}

/// Rebuild a segment's membership as of `target_date` from its event log.
/// Dates are compared as ISO-8601 strings (lexicographic order), so they must
/// be in a sortable `YYYY-MM-DD`-style format.
#[pyfunction]
fn reconstruct_segment_history(
    segment_id: &str,
    target_date: &str,
    historical_events: Vec<(String, String, String)>,
) -> PyResult<PyHistoricalSegmentState> {
    if segment_id.trim().is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "segment_id must not be empty",
        ));
    }
    if target_date.trim().is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "target_date must not be empty",
        ));
    }
    for (index, (event_date, event_type, member_id)) in historical_events.iter().enumerate() {
        if event_date.trim().is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "historical_events[{index}] event_date must not be empty"
            )));
        }
        if event_type != "added" && event_type != "removed" {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "historical_events[{index}] event_type must be 'added' or 'removed'"
            )));
        }
        if member_id.trim().is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "historical_events[{index}] member_id must not be empty"
            )));
        }
    }

    let state =
        HistoricalReconstruction::reconstruct_at_date(segment_id, target_date, &historical_events)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyHistoricalSegmentState {
        segment_id: state.segment_id,
        date: state.date,
        members: state.members,
        size_trend: state.size_trend,
        composition_changes: state
            .composition_changes
            .into_iter()
            .map(|c| (c.days_ago, c.change_type, c.affected_members, c.reason))
            .collect(),
    })
}

/// Momentum read-out for a metric series: recent change, acceleration, and a
/// continuation/reversal split.
#[pyclass]
pub struct PyTrendMomentum {
    #[pyo3(get)]
    pub metric_name: String,
    #[pyo3(get)]
    pub current_value: f64,
    #[pyo3(get)]
    pub momentum_score: f64,
    #[pyo3(get)]
    pub acceleration: f64,
    #[pyo3(get)]
    pub trend_strength: String,
    #[pyo3(get)]
    pub continuation_probability: f64,
    #[pyo3(get)]
    pub reversal_risk: f64,
}

#[pymethods]
impl PyTrendMomentum {
    fn __repr__(&self) -> String {
        format!(
            "TrendMomentum(momentum={:.4}, strength={}, continuation={:.4})",
            self.momentum_score, self.trend_strength, self.continuation_probability
        )
    }
}

/// Measure momentum over `lookback_periods` of a finite metric series. Needs
/// at least three points and a lookback of at least one period.
#[pyfunction]
fn analyze_trend_momentum(
    metric_history: Vec<f64>,
    lookback_periods: usize,
) -> PyResult<PyTrendMomentum> {
    if lookback_periods == 0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "lookback_periods must be greater than zero",
        ));
    }
    if metric_history.iter().any(|value| !value.is_finite()) {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "metric_history must contain finite values",
        ));
    }

    let momentum = MomentumAnalyzer::analyze_momentum(&metric_history, lookback_periods)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyTrendMomentum {
        metric_name: momentum.metric_name,
        current_value: momentum.current_value,
        momentum_score: momentum.momentum_score,
        acceleration: momentum.acceleration,
        trend_strength: momentum.trend_strength,
        continuation_probability: momentum.continuation_probability,
        reversal_risk: momentum.reversal_risk,
    })
}

// ---------------------------------------------------------------------------
// Price intelligence: tier migration and category affinity
// ---------------------------------------------------------------------------

/// Probability of a customer staying in, upgrading from, or downgrading out
/// of a pricing tier.
#[pyclass]
pub struct PyTierMigrationForecast {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub current_tier: String,
    #[pyo3(get)]
    pub tier_migration_probabilities: HashMap<String, f64>,
    #[pyo3(get)]
    pub most_likely_tier: String,
    #[pyo3(get)]
    pub migration_timeline_months: u32,
    #[pyo3(get)]
    pub expansion_likelihood: f64,
    #[pyo3(get)]
    pub churn_risk: f64,
}

#[pymethods]
impl PyTierMigrationForecast {
    fn __repr__(&self) -> String {
        format!(
            "TierMigrationForecast(segment={}, tier={}, most_likely={}, churn={:.4})",
            self.segment_id, self.current_tier, self.most_likely_tier, self.churn_risk
        )
    }
}

/// Predict tier movement from current usage versus historical usage. Needs a
/// non-empty historical series of finite values; a flat history yields a
/// "stay" forecast.
#[pyfunction]
fn predict_tier_migration(
    segment_id: &str,
    current_tier: &str,
    current_usage: f64,
    historical_usage: Vec<f64>,
) -> PyResult<PyTierMigrationForecast> {
    if segment_id.trim().is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "segment_id must not be empty",
        ));
    }
    if current_tier.trim().is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "current_tier must not be empty",
        ));
    }
    if !current_usage.is_finite() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "current_usage must be finite",
        ));
    }
    if historical_usage.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "historical_usage must contain at least one value",
        ));
    }
    if historical_usage.iter().any(|value| !value.is_finite()) {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "historical_usage must contain finite values",
        ));
    }

    let forecast = TierMigrationPredictor::predict_migration(
        segment_id,
        current_tier,
        current_usage,
        &historical_usage,
    )
    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyTierMigrationForecast {
        segment_id: forecast.segment_id,
        current_tier: forecast.current_tier,
        tier_migration_probabilities: forecast.tier_migration_probabilities,
        most_likely_tier: forecast.most_likely_tier,
        migration_timeline_months: forecast.migration_timeline_months,
        expansion_likelihood: forecast.expansion_likelihood,
        churn_risk: forecast.churn_risk,
    })
}

/// Cross-category purchase affinity for one segment.
#[pyclass]
pub struct PyCategoryAffinity {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub category_pairs: Vec<(String, String, f64)>,
    #[pyo3(get)]
    pub affinity_scores: HashMap<String, f64>,
    #[pyo3(get)]
    pub recommended_bundles: Vec<String>,
    #[pyo3(get)]
    pub cross_sell_opportunities: f64,
}

#[pymethods]
impl PyCategoryAffinity {
    fn __repr__(&self) -> String {
        format!(
            "CategoryAffinity(segment={}, categories={}, cross_sell={:.4})",
            self.segment_id,
            self.affinity_scores.len(),
            self.cross_sell_opportunities
        )
    }
}

/// Score category affinity from `(category, purchase_value)` pairs. Needs at
/// least one category with finite, non-negative purchase values.
#[pyfunction]
fn analyze_category_affinity(
    segment_id: &str,
    category_purchases: Vec<(String, f64)>,
) -> PyResult<PyCategoryAffinity> {
    if segment_id.trim().is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "segment_id must not be empty",
        ));
    }
    if category_purchases.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "category_purchases must contain at least one category",
        ));
    }
    for (index, (category, purchases)) in category_purchases.iter().enumerate() {
        if category.trim().is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "category_purchases[{index}] category must not be empty"
            )));
        }
        if !purchases.is_finite() || *purchases < 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "category_purchases[{index}] purchases must be finite and non-negative"
            )));
        }
    }

    let affinity = AffiniCalculator::analyze_category_affinity(segment_id, &category_purchases)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyCategoryAffinity {
        segment_id: affinity.segment_id,
        category_pairs: affinity.category_pairs,
        affinity_scores: affinity.affinity_scores,
        recommended_bundles: affinity.recommended_bundles,
        cross_sell_opportunities: affinity.cross_sell_opportunities,
    })
}

// ---------------------------------------------------------------------------
// Revenue intelligence: customer acquisition cost
// ---------------------------------------------------------------------------

/// Acquisition cost, payback, and efficiency for one segment.
#[pyclass]
pub struct PyCustomerAcquisitionCost {
    #[pyo3(get)]
    pub segment_id: String,
    #[pyo3(get)]
    pub total_acquisition_spend: f64,
    #[pyo3(get)]
    pub customers_acquired: usize,
    #[pyo3(get)]
    pub cac_per_customer: f64,
    #[pyo3(get)]
    pub payback_period_months: f64,
    #[pyo3(get)]
    pub efficiency_score: f64,
}

#[pymethods]
impl PyCustomerAcquisitionCost {
    fn __repr__(&self) -> String {
        format!(
            "CustomerAcquisitionCost(segment={}, cac={:.2}, payback_months={:.2})",
            self.segment_id, self.cac_per_customer, self.payback_period_months
        )
    }
}

/// Calculate CAC (`spend / customers_acquired`) and its payback period in
/// months. Payback is capped at 1000 months when monthly revenue is zero.
#[pyfunction]
fn calculate_cac(
    segment_id: &str,
    acquisition_spend: f64,
    customers_acquired: usize,
    avg_monthly_revenue_per_customer: f64,
) -> PyResult<PyCustomerAcquisitionCost> {
    if segment_id.trim().is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "segment_id must not be empty",
        ));
    }
    if !acquisition_spend.is_finite() || acquisition_spend < 0.0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "acquisition_spend must be finite and non-negative",
        ));
    }
    if customers_acquired == 0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "customers_acquired must be greater than zero",
        ));
    }
    if !avg_monthly_revenue_per_customer.is_finite() || avg_monthly_revenue_per_customer < 0.0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "avg_monthly_revenue_per_customer must be finite and non-negative",
        ));
    }

    let cac = CACCalculator::calculate_cac(
        segment_id,
        acquisition_spend,
        customers_acquired,
        avg_monthly_revenue_per_customer,
    )
    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyCustomerAcquisitionCost {
        segment_id: cac.segment_id,
        total_acquisition_spend: cac.total_acquisition_spend,
        customers_acquired: cac.customers_acquired,
        cac_per_customer: cac.cac_per_customer,
        payback_period_months: cac.payback_period_months,
        efficiency_score: cac.efficiency_score,
    })
}

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
    m.add_class::<PyMiniBatchKMeans>()?;

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
    m.add_class::<PyReclusterConfig>()?;
    m.add_class::<PyReclusterEvent>()?;

    // Drift detection
    m.add_class::<PyFeatureDrift>()?;
    m.add_class::<PySegmentTrend>()?;
    m.add_class::<PyEmergingAudience>()?;
    m.add_class::<PySegmentCompositionChange>()?;
    m.add_function(wrap_pyfunction!(kolmogorov_smirnov, m)?)?;
    m.add_function(wrap_pyfunction!(hellinger_distance, m)?)?;
    m.add_function(wrap_pyfunction!(chi_square_drift, m)?)?;
    m.add_function(wrap_pyfunction!(detect_feature_drift, m)?)?;
    m.add_function(wrap_pyfunction!(detect_segment_composition_change, m)?)?;

    // Segment intelligence: confidence, entropy, differentiation,
    // predictability, and aging metrics
    m.add_class::<PySegmentConfidence>()?;
    m.add_class::<PySegmentEntropy>()?;
    m.add_class::<PySegmentDifferentiation>()?;
    m.add_class::<PySegmentPredictability>()?;
    m.add_class::<PySegmentAging>()?;

    // Temporal analytics
    m.add_class::<PySizeForecast>()?;
    m.add_class::<PyCompositionForecast>()?;
    m.add_class::<PyWhatIfScenario>()?;
    m.add_class::<PyHistoricalSegmentState>()?;
    m.add_class::<PyTrendMomentum>()?;
    m.add_function(wrap_pyfunction!(forecast_segment_sizes, m)?)?;
    m.add_function(wrap_pyfunction!(forecast_composition, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_what_if, m)?)?;
    m.add_function(wrap_pyfunction!(reconstruct_segment_history, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_trend_momentum, m)?)?;

    // Price intelligence
    m.add_class::<PyElasticityAnalysis>()?;
    m.add_class::<PyDiscountStrategy>()?;
    m.add_class::<PyTierMigrationForecast>()?;
    m.add_class::<PyCategoryAffinity>()?;
    m.add_function(wrap_pyfunction!(analyze_price_elasticity, m)?)?;
    m.add_function(wrap_pyfunction!(optimize_discount, m)?)?;
    m.add_function(wrap_pyfunction!(predict_tier_migration, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_category_affinity, m)?)?;

    // Revenue intelligence
    m.add_class::<PySegmentROI>()?;
    m.add_class::<PyRevenueAttribution>()?;
    m.add_class::<PyRevenueForecast>()?;
    m.add_class::<PyCustomerAcquisitionCost>()?;
    m.add_function(wrap_pyfunction!(calculate_segment_roi, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_revenue_attribution, m)?)?;
    m.add_function(wrap_pyfunction!(forecast_revenue, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_cac, m)?)?;

    // Neural networks
    m.add_class::<PyTrainingStats>()?;
    m.add_class::<PyNeuralNetwork>()?;

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
    let info = PyModule::new(py, "info")?;
    info.add(
        "algorithms",
        vec!["kmeans", "kprototypes", "minibatchkmeans"],
    )?;
    info.add(
        "metrics",
        vec!["silhouette", "davies_bouldin", "calinski_harabasz"],
    )?;
    info.add("segments", 13usize)?;
    m.add_submodule(&info)?;

    Ok(())
}
