use std::{env, process};

use corrator::Config;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "~/.corrator.toml")]
    config_path: String,
}

fn main() {
    let args = Args::parse();
    let file_path = match env::var("CORRATOR_CONFIG_PATH") {
        Ok(x) => x,
        _ => args.config_path,
    };

    let config = Config::new(&file_path).unwrap_or_else(|err| {
        eprintln!("unable to parse config file: {err}");
        process::exit(1);
    });

    config.run();
}
