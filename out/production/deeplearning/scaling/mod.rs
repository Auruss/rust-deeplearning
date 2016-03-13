use evolution::Evolvable;

use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::env;
use std::process::Command;

use rustc_serialize::*;
use bincode::rustc_serialize::{encode_into, decode_from};
use bincode::SizeLimit;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub struct Options {
    /// The port the master should listen on
    pub port: u16,

    /// Sets the condition when master should start to command slaves
    pub startCondition: StartCondition,
    /// Sets sync condition
    pub syncCondition: SynchronizeCondition
}

impl Options {
    pub fn defaults() -> Self {
        Options {
            port: 1337,

            startCondition: StartCondition::NoConnectsSinceSeconds(15),
            syncCondition: SynchronizeCondition::AfterGenerationsAdvanced(6),
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

struct NetworkEvolvable<'a> {
    client: &'a TcpStream,
    fitness: f64
}

impl<'a> Evolvable for NetworkEvolvable<'a> {
    fn cross_over(&self, other: &Self) -> Self {
        panic!("TODO")
    }

    fn mutate(&mut self) {
        // send train request
        self.client.write(&[0x00, 0x02]);

        // read answer
    }
}

/// Creates an host for scaling
pub fn create_scaling_host(options: Options) {
    // create listener
    let listener = TcpListener::bind(("0.0.0.0", options.port)).unwrap();

    // start to listen
    let mut connections = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // connection succeeded
                connections.push(stream);
                info!(target: "scaling", "Client connection, No: {}", connections.len());

                // handle startup
                match options.startCondition {
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

    // send start command to all client
    for mut stream in connections {
        let _ = stream.write(&[0x00, 0x01]); // start command
    }
}

/// Creates an client for scaling
pub fn create_scaling_client<T: Encodable + Decodable, Finit, Fcontinue>(master_port: u16, init: Finit, train: Fcontinue)
    where Finit:     Fn() -> (T, f64),
          Fcontinue: Fn(&mut T) -> f64
{
    let mut current_object: Option<&mut T> = None;
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

            stream.write(&[0x00, 0x01]);
            stream.write_f64::<BigEndian>(fitness);
            encode_into(&current_object, &mut stream, SizeLimit::Infinite);
        }

        // train action
        else if opcode[0] == 0x00 && opcode[1] == 0x02 {
            info!(target: "scaling", "Received train request");
            let new_fitness = train(current_object.unwrap_or_else(|e| {
                error!(target: "scaling", "Tried to train without initializing first. Possibly an error in init function.");
                panic!("can't continue")
            }));

            stream.write(&[0x00, 0x02]);
            stream.write_f64::<BigEndian>(new_fitness);
        }

        // get current object request
        else if opcode[0] == 0x00 && opcode[1] == 0x03 {
            info!(target: "scaling", "Received get current object request");

            stream.write(&[0x00, 0x03]);
            encode_into(&current_object, &mut stream, SizeLimit::Infinite);
        }

        // set current object request
        else if opcode[0] == 0x00 && opcode[1] == 0x04 {
            info!(target: "scaling", "Received set current object request");

            match decode_from(&mut stream, SizeLimit::Infinite) {
                Ok(object) => {
                    current_object = object;
                },

                Err(err) => {
                    error!(target: "scaling", "Could not decode set current object request. Ignoring request.");
                }
            }
        }
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
