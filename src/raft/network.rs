use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRequest, State},
    handler::Handler,
    http::{Request, StatusCode},
    Json,
};
use axum_macros::{debug_handler, FromRequest};
use bytes::Bytes;
use hyper::body::HttpBody;
use little_raft::{
    message::{LogEntry, Message},
    replica::ReplicaID,
    state_machine::StateMachineTransition,
};
use serde::Deserialize;

use crate::{raft_server::App, DbOp, KvStore, KvsEngine};

use super::msg::Msg;

pub async fn handle_msg<K>(
    State(state): State<Arc<App<K>>>,
    Json(msg): Json<Msg>,
) -> Result<String, StatusCode>
where
    K: KvsEngine + Sync + Send + 'static,
{
    let message = Message::<DbOp, Bytes>::from(msg);

    // println!("\n\nReceived Message {:?}\n\n", message);

    {
        let mut cluster = state.cluster.lock().unwrap();

        cluster.pending_messages.push(message);
    }

    let msg_tx = state.msg_tx.clone();

    msg_tx.send(()).expect("Unable to send message");

    Ok("".to_string())
}
