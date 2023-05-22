use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Container {
	pub path: String,
	pub apps: Vec<String>,
}

#[derive(Serialize)]
pub struct Status {
	pub name: String,
	pub apps: Vec<crate::application::Status>,
}

impl Status {
	pub fn new(name: String) -> Self {
		Self { name, apps: vec![] }
	}

	pub fn to_json(&self) -> String {
		serde_json::to_string(&self).unwrap()
	}
}

impl From<Status> for String {
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
