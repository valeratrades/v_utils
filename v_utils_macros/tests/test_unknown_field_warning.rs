//! Test that unknown fields in config files produce warnings.
//! Run with: cargo test --package v_utils_macros test_unknown_field_warning -- --nocapture

use clap::Parser;
use serde::{Deserialize, Serialize};
use v_utils_macros::{Settings, SettingsNested};

#[derive(Clone, Debug, Default, v_utils_macros::MyConfigPrimitives, Serialize, Settings)]
struct TestConfig {
	#[serde(default)]
	host: String,
	#[serde(default)]
	port: u16,
	#[serde(default)]
	debug: bool,
	#[settings(flatten)]
	#[serde(default)]
	database: Database,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, SettingsNested)]
struct Database {
	#[serde(default)]
	url: String,
	#[serde(default)]
	max_connections: u32,
}

#[derive(Debug, Parser)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}

#[test]
fn test_unknown_field_warning() {
	// Create the SettingsFlags pointing to our test config file
	let flags = SettingsFlags {
		config: Some(v_utils::io::ExpandedPath(std::path::PathBuf::from(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/tests/test_unknown_field.toml"
		)))),
		host: None,
		port: None,
		debug: None,
		database: __SettingsNestedDatabase {
			database_url: None,
			database_max_connections: None,
		},
	};

	eprintln!("\n=== Testing unknown field warning ===");
	eprintln!("Config file: {:?}", flags.config);
	eprintln!("Expected warnings for: unknown_top_level, some_other_field, unknown_section");
	eprintln!("Should NOT warn about: host, port, debug, database (valid fields)");

	// This should load the config and print warnings for unknown fields
	match TestConfig::try_build(flags) {
		Ok(config) => {
			eprintln!("Config loaded: {:?}", config);
			eprintln!("(Unknown field warnings should have been printed above)");
		}
		Err(e) => {
			eprintln!("Error loading config: {}", e);
		}
	}
	eprintln!("=== End of unknown field warning test ===\n");
}
