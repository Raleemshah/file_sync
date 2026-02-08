use std::{env, path::PathBuf, process};

use file_sync::sync;

fn main() {
    let args: Vec<PathBuf> = env::args().map(PathBuf::from).collect();
    let config = Config::Build(&args).unwrap_or_else(|err| {
        println!("Error: {0}", err);
        process::exit(1);
    });

    sync(&config.source, &config.destination);
}

struct Config {
    source: PathBuf,
    destination: PathBuf,
}

impl Config {
    fn Build(args: &[PathBuf]) -> Result<Self, &'static str> {
        if args.len() < 3 {
            return Err("Less args provided");
        } else {
            Ok(Self {
                source: args[1].clone(),
                destination: args[2].clone(),
            })
        }
    }
}
