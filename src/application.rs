use std::error::Error;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Application {
    pub name: String,
    #[serde(with = "serde_regex")]
    pub version_regex: Regex,
    pub version_command: String,
    pub eol_api_supported: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_query_returns_version() {
        let app = Application {
            name: String::from("test"),
            version_regex: Regex::new(r"test: (?P<version>[0-9.]+)").unwrap(),
            version_command: String::from(""),
            eol_api_supported: false,
        };

        assert_eq!(String::from("1.2.3"), app.query_version("test: 1.2.3").unwrap());
    }
}
