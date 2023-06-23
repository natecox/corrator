use serde::{Deserialize, Serialize};

/// Configuration details for a container
///
/// A representation of a corrator.toml config provided by a user.
#[derive(Serialize, Deserialize, Debug)]
pub struct Container {
	/// A full path to a docker image
	pub path: String,

	/// A list of the apps to query, by name
	pub apps: Vec<String>,

	/// A list of tags for this container
	pub tags: Option<Vec<String>>,
}

// Added to simplify tests and documentation
impl Default for Container {
	fn default() -> Self {
		Self {
			path: String::from("path"),
			apps: vec![String::from("bash")],
			tags: None,
		}
	}
}

/// The currency status of a container
///
/// Contains a list of [`application::Status`] providing currency details
/// for each defined application in a container.
///
/// [`application::Status`]: ../application/Struct.status.html
#[derive(Serialize)]
pub struct Status {
	pub name: String,
	pub apps: Vec<crate::application::Status>,
}

impl Status {
	/// Create a new Status instance from a [`String`]
	///
	/// # Example
	/// ```rust
	/// corrator::container::Status::new(String::from("corrator"));
	/// ```
	pub fn new(name: String) -> Self {
		Self { name, apps: vec![] }
	}

	/// Serialize a Status instance as JSON data
	///
	/// # Example
	/// ```rust
	/// let name = String::from("corrator");
	/// let status = corrator::container::Status::new(name);
	/// status.to_json();
	/// ```
	pub fn to_json(&self) -> String {
		serde_json::to_string(&self).unwrap()
	}
}

impl From<Status> for String {
	/// A human readable representation of a container's currency
	///
	/// Includes version and end-of-life details for each configured
	/// app inside a container.
	///
	/// # Example
	///
	/// ---Container: ubuntu-----------------------------
	///     bash           5.1.16     
	///     grep           3.7        
	///     ubuntu         22.04      

	fn from(value: Status) -> Self {
		let mut output = vec![];
		output.push(format!("\n---Container: {:-<35}", value.name));

		for app in value.apps.iter() {
			let eol_status: String = match &app.eol_status {
				Some(x) => x.to_string(),
				None => String::from(""),
			};

			output.push(format!(
				"\t{: <15}{: <10} {}",
				&app.name, &app.version, eol_status,
			));
		}

		output.join("\n")
	}
}
