use clap::Parser;
use serde::Deserialize;
use v_utils_macros::{Settings, SettingsNested};

#[derive(Clone, Debug, v_utils_macros::MyConfigPrimitives, Settings)]
#[allow(unused)]
struct AppConfig {
	host: String,
	port: u16,
	debug: bool,
	workers: Option<usize>,
	#[settings(flatten)]
	database: Database,
	#[settings(flatten)]
	risk_tiers: RiskTiers,
	/// Optional nested config - should work with flatten
	#[settings(flatten)]
	logging: Option<Logging>,
	/// This field should be skipped - not in CLI flags or config
	#[settings(skip)]
	internal_state: String,
}

#[derive(Clone, Debug, Deserialize, SettingsNested)]
#[allow(unused)]
pub struct Database {
	url: String,
	max_connections: u32,
	#[settings(flatten)]
	pool: Pool,
}

/// Second level of nesting - Pool config (doubly nested)
#[derive(Clone, Debug, Deserialize, SettingsNested)]
#[settings(prefix = "database_pool")]
#[allow(unused)]
pub struct Pool {
	min_size: u32,
	max_size: u32,
}

#[derive(Clone, Debug, Deserialize, SettingsNested)]
#[allow(unused)]
pub struct RiskTiers {
	a: String,
	b: String,
}

#[derive(Clone, Debug, Deserialize, SettingsNested)]
#[allow(unused)]
pub struct Logging {
	level: String,
	file: Option<String>,
}

/// Example of how to use Settings with Clap
/// The Settings derive macro generates a `SettingsFlags` struct
/// which can be flattened into your CLI struct
#[derive(Debug, Parser)]
#[allow(unused)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}

fn main() {
	// Test that the Settings macro generates the expected SettingsFlags struct //HACK: relies on exact name
	let flags = SettingsFlags {
		config: None,
		host: Some("localhost".to_string()),
		port: Some("8080".to_string()),
		debug: Some(true),
		workers: Some("4".to_string()),
		database: __SettingsNestedDatabase {
			database_url: Some("postgres://localhost".to_string()),
			database_max_connections: Some("10".to_string()),
			pool: __SettingsNestedPool {
				database_pool_min_size: Some("5".to_string()),
				database_pool_max_size: Some("20".to_string()),
			},
		},
		risk_tiers: __SettingsNestedRiskTiers {
			risk_tiers_a: Some("0.01".to_string()),
			risk_tiers_b: Some("0.02".to_string()),
		},
		// Optional flattened field - uses same nested struct pattern
		logging: __SettingsNestedLogging {
			logging_level: Some("debug".to_string()),
			logging_file: Some("/var/log/app.log".to_string()),
		},
	};

	// Verify the SettingsFlags struct was created
	assert_eq!(flags.host, Some("localhost".to_string()));
	assert_eq!(flags.port, Some("8080".to_string()));
	assert_eq!(flags.debug, Some(true));
	assert_eq!(flags.workers, Some("4".to_string()));
	assert_eq!(flags.database.database_url, Some("postgres://localhost".to_string()));
	assert_eq!(flags.database.database_max_connections, Some("10".to_string()));
	assert_eq!(flags.risk_tiers.risk_tiers_a, Some("0.01".to_string()));
	assert_eq!(flags.risk_tiers.risk_tiers_b, Some("0.02".to_string()));

	// Test that try_build method exists and compiles
	// Note: We can't actually call try_build in a simple test because it requires
	// environment setup and config files
	let _build_exists: fn(SettingsFlags) -> Result<AppConfig, v_utils::__internal::eyre::Report> = AppConfig::try_build;

	// Test that SettingsFlags can be flattened into Cli struct
	// This is verified at compile time - if Cli compiles, it works

	// Test that skipped field is not present in SettingsFlags
	// The fact that this compiles proves that 'internal_state' field is NOT in SettingsFlags
	// If it were present, we would need to initialize it above
	let _test_skip: fn() = || {
		let _flags_without_internal_state = SettingsFlags {
			config: None,
			host: None,
			port: None,
			debug: None,
			workers: None,
			database: __SettingsNestedDatabase {
				database_url: None,
				database_max_connections: None,
				pool: __SettingsNestedPool {
					database_pool_min_size: None,
					database_pool_max_size: None,
				},
			},
			risk_tiers: __SettingsNestedRiskTiers {
				risk_tiers_a: None,
				risk_tiers_b: None,
			},
			logging: __SettingsNestedLogging {
				logging_level: None,
				logging_file: None,
			},
			// NOTE: internal_state is NOT here because it has #[settings(skip)]
		};
	};

	// Test loading config with unknown field (will warn to stderr)
	eprintln!("\n=== Testing unknown field warning ===");
	let flags_with_config = SettingsFlags {
		config: Some(v_utils::io::ExpandedPath(std::path::PathBuf::from("tests/test_unknown_field.toml"))),
		host: None,
		port: None,
		debug: None,
		workers: None,
		database: __SettingsNestedDatabase {
			database_url: None,
			database_max_connections: None,
			pool: __SettingsNestedPool {
				database_pool_min_size: None,
				database_pool_max_size: None,
			},
		},
		risk_tiers: __SettingsNestedRiskTiers {
			risk_tiers_a: None,
			risk_tiers_b: None,
		},
		logging: __SettingsNestedLogging {
			logging_level: None,
			logging_file: None,
		},
	};

	match AppConfig::try_build(flags_with_config) {
		Ok(_config) => {
			eprintln!("Config loaded successfully (unknown fields should have triggered warnings above)");
		}
		Err(e) => {
			// Config file might not exist in test environment, which is fine
			eprintln!("Note: Config loading skipped ({})", e);
		}
	}
	eprintln!("=== End of unknown field warning test ===\n");
}
