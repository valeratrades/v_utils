#![feature(default_field_values)]
use v_utils_macros::{MyConfigPrimitives, Settings, SettingsNested};

fn main() {
	let config = v_utils::utils::exit_on_error(Config::try_build(SettingsFlags::default()));
	println!("host={:?} port={} db.url={:?}", config.host, config.port, config.db.url);
}
#[derive(Clone, Debug, Default, MyConfigPrimitives, Settings)]
struct Config {
	#[settings(required_in("production"))]
	host: Option<String>,
	#[serde(default)]
	port: u16,
	#[settings(flatten)]
	#[serde(default)]
	db: Db,
}

#[derive(Clone, Debug, Default, SettingsNested, serde::Deserialize, serde::Serialize)]
struct Db {
	#[settings(required_in("production", "staging"))]
	url: Option<String>,
}
