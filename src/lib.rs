#[macro_use]
extern crate log;
extern crate rand;
extern crate scoped_threadpool;
extern crate bincode;
extern crate rustc_serialize;
extern crate byteorder;

pub mod neural_network;
pub mod evolution;
pub mod scaling;

pub use neural_network::NeuralNetwork;
pub use neural_network::NeuronType;
pub use neural_network::Instance;

pub use neural_network::cpu::CpuInstance;

pub use evolution::*;

pub use scaling::*;
