use clap::Parser;
use serde::Deserialize;
use v_utils_macros::{Settings, SettingsNested};

/// Top-level config with a nested Database section
#[derive(Clone, Debug, v_utils_macros::MyConfigPrimitives, Settings)]
pub struct AppConfig {
	host: String,
	port: u16,
	#[settings(flatten)]
	database: Database,
}

/// First level of nesting - Database config with nested Pool config
#[derive(Clone, Debug, Deserialize, SettingsNested)]
#[settings(prefix = "database")]
pub struct Database {
	url: String,
	#[settings(flatten)]
	pool: Pool,
}

/// Second level of nesting - Pool config (doubly nested)
/// Note: prefix is the full path with underscores
#[derive(Clone, Debug, Deserialize, SettingsNested)]
#[settings(prefix = "database_pool")]
pub struct Pool {
	min_size: u32,
	max_size: u32,
	timeout_ms: u64,
}

#[derive(Debug, Parser)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}

#[test]
fn test_double_nesting() {
	// Verify double nesting generates correct CLI flag names:
	// - database_url (from Database)
	// - database_pool_min_size (from Pool, nested under Database)
	// - database_pool_max_size
	// - database_pool_timeout_ms
	let flags = SettingsFlags {
		config: None,
		host: Some("localhost".to_string()),
		port: Some("8080".to_string()),
		database: __SettingsNestedDatabase {
			database_url: Some("postgres://localhost/mydb".to_string()),
			pool: __SettingsNestedPool {
				database_pool_min_size: Some("5".to_string()),
				database_pool_max_size: Some("20".to_string()),
				database_pool_timeout_ms: Some("5000".to_string()),
			},
		},
	};

	// Verify the nested flags structure
	assert_eq!(flags.host, Some("localhost".to_string()));
	assert_eq!(flags.port, Some("8080".to_string()));
	assert_eq!(flags.database.database_url, Some("postgres://localhost/mydb".to_string()));
	assert_eq!(flags.database.pool.database_pool_min_size, Some("5".to_string()));
	assert_eq!(flags.database.pool.database_pool_max_size, Some("20".to_string()));
	assert_eq!(flags.database.pool.database_pool_timeout_ms, Some("5000".to_string()));

	// Verify CLI struct compiles with flattened settings
	let _build_exists: fn(SettingsFlags) -> Result<AppConfig, v_utils::__internal::eyre::Report> = AppConfig::try_build;
}
