use std::sync::Arc;

use axum::{extract::State, handler::Handler, http::StatusCode, Json};
use bytes::Bytes;
use little_raft::message::Message;

use crate::{raft_server::App, DbOp, KvsEngine};

pub async fn handle_msg<K>(
    Json(msg): Json<Message<DbOp, Bytes>>,
    State(state): State<Arc<App<K>>>,
) -> Result<String, StatusCode>
where
    K: KvsEngine + Sync + Send + 'static,
{
    let mut cluster = state.cluster.lock().unwrap();

    cluster.pending_messages.push(msg);

    Ok("".to_string())
}

#[async_trait]
impl<S, T, D> FromRequest<S> for Message<T, D>
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {}
}
