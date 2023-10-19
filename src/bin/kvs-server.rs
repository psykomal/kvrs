extern crate slog;
extern crate slog_async;
extern crate slog_term;

use std::net::{IpAddr, SocketAddr};

use ::clap::{Args, Parser, Subcommand};
use kvs::{KvsServer, Result};

use slog::{info, o, Drain};

#[derive(Parser)]
#[clap(author, version)]
#[clap(about = "KV server")]
struct Cli {
    #[arg(short = 'a', long = "addr", default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[arg(short = 'e', long = "engine", default_value = "kvs")]
    engine: String,
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

    let srv = KvsServer::new(cli.addr, cli.engine.clone(), cli.dir, logger);

    srv.start();

    Ok(())
}
