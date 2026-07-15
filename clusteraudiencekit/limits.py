"""Resource limits to prevent DoS attacks."""

from dataclasses import dataclass
import logging

logger = logging.getLogger(__name__)


@dataclass
class ResourceLimits:
    """Resource constraints for segmentation."""
    
    # Customer count limits
    max_customers: int = 10_000_000
    min_customers: int = 10
    
    # Clustering parameters
    max_clusters: int = 1000
    min_clusters: int = 2
    
    # Memory/processing limits
    max_memory_mb: int = 8192  # 8GB
    max_execution_time_seconds: int = 3600  # 1 hour
    
    # Feature limits
    max_features: int = 100
    min_features: int = 1
    
    @staticmethod
    def validate_customer_count(count: int) -> None:
        """Validate customer count."""
        if count < ResourceLimits.min_customers:
            raise ValueError(f"Too few customers ({count}). Min: {ResourceLimits.min_customers}")
        if count > ResourceLimits.max_customers:
            raise ValueError(f"Too many customers ({count}). Max: {ResourceLimits.max_customers}")
    
    @staticmethod
    def validate_cluster_count(count: int) -> None:
        """Validate number of clusters."""
        if count < ResourceLimits.min_clusters:
            raise ValueError(f"Too few clusters ({count}). Min: {ResourceLimits.min_clusters}")
        if count > ResourceLimits.max_clusters:
            raise ValueError(f"Too many clusters ({count}). Max: {ResourceLimits.max_clusters}")
    
    @staticmethod
    def validate_features_count(count: int) -> None:
        """Validate number of features."""
        if count < ResourceLimits.min_features:
            raise ValueError(f"Too few features ({count}). Min: {ResourceLimits.min_features}")
        if count > ResourceLimits.max_features:
            raise ValueError(f"Too many features ({count}). Max: {ResourceLimits.max_features}")
    
    @staticmethod
    def estimate_memory(customer_count: int, feature_count: int) -> float:
        """
        Estimate memory usage in MB.
        
        Rough estimate: customer_count * feature_count * 8 bytes (float64)
        """
        bytes_used = customer_count * feature_count * 8
        mb_used = bytes_used / (1024 * 1024)
        
        if mb_used > ResourceLimits.max_memory_mb:
            raise ValueError(
                f"Estimated memory usage ({mb_used:.1f}MB) exceeds limit "
                f"({ResourceLimits.max_memory_mb}MB)"
            )
        
        logger.info(f"Estimated memory: {mb_used:.1f}MB")
        return mb_used
    
    @staticmethod
    def estimate_execution_time(customer_count: int, cluster_count: int) -> float:
        """
        Estimate execution time in seconds.
        
        Rough estimate: O(n * k) where n=customers, k=clusters
        Assumes ~0.001s per customer per cluster on modern hardware
        """
        estimated_seconds = (customer_count * cluster_count * 0.001)
        
        if estimated_seconds > ResourceLimits.max_execution_time_seconds:
            raise ValueError(
                f"Estimated execution time ({estimated_seconds:.1f}s) exceeds limit "
                f"({ResourceLimits.max_execution_time_seconds}s)"
            )
        
        logger.info(f"Estimated execution time: {estimated_seconds:.1f}s")
        return estimated_seconds


def validate_resources(customer_count: int, cluster_count: int, feature_count: int) -> None:
    """
    Validate all resource constraints before processing.
    
    Raises:
        ValueError: If any limit is exceeded
    """
    ResourceLimits.validate_customer_count(customer_count)
    ResourceLimits.validate_cluster_count(cluster_count)
    ResourceLimits.validate_features_count(feature_count)
    
    # Estimate and validate memory
    ResourceLimits.estimate_memory(customer_count, feature_count)
    
    # Estimate and validate execution time
    ResourceLimits.estimate_execution_time(customer_count, cluster_count)
