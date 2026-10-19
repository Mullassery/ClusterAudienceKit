"""Preprocessing pipeline tests with synthetic mixed-type customer data."""

from __future__ import annotations

import numpy as np
import pandas as pd

from clusteraudiencekit import AudienceSegmenter
from clusteraudiencekit.preprocessing import (
    PreprocessingPipeline,
    fit_with_preprocessing,
)


def _data():
    return pd.DataFrame(
        {
            "recency": [1, 30, 400, 2, 50, 380],
            "monetary": [100.0, 50.0, 10.0, 120.0, 40.0, 15.0],
            "channel": ["web", "web", "retail", "web", "retail", "retail"],
        }
    )


def test_preprocessing_scales_and_encodes():
    df = _data()
    pipeline = PreprocessingPipeline(
        numeric_features=["recency", "monetary"],
        categorical_features=["channel"],
    )
    matrix = pipeline.fit_transform(df)
    assert matrix.shape[0] == len(df)
    assert matrix.shape[1] >= 4
    assert np.isfinite(matrix).all()
    # Standardized numeric columns have ~zero mean.
    assert abs(float(matrix[:, 0].mean())) < 1e-9


def test_transform_uses_fitted_statistics():
    df = _data()
    pipeline = PreprocessingPipeline(
        numeric_features=["recency", "monetary"],
        categorical_features=["channel"],
    )
    pipeline.fit(df)
    transformed = pipeline.transform(df.iloc[[0]].assign(recency=9999))
    assert transformed.shape[0] == 1


def test_fit_segmenter_with_preprocessing():
    df = _data()
    segmenter = AudienceSegmenter(n_clusters=2)
    wrapped = fit_with_preprocessing(
        segmenter,
        df,
        numeric_features=["recency", "monetary"],
        categorical_features=["channel"],
    )
    labels = wrapped.predict(df)
    assert len(labels) == len(df)
