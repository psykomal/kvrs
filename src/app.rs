use std::sync::Arc;

use crate::KvsEngine;
use crate::NodeId;
use crate::Raft;
use crate::Store;

// Representation of an application state. This struct can be shared around to share
// instances of raft, store and more.
pub struct App<K: KvsEngine + Sync> {
    pub id: NodeId,
    pub addr: String,
    pub raft: Raft,
    pub store: Arc<Store<K>>,
    pub config: Arc<openraft::Config>,
}
