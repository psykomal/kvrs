use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
};

use crate::{raft_server::App, KvsEngine};

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
