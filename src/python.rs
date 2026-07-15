//! Python bindings using PyO3

use pyo3::prelude::*;

/// Python module initialization
#[pymodule]
fn clusteraudiencekit(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add("__version__", crate::VERSION)?;
    m.add("__author__", "Georgi Mammen Mullassery")?;

    // Add module info
    let info = PyModule::new_bound(py, "info")?;
    info.add("algorithms", vec!["kmeans", "dbscan", "hierarchical", "gmm"])?;
    info.add("metrics", vec!["silhouette", "davies_bouldin", "calinski_harabasz"])?;
    info.add("segments", 13usize)?;
    m.add_submodule(&info)?;

    Ok(())
}
