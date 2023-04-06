use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

pub mod application;
pub mod container;
pub mod docker;
pub mod end_of_life;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub containers: BTreeMap<String, container::Container>,
    pub applications: BTreeMap<String, application::Application>,
}

impl Config {
    pub fn new(file_path: &String) -> Result<Self, Box<dyn Error>> {
        let contents = fs::read_to_string(file_path)?;
        let results: Self = toml::from_str(&contents)?;

        Ok(results)
    }
}

pub fn run(config: Config) -> Result<Vec<container::Status>, Box<dyn Error>> {
    let mut data = vec![];

    for (name, container) in config.containers.iter() {
        let mut container_status = container::Status::new(name.to_string());
        let mut apps = container.apps.clone();
        apps.sort();

        docker::run(name, &container.path).expect("Unable to start docker container");

        for app_name in apps.iter() {
            let app = config.applications.get(app_name).unwrap();
            let output = docker::execute(name, &app.version_command);
            let version = app.query_version(&output).unwrap();

            let eol_status: String = match &app.eol {
                Some(x) => {
                    let status: String = x.query(&version).unwrap().into();
                    format!("(eol: {status})")
                }
                _ => String::from(""),
            };

            container_status.apps.push(application::Status {
                name: app_name.to_string(),
                version,
                eol_status: Some(eol_status),
            });
        }

        docker::stop(name).expect("Unable to clean up docker container");

        data.push(container_status);
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_application_by_name() {
        let config = Config::new(&String::from("corrator.toml")).unwrap();

        assert_eq!(
            config.applications.get("bash").unwrap().version_command,
            "bash --version"
        );
    }
}
