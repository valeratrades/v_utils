//! A struct that is not `Default` as a whole used to disable config scaffolding entirely:
//! `write_defaults` bailed with "requires `X` to impl Default". Now it scaffolds field by
//! field — every field whose *own* type has a default gets it, and the rest are written as
//! `"REQUIRED"` placeholders (warned about on stderr).
//!
//! The other half of the contract is that a placeholder must never load as a value. `ApiKey`
//! is a newtype over `String`, so `api_key = "REQUIRED"` would deserialize perfectly happily —
//! `try_build` has to reject it *before* deserialization, and name the file and the field.

use v_utils_macros::Settings;

/// No `Default` — this is the field the config author must fill in themselves.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct ApiKey(String);

#[derive(Clone, Debug, Settings, v_utils_macros::MyConfigPrimitives)]
#[settings(config_name = "v_utils_required_placeholders")]
struct AppConfig {
	host: String,
	port: u16,
	api_key: ApiKey,
}

#[test]
fn scaffolds_placeholders_then_refuses_to_load_them() {
	let tmp = tempfile::tempdir().unwrap();
	// SAFETY: single-threaded test, and this is the only `#[test]` in the binary.
	unsafe {
		std::env::set_var("XDG_CONFIG_HOME", tmp.path());
	}
	assert!(
		std::env::var_os("V_UTILS_MACROS__API_KEY").is_none(),
		"test env is polluted with V_UTILS_MACROS__API_KEY — it would override the placeholder and void the assertions below"
	);

	// An existing config that sets only *some* of the fields: `write_defaults` merges into it
	// rather than creating a fresh `.nix` (which would need `nix eval` to read back).
	let config_path = tmp.path().join("v_utils_required_placeholders.toml");
	std::fs::write(&config_path, "host = \"example.com\"\n").unwrap();

	let written = AppConfig::write_defaults().expect("a struct without a whole-struct `Default` must still scaffold");
	assert_eq!(written, config_path);

	let content = std::fs::read_to_string(&config_path).unwrap();
	assert!(content.contains("host = \"example.com\""), "pre-existing values must not be clobbered, got:\n{content}");
	assert!(content.contains("port = 0"), "`u16` has a default and must be filled with it, got:\n{content}");
	assert!(content.contains("api_key = \"REQUIRED\""), "`ApiKey` has no default and must get a placeholder, got:\n{content}");

	let flags = SettingsFlags {
		config: None,
		yes: false,
		host: None,
		port: None,
		api_key: None,
	};
	let err = AppConfig::try_build(flags.clone()).expect_err("a config still holding a placeholder must not load");
	let msg = err.to_string();
	assert!(msg.contains("api_key"), "the error must name the unset field, got: {msg}");
	assert!(msg.contains(&config_path.display().to_string()), "the error must name the file to edit, got: {msg}");
	assert!(!msg.contains("  - host"), "fields the author already set must not be reported, got: {msg}");
	assert!(!msg.contains("  - port"), "fields with a real default must not be reported, got: {msg}");

	// Filling the placaholder in resolves it — and so does any higher-precedence source.
	std::fs::write(&config_path, "host = \"example.com\"\nport = 0\napi_key = \"sk-live\"\n").unwrap();
	let cfg = AppConfig::try_build(flags).expect("a fully-specified config must load");
	assert_eq!(cfg.api_key, ApiKey("sk-live".to_owned()));
}
