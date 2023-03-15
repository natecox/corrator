use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::process::{self, Command};

mod application;
mod container;
mod end_of_life;

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

    pub fn run(&self) {
        for (name, container) in self.containers.iter() {
            println!("\n---Container: {name:-<35}");

            let mut apps = container.apps.clone();
            apps.sort();

            for app_name in apps.iter() {
                let app = self.get_application(String::from(app_name));
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

                let version = app
                    .query_version(&String::from_utf8(output.stdout).unwrap())
                    .unwrap();

                let eol_status: String = match &app.eol {
                    Some(x) => {
                        let version = x.version_regex.find(&version).unwrap().as_str();
                        format!(
                            "(eol: {})",
                            end_of_life::query(app_name, version).unwrap().eol
                        )
                    }
                    _ => String::from(""),
                };

                println!("\t{app_name: <15}{version: <10} {eol_status}");
            }
        }
    }

    fn get_application(&self, name: String) -> &application::Application {
        self.applications.get(&name).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_application_by_name() {
        let config = Config::new(&String::from("corrator.toml")).unwrap();

        println!("{:?}", config);

        assert_eq!(
            config.get_application(String::from("bash")).version_command,
            "bash --version"
        );
    }
}
