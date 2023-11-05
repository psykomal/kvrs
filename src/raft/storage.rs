use std::net::SocketAddr;

use bytes::Bytes;
use crossbeam_channel::Sender;
use little_raft::{
    cluster::Cluster,
    message::Message,
    replica::Replica,
    state_machine::{Snapshot, StateMachine, StateMachineTransition, TransitionState},
};

use crate::KvsEngine;

#[derive(Clone, Debug)]
pub enum DbOpType {
    Set(String, String),
    Delete(String),
    Noop,
}

#[derive(Clone, Debug)]
pub struct DbOp {
    pub id: usize,
    pub op_type: DbOpType,
}

impl StateMachineTransition for DbOp {
    type TransitionID = usize;
    fn get_id(&self) -> Self::TransitionID {
        self.id
    }
}

#[derive(Clone, Debug)]
pub struct Storage<K: KvsEngine> {
    pub engine: K,
    pub pending_transitions: Vec<DbOp>,
    pub applied_transitions: Sender<usize>,
}

impl<K> StateMachine<DbOp, Bytes> for Storage<K>
where
    K: KvsEngine,
{
    fn apply_transition(&mut self, transition: DbOp) {
        match transition.op_type {
            DbOpType::Set(key, value) => {
                self.engine.set(key, value);
            }
            DbOpType::Delete(key) => {
                self.engine.remove(key);
            }
            DbOpType::Noop => {}
        }
    }

    fn register_transition_state(
        &mut self,
        transition_id: <DbOp as StateMachineTransition>::TransitionID,
        state: TransitionState,
    ) {
        // Send IDs of applied transitions down the channel so we can confirm
        // they were applied in the right order.
        if state == TransitionState::Applied {
            self.applied_transitions
                .send(transition_id)
                .expect("could not send applied transition id");
        }
    }

    fn get_pending_transitions(&mut self) -> Vec<DbOp> {
        let cur = self.pending_transitions.clone();
        self.pending_transitions = Vec::new();
        cur
    }

    fn get_snapshot(&mut self) -> Option<Snapshot<Bytes>> {
        println!("checked for snapshot");
        None
    }

    fn create_snapshot(&mut self, index: usize, term: usize) -> Snapshot<Bytes> {
        println!("created snapshot");
        Snapshot {
            last_included_index: index,
            last_included_term: term,
            data: Bytes::from(""),
        }
    }

    fn set_snapshot(&mut self, snapshot: Snapshot<Bytes>) {
        println!("Not Implemented");
    }
}

#[derive(Clone)]
pub struct Node {
    pub id: usize,
    pub addr: SocketAddr,
}

pub struct StorageCluster {
    pub id: usize,
    pub is_leader: bool,
    pub peers: Vec<Node>,
    pub pending_messages: Vec<Message<DbOp, Bytes>>,
    pub halt: bool,
}

impl Cluster<DbOp, Bytes> for StorageCluster {
    fn register_leader(&mut self, leader_id: Option<usize>) {
        if let Some(id) = leader_id {
            if id == self.id {
                self.is_leader = true;
            } else {
                self.is_leader = false;
            }
        } else {
            self.is_leader = false;
        }
    }

    fn send_message(&mut self, to_id: usize, message: Message<DbOp, Bytes>) {}

    fn halt(&self) -> bool {
        self.halt
    }

    fn receive_messages(&mut self) -> Vec<Message<DbOp, Bytes>> {
        let cur = self.pending_messages.clone();
        self.pending_messages = Vec::new();
        cur
    }
}
