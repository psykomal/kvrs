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

use crate::engines::SledKvsEngine;
use crate::{common::*, engines, ThreadPool};
use crate::{KvStore, KvsEngine};

pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    addr: SocketAddr,
    listener: TcpListener,
    engine: E,
    dir: PathBuf,
    logger: Logger,
    pool: P,
}

impl<E, P> KvsServer<E, P>
where
    E: KvsEngine,
    P: ThreadPool,
{
    pub fn new(addr: SocketAddr, engine: E, dir: String, logger: Logger, pool: P) -> Self {
        let dir = PathBuf::from(dir);

        Self {
            addr,
            engine,
            listener: TcpListener::bind(addr).expect("Failed to bind to address"),
            dir,
            logger,
            pool,
        }
    }

    pub fn start(&self) {
        info!(self.logger, "Started listening on {}", self.addr);

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    // println!("New connection: {:?}", stream.peer_addr());
                    // std::thread::spawn(move || {
                    // });
                    let engine = self.engine.clone();

                    self.pool.spawn(move || {
                        handle_client(engine, stream);
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

fn handle_client<T>(engine: T, stream: TcpStream)
where
    T: KvsEngine,
{
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
