use chrono::NaiveDate;
use regex::Regex;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::{convert::From, process};

pub mod cache;

/// Configuation details for an endoflife.date cycle
#[derive(Serialize, Deserialize, Debug)]
pub struct EolConfig {
	pub product_name: String,
	#[serde(with = "serde_regex")]
	pub version_regex: Regex,
}

impl EolConfig {
	/// Fetch EOL data for an application version from endoflife.dat
	///
	/// First looks to see if this application/version combo has been previously
	/// cached and avoids the network call if possible.
	pub fn query(&self, input: &str) -> Result<Cycle, Error> {
		let version = self.version_regex.find(input).unwrap().as_str();

		if let Some(x) = cache::get_cycle(&self.product_name, version).unwrap() {
			Ok(x)
		} else {
			let request_url = format!(
				"https://endoflife.date/api/{}/{}.json",
				&self.product_name, &version
			);

			let response = reqwest::blocking::get(&request_url)?;
			if response.status() != 200 {
				eprintln!("Unable to query {request_url}");
				eprintln!("-- hint: You may want to check the version number with endoflife.date.");
				eprintln!("         If your url has extra digits at the end you may need to add");
				eprintln!("         a version_regex to the application's eol config.");
				process::exit(1);
			}

			let response = response.json::<Cycle>()?;

			// Don't cache responses where an end of life date hasn't yet been set
			match response.eol {
				EOLDate::String(_) => {
					Ok(cache::insert_cycle(&self.product_name, version, response)
						.expect("failed to insert cached cycle"))
				}
				EOLDate::Boolean(_) => Ok(response),
			}
		}
	}
}

/// Representation of an endoflife.date cycle object returned by their API
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
	pub eol: EOLDate,
	pub support: Option<EOLDate>,
	pub latest: String,
	pub latest_release_date: String,
	pub release_date: String,
	pub lts: bool,
}

impl From<Cycle> for String {
	fn from(item: Cycle) -> Self {
		item.eol.into()
	}
}

/// Fix for `false` returned by endoflife.date rather than null
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EOLDate {
	String(String),
	Boolean(bool),
}

impl Default for EOLDate {
	fn default() -> Self {
		Self::Boolean(false)
	}
}

impl From<EOLDate> for String {
	/// Sanitize endoflife.date output to either a date or "alive"
	///
	/// A value of "alive" represents that no EOL date has yet been set for the
	/// requested version.
	fn from(value: EOLDate) -> Self {
		match value {
			EOLDate::String(x) => NaiveDate::parse_from_str(&x, "%Y-%m-%d")
				.unwrap()
				.format("%x")
				.to_string(),
			EOLDate::Boolean(_) => "alive".into(),
		}
	}
}
