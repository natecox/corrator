use std::error::Error;
use std::process::{self, Command};

pub struct Docker<'a> {
	name: &'a str,
	path: &'a str,
}

impl Docker<'_> {
	pub fn new<'a>(name: &'a str, path: &'a str) -> Docker<'a> {
		Docker { name, path }
	}

	pub fn run(&self) -> Result<(), Box<dyn Error>> {
		Command::new("docker")
			.arg("run")
			.arg("--rm")
			.args(["--entrypoint", ""])
			.args(["--pull", "always"])
			.args(["--name", (self.name)])
			.arg("-dit")
			.arg(self.path)
			.arg("bash")
			.output()
			.unwrap_or_else(|err| {
				eprintln!("Unable to stand docker container up: {err}");
				process::exit(1);
			});

		Ok(())
	}

	pub fn stop(&self, remove: bool) -> Result<(), Box<dyn Error>> {
		Command::new("docker")
			.args(["stop", (self.name)])
			.output()
			.unwrap_or_else(|err| {
				eprintln!("Unable to stop docker container: {err}");
				process::exit(1);
			});

		if remove {
			self.clean().expect("Unable to clean up image");
		}

		Ok(())
	}

	pub fn execute(&self, args: &str) -> String {
		let output = Command::new("docker")
			.args(["exec", (self.name)])
			.args(args.split(' '))
			.output()
			.unwrap_or_else(|err| {
				eprintln!("Unable to run docker command: {err}");
				process::exit(1);
			});

		String::from_utf8(output.stdout).unwrap()
	}

	pub fn clean(&self) -> Result<(), Box<dyn Error>> {
		Command::new("docker")
			.args(["rmi", "-f", (self.path)])
			.output()
			.unwrap_or_else(|err| {
				eprintln!("Unable to remove docker image: {err}");
				process::exit(1);
			});

		Ok(())
	}
}
