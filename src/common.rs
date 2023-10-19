extern crate serde;
extern crate serde_bytes;

pub use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Request {
    Get(GetRequest),
    Set(SetRequest),
    Remove(RemoveRequest),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SetRequest {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GetRequest {
    pub key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct RemoveRequest {
    pub key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Success(Vec<u8>),
    Error(String),
}
