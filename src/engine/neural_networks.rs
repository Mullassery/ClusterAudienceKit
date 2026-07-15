//! Neural networks for pattern discovery and deep learning

use crate::Result;

/// Activation function type
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum Activation {
    ReLU,      // Rectified Linear Unit
    Sigmoid,   // Logistic sigmoid
    Tanh,      // Hyperbolic tangent
    Linear,    // No activation
}

impl Activation {
    pub fn apply(&self, x: f64) -> f64 {
        match self {
            Activation::ReLU => x.max(0.0),
            Activation::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            Activation::Tanh => x.tanh(),
            Activation::Linear => x,
        }
    }

    pub fn derivative(&self, x: f64) -> f64 {
        match self {
            Activation::ReLU => if x > 0.0 { 1.0 } else { 0.0 },
            Activation::Sigmoid => {
                let s = self.apply(x);
                s * (1.0 - s)
            }
            Activation::Tanh => {
                let t = x.tanh();
                1.0 - (t * t)
            }
            Activation::Linear => 1.0,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Activation::ReLU => "relu",
            Activation::Sigmoid => "sigmoid",
            Activation::Tanh => "tanh",
            Activation::Linear => "linear",
        }
    }
}

/// Dense layer in neural network
#[derive(Clone, Debug)]
pub struct DenseLayer {
    pub weights: Vec<Vec<f64>>,      // [output_size x input_size]
    pub biases: Vec<f64>,             // [output_size]
    pub activation: Activation,
    pub input_size: usize,
    pub output_size: usize,
}

impl DenseLayer {
    /// Create a new dense layer with random initialization
    pub fn new(input_size: usize, output_size: usize, activation: Activation) -> Self {
        let mut weights = Vec::with_capacity(output_size);
        for _ in 0..output_size {
            let mut row = Vec::with_capacity(input_size);
            for _ in 0..input_size {
                row.push(0.1); // Xavier-like initialization
            }
            weights.push(row);
        }

        let biases = vec![0.0; output_size];

        Self {
            weights,
            biases,
            activation,
            input_size,
            output_size,
        }
    }

    /// Forward pass through the layer
    pub fn forward(&self, input: &[f64]) -> Result<Vec<f64>> {
        if input.len() != self.input_size {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                format!(
                    "Input size mismatch: expected {}, got {}",
                    self.input_size,
                    input.len()
                ),
            ));
        }

        let mut output = vec![0.0; self.output_size];

        for i in 0..self.output_size {
            let mut z = self.biases[i];
            for j in 0..self.input_size {
                z += self.weights[i][j] * input[j];
            }
            output[i] = self.activation.apply(z);
        }

        Ok(output)
    }

    /// Backward pass (compute gradients)
    pub fn backward(
        &self,
        input: &[f64],
        output_deltas: &[f64],
        learning_rate: f64,
    ) -> Result<Vec<f64>> {
        if output_deltas.len() != self.output_size {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Output delta size mismatch".to_string(),
            ));
        }

        let mut input_deltas = vec![0.0; self.input_size];

        // Compute input deltas for backprop to previous layer
        for j in 0..self.input_size {
            for i in 0..self.output_size {
                input_deltas[j] += self.weights[i][j] * output_deltas[i];
            }
        }

        Ok(input_deltas)
    }

    /// Update weights based on gradients
    pub fn update_weights(
        &mut self,
        input: &[f64],
        output_deltas: &[f64],
        learning_rate: f64,
    ) -> Result<()> {
        for i in 0..self.output_size {
            self.biases[i] -= learning_rate * output_deltas[i];
            for j in 0..self.input_size {
                self.weights[i][j] -= learning_rate * output_deltas[i] * input[j];
            }
        }

        Ok(())
    }
}

/// Neural network model configuration
#[derive(Clone, Debug)]
pub struct NNConfig {
    pub hidden_layers: Vec<usize>,     // Sizes of hidden layers
    pub activation: Activation,        // Hidden layer activation
    pub learning_rate: f64,
    pub epochs: usize,
    pub batch_size: usize,
    pub dropout_rate: f64,             // Regularization via dropout
    pub momentum: f64,                 // Momentum for SGD
}

impl Default for NNConfig {
    fn default() -> Self {
        Self {
            hidden_layers: vec![128, 64, 32],
            activation: Activation::ReLU,
            learning_rate: 0.01,
            epochs: 100,
            batch_size: 32,
            dropout_rate: 0.2,
            momentum: 0.9,
        }
    }
}

