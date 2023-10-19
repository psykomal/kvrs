use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;

use slog::{info, Logger};
extern crate bincode;
extern crate serde;
extern crate serde_bytes;
use bincode::{deserialize, serialize, Error};
use std::thread;

use crate::common::*;
use crate::{KvStore, KvsEngine};

pub struct KvsServer {
    addr: SocketAddr,
    listener: TcpListener,
    engine: String,
    dir: PathBuf,
    logger: Logger,
}

impl KvsServer {
    pub fn new(addr: SocketAddr, engine: String, dir: String, logger: Logger) -> Self {
        Self {
            addr,
            engine,
            listener: TcpListener::bind(addr).expect("Failed to bind to address"),
            dir: PathBuf::from(dir),
            logger,
        }
    }

    pub fn start(&self) {
        info!(self.logger, "Started listening on {}", self.addr);

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {:?}", stream.peer_addr());
                    std::thread::spawn(move || {
                        handle_client(stream);
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut stream = BufReader::new(stream);

    let mut buf = Vec::new();
    if let Err(e) = stream.read_until(b'\n', &mut buf) {
        eprintln!("Error reading from stream: {}", e);
        return;
    }

    println!("Received buf: {:?}", buf);

    let request: Result<Request, Error> = deserialize(&buf);

    println!("Received request: {:?}", request);

    let response = match request {
        Ok(Request::Set(set_request)) => {
            // Handle SET operation
            // Implement your SET logic here
            // In this example, we return a generic success response
            Response::Success(b"SET operation successful".to_vec())
        }
        Ok(Request::Get(get_request)) => {
            // Handle GET operation
            // Implement your GET logic here
            // In this example, we return a generic success response
            Response::Success(b"GET operation successful".to_vec())
        }
        Ok(Request::Remove(remove_request)) => {
            // Handle REMOVE operation
            // Implement your REMOVE logic here
            // In this example, we return a generic success response
            Response::Success(b"REMOVE operation successful".to_vec())
        }
        Err(e) => Response::Error(format!("Invalid request: {}", e)),
    };

    println!("Sending response: {:?}", response);

    let response_buf = serialize(&response).unwrap();
    if let Err(e) = stream.get_mut().write_all(&response_buf) {
        eprintln!("Error writing to stream: {}", e);
    }
}

// fn get_engine(engine: String, dir: PathBuf) -> Box<dyn KvsEngine> {
//     match engine.as_str() {
//         "kvs" => Box::new(KvStore::open(dir)?),
//     }
// }
