use rand::Rng;

#[derive(Clone)]
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
    pub fn mutate_layer(
        &self, 
        layer: &Vec<Vec<f32>>, 
        mut_prob: f32,
        mut_amount: f32,
    ) -> Vec<Vec<f32>> {
        let mut rng = rand::thread_rng();
        let mut new_layer = layer.clone();

        for i in 0..layer.len() {
            for j in 0..layer[i].len() {
                if rng.gen::<f32>() < mut_prob {
                    new_layer[i][j] += rng.gen_range(-mut_amount..mut_amount);
                }
            }
        }
        new_layer 
    }

    pub fn mutate_biases(
        &self, 
        biases: &Vec<f32>, 
        mut_prob: f32,
        mut_amount: f32,
    ) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        let mut new_biases = biases.clone();

        for i in 0..biases.len() {
            if rng.gen::<f32>() < mut_prob {
                new_biases[i] += rng.gen_range(-mut_amount..mut_amount);
            }
        }
        new_biases 
    }

    pub fn mutate(
        &self,
        mut_prob: f32,
        mut_amount: f32,
    ) -> Self {
        Self {
            weights1: self.mutate_layer(&self.weights1, mut_prob, mut_amount),
            biases1: self.mutate_biases(&self.biases1, mut_prob, mut_amount),
            weights2: self.mutate_layer(&self.weights2, mut_prob, mut_amount),
            biases2: self.mutate_biases(&self.biases2, mut_prob, mut_amount),
            weights3: self.mutate_layer(&self.weights3, mut_prob, mut_amount),
            biases3: self.mutate_biases(&self.biases3, mut_prob, mut_amount),
            weights4: self.mutate_layer(&self.weights4, mut_prob, mut_amount),
            biases4: self.mutate_biases(&self.biases4, mut_prob, mut_amount),
        }
    }

}