use std::net::SocketAddr;

use ::clap::{Args, Parser, Subcommand};
use kvs::KvStore;
use kvs::KvsClient;
use kvs::Result;
use slog::info;
use slog::o;
use slog::Drain;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "KV client")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Get(Get),
    Set(Set),
    Rm(Rm),
}

#[derive(Args)]
struct Get {
    key: String,
    #[arg(short, long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
}

#[derive(Args)]
struct Set {
    key: String,
    value: String,
    #[arg(short, long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
}

#[derive(Args)]
struct Rm {
    key: String,
    #[arg(short, long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!());

    // start server

    let logger = log.new(o!());

    match &cli.command {
        Some(Commands::Get(args)) => {
            let client = KvsClient::new(args.addr, logger);
            let val = client.get(args.key.clone())?;
            println!("{}", val.unwrap());
            Ok(())
        }
        Some(Commands::Set(args)) => {
            let client = KvsClient::new(args.addr, logger);
            client.set(args.key.clone(), args.value.clone())?;
            Ok(())
        }
        Some(Commands::Rm(args)) => {
            let client = KvsClient::new(args.addr, logger);
            client.remove(args.key.clone())?;
            Ok(())
        }
        _ => {
            println!("Unknown method");
            std::process::exit(1);
        }
    }
}
