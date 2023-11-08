use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use axum::{
    routing::{get, post},
    Router,
};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use little_raft::replica::Replica;
use tokio::sync::broadcast;

use crate::{
    api, engines::inmem::InMemEngine, raft::network, DbOp, DbOpType, KvStore, KvsEngine, Node,
    Storage, StorageCluster,
};

const HEARTBEAT_TIMEOUT: Duration = Duration::from_millis(250); // 5000
const MIN_ELECTION_TIMEOUT: Duration = Duration::from_millis(30000); //30000
const MAX_ELECTION_TIMEOUT: Duration = Duration::from_millis(600000); //60000

// Representation of an application state. This struct can be shared around to share
// instances of raft, store and more.
pub struct App<K: KvsEngine + Sync> {
    pub id: usize,
    pub addr: String,
    pub state_machine: Arc<Mutex<Storage<K>>>,
    pub cluster: Arc<Mutex<StorageCluster>>,
    pub msg_tx: Sender<()>,
    pub tsn_tx: Sender<()>,
    pub applied_tsns_rx: broadcast::Receiver<usize>,
    pub counter: Arc<Mutex<usize>>,
}

pub async fn run_raft_node(
    node_id: usize,
    engine: &str,
    addr: SocketAddr,
    dir: PathBuf,
    is_leader: bool,
) {
    // Create KV Engine

    let noop = DbOp {
        id: 0,
        op_type: DbOpType::Noop,
    };
    let mut peers = vec![
        Node {
            id: 1,
            addr: "127.0.0.1:4001".parse().unwrap(),
        },
        Node {
            id: 2,
            addr: "127.0.0.1:4002".parse().unwrap(),
        },
        Node {
            id: 3,
            addr: "127.0.0.1:4003".parse().unwrap(),
        },
    ];
    peers = peers
        .into_iter()
        .filter(|node| node.id != node_id)
        .collect();

    let engine = InMemEngine::open(PathBuf::from(""));

    let (applied_tsns_tx, applied_tsns_rx) = broadcast::channel::<usize>(50);

    // Create State Machine
    let state_machine = Storage {
        engine,
        pending_transitions: Vec::new(),
        applied_transitions: applied_tsns_tx,
    };

    // Create Cluster
    let cluster = StorageCluster {
        id: node_id,
        is_leader: is_leader,
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
        msg_tx: msg_tx,
        tsn_tx: tsn_tx,
        applied_tsns_rx: applied_tsns_rx,
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
            0,
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
        // .route("/rm", get(api::handle_remove))
        .route("/msg", post(network::handle_msg))
        .with_state(state);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
