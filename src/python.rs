//! Python bindings using PyO3

use pyo3::prelude::*;
use crate::engine::sql_export::{SQLExporter, SQLDialect, ColumnMapping};
use std::collections::HashMap;

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

/// Python module initialization
#[pymodule]
fn clusteraudiencekit(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add("__version__", crate::VERSION)?;
    m.add("__author__", "Georgi Mammen Mullassery")?;

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
