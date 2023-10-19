use std::net::IpAddr;

use ::clap::{Args, Parser, Subcommand};
use kvs::Result;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "KV server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long, default_value = ".")]
    dir: String,
}

#[derive(Subcommand)]
enum Commands {
    Start(Start),
}

#[derive(Args)]
struct Start {
    #[arg(short, long, default_value = "127.0.0.1:4000")]
    addr: IpAddr,
    #[arg(short, long, default_value = "kvs")]
    engine: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Start(args)) => {
            // start server
        }
        None => {}
    }

    Ok(())
}
