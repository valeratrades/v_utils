//! Test that Settings macro compiles and works without Default + Serialize.
//! In this case, the config auto-extension feature is silently disabled.
#![allow(dead_code, unused_imports)]

use clap::Parser;
use v_utils_macros::Settings;

/// Settings struct WITHOUT Default and Serialize.
/// Config auto-extension will not be available, but it should still compile
/// and work normally for loading config.
#[derive(Clone, Debug, v_utils_macros::MyConfigPrimitives, Settings)]
struct AppConfigNoDefault {
	host: String,
	port: u16,
	debug: bool,
}

#[derive(Debug, Parser)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}

fn main() {
	// Verify the SettingsFlags struct was created
	let flags = SettingsFlags {
		config: None,
		host: Some("localhost".to_string()),
		port: Some("8080".to_string()),
		debug: Some(true),
	};

	// Verify try_build exists and has correct signature
	let _build_exists: fn(SettingsFlags) -> Result<AppConfigNoDefault, v_utils::__internal::eyre::Report> = AppConfigNoDefault::try_build;

	// Suppress unused warnings
	let _ = flags;
}
