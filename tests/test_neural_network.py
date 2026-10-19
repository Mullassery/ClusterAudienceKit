"""Exposed neural-network API: construction, prediction, training, validation."""

from __future__ import annotations

import pytest

from clusteraudiencekit import NeuralNetwork, TrainingStats


def test_untrained_linear_network_has_expected_output_shape():
    net = NeuralNetwork(1, 1, hidden_layers=[], learning_rate=0.05, epochs=1)
    assert net.input_size == 1
    assert net.output_size == 1
    assert net.layer_info() == ["Input: 1", "Layer 0: 1 → 1 [linear]"]
    assert net.predict([1.0]) == pytest.approx([0.1])


def test_training_reduces_loss_and_fits_identity():
    net = NeuralNetwork(1, 1, hidden_layers=[], learning_rate=0.05, epochs=500)
    xs = [[0.0], [1.0], [2.0], [3.0]]
    stats = net.train(xs, xs)

    assert isinstance(stats, TrainingStats)
    assert stats.epochs == 500
    assert stats.samples_trained == 4
    assert stats.final_loss < 1e-3
    for x in (0.0, 1.0, 2.0, 3.0):
        assert net.predict([x])[0] == pytest.approx(x, abs=0.05)


def test_multiple_outputs():
    net = NeuralNetwork(2, 2, hidden_layers=[], learning_rate=0.01, epochs=1)
    assert net.output_size == 2
    assert len(net.predict([1.0, 2.0])) == 2


def test_predict_rejects_wrong_input_size():
    net = NeuralNetwork(2, 1, hidden_layers=[])
    with pytest.raises(ValueError):
        net.predict([1.0])


def test_train_rejects_empty_data():
    net = NeuralNetwork(1, 1, hidden_layers=[])
    with pytest.raises(ValueError):
        net.train([], [])


def test_train_rejects_length_mismatch():
    net = NeuralNetwork(1, 1, hidden_layers=[])
    with pytest.raises(ValueError):
        net.train([[1.0], [2.0]], [[1.0]])


def test_train_rejects_bad_row_widths_instead_of_panicking():
    net = NeuralNetwork(2, 1, hidden_layers=[])
    with pytest.raises(ValueError):
        net.train([[1.0], [2.0]], [[1.0], [2.0]])
    with pytest.raises(ValueError):
        net.train([[1.0, 2.0]], [[1.0, 2.0]])


def test_constructor_validation():
    with pytest.raises(ValueError):
        NeuralNetwork(0, 1)
    with pytest.raises(ValueError):
        NeuralNetwork(1, 0)
    with pytest.raises(ValueError):
        NeuralNetwork(1, 1, hidden_layers=[4, 0])
    with pytest.raises(ValueError):
        NeuralNetwork(1, 1, learning_rate=0.0)
    with pytest.raises(ValueError):
        NeuralNetwork(1, 1, learning_rate=float("nan"))


def test_classes_exported_in_all():
    import clusteraudiencekit as cak

    for name in ("NeuralNetwork", "TrainingStats"):
        assert name in cak.__all__
    assert NeuralNetwork.__name__ == "PyNeuralNetwork"
    assert TrainingStats.__name__ == "PyTrainingStats"
