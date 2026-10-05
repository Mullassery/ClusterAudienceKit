//! Data validation utilities

use crate::Result;

/// Validate configuration parameters
pub fn validate_config(n_clusters: usize, recency_window: u32) -> Result<()> {
    if n_clusters == 0 {
        return Err(crate::ClusterAudienceKitError::InvalidConfig(
            "n_clusters must be >= 1".to_string(),
        ));
    }

    if recency_window == 0 {
        return Err(crate::ClusterAudienceKitError::InvalidConfig(
            "recency_window must be > 0".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_config() {
        assert!(validate_config(4, 90).is_ok());
    }

    #[test]
    fn test_validate_invalid_clusters() {
        assert!(validate_config(0, 90).is_err());
    }

    #[test]
    fn test_validate_invalid_window() {
        assert!(validate_config(4, 0).is_err());
    }
}
