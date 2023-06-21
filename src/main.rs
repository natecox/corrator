extern crate xdg;

use clap::Parser;
use corrator::{ApplicationMap, Config, ContainerMap};
use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(short, long, default_value_t = default_config_path())]
	config_path: String,

	#[arg(short, long, default_value = "text", value_parser = ["text", "json"])]
	format: String,

	/// Enable flag to remove images after version queries
	#[arg(long)]
	clean: bool,
}

#[derive(Serialize, Deserialize)]
struct ConfigData {
	containers: ContainerMap,
	applications: ApplicationMap,
}

fn default_config_path() -> String {
	match xdg::BaseDirectories::with_prefix("corrator") {
		Ok(x) => String::from(x.get_config_file("corrator.toml").to_str().unwrap()),
		_ => String::from("~/.corrator"),
	}
}

fn main() {
	let args = Args::parse();

	let config_data = match env::var("CORRATOR_CONFIG_PATH") {
		Ok(x) => x,
		_ => args.config_path,
	};
	let config_data = shellexpand::tilde(&config_data).to_string();
	let config_data = fs::read_to_string(config_data).expect("Cound not read config file");
	let config_data: ConfigData =
		toml::from_str(&config_data).expect("Could not parse config file");

	let config = Config::new(config_data.containers, config_data.applications, args.clean);
	if let Ok(data) = corrator::run(config) {
		match args.format.as_str() {
			"text" => output_as_text(data),
			"json" => output_as_json(data),
			_ => println!("unknown format"),
		}
	}
}

fn output_as_text(data: Vec<corrator::container::Status>) {
	for x in data {
		println!("{}", String::from(x));
	}
}

fn output_as_json(data: Vec<corrator::container::Status>) {
	println!("{}", serde_json::to_string(&data).unwrap());
}
