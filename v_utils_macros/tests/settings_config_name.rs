//! Struct-level `#[settings(config_name = "...")]` redirects config-file resolution away
//! from `CARGO_PKG_NAME`, including `/`-nesting inside another app's config dir — the
//! sub-tool-of-a-larger-app layout (`~/.config/parent_app/sub_tool.toml`). The test pins the
//! resolution contract through `try_build`: a file at the overridden location is found and
//! its values win over field defaults.

use v_utils_macros::Settings;

// `use_env = false` is the default; it rides along to exercise the comma-separated
// struct-attr parse path at compile time.
#[derive(Clone, Debug, Default, v_utils_macros::MyConfigPrimitives, Settings)]
#[settings(use_env = false, config_name = "parent_app/sub_tool")]
struct SubToolConfig {
	#[serde(default)]
	host: String,
	#[serde(default)]
	port: u16,
}

#[test]
fn resolves_config_at_overridden_nested_path() {
	let tmp = tempfile::tempdir().unwrap();
	// Both the xdg and fallback branches of the macro honor `$XDG_CONFIG_HOME`.
	// SAFETY: single-threaded test, no other thread reads env concurrently.
	unsafe {
		std::env::set_var("XDG_CONFIG_HOME", tmp.path());
	}

	let config_dir = tmp.path().join("parent_app");
	std::fs::create_dir_all(&config_dir).unwrap();
	std::fs::write(config_dir.join("sub_tool.toml"), "host = \"example.com\"\nport = 8080\n").unwrap();

	let flags = SettingsFlags {
		config: None,
		yes: false,
		host: None,
		port: None,
	};
	let cfg = SubToolConfig::try_build(flags).expect("config at the overridden location must be found and parsed");

	assert_eq!(cfg.host, "example.com");
	assert_eq!(cfg.port, 8080);
}
