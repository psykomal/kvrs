use clap::Parser;
use kvs::KvStore;
use kvs::Result;

#[derive(Parser, Debug)]
struct Cli {
    /// The pattern to look for
    #[arg(short, long, default_value_t = env!("CARGO_PKG_VERSION").to_string())]
    version: String,

    #[arg(short = 'a', long, default_value_t = env!("CARGO_PKG_AUTHORS").to_string())]
    author: String,

    #[arg(short = 'b', long, default_value_t = env!("CARGO_PKG_DESCRIPTION").to_string())]
    about: String,

    method: String,
    key: String,
    value: Option<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut kv = KvStore::open("./data")?;
    println!("args: {:?}", args);

    match args.method.as_str() {
        "get" => {
            let value = kv.get(args.key).unwrap_or_else(|_| {
                println!("Key not found");
                std::process::exit(1);
            });
            println!("{:?}", value.unwrap());
            Ok(())
        }
        "set" => {
            return kv.set(args.key, args.value.unwrap());
        }
        "rm" => {
            return kv.remove(args.key);
        }
        _ => {
            println!("Unknown method");
            std::process::exit(1);
        }
    }
}
