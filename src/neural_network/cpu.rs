use neural_network::*;

pub struct CpuInstance<'a> {
    network: &'a NeuralNetwork
}

/// Errors for CpuInstace
#[derive(Debug)]
pub enum CpuInstanceError {
    /// When internal thread creation or usage fails
    ThreadFailure,

    /// CPU instance does not support all neuron types yet
    ///  (type_of_neuron)
    UnsupportedNeuronType(NeuronType),

    /// Currently mixing multiple activation functions within one layer is not supported
    ///  (layer_number)
    UnsupportedActivationMix(usize)
}

/// applies activation over an set of values
fn apply_activation(values:&mut Vec<f64>, neuron_type: NeuronType) {
    match neuron_type {
        NeuronType::TanH => {
            for i in 0..values.len() {
                values[i] = values[i].tanh();
            }
        },
        _ => {

        }
    }
}

/// validates a neural network and returns either Some(error) or None
fn validate(network: &NeuralNetwork) -> Option<CpuInstanceError> {
    // TODO do optional validation (feature flag)
    None
}

impl<'a> Instance<'a, CpuInstanceError> for CpuInstance<'a> {
    fn new (network: &'a NeuralNetwork) -> Result<Self, CpuInstanceError> {
        match validate(network) {
            Some(err) => {
                return Err(err);
            },
            None => {
                return Ok(CpuInstance {
                    network: network
                })
            }
        }
    }

    fn calculate(&mut self, inputs: &Vec<f64>, outputs: &mut Vec<f64>) -> Result<(), CpuInstanceError> {
        let mut previous_values: Vec<f64> = Vec::new();
        let mut current_values: Vec<f64> = Vec::new();

        let mut previous_layer = 0;

        // pretend inputs form the previous layer
        for val in inputs {
            previous_values.push(*val);
        }

        // loop through neurons
        for (layer_index, neuron) in self.network.iter() {
            if previous_layer != layer_index {
                // new layer reached
                previous_values = current_values.clone();
                current_values.clear();

                // apply activation
                apply_activation(&mut previous_values, NeuronType::TanH);
            }

            // new neuron
            let mut val = 0.0;
            for i in 0..neuron.weights.len() {
                val += neuron.weights[i] * previous_values[i];
            }
            val += neuron.bias;

            current_values.push(val);

            previous_layer = layer_index;
        }

        // since technically the output layer is represented as an additional output layer we are done here
        for val in current_values {
            outputs.push(val);
        }

        // Also apply activation over outputs
        apply_activation(outputs, NeuronType::TanH);

        Ok(())
    }
}
