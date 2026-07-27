"""User-friendly error messages for segmentation."""


class SegmentationError:
    """Error with suggestions for fixing."""
    
    def __init__(self, title: str, message: str, fix_steps: list = None):
        self.title = title
        self.message = message
        self.fix_steps = fix_steps or []
    
    def format(self) -> str:
        """Format error."""
        lines = [f"\n❌ {self.title}\n", f"   {self.message}\n"]
        if self.fix_steps:
            lines.append("   🔧 Fix steps:")
            for i, step in enumerate(self.fix_steps, 1):
                lines.append(f"      {i}. {step}")
        return "\n".join(lines)
    
    def __str__(self) -> str:
        return self.format()


# Data errors
MISSING_REQUIRED_COLUMNS = SegmentationError(
    title="Missing Required Transaction Columns",
    message="Data must have: customer_id, transaction_date, amount",
    fix_steps=[
        "Verify your CSV/DataFrame has these exact column names",
        "Column names are case-sensitive",
        "Example structure: df[['customer_id', 'transaction_date', 'amount']]",
        "Use df.rename() to fix column names if needed",
    ]
)

DATA_CONTAINS_NULLS = SegmentationError(
    title="Data Contains Null/Missing Values",
    message="Cannot process data with null values. All rows must be complete.",
    fix_steps=[
        "Drop rows with missing values: df.dropna()",
        "Fill missing values: df.fillna(0) or df.fillna(method='ffill')",
        "Check which columns have nulls: df.isnull().sum()",
        "For transactions, null amounts might indicate incomplete records",
    ]
)

INVALID_AMOUNT_TYPE = SegmentationError(
    title="Invalid Transaction Amount Type",
    message="Transaction amounts must be numeric (int or float).",
    fix_steps=[
        "Convert to numeric: df['amount'] = pd.to_numeric(df['amount'])",
        "Check for non-numeric values: df[~df['amount'].apply(is_numeric)]",
        "Remove or fix invalid rows before segmentation",
    ]
)

TOO_MANY_CUSTOMERS = SegmentationError(
    title="Too Many Customers (Exceeds Limit)",
    message="Dataset has too many customers for processing. Max: 10,000,000",
    fix_steps=[
        "Sample your data: df_sample = df.sample(frac=0.1)",
        "Filter to a subset: df_recent = df[df['date'] > '2025-01-01']",
        "Process by region/cohort separately",
        "Check if you need all customers or can focus on active ones",
    ]
)

TOO_FEW_CUSTOMERS = SegmentationError(
    title="Too Few Customers",
    message="Dataset has too few customers for meaningful segmentation. Min: 10",
    fix_steps=[
        "Ensure you have at least 10 unique customers",
        "Check data filtering - may have removed too much",
        "Current count: Use df['customer_id'].nunique() to check",
    ]
)

# Clustering errors
INVALID_CLUSTER_COUNT = SegmentationError(
    title="Invalid Number of Clusters",
    message="Number of clusters must be between 2 and 1,000",
    fix_steps=[
        "Use reasonable n_clusters based on data size",
        "Rule of thumb: n_clusters = sqrt(n_customers / 2)",
        "For 1M customers: try n_clusters = 700",
        "Too many clusters = over-segmentation",
    ]
)

CLUSTERING_DIVERGED = SegmentationError(
    title="Clustering Algorithm Diverged",
    message="KMeans failed to converge. Try different parameters.",
    fix_steps=[
        "Increase max_iter: AudienceSegmenter(..., max_iter=500)",
        "Use different random_state: AudienceSegmenter(..., random_state=42)",
        "Check data for outliers that prevent convergence",
        "Try fewer clusters: n_clusters = 3 instead of 10",
    ]
)

SILHOUETTE_SCORE_LOW = SegmentationError(
    title="Segmentation Quality Is Low",
    message="Silhouette score indicates clusters are not well separated.",
    fix_steps=[
        f"Current score (0-1): check via segmenter.silhouette_score()",
        "Try different n_clusters - may need more or fewer",
        "Verify data quality - remove outliers if present",
        "Different RFM calculation may help: check settings",
    ]
)


def get_customer_count_error(actual: int, limit: int) -> SegmentationError:
    """Error for customer count exceeding limit."""
    return SegmentationError(
        title=f"Too Many Customers ({actual:,})",
        message=f"Exceeds limit of {limit:,} customers. Dataset is too large.",
        fix_steps=[
            f"Current: {actual:,} customers",
            f"Limit: {limit:,} customers",
            "Sample data to smaller subset",
            "Or process in batches by region",
        ]
    )


def get_memory_error(estimated_mb: float, limit_mb: int) -> SegmentationError:
    """Error for estimated memory usage."""
    return SegmentationError(
        title=f"Estimated Memory Usage Too High ({estimated_mb:.1f}MB)",
        message=f"Exceeds system limit of {limit_mb}MB",
        fix_steps=[
            f"Estimated: {estimated_mb:.1f}MB",
            f"Limit: {limit_mb}MB",
            "Reduce number of customers or features",
            "Use float32 instead of float64 if possible",
        ]
    )