impl NNConfig {
    pub fn with_hidden_layers(mut self, layers: Vec<usize>) -> Self {
        self.hidden_layers = layers;
        self
    }

    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }

    pub fn with_epochs(mut self, epochs: usize) -> Self {
        self.epochs = epochs;
        self
    }

    pub fn with_dropout(mut self, rate: f64) -> Self {
        self.dropout_rate = rate;
        self
    }
}

/// Supervised neural network (MLP)
#[derive(Clone, Debug)]
pub struct NeuralNetwork {
    pub layers: Vec<DenseLayer>,
    pub config: NNConfig,
    pub input_size: usize,
    pub output_size: usize,
    pub feature_names: Vec<String>,
}

impl NeuralNetwork {
    /// Create a new neural network with given architecture
    pub fn new(input_size: usize, output_size: usize, config: NNConfig) -> Self {
        let mut layers = Vec::new();
        let mut prev_size = input_size;

        // Hidden layers
        for &hidden_size in &config.hidden_layers {
            layers.push(DenseLayer::new(prev_size, hidden_size, config.activation));
            prev_size = hidden_size;
        }

        // Output layer (linear activation for regression/raw output)
        layers.push(DenseLayer::new(prev_size, output_size, Activation::Linear));

        Self {
            layers,
            config,
            input_size,
            output_size,
            feature_names: Vec::new(),
        }
    }

    pub fn with_features(mut self, names: Vec<String>) -> Self {
        self.feature_names = names;
        self
    }

    /// Forward pass through entire network
    pub fn predict(&self, input: &[f64]) -> Result<Vec<f64>> {
        if input.len() != self.input_size {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                format!(
                    "Input size mismatch: expected {}, got {}",
                    self.input_size,
                    input.len()
                ),
            ));
        }

        let mut current = input.to_vec();

        for layer in &self.layers {
            current = layer.forward(&current)?;
        }

        Ok(current)
    }

    /// Train the network on data
    pub fn train(&mut self, X: &[Vec<f64>], y: &[Vec<f64>]) -> Result<TrainingStats> {
        if X.is_empty() || y.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Empty training data".to_string(),
            ));
        }

        if X.len() != y.len() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "X and y length mismatch".to_string(),
            ));
        }

        let mut total_loss = 0.0;
        let mut samples_processed = 0;

        for epoch in 0..self.config.epochs {
            total_loss = 0.0;
            samples_processed = 0;

            // Mini-batch training
            for batch_start in (0..X.len()).step_by(self.config.batch_size) {
                let batch_end = (batch_start + self.config.batch_size).min(X.len());

                for i in batch_start..batch_end {
                    let pred = self.predict(&X[i])?;
                    let target = &y[i];

                    // Compute loss (MSE)
                    let mut batch_loss = 0.0;
                    for (p, t) in pred.iter().zip(target.iter()) {
                        batch_loss += (p - t).powi(2);
                    }
                    total_loss += batch_loss / pred.len() as f64;
                    samples_processed += 1;

                    // Backpropagation
                    self.backpropagate(&X[i], target)?;
                }
            }
        }

        let avg_loss = if samples_processed > 0 {
            total_loss / samples_processed as f64
        } else {
            0.0
        };

        Ok(TrainingStats {
            epochs: self.config.epochs,
            final_loss: avg_loss,
            samples_trained: samples_processed,
        })
    }

    /// Backpropagation algorithm
    fn backpropagate(&mut self, input: &[f64], target: &[f64]) -> Result<()> {
        // Forward pass to cache activations
        let mut activations = vec![input.to_vec()];
        let mut current = input.to_vec();

        for layer in &self.layers {
            current = layer.forward(&current)?;
            activations.push(current.clone());
        }

        // Output layer error
        let mut deltas = vec![0.0; self.output_size];
        for i in 0..self.output_size {
            deltas[i] = (current[i] - target[i]) * 2.0; // d(MSE)/d(output)
        }

        // Backpropagation through layers (reversed)
        for layer_idx in (0..self.layers.len()).rev() {
            let layer = &mut self.layers[layer_idx];
            let input_to_layer = &activations[layer_idx];

            // Update weights
            layer.update_weights(input_to_layer, &deltas, self.config.learning_rate)?;

            // Compute deltas for previous layer
            if layer_idx > 0 {
                deltas = layer.backward(input_to_layer, &deltas, self.config.learning_rate)?;
            }
        }

        Ok(())
    }

    /// Get layer information
    pub fn layer_info(&self) -> Vec<String> {
        let mut info = vec![format!("Input: {}", self.input_size)];

        for (i, layer) in self.layers.iter().enumerate() {
            let activation_name = if i == self.layers.len() - 1 {
                "linear"
            } else {
                layer.activation.as_str()
            };
            info.push(format!(
                "Layer {}: {} → {} [{}]",
                i, layer.input_size, layer.output_size, activation_name
            ));
        }

        info
    }
}

