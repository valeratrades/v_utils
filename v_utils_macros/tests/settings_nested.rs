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

/// First level of nesting - no prefix needed, defaults to "database"
#[derive(Clone, Debug, Deserialize, SettingsNested)]
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

#[test]
fn test_missing_fields_error_message() {
	// Test that the error message lists ALL missing fields, not just the first one
	let empty_flags = SettingsFlags {
		config: None,
		host: None, // Missing required field
		port: None, // Missing required field
		database: __SettingsNestedDatabase {
			database_url: None, // Missing required field
			pool: __SettingsNestedPool {
				database_pool_min_size: None,   // Missing required field
				database_pool_max_size: None,   // Missing required field
				database_pool_timeout_ms: None, // Missing required field
			},
		},
	};

	let result = AppConfig::try_build(empty_flags);
	assert!(result.is_err(), "Should fail with missing required fields");

	let err_msg = result.unwrap_err().to_string();

	// Verify the error message mentions all missing fields
	assert!(err_msg.contains("host"), "Error should mention missing 'host' field: {}", err_msg);
	assert!(err_msg.contains("port"), "Error should mention missing 'port' field: {}", err_msg);
	assert!(err_msg.contains("database.url"), "Error should mention missing 'database.url' field: {}", err_msg);
	assert!(
		err_msg.contains("database.pool.min_size"),
		"Error should mention missing 'database.pool.min_size' field: {}",
		err_msg
	);
	assert!(
		err_msg.contains("database.pool.max_size"),
		"Error should mention missing 'database.pool.max_size' field: {}",
		err_msg
	);
	assert!(
		err_msg.contains("database.pool.timeout_ms"),
		"Error should mention missing 'database.pool.timeout_ms' field: {}",
		err_msg
	);

	// Verify the error message has the correct format (lists all fields)
	assert!(err_msg.contains("Missing required configuration fields"), "Error should have correct prefix: {}", err_msg);
}
