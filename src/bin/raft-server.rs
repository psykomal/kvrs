extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use ::clap::{Args, Parser, Subcommand};
use kvs::{
    run_raft_node, KvsEngine, KvsServer, NaiveThreadPool, RayonThreadPool, Result,
    SharedQueueThreadPool, ThreadPool,
};

use slog::{info, o, Drain, Logger};

#[derive(Parser)]
#[clap(author, version)]
#[clap(about = "KV server")]
struct Cli {
    #[arg(short = 'i', long = "id")]
    id: u64,
    #[arg(short = 'a', long = "addr", default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[arg(short = 'e', long = "engine", default_value = "kvs")]
    engine: String,
    #[arg(short, long, default_value = ".")]
    dir: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!());

    // start server

    let logger = log.new(o!("ip" => cli.addr.to_string() ,
    "version" => env!("CARGO_PKG_VERSION"),
    "engine" => cli.engine.clone(),
    ));

    info!(logger, "Starting server");

    run_raft_node(
        cli.id as usize,
        cli.engine.as_str(),
        SocketAddr::from(cli.addr),
        PathBuf::from(cli.dir),
    )
    .await;
}
