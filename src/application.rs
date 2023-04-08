use std::error::Error;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::end_of_life;

#[derive(Serialize, Deserialize)]
pub struct Application {
	#[serde(with = "serde_regex")]
	pub version_regex: Regex,
	pub version_command: String,
	pub eol: Option<end_of_life::EolConfig>,
}

impl Application {
	pub fn query_version(&self, input: &str) -> Result<String, Box<dyn Error>> {
		let results = self
			.version_regex
			.captures(input)
			.and_then(|cap| cap.name("version").map(|version| version.as_str()))
			.unwrap();

		Ok(results.to_string())
	}
}

#[derive(Debug)]
pub struct Status {
	pub name: String,
	pub version: String,
	pub eol_status: Option<String>,
}

impl Status {
	pub fn as_vec(&self) -> Vec<String> {
		let eol_status = match &self.eol_status {
			Some(x) => String::from(x),
			None => String::from(""),
		};

		vec![
			String::from(&self.name),
			String::from(&self.version),
			String::from(eol_status),
		]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn valid_query_returns_version() {
		let app = Application {
			version_regex: Regex::new(r"test: (?P<version>[0-9.]+)").unwrap(),
			version_command: String::from(""),
			eol: None,
		};

		assert_eq!(
			String::from("1.2.3"),
			app.query_version("test: 1.2.3").unwrap()
		);
	}
}
