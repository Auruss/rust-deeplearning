use evolution::{Evolvable, StopRule, genetic_evolution};

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::env;
use std::process::Command;

use rustc_serialize::serialize::{Encodable, Decodable};
use bincode::rustc_serialize::{encode_into, decode_from};
use bincode::SizeLimit;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub struct Options {
    /// The port the master should listen on
    pub port: u16,

    /// Sets the condition when master should start to command slaves
    pub start_condition: StartCondition,
    /// Sets sync condition
    pub sync_condition: SynchronizeCondition
}

impl Options {
    pub fn defaults() -> Self {
        Options {
            port: 1337,

            start_condition: StartCondition::AmountClientsReady(6),
            sync_condition: SynchronizeCondition::AfterGenerationsAdvanced(6),
        }
    }
}

pub enum StartCondition {
    /// Starts when x clients are connected and ready
    AmountClientsReady(usize),
    /// Starts when the last connection was x seconds ago
    NoConnectsSinceSeconds(usize)
}

pub enum SynchronizeCondition {
    /// Synchronizes every x seconds
    AfterTimeInSeconds(usize),

    /// Synchronizes every x generations
    AfterGenerationsAdvanced(usize)
}

#[derive(Copy, Clone)]
struct NetworkEvolvable {
    client: usize,
    fitness: f64,
    crossWith: usize,
    crossOver: bool
}

impl Evolvable for NetworkEvolvable {
    fn cross_over(&self, other: &Self) -> Self {
        NetworkEvolvable {
            client: self.client,
            fitness: 0.0,
            crossWith: other.client,
            crossOver: true
        }
    }

    fn mutate(&mut self) {
    }
}

pub fn create_scaling_host<T: Decodable>(options: Options) -> Option<T> {
    // create listener
    let listener = TcpListener::bind(("0.0.0.0", options.port)).unwrap();

    // start to listen
    let mut connections = Vec::new();

    //for stream in listener.incoming() {
    loop {
        match listener.accept() {
            Ok((stream, adr)) => {
                // connection succeeded
                connections.push(stream);

                info!(target: "scaling", "Client connection, No: {} Adr: {}", connections.len(), adr);

                // handle startup
                match options.start_condition {
                    StartCondition::AmountClientsReady(x) => {
                        if connections.len() == x {
                            break;
                        }
                    },
                    _ => { }
                }
            }
            Err(_) => {
                // client connection failed
                error!(target: "scaling", "Failed to handle tcp client connection");
            }
        }
    }

    // create initial individuals

    // send request to every client
    for connection in connections.iter_mut() {
        let _ = connection.write(&[0x00, 0x01]).unwrap(); // start command
    }

    // wait for answer of every client
    //let mut fitnessResults = Vec::new();
    let first: usize = 0;
    let second: usize = 0;

    for mut connection in connections.iter_mut() {
        let mut opcode = [0; 2];
        let _ = connection.read(&mut opcode).unwrap();
        let fitness = connection.read_f64::<BigEndian>().unwrap();
        let _ = decode_from::<_, T>(&mut connection, SizeLimit::Infinite).unwrap_or_else(|e| {
            error!("{}", e);
            panic!("{}", e);
        });

        info!(target: "scaling", "retrieved inital response");

    }

    None
}

