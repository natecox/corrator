extern crate xdg;

use clap::Parser;
use corrator::Config;
use std::{env, error::Error, process};

mod tui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(short, long, default_value_t = default_config_path())]
	config_path: String,

	#[arg(short, long, default_value = "text", value_parser = ["text", "tui"])]
	format: String,
}

fn default_config_path() -> String {
	match xdg::BaseDirectories::with_prefix("corrator") {
		Ok(x) => String::from(x.get_config_file("corrator.toml").to_str().unwrap()),
		_ => String::from("~/.corrator"),
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let args = Args::parse();
	let file_path = match env::var("CORRATOR_CONFIG_PATH") {
		Ok(x) => x,
		_ => args.config_path,
	};
	let file_path = shellexpand::tilde(&file_path).to_string();

	let config = Config::new(&file_path).unwrap_or_else(|err| {
		eprintln!("unable to parse config file: {err}");
		process::exit(1);
	});

	match args.format.as_str() {
		"tui" => tui::render_tui(config),
		_ => render_text(config),
	}
}

fn render_text(config: Config) -> Result<(), Box<dyn Error>> {
	let data = corrator::run(config).expect("Could not process docker commands.");
	for x in data {
		println!("{}", String::from(x));
	}

	Ok(())
}
