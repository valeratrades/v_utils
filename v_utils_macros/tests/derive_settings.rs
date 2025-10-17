use serde::Deserialize;
use v_utils_macros::Settings;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Settings)]
pub struct AppConfig {
	host: String,
	port: u16,
	debug: bool,
	workers: Option<usize>,
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

	// Test that try_build method exists (compile-time check)
	// Note: We can't actually call try_build in a simple test because it requires
	// environment setup and config files, but we can verify it compiles
	let _settings = AppConfig::try_build(flags).unwrap();
	println!("{_settings:?}")
}
