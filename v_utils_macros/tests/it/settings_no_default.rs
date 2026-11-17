//! Test that Settings macro compiles and works without Default + Serialize.
//! In this case the interactive extend-on-missing-field prompt is silently disabled;
//! `write_defaults` still works field-wise (see `settings_required_placeholders.rs`).
#![allow(dead_code, unused_imports)]

use clap::Parser;
use v_utils_macros::Settings;

#[derive(Debug, Parser)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}
#[test]
fn test() {
	// Verify the SettingsFlags struct was created
	let flags = SettingsFlags {
		config: None,
		yes: false,
		host: Some("localhost".to_string()),
		port: Some("8080".to_string()),
		debug: Some(true),
	};

	// Verify try_build exists and has correct signature
	let _build_exists: fn(SettingsFlags) -> Result<AppConfigNoDefault, v_utils::__internal::SettingsError> = AppConfigNoDefault::try_build;

	// Suppress unused warnings
	let _ = flags;
}
/// Settings struct WITHOUT Default and Serialize.
/// Config auto-extension will not be available, but it should still compile
/// and work normally for loading config.
#[derive(Clone, Debug, Settings, v_utils_macros::MyConfigPrimitives)]
struct AppConfigNoDefault {
	host: String,
	port: u16,
	debug: bool,
}
