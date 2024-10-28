use rand::Rng;

pub struct NeuralNetwork {
    // Weights and biases for each layer
    weights1: Vec<Vec<f32>>, // Input to first hidden layer
    biases1: Vec<f32>,
    weights2: Vec<Vec<f32>>, // First hidden to second hidden layer
    biases2: Vec<f32>,
    weights3: Vec<Vec<f32>>, // Second hidden to third hidden layer
    biases3: Vec<f32>,
    weights4: Vec<Vec<f32>>, // Third hidden to output layer
    biases4: Vec<f32>,
}

impl NeuralNetwork {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        
        // Initialize weights and biases with random values
        let mut weights1 = vec![vec![0.0; hidden_size]; input_size];
        let mut weights2 = vec![vec![0.0; hidden_size]; hidden_size];
        let mut weights3 = vec![vec![0.0; hidden_size]; hidden_size];
        let mut weights4 = vec![vec![0.0; output_size]; hidden_size];

        let biases1 = vec![0.0; hidden_size];
        let biases2 = vec![0.0; hidden_size];
        let biases3 = vec![0.0; hidden_size];
        let biases4 = vec![0.0; output_size];

        // Randomize initial weights
        for i in 0..input_size {
            for j in 0..hidden_size {
                weights1[i][j] = rng.gen::<f32>() - 0.5;
            }
        }
        for i in 0..hidden_size {
            for j in 0..hidden_size {
                weights2[i][j] = rng.gen::<f32>() - 0.5;
                weights3[i][j] = rng.gen::<f32>() - 0.5;
            }
        }
        for i in 0..hidden_size {
            for j in 0..output_size {
                weights4[i][j] = rng.gen::<f32>() - 0.5;
            }
        }

        Self { weights1, biases1, weights2, biases2, weights3, biases3, weights4, biases4 }
    }

    // Forward propagation with three hidden layers
    pub fn forward(&self, input: &Vec<f32>) -> Vec<f32> {
        let hidden_output1 = self.activate(&self.layer_forward(&input, &self.weights1, &self.biases1));
        let hidden_output2 = self.activate(&self.layer_forward(&hidden_output1, &self.weights2, &self.biases2));
        let hidden_output3 = self.activate(&self.layer_forward(&hidden_output2, &self.weights3, &self.biases3));
        let output = self.softmax(&self.layer_forward(&hidden_output3, &self.weights4, &self.biases4));
        output
    }

    // Layer forward function: Multiply input with weights and add biases
    fn layer_forward(&self, input: &Vec<f32>, weights: &Vec<Vec<f32>>, biases: &Vec<f32>) -> Vec<f32> {
        let mut output = vec![0.0; biases.len()];

        for j in 0..output.len() {
            output[j] = biases[j];
            for i in 0..input.len() {
                output[j] += input[i] * weights[i][j];
            }
        }
        output
    }

    // ReLU activation function
    fn activate(&self, layer_output: &Vec<f32>) -> Vec<f32> {
        let mut output = vec![0.0; layer_output.len()];
        for i in 0..layer_output.len() {
            output[i] = if layer_output[i] > 0.0 { layer_output[i] } else { 0.0 };
        }
        output
    }

    // Softmax function for output layer
    pub fn softmax(&self, layer_output: &Vec<f32>) -> Vec<f32> {
        let mut exp_sum = 0.0;
        for i in 0..layer_output.len() {
            exp_sum += layer_output[i].exp();
        }

        let mut output = vec![0.0; layer_output.len()];
        for i in 0..layer_output.len() {
            output[i] = layer_output[i].exp() / exp_sum;
        }
        output
    }
}


/////// TRAINING ///////


impl NeuralNetwork {
    // Mean Squared Error (MSE) Loss
    pub fn calculate_loss(&self, output: &Vec<f32>, target: &Vec<f32>) -> f32 {
        let mut sum = 0.0;
        for i in 0..output.len() {
            sum += (output[i] - target[i]).powi(2);
        }
        sum / output.len() as f32
    }

