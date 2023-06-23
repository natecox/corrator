extern crate xdg;

use clap::Parser;
use corrator::{Config, Options};
use serde::de::DeserializeOwned;
use std::{fs, path::Path};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(short, long, default_value_t = default_config_path())]
	config_directory: String,

	#[arg(short, long, default_value = "text", value_parser = ["text", "json"])]
	format: String,

	/// Enable flag to remove images after version queries
	#[arg(long)]
	clean: bool,

	/// Filter containers by tag; can be used multiple times
	#[arg(short, long)]
	tag: Option<Vec<String>>,

	/// Filter function for tagging
	#[arg(long, value_enum, default_value_t = corrator::FilterFunction::Any)]
	filter: corrator::FilterFunction,

	/// Filter containers by name; can be used multiple times
	#[arg(short, long)]
	name: Option<Vec<String>>,
}

impl From<&Args> for Options {
	fn from(args: &Args) -> Self {
		Self::new(
			args.clean,
			args.tag.clone(),
			args.name.clone(),
			args.filter.clone(),
		)
	}
}

fn default_config_path() -> String {
	xdg::BaseDirectories::with_prefix("corrator")
		.unwrap()
		.get_config_home()
		.to_str()
		.unwrap()
		.into()
}

fn main() {
	let args = Args::parse();
	let options = Options::from(&args);

	let config = Path::new(&args.config_directory);
	let config = Config::new(
		parse_config_file(config, "containers.toml"),
		parse_config_file(config, "applications.toml"),
		options,
	);

	if let Ok(data) = config.run() {
		match args.format.as_str() {
			"text" => output_as_text(data),
			"json" => output_as_json(data),
			_ => println!("unknown format"),
		}
	}
}

fn parse_config_file<T: DeserializeOwned>(config_directory: &Path, file_name: &str) -> T {
	let mut config_directory = config_directory.to_path_buf();
	config_directory.push(file_name);

	let data = fs::read_to_string(config_directory).expect("Could not read config file");

	toml::from_str(&data).expect("Cound not read applications config file")
}

fn output_as_text(data: Vec<corrator::container::Status>) {
	for x in data {
		println!("{}", String::from(x));
	}
}

fn output_as_json(data: Vec<corrator::container::Status>) {
	println!("{}", serde_json::to_string(&data).unwrap());
}
