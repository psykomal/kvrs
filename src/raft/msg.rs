use bytes::Bytes;
use little_raft::{
    message::{LogEntry, Message},
    replica::ReplicaID,
};

use crate::DbOp;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Deserialize, Serialize)]
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

impl From<LogEntry<DbOp>> for MsgLogEntry {
    fn from(value: LogEntry<DbOp>) -> Self {
        Self {
            transition: value.transition,
            index: value.index,
            term: value.term,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Deserialize, Serialize)]
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

impl From<Msg> for Message<DbOp, Bytes> {
    fn from(value: Msg) -> Self {
        match value {
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
        }
    }
}

impl From<Message<DbOp, Bytes>> for Msg {
    fn from(value: Message<DbOp, Bytes>) -> Self {
        match value {
            Message::AppendEntryRequest {
                from_id,
                term,
                prev_log_index,
                prev_log_term,
                entries,
                commit_index,
            } => {
                let ents = entries
                    .into_iter()
                    .map(|entry| MsgLogEntry::from(entry))
                    .collect();

                Msg::AppendEntryRequest {
                    from_id,
                    term,
                    prev_log_index,
                    prev_log_term,
                    entries: ents,
                    commit_index,
                }
            }

            Message::AppendEntryResponse {
                from_id,
                term,
                success,
                last_index,
                mismatch_index,
            } => Msg::AppendEntryResponse {
                from_id: from_id,
                term: term,
                success: success,
                last_index: last_index,
                mismatch_index: mismatch_index,
            },

            Message::VoteRequest {
                from_id,
                term,
                last_log_index,
                last_log_term,
            } => Msg::VoteRequest {
                from_id,
                term,
                last_log_index,
                last_log_term,
            },

            Message::VoteResponse {
                from_id,
                term,
                vote_granted,
            } => Msg::VoteResponse {
                from_id,
                term,
                vote_granted,
            },

            _ => panic!("Invalid message type"),
        }
    }
}
