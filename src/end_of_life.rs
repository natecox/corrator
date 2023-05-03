use chrono::NaiveDate;
use regex::Regex;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Serialize, Deserialize, Debug)]
pub struct EolConfig {
    pub product_name: String,
    #[serde(with = "serde_regex")]
    pub version_regex: Regex,
}

impl EolConfig {
    pub fn query(&self, input: &str) -> Result<Cycle, Error> {
        let version = self.version_regex.find(input).unwrap().as_str();
        let request_url = format!(
            "https://endoflife.date/api/{}/{}.json",
            &self.product_name, &version
        );
        let response = reqwest::blocking::get(request_url)?.json::<Cycle>()?;

        Ok(response)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
    pub eol: EOLDate,
    pub support: Option<EOLDate>,
    pub latest: String,
    pub latest_release_date: String,
    pub release_date: String,
}

impl From<Cycle> for String {
    fn from(item: Cycle) -> Self {
        item.eol.into()
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum EOLDate {
    String(String),
    Boolean(bool),
}

impl From<EOLDate> for String {
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
