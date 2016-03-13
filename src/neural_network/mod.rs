use std::vec::Vec;
use rand::*;
use rustc_serialize::{Encodable, Decodable, Encoder, Decoder};

use evolution::*;

pub mod cpu;

struct RngWrapper(OsRng);

impl Encodable for RngWrapper {
    fn encode<S: Encoder>(&self, _: &mut S) -> Result<(), S::Error> {
        Ok(())
    }
}

impl Decodable for RngWrapper {
    fn decode<D: Decoder>(_: &mut D) -> Result<Self, D::Error> {
        Ok(RngWrapper(OsRng::new().unwrap()))
    }
}

impl Clone for RngWrapper {
    fn clone(&self) -> Self {
        RngWrapper(OsRng::new().unwrap())
    }
}

/// Structure that describes a neural network
#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct NeuralNetwork {
    inputs: usize,

    hidden_layers: Vec<Vec<Neuron>>,
    neuron_count: usize,

    random_generator: RngWrapper
}

impl Evolvable for NeuralNetwork {
    fn cross_over(&self, other: &Self) -> Self {
        self.clone()
    }

    fn mutate(&mut self) {
        // TODO: this is a super ugly implementation
        let maxLayers = self.hidden_layers.len();

        let randomIndex: usize = self.random_generator.0.gen_range(0, maxLayers - 1);
        let randomIndex2: usize = self.random_generator.0.gen_range(0, maxLayers - 1);

        // find random neuron to adjust bias
        {
            let mut randomLayer: &mut Vec<Neuron> = &mut self.hidden_layers[randomIndex];
            let maxLen = randomLayer.len();
            let mut randomNeuron: &mut Neuron = &mut randomLayer[self.random_generator.0.gen_range(0, maxLen - 1)];

            // randomize its bias
            randomNeuron.bias = self.random_generator.0.gen_range(-1.0, 1.0);
        }

        // find random connection to adjust weight
        let mut randomLayer2: &mut Vec<Neuron> = &mut self.hidden_layers[randomIndex2];
        let maxLen2 = randomLayer2.len();
        let mut randomNeuron2: &mut Neuron = &mut randomLayer2[self.random_generator.0.gen_range(0, maxLen2 - 1)];
        let mut randomWeightIndex = self.random_generator.0.gen_range(0, randomNeuron2.weights.len() - 1);

        // randmoize its weight
        randomNeuron2.weights[randomWeightIndex] = self.random_generator.0.gen_range(-1.0, 1.0);
    }

}

/// Describes an neuron type
#[derive(RustcEncodable, RustcDecodable, Copy, Clone, Debug)]
pub enum NeuronType {
    Identity,
    SigMoid,
    TanH,
    DeLu,
}

/// A neuron
#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct Neuron {
    weights: Vec<f64>,
    bias: f64,
    //neuron_type: NeuronType
}

/// Trait for neural network instances
pub trait Instance<'a, Err>: Sized {
    /// Creates a new instance by a given neural network
    fn new(nn: &'a NeuralNetwork) -> Result<Self, Err>;

    /// Calulates using the neural network by given inputs
    fn calculate(&mut self, inputs: &Vec<f64>, outputs: &mut Vec<f64>) -> Result<(), Err>;
}

/// Iterator that iterates through all neurons within a neural network
pub struct NeuronIterator<'a> {
    network: &'a NeuralNetwork,
    current_layer: usize,
    current_neuron: usize
}

impl<'a> Iterator for NeuronIterator<'a> {
    /// Layer index, Reference to neuron
    type Item = (usize, &'a Neuron);

    fn next(&mut self) -> Option<Self::Item> {

        // check neuron bounds
        if self.current_neuron >= self.network.hidden_layers[self.current_layer].len() {
            self.current_neuron = 0;
            self.current_layer += 1;
        }

        // check layer bounds
        if self.current_layer >= self.network.hidden_layers.len() {
            return None;
        }

        // return instance
        let res = (self.current_layer, &self.network.hidden_layers[self.current_layer][self.current_neuron]);
        self.current_neuron += 1;

        Some(res)
    }
}

impl NeuralNetwork {
    /// Creates a new instance of an neural network
    pub fn new() -> Self {
        NeuralNetwork {
            inputs: 0,
            hidden_layers: Vec::new(),
            neuron_count: 0,
            random_generator: RngWrapper(OsRng::new().unwrap())
        }
    }

    /// Creates an iterator over all neurons
    pub fn iter(&self) -> NeuronIterator {
        NeuronIterator {
            network: self,
            current_layer: 0,
            current_neuron: 0
        }
    }

    /// Generates a random f64 from 0.0 to 1.0 (both inclusive)
    pub fn random(&mut self, min: f64, max: f64) -> f64 {
        self.random_generator.0.gen_range(min, max)
    }

    /// Sets the amount of inputs the nn should have
    pub fn set_inputs(&mut self, amount: usize) {
        self.inputs = amount;
    }

    /// Adds a group of neurons to an hidden layer
    pub fn add_neuron_group(&mut self, layer_index: usize, neuron_type: NeuronType, amount: usize, min: f64, max: f64) {
        self.neuron_count += amount;

        if self.hidden_layers.len() <= layer_index {
            self.hidden_layers.push(Vec::new());
        }

        // generate weights
        let weightsAmount = {
            if layer_index == 0 {
                self.inputs
            } else {
                self.hidden_layers[layer_index - 1].len()
            }
        };

        // create neurons
        for _ in 0..amount {
            let mut weights: Vec<f64> = Vec::new();

            for _ in 0..weightsAmount {
                weights.push(self.random(min, max));
            }

            // save neurons
            let neuron = Neuron {
                weights: weights.clone(),
                bias: self.random(min, max),
                //neuron_type: neuron_type
            };

            self.hidden_layers[layer_index].push(neuron);
        }
    }

    /// Finalizes the neural networks strcuture
    pub fn build(&self) {
    }
}