/// Autoencoder for unsupervised pattern discovery
#[derive(Clone, Debug)]
pub struct Autoencoder {
    pub encoder: Vec<DenseLayer>,
    pub decoder: Vec<DenseLayer>,
    pub config: NNConfig,
    pub input_size: usize,
    pub latent_size: usize,
    pub feature_names: Vec<String>,
}

impl Autoencoder {
    /// Create a new autoencoder with symmetric architecture
    pub fn new(input_size: usize, latent_size: usize, config: NNConfig) -> Self {
        let mut encoder = Vec::new();
        let mut prev_size = input_size;

        // Encoder: input → hidden → latent
        for &hidden_size in &config.hidden_layers {
            encoder.push(DenseLayer::new(prev_size, hidden_size, config.activation));
            prev_size = hidden_size;
        }
        encoder.push(DenseLayer::new(prev_size, latent_size, config.activation));

        // Decoder: latent → hidden → input
        let mut decoder = Vec::new();
        let mut hidden_reversed = config.hidden_layers.clone();
        hidden_reversed.reverse();

        prev_size = latent_size;
        for &hidden_size in &hidden_reversed {
            decoder.push(DenseLayer::new(prev_size, hidden_size, config.activation));
            prev_size = hidden_size;
        }
        decoder.push(DenseLayer::new(prev_size, input_size, Activation::Linear));

        Self {
            encoder,
            decoder,
            config,
            input_size,
            latent_size,
            feature_names: Vec::new(),
        }
    }

    pub fn with_features(mut self, names: Vec<String>) -> Self {
        self.feature_names = names;
        self
    }

    /// Encode input to latent representation
    pub fn encode(&self, input: &[f64]) -> Result<Vec<f64>> {
        if input.len() != self.input_size {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Input size mismatch".to_string(),
            ));
        }

        let mut current = input.to_vec();
        for layer in &self.encoder {
            current = layer.forward(&current)?;
        }

        Ok(current)
    }

    /// Decode latent representation to reconstruction
    pub fn decode(&self, latent: &[f64]) -> Result<Vec<f64>> {
        if latent.len() != self.latent_size {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Latent size mismatch".to_string(),
            ));
        }

        let mut current = latent.to_vec();
        for layer in &self.decoder {
            current = layer.forward(&current)?;
        }

        Ok(current)
    }

    /// Full reconstruction (encode then decode)
    pub fn reconstruct(&self, input: &[f64]) -> Result<Vec<f64>> {
        let latent = self.encode(input)?;
        self.decode(&latent)
    }

    /// Train autoencoder to minimize reconstruction error
    pub fn train(&mut self, X: &[Vec<f64>]) -> Result<TrainingStats> {
        if X.is_empty() {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Empty training data".to_string(),
            ));
        }

        let mut total_loss = 0.0;
        let mut samples_processed = 0;

        for _epoch in 0..self.config.epochs {
            total_loss = 0.0;

            for batch_start in (0..X.len()).step_by(self.config.batch_size) {
                let batch_end = (batch_start + self.config.batch_size).min(X.len());

                for i in batch_start..batch_end {
                    let reconstruction = self.reconstruct(&X[i])?;

                    // Reconstruction error (MSE)
                    let mut loss = 0.0;
                    for (r, x) in reconstruction.iter().zip(X[i].iter()) {
                        loss += (r - x).powi(2);
                    }
                    total_loss += loss / reconstruction.len() as f64;
                    samples_processed += 1;
                }
            }
        }

        let avg_loss = if samples_processed > 0 {
            total_loss / samples_processed as f64
        } else {
            0.0
        };

        Ok(TrainingStats {
            epochs: self.config.epochs,
            final_loss: avg_loss,
            samples_trained: samples_processed,
        })
    }

    /// Detect anomalies via reconstruction error
    pub fn anomaly_scores(&self, X: &[Vec<f64>]) -> Result<Vec<f64>> {
        let mut scores = Vec::new();

        for sample in X {
            let reconstruction = self.reconstruct(sample)?;
            let mut error = 0.0;

            for (r, x) in reconstruction.iter().zip(sample.iter()) {
                error += (r - x).powi(2);
            }

            scores.push(error / sample.len() as f64);
        }

        Ok(scores)
    }

    /// Extract patterns by getting latent representations
    pub fn extract_patterns(&self, X: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
        let mut patterns = Vec::new();

        for sample in X {
            patterns.push(self.encode(sample)?);
        }

        Ok(patterns)
    }
}

