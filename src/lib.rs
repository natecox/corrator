use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Mutex;
use std::thread;

pub mod application;
pub mod container;
pub mod docker;
pub mod end_of_life;

pub type ContainerMap = BTreeMap<String, container::Container>;
pub type ApplicationMap = BTreeMap<String, application::Application>;

/// Function to use when filtering by tags
#[derive(ValueEnum, Serialize, Deserialize, Clone, Debug, Default)]
pub enum FilterFunction {
	#[default]
	Any,
	All,
}

/// Various runtime options
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Options {
	/// Remove the docker image after execution
	clean_after_query: bool,
	/// Container tags to filter by
	tags: Option<Vec<String>>,
	/// Filter function for tags, e.g., "any"
	filter_function: FilterFunction,
	/// Container names to filter by
	names: Option<Vec<String>>,
}

impl Options {
	pub fn new(
		clean_after_query: bool,
		tags: Option<Vec<String>>,
		names: Option<Vec<String>>,
		filter_function: FilterFunction,
	) -> Self {
		Self {
			clean_after_query,
			tags,
			names,
			filter_function,
		}
	}
}

/// Runtime config required to run the app
#[derive(Serialize, Deserialize, Default)]
pub struct Config {
	containers: ContainerMap,
	applications: ApplicationMap,
	options: Options,
}

impl Config {
	/// Create a new runtime configuration dataset
	///
	/// # Example
	/// ```rust
	/// # use std::error::Error;
	/// # use corrator::{Config, ContainerMap, ApplicationMap, Options};
	/// # let containers = ContainerMap::new();
	/// # let applications = ApplicationMap::new();
	/// # let options = Options::default();
	/// Config::new(containers, applications, options);
	/// ```
	pub fn new(containers: ContainerMap, applications: ApplicationMap, options: Options) -> Self {
		let containers = Self::filter_by_names(containers, &options.names);
		let containers = Self::filter_by_tags(containers, &options.tags, &options.filter_function);

		Self {
			containers,
			applications,
			options,
		}
	}

	/// Consume this Config to generate a result set.
	///
	/// # Example
	///
	/// ```no_run
	/// # let config = corrator::Config::default();
	/// config.run();
	/// ```
	pub fn run(self) -> Result<Vec<container::Status>, Box<dyn Error>> {
		run(self)
	}

	fn filter_by_tags(
		containers: ContainerMap,
		tags: &Option<Vec<String>>,
		filter_function: &FilterFunction,
	) -> ContainerMap {
		match tags {
			Some(x) => containers
				.into_iter()
				.filter(|(_, c)| match &c.tags {
					Some(t) => match &filter_function {
						FilterFunction::Any => t.iter().any(|y| x.contains(y)),
						FilterFunction::All => x.iter().all(|y| t.contains(y)),
					},
					None => false,
				})
				.collect(),
			None => containers,
		}
	}

	fn filter_by_names(containers: ContainerMap, names: &Option<Vec<String>>) -> ContainerMap {
		match names {
			Some(x) => containers
				.into_iter()
				.filter(|(name, _)| x.contains(name))
				.collect(),
			None => containers,
		}
	}
}

fn run(config: Config) -> Result<Vec<container::Status>, Box<dyn Error>> {
	let data = Mutex::new(vec![]);

	thread::scope(|s| {
		for entry in config.containers {
			s.spawn(|| {
				let (name, mut container) = entry;
				let mut container_status = container::Status::new(name.clone());
				let instance = docker::Docker::new(&name, &container.path);

				container.apps.sort();
				instance.run().expect("Unable to start docker container");

				for app_name in container.apps {
					let app =
						match config.applications.get(&app_name) {
							Some(app) => app,
							None => {
								eprintln!(
									"Config error for {} on {}: App is not defined",
									&app_name, &name
								);
								eprintln!("-- hint: If you're sure you have a config for {}, look for typos.", &app_name);
								continue;
							}
						};
					let output = instance.execute(&app.version_command);

					match app.query_version(&output) {
						Ok(version) => {
							let eol_status: Option<String> = match &app.eol {
								Some(x) => match x.query(&version) {
									Ok(cycle) => Some(cycle.into()),
									_ => None,
								},
								_ => None,
							};

							container_status.apps.push(application::Status {
								name: app_name,
								version,
								eol_status,
							});
						}
						_ => {
							eprintln!("Error querying app version for {} on {}", &app_name, &name);
							eprintln!("-- hint: Your version command was: {}", app.version_command);
							eprintln!(
								"         Your regex query was: {}",
								app.version_regex.as_str()
							);
							eprintln!("         Your regex input was: {output}");
						}
					}
				}

				instance
					.stop(config.options.clean_after_query)
					.expect("Unable to clean up docker container");

				let mut data = data.lock().unwrap();
				data.push(container_status);
			});
		}
	});

	let mut data = data.into_inner().unwrap();
	data.sort_by_key(|x| x.name.clone());
	Ok(data)
}

#[cfg(test)]
mod tests {
	use crate::{
		container::Container, ApplicationMap, Config, ContainerMap, FilterFunction, Options,
	};

	#[test]
	fn filter_by_any() {
		let containers = ContainerMap::from([
			(
				String::from("test"),
				Container {
					tags: Some(vec![String::from("one"), String::from("two")]),
					..Default::default()
				},
			),
			(
				String::from("again"),
				Container {
					tags: None,
					..Default::default()
				},
			),
		]);

		let applications = ApplicationMap::new();
		let options = Options {
			tags: Some(vec![String::from("one")]),
			filter_function: FilterFunction::Any,
			..Default::default()
		};

		let config = Config::new(containers, applications, options);

		assert_eq!(config.containers.len(), 1)
	}

	#[test]
	fn filter_by_all() {
		let containers = ContainerMap::from([
			(
				String::from("test"),
				Container {
					tags: Some(vec![String::from("one"), String::from("two")]),
					..Default::default()
				},
			),
			(
				String::from("again"),
				Container {
					tags: Some(vec![String::from("one")]),
					..Default::default()
				},
			),
		]);

		let applications = ApplicationMap::new();
		let options = Options {
			tags: Some(vec![String::from("one"), String::from("two")]),
			filter_function: FilterFunction::All,
			..Default::default()
		};

		let config = Config::new(containers, applications, options);

		assert_eq!(config.containers.len(), 1)
	}

	#[test]
	fn filter_by_name() {
		let containers = ContainerMap::from([
			(String::from("test"), Container::default()),
			(String::from("again"), Container::default()),
		]);
		let applications = ApplicationMap::new();
		let options = Options {
			names: Some(vec![String::from("test")]),
			..Default::default()
		};

		let config = Config::new(containers, applications, options);

		assert_eq!(config.containers.len(), 1);
	}
}
