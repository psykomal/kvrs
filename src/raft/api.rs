use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use crossbeam_channel::select;

use crate::{raft_server::App, DbOp, DbOpType, KvsEngine};

pub async fn handle_get<K>(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<App<K>>>,
) -> Result<String, StatusCode>
where
    K: KvsEngine + Sync,
{
    let store = state.state_machine.lock().unwrap();

    let key = match params.get("key") {
        Some(k) => k.to_owned(),
        None => return Err(StatusCode::BAD_REQUEST),
    };

    match store.engine.get(key) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Ok("Key not found".to_string()),
        Err(e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_set<K>(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<App<K>>>,
) -> Result<StatusCode, StatusCode>
where
    K: KvsEngine + Sync,
{
    let key = match params.get("key") {
        Some(k) => k.to_owned(),
        None => return Err(StatusCode::BAD_REQUEST),
    };
    let val = match params.get("val") {
        Some(k) => k.to_owned(),
        None => return Err(StatusCode::BAD_REQUEST),
    };

    let tsn_tx = state.tsn_tx.lock().unwrap();
    let applied_tsns_rx = state.applied_tsns_rx.clone();
    let id;
    {
        let mut next_id = state.counter.lock().unwrap();
        *next_id = *next_id + 1;
        id = *next_id;
    }

    let op = DbOp {
        id,
        op_type: DbOpType::Set(key, val),
    };

    state
        .state_machine
        .lock()
        .unwrap()
        .pending_transitions
        .push(op);

    tsn_tx.send(()).expect("Unable to send message to raft");

    loop {
        select! {
            recv(applied_tsns_rx) -> applied_id => {
                if applied_id == Ok(id) {
                    return Ok(StatusCode::OK);
                }
            },
            default(Duration::from_secs(5)) => return Err(StatusCode::REQUEST_TIMEOUT),
        }
    }
}

pub async fn handle_remove<K>(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<App<K>>>,
) -> Result<StatusCode, StatusCode>
where
    K: KvsEngine + Sync,
{
    let key = match params.get("key") {
        Some(k) => k.to_owned(),
        None => return Err(StatusCode::BAD_REQUEST),
    };

    let tsn_tx = state.tsn_tx.lock().unwrap();
    let applied_tsns_rx = state.applied_tsns_rx.clone();

    let id;
    {
        let mut next_id = state.counter.lock().unwrap();
        *next_id = *next_id + 1;
        id = *next_id;
    }

    let op = DbOp {
        id,
        op_type: DbOpType::Delete(key),
    };

    state
        .state_machine
        .lock()
        .unwrap()
        .pending_transitions
        .push(op);

    tsn_tx.send(()).expect("Unable to send message to raft");

    loop {
        select! {
            recv(applied_tsns_rx) -> applied_id => {
                if applied_id == Ok(id) {
                    return Ok(StatusCode::OK);
                }
            },
            default(Duration::from_secs(5)) => return Err(StatusCode::REQUEST_TIMEOUT),
        }
    }
}