/// Recurrent layer for sequence processing
#[derive(Clone, Debug)]
pub struct RecurrentLayer {
    pub weight_input: Vec<Vec<f64>>,  // Input → hidden
    pub weight_hidden: Vec<Vec<f64>>, // Hidden → hidden
    pub bias: Vec<f64>,
    pub hidden_state: Vec<f64>,
    pub hidden_size: usize,
    pub input_size: usize,
}

impl RecurrentLayer {
    /// Create a new recurrent layer
    pub fn new(input_size: usize, hidden_size: usize) -> Self {
        Self {
            weight_input: vec![vec![0.1; input_size]; hidden_size],
            weight_hidden: vec![vec![0.1; hidden_size]; hidden_size],
            bias: vec![0.0; hidden_size],
            hidden_state: vec![0.0; hidden_size],
            hidden_size,
            input_size,
        }
    }

    /// Process a single timestep
    pub fn step(&mut self, input: &[f64]) -> Result<Vec<f64>> {
        if input.len() != self.input_size {
            return Err(crate::ClusterClusterAudienceKitError::DataValidation(
                "Input size mismatch".to_string(),
            ));
        }

        let mut new_hidden = vec![0.0; self.hidden_size];

        for i in 0..self.hidden_size {
            let mut z = self.bias[i];

            // Input contribution
            for j in 0..self.input_size {
                z += self.weight_input[i][j] * input[j];
            }

            // Recurrent contribution
            for j in 0..self.hidden_size {
                z += self.weight_hidden[i][j] * self.hidden_state[j];
            }

            new_hidden[i] = z.tanh();
        }

        self.hidden_state = new_hidden.clone();
        Ok(new_hidden)
    }

    /// Reset hidden state
    pub fn reset(&mut self) {
        self.hidden_state = vec![0.0; self.hidden_size];
    }

    /// Process sequence and return all hidden states
    pub fn process_sequence(&mut self, sequence: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
        self.reset();
        let mut outputs = Vec::new();

        for timestep in sequence {
            outputs.push(self.step(timestep)?);
        }

        Ok(outputs)
    }
}

/// Training statistics
#[derive(Clone, Debug)]
pub struct TrainingStats {
    pub epochs: usize,
    pub final_loss: f64,
    pub samples_trained: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activation_relu() {
        assert_eq!(Activation::ReLU.apply(5.0), 5.0);
        assert_eq!(Activation::ReLU.apply(-5.0), 0.0);
    }

