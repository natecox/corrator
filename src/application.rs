use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

use crate::end_of_life;

#[derive(Debug)]
pub struct RegexCaptureError;
impl Error for RegexCaptureError {}
impl fmt::Display for RegexCaptureError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Could not capture version from input")
	}
}

/// Configuration details for an application
///
/// A representation of a corrator.toml config provided by a user.
#[derive(Serialize, Deserialize)]
pub struct Application {
	/// A regex pattern for pulling version numbers from command output
	#[serde(with = "serde_regex")]
	pub version_regex: Regex,

	/// The command to run to determine current version, e.g., `bash --version`
	pub version_command: String,

	/// An optional endoflife.date config
	pub eol: Option<end_of_life::EolConfig>,
}

impl Application {
	/// Given the output of version_command, retrieve the version number via regex
	///
	/// # Example
	/// ```rust
	/// # use std::error::Error;
	/// # use regex::Regex;
	/// # fn main() -> Result<(), Box<dyn Error>> {
	/// let application = corrator::application::Application {
	///     version_regex: Regex::new(r"test: (?P<version>[0-9.]+)")?,
	///     version_command: String::from(""),
	///     eol: None,
	/// };
	///
	/// let version = application.query_version("test: 1.2.3")?;
	/// #     assert_eq!(version, String::from("1.2.3"));
	/// #     Ok(())
	/// # }
	/// ```
	pub fn query_version(&self, input: &str) -> Result<String, Box<dyn Error>> {
		let results = self.version_regex.captures(input).and_then(|cap| {
			cap.name("version")
				.map(|version| String::from(version.as_str()))
		});

		match results {
			Some(x) => Ok(x),
			_ => Err(Box::new(RegexCaptureError)),
		}
	}
}

/// The currency status of an application
///
/// Contains the current version and an optional endoflife.date response
#[derive(Serialize, Debug)]
pub struct Status {
	pub name: String,
	pub version: String,

	/// A parsed output from endoflife.date; either a date representing
	/// the end of support, or "alive" if no date has been set
	pub eol_status: Option<String>,
}
