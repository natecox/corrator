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
/// let db = cache::eol_cache_db().unwrap();
/// cache::get_cycle(&db, "ubuntu", "22.10");
/// ```
pub fn get_cycle(
	db: &Database,
	product: &str,
	cycle: &str,
) -> Result<Option<Cycle>, bonsaidb::core::Error> {
	let key = format!("{product}::{cycle}");
	let entry = CachedCycle::get(&key, db)?;

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
/// let db = cache::eol_cache_db().unwrap();
/// let cycle: Cycle = Default::default();
/// cache::insert_cycle(&db, "ubuntu", "22.10", cycle);
/// ```
pub fn insert_cycle(
	db: &Database,
	product: &str,
	cycle: &str,
	data: Cycle,
) -> Result<Cycle, Error> {
	let key = format!("{product}::{cycle}");
	let entry = CachedCycle {
		product: String::from(product),
		cycle: String::from(cycle),
		data,
	};

	if let Ok(None) = get_cycle(db, product, cycle) {
		db.collection::<CachedCycle>().insert(&key, &entry)?;
	}

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
	drop(std::fs::remove_dir_all(eol_cache_db_path()));

	Ok(())
}

/// Returns a reusable bonsaidb instance
///
/// This was made a public function to reduce the number of times a DB
/// connection was created, when bonsaidb started producting intermittent
/// connection refusals.
///
/// # Examples
/// ```no_run
/// use corrator::end_of_life::cache;
///
/// let db = cache::eol_cache_db().unwrap();
/// ```
pub fn eol_cache_db() -> Result<Database, Error> {
	let db_path = eol_cache_db_path();

	Ok(
		Database::open::<CachedCycle>(StorageConfiguration::new(db_path))
			.expect("Couldn't connect to DB"),
	)
}

fn eol_cache_db_path() -> PathBuf {
	directories::ProjectDirs::from("rs", "", "corrator")
		.expect("could not find project directory")
		.data_dir()
		.join("corrator.bonsaidb")
}