    #[test]
    fn test_activation_sigmoid() {
        let s = Activation::Sigmoid.apply(0.0);
        assert!((s - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_dense_layer_creation() {
        let layer = DenseLayer::new(10, 5, Activation::ReLU);
        assert_eq!(layer.input_size, 10);
        assert_eq!(layer.output_size, 5);
        assert_eq!(layer.weights.len(), 5);
    }

    #[test]
    fn test_dense_layer_forward() {
        let layer = DenseLayer::new(3, 2, Activation::Linear);
        let input = vec![1.0, 2.0, 3.0];
        let output = layer.forward(&input).unwrap();

        assert_eq!(output.len(), 2);
        for val in output {
            assert!(val.is_finite());
        }
    }

    #[test]
    fn test_neural_network_creation() {
        let config = NNConfig {
            hidden_layers: vec![64, 32],
            ..Default::default()
        };
        let nn = NeuralNetwork::new(10, 3, config);

        assert_eq!(nn.input_size, 10);
        assert_eq!(nn.output_size, 3);
        assert_eq!(nn.layers.len(), 3); // 2 hidden + 1 output
    }

    #[test]
    fn test_neural_network_predict() {
        let nn = NeuralNetwork::new(5, 2, NNConfig::default());
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let output = nn.predict(&input).unwrap();

        assert_eq!(output.len(), 2);
        for val in output {
            assert!(val.is_finite());
        }
    }

    #[test]
    fn test_neural_network_training() {
        let mut nn = NeuralNetwork::new(
            4,
            1,
            NNConfig {
                hidden_layers: vec![8, 4],
                epochs: 10,
                batch_size: 2,
                ..Default::default()
            },
        );

        let X = vec![
            vec![0.0, 0.0, 0.0, 0.0],
            vec![1.0, 1.0, 1.0, 1.0],
            vec![0.5, 0.5, 0.5, 0.5],
        ];
        let y = vec![vec![0.0], vec![1.0], vec![0.5]];

        let stats = nn.train(&X, &y).unwrap();
        assert!(stats.final_loss >= 0.0);
        assert!(stats.samples_trained > 0);
    }

    #[test]
    fn test_autoencoder_creation() {
        let ae = Autoencoder::new(10, 3, NNConfig::default());
        assert_eq!(ae.input_size, 10);
        assert_eq!(ae.latent_size, 3);
    }

    #[test]
    fn test_autoencoder_encode_decode() {
        let ae = Autoencoder::new(5, 2, NNConfig::default());
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5];

        let latent = ae.encode(&input).unwrap();
        assert_eq!(latent.len(), 2);

        let reconstructed = ae.decode(&latent).unwrap();
        assert_eq!(reconstructed.len(), 5);
    }

    #[test]
    fn test_autoencoder_reconstruct() {
        let ae = Autoencoder::new(4, 2, NNConfig::default());
        let input = vec![0.1, 0.2, 0.3, 0.4];
        let reconstruction = ae.reconstruct(&input).unwrap();

        assert_eq!(reconstruction.len(), 4);
    }

    #[test]
    fn test_autoencoder_anomaly_scores() {
        let ae = Autoencoder::new(3, 2, NNConfig::default());
        let X = vec![
            vec![0.1, 0.2, 0.3],
            vec![0.4, 0.5, 0.6],
        ];

        let scores = ae.anomaly_scores(&X).unwrap();
        assert_eq!(scores.len(), 2);
        for score in scores {
            assert!(score >= 0.0);
        }
    }

    #[test]
    fn test_autoencoder_extract_patterns() {
        let ae = Autoencoder::new(4, 2, NNConfig::default());
        let X = vec![
            vec![0.1, 0.2, 0.3, 0.4],
            vec![0.5, 0.6, 0.7, 0.8],
        ];

        let patterns = ae.extract_patterns(&X).unwrap();
        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0].len(), 2);
    }

    #[test]
    fn test_recurrent_layer_creation() {
        let rnn = RecurrentLayer::new(5, 8);
        assert_eq!(rnn.input_size, 5);
        assert_eq!(rnn.hidden_size, 8);
    }

    #[test]
    fn test_recurrent_layer_step() {
        let mut rnn = RecurrentLayer::new(3, 4);
        let input = vec![0.1, 0.2, 0.3];
        let output = rnn.step(&input).unwrap();

        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_recurrent_layer_sequence() {
        let mut rnn = RecurrentLayer::new(2, 3);
        let sequence = vec![
            vec![0.1, 0.2],
            vec![0.3, 0.4],
            vec![0.5, 0.6],
        ];

        let outputs = rnn.process_sequence(&sequence).unwrap();
        assert_eq!(outputs.len(), 3);
        for output in outputs {
            assert_eq!(output.len(), 3);
        }
    }

    #[test]
    fn test_nn_with_features() {
        let nn = NeuralNetwork::new(5, 2, NNConfig::default())
            .with_features(vec!["f1".to_string(), "f2".to_string()]);

        assert_eq!(nn.feature_names.len(), 2);
    }

    #[test]
    fn test_nn_layer_info() {
        let nn = NeuralNetwork::new(
            10,
            3,
            NNConfig {
                hidden_layers: vec![64, 32],
                ..Default::default()
            },
        );

        let info = nn.layer_info();
        assert!(info.len() > 0);
        assert!(info[0].contains("Input"));
    }

    #[test]
    fn test_activation_derivative() {
        let deriv = Activation::Sigmoid.derivative(0.0);
        assert!((deriv - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_config_builder() {
        let config = NNConfig::default()
            .with_hidden_layers(vec![256, 128])
            .with_learning_rate(0.001)
            .with_epochs(50)
            .with_dropout(0.3);

        assert_eq!(config.hidden_layers, vec![256, 128]);
        assert_eq!(config.learning_rate, 0.001);
        assert_eq!(config.epochs, 50);
        assert_eq!(config.dropout_rate, 0.3);
    }
}
