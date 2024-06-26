use std::path::PathBuf;

use bonsaidb::core::Error;

use crate::end_of_life::Cycle;
use bonsaidb::{
	core::{
		connection::Connection,
		schema::{Collection, SerializedCollection},
	},
	local::{
		config::{Builder, StorageConfiguration},
		Database,
	},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Collection)]
#[collection(
    name = "eol_cycles",
    primary_key = String,
    natural_id = Some(format!("{}::{}", self.product, self.cycle))
)]
struct CachedCycle {
	product: String,
	cycle: String,
	data: Cycle,
}

/// Retrieve an existing cached endoflife.date product cycle
///
/// # Examples
///
/// ```no_run
/// use corrator::end_of_life::cache;
///
/// cache::get_cycle("ubuntu", "22.10");
/// ```
pub fn get_cycle(product: &str, cycle: &str) -> Result<Option<Cycle>, bonsaidb::core::Error> {
	let db = corrator_db()?;
	let key = format!("{product}::{cycle}");
	let entry = CachedCycle::get(&key, &db)?;

	match entry {
		Some(x) => Ok(Some(x.contents.data)),
		None => Ok(None),
	}
}

/// Insert a new endoflife.date product cycle
///
/// # Examples
///
/// ```no_run
/// use corrator::end_of_life::{cache, Cycle};
///
/// let cycle: Cycle = Default::default();
/// cache::insert_cycle("ubuntu", "22.10", cycle);
/// ```
pub fn insert_cycle(product: &str, cycle: &str, data: Cycle) -> Result<Cycle, Error> {
	let key = format!("{product}::{cycle}");
	let db = corrator_db()?;
	let entry = CachedCycle {
		product: String::from(product),
		cycle: String::from(cycle),
		data,
	};

	db.collection::<CachedCycle>().insert(&key, &entry)?;
	Ok(entry.data)
}

/// Clears the cache entirely
///
/// # Examples
/// ```no_run
/// use corrator::end_of_life::cache;
///
/// cache::clear();
/// ```
pub fn clear() -> Result<(), Error> {
	drop(std::fs::remove_dir_all(corrator_db_path()));

	Ok(())
}

fn corrator_db_path() -> PathBuf {
	directories::ProjectDirs::from("rs", "", "corrator")
		.expect("could not find project directory")
		.data_dir()
		.join("corrator.bonsaidb")
}

fn corrator_db() -> Result<Database, Error> {
	let db_path = corrator_db_path();

	Ok(
		Database::open::<CachedCycle>(StorageConfiguration::new(db_path))
			.expect("Couldn't connect to DB"),
	)
}
