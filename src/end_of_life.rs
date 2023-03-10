use chrono::NaiveDate;
use reqwest::Error;
use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
    pub eol: EOLDate,
    pub support: Option<EOLDate>,
    pub latest: String,
    pub latest_release_date: String,
    pub release_date: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum EOLDate {
    String(String),
    Boolean(bool),
}

impl fmt::Display for EOLDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EOLDate::String(x) => write!(
                f,
                "{}",
                NaiveDate::parse_from_str(x, "%Y-%m-%d")
                    .unwrap()
                    .format("%x")
            ),
            EOLDate::Boolean(_) => write!(f, "alive"),
        }
    }
}

pub fn query(product: &str, version: &str) -> Result<Cycle, Error> {
    let request_url = format!("https://endoflife.date/api/{}/{}.json", &product, &version);
    let response = reqwest::blocking::get(request_url)?.json::<Cycle>()?;

    Ok(response)
}
