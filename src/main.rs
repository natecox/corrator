use clap::Parser;
use core::panic;
use corrator::{ApplicationMap, Config, ContainerMap, Options};
use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Write, fs, path::Path, process::exit};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Specify a directory to load toml files from
	#[arg(short = 'd', long, default_value_t = default_config_path(), conflicts_with = "config_url", help_heading="Config Settings")]
	config_directory: String,

	/// URL to fetch a JSON formatted config
	///
	/// See corrator github repo for a JSON schema
	#[arg(
		short = 'u',
		long,
		conflicts_with = "config_directory",
		help_heading = "Config Settings"
	)]
	config_url: Option<String>,

	/// Validate config URL only and then exit
	///
	/// Program will exit with a failing status if validation is not successful
	#[arg(
		short = 'v',
		long,
		requires = "config_url",
		help_heading = "Config Settings"
	)]
	validate_config_url: bool,

	#[arg(short, long, default_value = "text", value_parser = ["text", "json"], help_heading = "Output")]
	format: String,

	/// Writes output to a file at this given path if provided
	///
	/// Will write to stdout if this option is not used
	#[arg(short, long, help_heading = "Output")]
	output: Option<String>,

	/// Enable flag to remove images after version queries
	#[arg(long)]
	clean: bool,

	/// Filter containers by tag; can be used multiple times
	#[arg(short, long, help_heading = "Filtering")]
	tag: Option<Vec<String>>,

	/// Filter function for tagging
	#[arg(long, value_enum, default_value_t = corrator::FilterFunction::Any, help_heading = "Filtering")]
	filter: corrator::FilterFunction,

	/// Filter containers by name; can be used multiple times
	#[arg(short, long, help_heading = "Filtering")]
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

#[derive(Serialize, Deserialize)]
struct JsonConfig {
	containers: ContainerMap,
	applications: ApplicationMap,
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

	let config = match &args.config_url {
		Some(x) => {
			let schema = include_str!("config.schema.json");
			let schema = serde_json::from_str(schema).expect("Could not read json schema!");

			let validator =
				jsonschema::validator_for(&schema).expect("Could not initialize json validator!");
			let config = get_config_from_url(x);
			let is_valid = validator.is_valid(&config);

			if args.validate_config_url {
				let exit_code = if is_valid { 0 } else { 1 };
				println!("is valid: {}", is_valid);
				exit(exit_code);
			}

			if !is_valid {
				panic!("Unable to validate the fetched config json!")
			}

			let json_config: JsonConfig = parse_config_url(x);
			Config::new(json_config.containers, json_config.applications, options)
		}

		None => {
			let directory = Path::new(&args.config_directory);
			Config::new(
				parse_config_file(directory, "containers.toml"),
				parse_config_file(directory, "applications.toml"),
				options,
			)
		}
	};

	if !args.keep_eol_cache {
		corrator::end_of_life::cache::clear().expect("Unable to clear EOL cache");
	}

	if let Ok(data) = config.run() {
		match args.format.as_str() {
			"text" => {
				let data: String = data.into_iter().fold(String::new(), |mut output, b| {
					write!(output, "{}\n\n", String::from(b)).expect("Unable to build output text");
					output
				});
				write_results(data, args);
			}
			"json" => write_results(serde_json::to_string(&data).unwrap(), args),
			_ => eprintln!("unknown format"),
		}
	}
}

fn get_config_from_url(url: &String) -> serde_json::Value {
	let response = reqwest::blocking::get(url).expect("Unable to reach config");

	if response.status() != 200 {
		eprintln!("Bad response from {}: {}", url, response.status());
		panic!();
	}

	response.json().expect("Unable to parse config from URL")
}

fn parse_config_url<T: DeserializeOwned>(url: &String) -> T {
	let response = reqwest::blocking::get(url).expect("Unable to reach config");

	if response.status() != 200 {
		eprintln!("Bad response from {}: {}", url, response.status());
		panic!();
	}

	response
		.json::<T>()
		.expect("Unable to parse URL into config")
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