    // Backpropagation to update weights and biases
    pub fn train(&mut self, input: &Vec<f32>, target: &Vec<f32>, learning_rate: f32) {
        // Forward pass
        let hidden1_output = self.activate(&self.layer_forward(&input, &self.weights1, &self.biases1));
        let hidden2_output = self.activate(&self.layer_forward(&hidden1_output, &self.weights2, &self.biases2));
        let hidden3_output = self.activate(&self.layer_forward(&hidden2_output, &self.weights3, &self.biases3));
        let output = self.softmax(&self.layer_forward(&hidden3_output, &self.weights4, &self.biases4));

        // Output layer error and gradient
        let mut output_error = vec![0.0; target.len()];
        let mut output_delta = vec![0.0; target.len()];
        for i in 0..output.len() {
            output_error[i] = target[i] - output[i];
            output_delta[i] = output_error[i] * output[i] * (1.0 - output[i]);
        }

        // Update weights4 and biases4
        for i in 0..self.weights4.len() {
            for j in 0..self.weights4[i].len() {
                self.weights4[i][j] += learning_rate * output_delta[j] * hidden3_output[i];
            }
        }
        for j in 0..self.biases4.len() {
            self.biases4[j] += learning_rate * output_delta[j];
        }

        // Hidden layer 3 error and gradient
        let mut hidden3_error = vec![0.0; hidden3_output.len()];
        let mut hidden3_delta = vec![0.0; hidden3_output.len()];
        for i in 0..hidden3_output.len() {
            for j in 0..output.len() {
                hidden3_error[i] += output_delta[j] * self.weights4[i][j];
            }
            hidden3_delta[i] = hidden3_error[i] * if hidden3_output[i] > 0.0 { 1.0 } else { 0.0 };
        }

        // Update weights3 and biases3
        for i in 0..self.weights3.len() {
            for j in 0..self.weights3[i].len() {
                self.weights3[i][j] += learning_rate * hidden3_delta[j] * hidden2_output[i];
            }
        }
        for j in 0..self.biases3.len() {
            self.biases3[j] += learning_rate * hidden3_delta[j];
        }

        // Hidden layer 2 error and gradient
        let mut hidden2_error = vec![0.0; hidden2_output.len()];
        let mut hidden2_delta = vec![0.0; hidden2_output.len()];
        for i in 0..hidden2_output.len() {
            for j in 0..hidden3_output.len() {
                hidden2_error[i] += hidden3_delta[j] * self.weights3[i][j];
            }
            hidden2_delta[i] = hidden2_error[i] * if hidden2_output[i] > 0.0 { 1.0 } else { 0.0 };
        }

        // Update weights2 and biases2
        for i in 0..self.weights2.len() {
            for j in 0..self.weights2[i].len() {
                self.weights2[i][j] += learning_rate * hidden2_delta[j] * hidden1_output[i];
            }
        }
        for j in 0..self.biases2.len() {
            self.biases2[j] += learning_rate * hidden2_delta[j];
        }

        // Hidden layer 1 error and gradient
        let mut hidden1_error = vec![0.0; hidden1_output.len()];
        let mut hidden1_delta = vec![0.0; hidden1_output.len()];
        for i in 0..hidden1_output.len() {
            for j in 0..hidden2_output.len() {
                hidden1_error[i] += hidden2_delta[j] * self.weights2[i][j];
            }
            hidden1_delta[i] = hidden1_error[i] * if hidden1_output[i] > 0.0 { 1.0 } else { 0.0 };
        }

        // Update weights1 and biases1
        for i in 0..self.weights1.len() {
            for j in 0..self.weights1[i].len() {
                self.weights1[i][j] += learning_rate * hidden1_delta[j] * input[i];
            }
        }
        for j in 0..self.biases1.len() {
            self.biases1[j] += learning_rate * hidden1_delta[j];
        }
    }
}