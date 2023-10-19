use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

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
    engine: Rc<RefCell<dyn KvsEngine>>,
    dir: PathBuf,
    logger: Logger,
}

impl KvsServer {
    pub fn new(addr: SocketAddr, engine: String, dir: String, logger: Logger) -> Self {
        let dir = PathBuf::from(dir);

        Self {
            addr,
            engine: get_engine(engine, dir.clone()),
            listener: TcpListener::bind(addr).expect("Failed to bind to address"),
            dir,
            logger,
        }
    }

    pub fn start(&self) {
        info!(self.logger, "Started listening on {}", self.addr);

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {:?}", stream.peer_addr());
                    // std::thread::spawn(move || {
                    handle_client(Rc::clone(&self.engine), stream);
                    // });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

fn handle_client<T>(engine: Rc<RefCell<T>>, stream: TcpStream)
where
    T: KvsEngine + ?Sized,
{
    let mut engine = engine.as_ref().borrow_mut();
    let mut stream = BufReader::new(stream);

    let mut buf = Vec::new();
    if let Err(e) = stream.read_until(b'\n', &mut buf) {
        eprintln!("Error reading from stream: {}", e);
        return;
    }

    // println!("Received buf: {:?}", buf);

    let request: Result<Request, Error> = deserialize(&buf);

    // println!("Received request: {:?}", request);

    let response = match request {
        Ok(Request::Set(SetRequest { key, value })) => {
            let key = String::from_utf8(key).unwrap();
            let value = String::from_utf8(value).unwrap();
            match engine.set(key, value) {
                Ok(()) => Response::Success(b"SET operation successful".to_vec()),
                Err(e) => Response::Error(format!("Error: {}", e)),
            }
        }
        Ok(Request::Get(GetRequest { key })) => {
            let key = String::from_utf8(key).unwrap();
            match engine.get(key) {
                Ok(Some(value)) => Response::Success(value.into_bytes()),
                Ok(None) => Response::Success("Key not found".to_string().into_bytes()),
                Err(e) => Response::Error(format!("Error: {}", e)),
            }
        }
        Ok(Request::Remove(RemoveRequest { key })) => {
            let key = String::from_utf8(key).unwrap();
            match engine.remove(key) {
                Ok(()) => Response::Success(b"REMOVE operation successful".to_vec()),
                Err(e) => Response::Error(format!("Error: {}", e)),
            }
        }
        Err(e) => Response::Error(format!("Invalid request: {}", e)),
    };

    // println!("Sending response: {:?}", response);

    let response_buf = serialize(&response).unwrap();
    if let Err(e) = stream.get_mut().write_all(&response_buf) {
        eprintln!("Error writing to stream: {}", e);
    }
}

fn get_engine(engine: String, dir: PathBuf) -> Rc<RefCell<dyn KvsEngine>> {
    match engine.as_str() {
        "kvs" => Rc::new(RefCell::new(KvStore::open(dir).unwrap())),
        _ => Rc::new(RefCell::new(KvStore::open(dir).unwrap())),
    }
}
