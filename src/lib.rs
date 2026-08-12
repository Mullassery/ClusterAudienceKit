//! ClusterAudienceKit: High-performance audience segmentation engine
//!
//! This library provides fast customer segmentation using RFM (Recency-Frequency-Monetary)
//! analysis combined with clustering algorithms. Written in Rust for performance, with
//! Python bindings for ease of use.
//!
//! # Features
//!
//! - RFM calculation with customizable decay functions
//! - Multiple clustering algorithms (KMeans, K-Prototypes)
//! - Cluster quality metrics (Silhouette, Davies-Bouldin)
//! - Streaming/incremental updates
//! - State persistence
//!
//! # Example
//!
//! ```ignore
//! use clusteraudiencekit::AudienceSegmenter;
//!
//! let mut segmenter = AudienceSegmenter::new(4);
//! // Fit and predict...
//! ```

pub mod engine;
pub mod python;
pub mod utils;

pub use engine::{
    clustering::ClusteringMethod,
    rfm::{DecayFunction, RFMConfig},
    AudienceSegmenterCore,
};

use thiserror::Error;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Custom error type for the library
#[derive(Error, Debug)]
pub enum ClusterClusterAudienceKitError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Data validation error: {0}")]
    DataValidation(String),

    #[error("Clustering error: {0}")]
    ClusteringError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Arrow error: {0}")]
    Arrow(String),
}

pub type Result<T> = std::result::Result<T, ClusterClusterAudienceKitError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
