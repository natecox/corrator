use clap::Parser;
use corrator::{Config, Options};
use directories::ProjectDirs;
use serde::de::DeserializeOwned;
use std::{fs, path::Path};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(short, long, default_value_t = default_config_path())]
	config_directory: String,

	#[arg(short, long, default_value = "text", value_parser = ["text", "json"])]
	format: String,

	/// Writes output to a file at this given path if provided
	///
	/// Will write to stdout if this option is not used
	#[arg(short, long)]
	output: Option<String>,

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

	/// Do not clear the EOL cache before querying apps
	#[arg(short, long)]
	keep_eol_cache: bool,
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
	ProjectDirs::from("rs", "", "corrator")
		.expect("could not get project directory")
		.config_dir()
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

	if !args.keep_eol_cache {
		corrator::end_of_life::cache::clear().expect("Unable to clear EOL cache");
	}

	if let Ok(data) = config.run() {
		match args.format.as_str() {
			"text" => {
				let data: String = data
					.into_iter()
					.map(|x| format!("{}\n\n", String::from(x)))
					.collect();
				write_results(data, args);
			}
			"json" => write_results(serde_json::to_string(&data).unwrap(), args),
			_ => eprintln!("unknown format"),
		}
	}
}

fn parse_config_file<T: DeserializeOwned>(config_directory: &Path, file_name: &str) -> T {
	let config_directory = config_directory.join(file_name);

	let data = fs::read_to_string(config_directory).expect("Could not read config file");

	toml::from_str(&data).expect("Cound not read applications config file")
}

fn write_results(output: String, args: Args) {
	let output = output.trim();

	match args.output {
		Some(x) => fs::write(x, output).expect("Coult not write to file"),
		None => println!("{output}"),
	}
}
