"""Input validation for safe processing."""

from pydantic import BaseModel, field_validator, Field
from typing import Optional
import pandas as pd


class SegmentationRequest(BaseModel):
    """Validated segmentation request."""
    
    n_clusters: int = Field(ge=2, le=1000, description="Number of clusters (2-1000)")
    method: str = Field(pattern="^(rfm_kmeans|kprototypes)$", description="Segmentation method")
    random_state: Optional[int] = Field(default=42, ge=0, le=2**31-1)
    
    @field_validator('n_clusters')
    def validate_clusters(cls, v):
        if not 2 <= v <= 1000:
            raise ValueError('n_clusters must be between 2 and 1000')
        return v
    
    @field_validator('method')
    def validate_method(cls, v):
        if v not in ['rfm_kmeans', 'kprototypes']:
            raise ValueError('method must be rfm_kmeans or kprototypes')
        return v


def validate_transaction_data(df: pd.DataFrame) -> bool:
    """Validate transaction dataframe structure."""
    required_cols = {'customer_id', 'transaction_date', 'amount'}
    if not required_cols.issubset(df.columns):
        raise ValueError(f"Missing required columns: {required_cols - set(df.columns)}")
    
    # Check for NaN/null values
    if df.isnull().any().any():
        raise ValueError("Data contains null values")
    
    # Check for valid data types
    if not pd.api.types.is_numeric_dtype(df['amount']):
        raise ValueError("'amount' column must be numeric")
    
    if not pd.api.types.is_datetime64_any_dtype(df['transaction_date']):
        raise ValueError("'transaction_date' column must be datetime")
    
    return True


def validate_customer_count(df: pd.DataFrame, max_customers: int = 10_000_000):
    """Validate customer count doesn't exceed limits."""
    customer_count = df['customer_id'].nunique()
    if customer_count > max_customers:
        raise ValueError(f"Too many customers ({customer_count}). Max: {max_customers}")
    return True
