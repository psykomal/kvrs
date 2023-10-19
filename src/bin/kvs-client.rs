use ::clap::{Args, Parser, Subcommand};
use kvs::KvStore;
use kvs::Result;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "KV client")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long, default_value = ".")]
    dir: String,
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
    let mut kv = KvStore::open(cli.dir)?;

    match &cli.command {
        Some(Commands::Get(args)) => {
            if args.key == "" {
                std::process::exit(1);
            }

            if let Ok(value) = kv.get(args.key.clone()) {
                match value {
                    Some(value) => println!("{}", value),
                    None => {
                        println!("Key not found");
                        std::process::exit(0);
                    }
                }
            }
            Ok(())
        }
        Some(Commands::Set(args)) => {
            return kv.set(args.key.clone(), args.value.clone());
        }
        Some(Commands::Rm(args)) => {
            if let Ok(_) = kv.remove(args.key.clone()) {
                Ok(())
            } else {
                println!("Key not found");
                std::process::exit(1);
            }
        }
        _ => {
            println!("Unknown method");
            std::process::exit(1);
        }
    }
}
