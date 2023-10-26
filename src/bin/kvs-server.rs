extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use ::clap::{Args, Parser, Subcommand};
use kvs::{KvsServer, NaiveThreadPool, Result, ThreadPool};

use slog::{info, o, Drain};

#[derive(Parser)]
#[clap(author, version)]
#[clap(about = "KV server")]
struct Cli {
    #[arg(short = 'a', long = "addr", default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[arg(short = 'e', long = "engine", default_value = "kvs")]
    engine: String,
    #[arg(short = 'p', long = "pool", default_value = "naive")]
    pool: String,
    #[arg(short, long, default_value = ".")]
    dir: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!());

    // start server

    let logger = log.new(o!("ip" => cli.addr.to_string() , 
                                            "version" => env!("CARGO_PKG_VERSION"),
                                            "engine" => cli.engine.clone()));

    info!(logger, "Starting server");

    let engine = match cli.engine.as_str() {
        "kvs" => kvs::KvStore::open(PathBuf::from(&cli.dir))?,
        _ => panic!("Unknown engine"),
    };

    let pool = match cli.pool.as_str() {
        "naive" => NaiveThreadPool::new(10)?,
        _ => panic!("Unknown pool"),
    };

    let srv = KvsServer::new(cli.addr, engine, cli.dir, logger, pool);

    srv.start();

    Ok(())
}
