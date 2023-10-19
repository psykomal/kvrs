use std::net::SocketAddr;

use ::clap::{Args, Parser, Subcommand};
use kvs::KvStore;
use kvs::KvsClient;
use kvs::Result;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "KV client")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long, default_value = ".")]
    dir: String,
    #[arg(short, long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
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
}

#[derive(Args)]
struct Set {
    key: String,
    value: String,
}

#[derive(Args)]
struct Rm {
    key: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = KvsClient::new(cli.addr);

    match &cli.command {
        Some(Commands::Get(args)) => {
            let val = client.get(args.key.clone())?;
            println!("{}", val.unwrap());
            Ok(())
        }
        Some(Commands::Set(args)) => {
            client.set(args.key.clone(), args.value.clone())?;
            Ok(())
        }
        Some(Commands::Rm(args)) => {
            client.remove(args.key.clone())?;
            Ok(())
        }
        _ => {
            println!("Unknown method");
            std::process::exit(1);
        }
    }
}
