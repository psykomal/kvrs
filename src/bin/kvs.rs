use ::clap::{Args, Parser, Subcommand};
use kvs::KvStore;
use kvs::Result;

// #[derive(Parser, Debug)]
// struct Cli {
//     /// The pattern to look for
//     #[arg(short, long, default_value_t = env!("CARGO_PKG_VERSION").to_string())]
//     version: String,

//     #[arg(short = 'a', long, default_value_t = env!("CARGO_PKG_AUTHORS").to_string())]
//     author: String,

//     #[arg(short = 'b', long, default_value_t = env!("CARGO_PKG_DESCRIPTION").to_string())]
//     about: String,

//     method: String,
//     key: String,
//     value: Option<String>,
// }

#[derive(Parser)]
#[command(author, version)]
#[command(about = "KV store")]
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
    // #[arg(short, long)]
    key: String,
}

#[derive(Args)]
struct Set {
    // #[arg(short, long)]
    key: String,
    // #[arg(short, long)]
    value: String,
}

#[derive(Args)]
struct Rm {
    // #[arg(short, long)]
    key: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut kv = KvStore::open(".")?;

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
