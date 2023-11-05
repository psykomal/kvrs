//! A simple key/value store.

#[macro_use]
extern crate serde_derive;

pub use client::KvsClient;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::Result;
pub use raft::{api, DbOp, DbOpType, Node, Storage, StorageCluster};
pub use raft_server::run_raft_node;
pub use server::KvsServer;
pub use thread_pool::{NaiveThreadPool, RayonThreadPool, SharedQueueThreadPool, ThreadPool};

mod client;
pub mod common;
mod engines;
mod error;
mod raft;
mod raft_server;
mod server;
mod thread_pool;
