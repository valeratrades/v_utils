use std::time::Duration;

use clap::Parser;
use v_utils_macros::{LiveSettings, Settings};

#[allow(dead_code)]
#[derive(Clone, Debug, v_utils_macros::MyConfigPrimitives, Settings, LiveSettings)]
pub struct AppConfig {
	host: String,
	port: u16,
	debug: bool,
}

/// Example CLI struct using SettingsFlags
#[allow(dead_code)]
#[derive(Debug, Parser)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}

fn main() {
	// Test that LiveSettings struct was generated
	let flags = SettingsFlags {
		config: None,
		host: Some("localhost".to_string()),
		port: Some("8080".to_string()),
		debug: Some(true),
	};

	// Test that LiveSettings::new exists and has correct signature
	let _new_exists: fn(SettingsFlags, Duration) -> v_utils::__internal::eyre::Result<LiveSettings> = LiveSettings::new;

	// Test that LiveSettings is Clone
	fn assert_clone<T: Clone>() {}
	assert_clone::<LiveSettings>();

	// Test that LiveSettings is Debug
	fn assert_debug<T: std::fmt::Debug>() {}
	assert_debug::<LiveSettings>();

	// Test that config() method exists and returns AppConfig
	// We can't actually call it without a valid config file, but we can verify the signature
	fn check_config_method(ls: &LiveSettings) -> AppConfig {
		ls.config()
	}

	// Test that initial() method exists and returns AppConfig
	fn check_initial_method(ls: &LiveSettings) -> AppConfig {
		ls.initial()
	}

	// Suppress unused warnings
	let _ = flags;
	let _ = check_config_method;
	let _ = check_initial_method;

	println!("LiveSettings derive macro test passed!");
}
