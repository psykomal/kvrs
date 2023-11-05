use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use axum::{routing::get, Router};
use crossbeam_channel::{unbounded, Receiver, Sender};
use little_raft::replica::Replica;

use crate::{api, DbOp, DbOpType, KvStore, KvsEngine, Storage, StorageCluster};

const HEARTBEAT_TIMEOUT: Duration = Duration::from_millis(500);
const MIN_ELECTION_TIMEOUT: Duration = Duration::from_millis(750);
const MAX_ELECTION_TIMEOUT: Duration = Duration::from_millis(950);

// Representation of an application state. This struct can be shared around to share
// instances of raft, store and more.
pub struct App<K: KvsEngine + Sync> {
    pub id: usize,
    pub addr: String,
    pub state_machine: Arc<Mutex<Storage<K>>>,
    pub cluster: Arc<Mutex<StorageCluster>>,
    pub msg_tx: Arc<Mutex<Sender<()>>>,
    pub tsn_tx: Arc<Mutex<Sender<()>>>,
    pub applied_tsns_rx: Arc<Receiver<usize>>,
    pub counter: Arc<Mutex<usize>>,
}

pub async fn run_raft_node(node_id: usize, engine: &str, addr: SocketAddr, dir: PathBuf) {
    // Create KV Engine

    let noop = DbOp {
        id: 0,
        op_type: DbOpType::Noop,
    };
    let peers = vec![];

    let engine = KvStore::open(dir).unwrap();

    let (applied_tsns_tx, applied_tsns_rx) = unbounded();

    // Create State Machine
    let state_machine = Storage {
        engine,
        pending_transitions: Vec::new(),
        applied_transitions: applied_tsns_tx,
    };

    // Create Cluster
    let cluster = StorageCluster {
        id: node_id,
        is_leader: false,
        peers: peers.clone(),
        pending_messages: Vec::new(),
        halt: false,
    };

    // Create Channels
    let (msg_tx, msg_rx) = unbounded();
    let (tsn_tx, tsn_rx) = unbounded();

    // Create Axum State
    let state = Arc::new(App {
        id: node_id,
        addr: addr.to_string(),
        state_machine: Arc::new(Mutex::new(state_machine)),
        cluster: Arc::new(Mutex::new(cluster)),
        msg_tx: Arc::new(Mutex::new(msg_tx)),
        tsn_tx: Arc::new(Mutex::new(tsn_tx)),
        applied_tsns_rx: Arc::new(applied_tsns_rx),
        counter: Arc::new(Mutex::new(1)),
    });

    // Start Raft node
    let noop = noop.clone();
    let local_peer_ids = peers.iter().map(|x| x.id).collect();
    let cluster = state.cluster.clone();
    let state_machine = state.state_machine.clone();

    thread::spawn(move || {
        let mut replica = Replica::new(
            node_id,
            local_peer_ids,
            cluster,
            state_machine,
            1,
            noop.clone(),
            HEARTBEAT_TIMEOUT,
            (MIN_ELECTION_TIMEOUT, MAX_ELECTION_TIMEOUT),
        );

        replica.start(msg_rx, tsn_rx);
    });

    // Start Axum Server

    let app = Router::new()
        .route("/get", get(api::handle_get))
        .route("/set", get(api::handle_set))
        .with_state(state);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
