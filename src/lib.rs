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

/// Runtime config required to run the app
#[derive(Serialize, Deserialize)]
pub struct Config {
	containers: BTreeMap<String, container::Container>,
	applications: BTreeMap<String, application::Application>,
	clean_after_query: bool,
}

impl Config {
	/// Create a new runtime configuration dataset
	///
	/// # Example
	/// ```rust
	/// # use std::error::Error;
	/// # use corrator::{Config, ContainerMap, ApplicationMap};
	/// # use corrator::application::Application;
	/// # use corrator::container::Container;
	/// # fn main() -> Result<(), Box<dyn Error>> {
	/// #     let mut containers = ContainerMap::new();
	/// #     containers.insert(String::from("ubuntu"), Container::default());
	/// #     let mut applications = ApplicationMap::new();
	/// #     applications.insert(String::from("bash"), Application::default());
	/// #     let clean_after_query= false;
	/// #     
	/// Config::new(containers, applications, clean_after_query);
	/// #    Ok(())
	/// # }
	/// ```
	pub fn new(
		containers: ContainerMap,
		applications: ApplicationMap,
		clean_after_query: bool,
	) -> Self {
		Self {
			containers,
			applications,
			clean_after_query,
		}
	}
}

pub fn run(config: Config) -> Result<Vec<container::Status>, Box<dyn Error>> {
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
					.stop(config.clean_after_query)
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
