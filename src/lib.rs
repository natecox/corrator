use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::process::{self, Command};

mod application;
mod container;
mod end_of_life;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub containers: Vec<crate::container::Container>,
    pub applications: Vec<crate::application::Application>,
}

impl Config {
    pub fn new(file_path: &String) -> Result<Self, Box<dyn Error>> {
        let contents = fs::read_to_string(file_path)?;
        let results: Self = toml::from_str(&contents)?;

        Ok(results)
    }

    pub fn run(&self) {
        for container in self.containers.iter() {
            println!("\n-- Container: {} --------", container.name);

            for app in container.apps.iter() {
                let app = self.get_application(String::from(app));
                let output = Command::new("docker")
                    .arg("run")
                    .args(["--entrypoint", "", "--pull", "always"])
                    .arg(&container.path)
                    .args(app.version_command.split(' '))
                    .output()
                    .unwrap_or_else(|err| {
                        eprintln!("Unable to run docker command: {err}");
                        process::exit(1);
                    });

                let version = app.query_version(&String::from_utf8(output.stdout).unwrap()).unwrap();

                let eol_status = if app.eol_api_supported {
                    match end_of_life::query(&app.name, &version) {
                        Ok(x) => format!("(eol: {})", x.eol),
                        Err(e) => panic!("Unable to query endoflife.date: {e}"),
                    }
                } else {
                    String::from("")
                };

                println!("{}, {} {}", app.name, version, eol_status);
            }
        }
    }

    fn get_application(&self, name: String) -> &application::Application {
        self.applications
            .iter()
            .find(|x| x.name == name)
            .expect("Could not find application with name")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_application_by_name() {
        let config = Config::new(&String::from("corrator.toml")).unwrap();

        assert_eq!(config.get_application(String::from("bash")).name, "bash");
    }
}
