"""Type stubs for ClusterAudienceKit."""

from typing import Optional, Literal
import pandas as pd

class AudienceSegmenter:
    def __init__(
        self,
        method: Literal["rfm_kmeans", "rfm_kprototypes", "kmeans_only"] = "rfm_kmeans",
        n_clusters: int = 4,
        recency_window_days: int = 90,
        decay_function: Literal["linear", "exponential", "inverse"] = "linear",
        decay_half_life_days: int = 30,
        frequency_threshold: int = 1,
        monetary_threshold: float = 0.0,
        random_state: int = 42,
        n_jobs: int = -1,
    ) -> None: ...

    def fit(
        self,
        df: pd.DataFrame,
        date_column: str = "transaction_date",
        customer_column: str = "customer_id",
        transaction_column: str = "transaction_amount",
        categorical_columns: Optional[list[str]] = None,
    ) -> "AudienceSegmenter": ...

    def predict(
        self,
        df: pd.DataFrame,
        date_column: str = "transaction_date",
        customer_column: str = "customer_id",
        transaction_column: str = "transaction_amount",
    ) -> pd.Series: ...

    def fit_predict(
        self,
        df: pd.DataFrame,
        date_column: str = "transaction_date",
        customer_column: str = "customer_id",
        transaction_column: str = "transaction_amount",
    ) -> pd.Series: ...

    def transform(
        self,
        df: pd.DataFrame,
        date_column: str = "transaction_date",
        customer_column: str = "customer_id",
        transaction_column: str = "transaction_amount",
    ) -> pd.DataFrame: ...

    def segment_profiles(self) -> pd.DataFrame: ...

    def silhouette_score(self) -> float: ...

    def davies_bouldin_score(self) -> float: ...

    def inertia(self) -> float: ...

    def update(
        self,
        df: pd.DataFrame,
        date_column: str = "transaction_date",
        customer_column: str = "customer_id",
        transaction_column: str = "transaction_amount",
        refit: bool = False,
    ) -> "AudienceSegmenter": ...

    def segment_stability(self, previous_segments: pd.Series) -> float: ...

    def save_state(self, path: str) -> None: ...

    def load_state(self, path: str) -> None: ...

    def get_config(self) -> dict: ...
