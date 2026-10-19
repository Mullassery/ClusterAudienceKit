"""Scikit-learn preprocessing for customer segmentation.

The Rust segmenter expects a numeric matrix. Business datasets usually contain
mixed scales and categorical fields, and distance-based clustering on raw
values produces unstable segments. This module builds a fitted sklearn
``ColumnTransformer`` (impute + scale + one-hot) and exposes fit/transform
helpers that hand the Rust core a clean matrix.

scikit-learn/pandas are optional:

    pip install "clusteraudiencekit[preprocessing]"
"""

from __future__ import annotations

from typing import Any, Iterable

import numpy as np


class PreprocessingUnavailable(RuntimeError):
    """Raised when scikit-learn/pandas are not installed."""


def _require_sklearn():
    try:
        from sklearn.compose import ColumnTransformer
        from sklearn.impute import SimpleImputer
        from sklearn.pipeline import Pipeline
        from sklearn.preprocessing import OneHotEncoder, StandardScaler
    except ImportError as exc:
        raise PreprocessingUnavailable(
            "scikit-learn is required for preprocessing. Install with "
            '`pip install "clusteraudiencekit[preprocessing]"`.'
        ) from exc
    return ColumnTransformer, SimpleImputer, Pipeline, OneHotEncoder, StandardScaler


class PreprocessingPipeline:
    """Fit/transform a mixed-type customer dataframe into a numeric matrix."""

    def __init__(
        self,
        numeric_features: Iterable[str] | None = None,
        categorical_features: Iterable[str] | None = None,
        *,
        scale: bool = True,
        encode: bool = True,
        impute: bool = True,
    ):
        self.numeric_features = list(numeric_features or [])
        self.categorical_features = list(categorical_features or [])
        self.scale = scale
        self.encode = encode
        self.impute = impute
        self._transformer: Any = None
        self.feature_names_in_: list[str] = []
        self.feature_names_out_: list[str] = []

    def _build(self):
        ColumnTransformer, SimpleImputer, Pipeline, OneHotEncoder, StandardScaler = (
            _require_sklearn()
        )
        numeric_steps = []
        if self.impute:
            numeric_steps.append(("impute", SimpleImputer(strategy="median")))
        if self.scale:
            numeric_steps.append(("scale", StandardScaler()))
        numeric_pipe = Pipeline(numeric_steps) if numeric_steps else "passthrough"

        categorical_steps = []
        if self.impute:
            categorical_steps.append(
                ("impute", SimpleImputer(strategy="most_frequent"))
            )
        if self.encode:
            categorical_steps.append(
                (
                    "encode",
                    OneHotEncoder(handle_unknown="ignore", sparse_output=False),
                )
            )
        categorical_pipe = Pipeline(categorical_steps) if categorical_steps else "passthrough"

        transformers = []
        if self.numeric_features:
            transformers.append(("numeric", numeric_pipe, self.numeric_features))
        if self.categorical_features:
            transformers.append(("categorical", categorical_pipe, self.categorical_features))
        if not transformers:
            raise ValueError("at least one numeric or categorical feature is required")
        return ColumnTransformer(transformers, remainder="drop", verbose_feature_names_out=False)

    def fit(self, df: Any) -> PreprocessingPipeline:
        self.feature_names_in_ = list(df.columns)
        self._transformer = self._build()
        self._transformer.fit(df)
        try:
            self.feature_names_out_ = list(self._transformer.get_feature_names_out())
        except (AttributeError, ValueError):
            self.feature_names_out_ = []
        return self

    def transform(self, df: Any) -> np.ndarray:
        if self._transformer is None:
            raise RuntimeError("PreprocessingPipeline must be fitted before transform()")
        matrix = self._transformer.transform(df)
        return np.asarray(matrix, dtype=float)

    def fit_transform(self, df: Any) -> np.ndarray:
        return self.fit(df).transform(df)

    def to_matrix(self, df: Any, *, fit: bool = False) -> list[list[float]]:
        matrix = self.fit_transform(df) if fit else self.transform(df)
        return matrix.tolist()


class SegmenterWithPreprocessing:
    """Pair a fitted preprocessor with a Rust ``AudienceSegmenter``."""

    def __init__(self, segmenter: Any, preprocessing: PreprocessingPipeline | None = None):
        self.segmenter = segmenter
        self.preprocessing = preprocessing or PreprocessingPipeline()

    def fit(self, df: Any, **fit_kwargs: Any) -> SegmenterWithPreprocessing:
        matrix = self.preprocessing.fit_transform(df)
        self.segmenter.fit(matrix.tolist(), **fit_kwargs)
        return self

    def predict(self, df: Any) -> Any:
        matrix = self.preprocessing.transform(df)
        return self.segmenter.predict(matrix.tolist())


def fit_with_preprocessing(
    segmenter: Any,
    df: Any,
    *,
    numeric_features: Iterable[str] | None = None,
    categorical_features: Iterable[str] | None = None,
    **fit_kwargs: Any,
) -> SegmenterWithPreprocessing:
    """Convenience: build a preprocessor, fit it, then fit the segmenter."""
    pipeline = SegmenterWithPreprocessing(
        segmenter,
        PreprocessingPipeline(
            numeric_features=numeric_features,
            categorical_features=categorical_features,
        ),
    )
    return pipeline.fit(df, **fit_kwargs)


__all__ = [
    "PreprocessingPipeline",
    "PreprocessingUnavailable",
    "SegmenterWithPreprocessing",
    "fit_with_preprocessing",
]
