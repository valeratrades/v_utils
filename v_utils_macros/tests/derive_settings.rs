use clap::Parser;
use serde::Deserialize;
use v_utils_macros::Settings;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Settings)]
#[serde(crate = "v_utils::__internal::serde")]
pub struct AppConfig {
	host: String,
	port: u16,
	debug: bool,
	workers: Option<usize>,
}

/// Example of how to use Settings with Clap
/// The Settings derive macro generates a `SettingsFlags` struct
/// which can be flattened into your CLI struct
#[allow(dead_code)]
#[derive(Parser, Debug)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}

fn main() {
	// Test that the Settings macro generates the expected SettingsFlags struct
	let flags = SettingsFlags {
		config: None,
		host: Some("localhost".to_string()),
		port: Some("8080".to_string()),
		debug: Some(true),
		workers: Some("4".to_string()),
	};

	// Verify the SettingsFlags struct was created
	assert_eq!(flags.host, Some("localhost".to_string()));
	assert_eq!(flags.port, Some("8080".to_string()));
	assert_eq!(flags.debug, Some(true));
	assert_eq!(flags.workers, Some("4".to_string()));

	// Test that try_build method exists and compiles
	// Note: We can't actually call try_build in a simple test because it requires
	// environment setup and config files
	let _build_exists: fn(SettingsFlags) -> Result<AppConfig, v_utils::__internal::eyre::Report> = AppConfig::try_build;

	// Test that SettingsFlags can be flattened into Cli struct
	// This is verified at compile time - if Cli compiles, it works
}
