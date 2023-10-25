extern crate serde;
extern crate serde_bytes;

extern crate bincode;
use bincode::{deserialize, serialize};

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::common::*;
use crate::Result;

fn send_request<R: Serialize>(addr: SocketAddr, request: R) -> Result<Response> {
    let mut stream = TcpStream::connect(addr).expect("Failed to connect to server");

    let request_buf = serialize(&request)?;

    // println!("Request buf: {:?}", request_buf);

    stream.write_all(&request_buf)?;
    // send EOF
    stream.write_all(b"\n")?;

    let mut response_buf = Vec::new();
    stream.read_to_end(&mut response_buf)?;

    let response: Response = deserialize(&response_buf)?;

    Ok(response)
}

pub struct KvsClient {
    addr: SocketAddr,
}

impl KvsClient {
    pub fn new(addr: SocketAddr) -> KvsClient {
        KvsClient { addr }
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        let get_request = GetRequest {
            key: Vec::from(key.as_bytes()),
        };

        let request = Request::Get(get_request);

        match send_request(self.addr, request) {
            Ok(response) => match response {
                Response::Success(data) => {
                    let val = String::from_utf8(data).unwrap();
                    // println!("{}", val);
                    Ok(Some(val))
                }
                Response::Error(error) => {
                    eprintln!("GET Error: {}", error);
                    Err(failure::err_msg(error))
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(e)
            }
        }
    }

    pub fn set(&self, key: String, value: String) -> Result<()> {
        let set_request = SetRequest {
            key: Vec::from(key.as_bytes()),
            value: Vec::from(value.as_bytes()),
        };
        let request = Request::Set(set_request);

        match send_request(self.addr, request) {
            Ok(response) => match response {
                Response::Success(data) => {
                    // println!("SET Response: {:?}", String::from_utf8(data).unwrap());
                    Ok(())
                }
                Response::Error(error) => {
                    eprintln!("SET Error: {}", error);
                    Err(failure::err_msg(error))
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(e)
            }
        }
    }

    pub fn remove(&self, key: String) -> Result<()> {
        let rm_request = RemoveRequest {
            key: Vec::from(key.as_bytes()),
        };

        let request = Request::Remove(rm_request);

        match send_request(self.addr, request) {
            Ok(response) => match response {
                Response::Success(data) => {
                    let val = String::from_utf8(data).unwrap();
                    // println!("Remove Response: {:?}", val);
                    Ok(())
                }
                Response::Error(error) => {
                    eprintln!("Remove Error: {}", error);
                    Err(failure::err_msg(error))
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(e)
            }
        }
    }
}
