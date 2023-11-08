use bytes::Bytes;
use little_raft::message::Message;
use url_builder::URLBuilder;

use crate::{DbOp, Node};

use super::msg::Msg;

pub fn send_message(node: &Node, message: Message<DbOp, Bytes>) {
    let node_addr = node.addr;

    let msg = Msg::from(message);

    // let msg_body = serde_json::to_string(&msg).unwrap();

    let mut ub = URLBuilder::new();

    ub.set_protocol("http")
        .set_host(node_addr.ip().to_string().as_str())
        .set_port(node_addr.port())
        .add_route("msg");

    let url = ub.build();

    let echo_json = reqwest::blocking::Client::new().post(url).json(&msg).send();

    println!(
        "Sending message to id_{} {:?} \n response : {:?}",
        node.id, msg, echo_json
    );
}
