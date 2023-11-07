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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Deserialize)]
pub struct MsgLogEntry {
    pub transition: DbOp,
    pub index: usize,
    pub term: usize,
}

impl From<MsgLogEntry> for LogEntry<DbOp> {
    fn from(value: MsgLogEntry) -> Self {
        Self {
            transition: value.transition,
            index: value.index,
            term: value.term,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Deserialize)]
#[serde(untagged)]
pub enum Msg {
    /// AppendEntryRequest is used by the Leader to send out logs for other
    /// replicas to append to their log. It also has information on what logs
    /// are ready to be applied to the state machine. AppendEntryRequest is also
    /// used as a heartbeat message by the Leader even when no new logs need to
    /// be processed.
    AppendEntryRequest {
        from_id: ReplicaID,
        term: usize,
        prev_log_index: usize,
        prev_log_term: usize,
        entries: Vec<MsgLogEntry>,
        commit_index: usize,
    },

    /// AppendEntryResponse is used by replicas to respond to AppendEntryRequest
    /// messages.
    AppendEntryResponse {
        from_id: ReplicaID,
        term: usize,
        success: bool,
        last_index: usize,
        mismatch_index: Option<usize>,
    },

    /// VoteRequest is used by Candidates to solicit votes for themselves.
    VoteRequest {
        from_id: ReplicaID,
        term: usize,
        last_log_index: usize,
        last_log_term: usize,
    },

    /// VoteResponse is used by replicas to respond to VoteRequest messages.
    VoteResponse {
        from_id: ReplicaID,
        term: usize,
        vote_granted: bool,
    },
}

// pub struct AppendEntryRequest {
//     from_id: ReplicaID,
//     term: usize,
//     prev_log_index: usize,
//     prev_log_term: usize,
//     entries: Vec<MsgLogEntry>,
//     commit_index: usize,
// }

pub async fn handle_append_entry_request<K>(
    State(state): State<Arc<App<K>>>,
    Json(msg): Json<Msg>,
) -> Result<String, StatusCode>
where
    K: KvsEngine + Sync + Send + 'static,
{
    let message = match msg {
        Msg::AppendEntryRequest {
            from_id,
            term,
            prev_log_index,
            prev_log_term,
            entries,
            commit_index,
        } => {
            let ents = entries
                .into_iter()
                .map(|entry| LogEntry::<DbOp>::from(entry))
                .collect();

            Message::AppendEntryRequest {
                from_id: from_id,
                term: term,
                prev_log_index: prev_log_index,
                prev_log_term: prev_log_term,
                entries: ents,
                commit_index: commit_index,
            }
        }

        Msg::AppendEntryResponse {
            from_id,
            term,
            success,
            last_index,
            mismatch_index,
        } => Message::AppendEntryResponse {
            from_id: from_id,
            term: term,
            success: success,
            last_index: last_index,
            mismatch_index: mismatch_index,
        },

        Msg::VoteRequest {
            from_id,
            term,
            last_log_index,
            last_log_term,
        } => Message::VoteRequest {
            from_id,
            term,
            last_log_index,
            last_log_term,
        },

        Msg::VoteResponse {
            from_id,
            term,
            vote_granted,
        } => Message::VoteResponse {
            from_id,
            term,
            vote_granted,
        },
    };

    let mut cluster = state.cluster.lock().unwrap();

    cluster.pending_messages.push(message);

    println!("Here I am!");

    Ok("".to_string())
}