/*
/// Creates an host for scaling
pub fn create_scaling_host<T: Decodable>(options: Options) -> Option<T> {
    // create listener
    let listener = TcpListener::bind(("0.0.0.0", options.port)).unwrap();

    // start to listen
    let mut connections = RefCell::new(Vec::new());

    //for stream in listener.incoming() {
    loop {
        match listener.accept() {
            Ok((stream, adr)) => {
                // connection succeeded
                connections.borrow_mut().push(stream);

                info!(target: "scaling", "Client connection, No: {}", connections.borrow_mut().len());

                // handle startup
                match options.start_condition {
                    StartCondition::AmountClientsReady(x) => {
                        if connections.borrow_mut().len() == x {
                            break;
                        }
                    },
                    _ => { }
                }
            }
            Err(_) => {
                // client connection failed
                error!(target: "scaling", "Failed to handle tcp client connection");
            }
        }
    }

    debug!(target: "scaling", "starting swarm computation");

    // send start command to all client
    let amount = connections.borrow().len();
    let (trained, _) = genetic_evolution(amount, StopRule::GenerationReached(10), &mut |i| {
        debug!(target: "scaling", "contacting child to generate a new set");
        let _ = connections.borrow_mut()[i].write(&[0x00, 0x01]).unwrap(); // start command

        // wait for answer
        let mut opcode = [0; 2];
        let _ = connections.borrow_mut()[i].read(&mut opcode).unwrap();
        let fitness = connections.borrow_mut()[i].read_f64::<BigEndian>().unwrap();
        let _ = decode_from::<_, T>(&mut connections.borrow_mut()[i], SizeLimit::Infinite).unwrap();

        debug!(target: "scaling", "\t->Done!");

        // return
        NetworkEvolvable {
            client: i,
            fitness: fitness,
            crossWith: 0,
            crossOver: false
        }
    }, &mut | net: &mut NetworkEvolvable | {
        // cross over
        if net.crossOver {
            // TODO; impl cross over for scaling
            net.crossOver = false;
        }

        // train
        debug!(target: "scaling", "contacting child to train");
        let mut opcode = [0; 2];
        let _ = connections.borrow_mut()[net.client].write(&[0x00, 0x02]);
        let _ = connections.borrow_mut()[net.client].read(&mut opcode);
        let fitness = connections.borrow_mut()[net.client].read_f64::<BigEndian>().unwrap();
        net.fitness = fitness;
        debug!(target: "scaling", "\t->Done!");

        net.fitness
    }, None);

    // get winners object
    let _ = connections.borrow_mut()[trained.client].write(&[0x00, 0x03]);

    // TODO: remove temporary when https://github.com/rust-lang/rust/issues/22449 is fixed
    let temp = match decode_from(&mut connections.borrow_mut()[trained.client], SizeLimit::Infinite) {
        Ok(object) => {
            Some(object)
        },

        Err(_) => {
            None
        }
    };

    temp
}*/

/// Creates an client for scaling
pub fn create_scaling_client<T: Encodable + Decodable + Evolvable, Finit, Fcontinue>(master_port: u16, init: Finit, train: Fcontinue)
    where Finit:     Fn() -> (T, f64),
          Fcontinue: Fn(&mut T) -> f64
{
    let mut current_object: Option<T> = None;
    let mut opcode = [0; 2];
    let mut stream = TcpStream::connect(("127.0.0.1", master_port)).unwrap_or_else(|e| {
        error!(target: "scaling", "Failed to connect to master {}", e);
        panic!("can't continue")
    });

    loop {
        let _ = stream.read(&mut opcode);

        // start action
        if opcode[0] == 0x00 && opcode[1] == 0x01 {
            info!(target: "scaling", "Received init request");
            let (initial, fitness) = init();
            current_object = Some(initial);

            let _ = stream.write(&[0x00, 0x01]).unwrap();
            let _ = stream.write_f64::<BigEndian>(fitness).unwrap();
            let _ = encode_into(&current_object, &mut stream, SizeLimit::Infinite).unwrap();
            info!(target: "scaling", "Sent init response");
        }

        // train action
        else if opcode[0] == 0x00 && opcode[1] == 0x02 {
            info!(target: "scaling", "Received train request");

            // burrow current object
            let mut obj = current_object.unwrap_or_else(|| {
                error!(target: "scaling", "Tried to train without initializing first. Possibly an error in init function.");
                panic!("can't continue")
            });

            // mutate
            obj.mutate();

            // train
            let new_fitness = train(&mut obj);

            // put object back
            current_object = Some(obj);

            let _ = stream.write(&[0x00, 0x02]);
            let _ = stream.write_f64::<BigEndian>(new_fitness);
        }

        // get current object request
        else if opcode[0] == 0x00 && opcode[1] == 0x03 {
            info!(target: "scaling", "Received get current object request");

            let _ = stream.write(&[0x00, 0x03]);
            let _ = encode_into(&current_object, &mut stream, SizeLimit::Infinite);
        }

        // set current object request
        else if opcode[0] == 0x00 && opcode[1] == 0x04 {
            info!(target: "scaling", "Received set current object request");

            match decode_from(&mut stream, SizeLimit::Infinite) {
                Ok(object) => {
                    current_object = object;
                },

                Err(_) => {
                    error!(target: "scaling", "Could not decode set current object request. Ignoring request.");
                }
            }
        }

        opcode[0] = 0;
        opcode[1] = 0;
    }
}

/// Creates one new fork
pub fn fork_scaling() {
    // get path to self
    let self_path = env::current_exe().unwrap_or_else(|e| {
        error!(target: "scaling", "failed to fork process: {}", e);
        panic!("Can't continue")
    });

    // spawn child
    let _ = Command::new(self_path)
        .arg("--type child")
        .spawn();
}

pub enum ProcessType {
    /// Controlls slaves that will actually train
    Master,
    /// Slave that will train given individual from master
    Slave
}

/// Returns current process type
pub fn process_type() -> ProcessType {
    for argument in env::args() {
        if argument == "--type child" {
            return ProcessType::Slave;
        }
    }

    ProcessType::Master
}
