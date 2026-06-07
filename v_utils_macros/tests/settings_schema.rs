//! When a `#[derive(Settings)]` struct also derives `schemars::JsonSchema`, `write_schema()`
//! emits a valid schema file under `$XDG_CONFIG_HOME/<app>.schema.json`.
//! (The graceful-degradation path — `write_schema()` erroring without the derive — lives in
//! `settings_schema_optional.rs`, since each file may only host one `Settings` struct: the
//! derive generates module-level `SettingsFlags`/`SettingsCommand` items that would collide.)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use v_utils_macros::{Settings, SettingsNested};

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize, SettingsNested)]
#[allow(unused)]
struct Logging {
	#[serde(default)]
	level: String,
	#[serde(default)]
	file: Option<String>,
}

#[derive(Clone, Debug, Default, JsonSchema, v_utils_macros::MyConfigPrimitives, Settings)]
#[allow(unused)]
struct SchemaConfig {
	#[serde(default)]
	host: String,
	#[serde(default)]
	port: u16,
	#[settings(flatten)]
	#[serde(default)]
	logging: Logging,
}

#[test]
fn writes_valid_schema_when_jsonschema_derived() {
	let tmp = tempfile::tempdir().unwrap();
	// `write_schema` (non-xdg branch) resolves the config dir via `xdg_config_fallback`,
	// which honors `$XDG_CONFIG_HOME`. Point it at a throwaway dir so we don't touch the
	// real config location.
	// SAFETY: single-threaded test, no other thread reads env concurrently.
	unsafe {
		std::env::set_var("XDG_CONFIG_HOME", tmp.path());
	}

	let path = SchemaConfig::write_schema().expect("JsonSchema is derived, so this must succeed");
	assert!(path.exists(), "schema file should have been written to {}", path.display());
	assert_eq!(path.extension().and_then(|e| e.to_str()), Some("json"));

	let contents = std::fs::read_to_string(&path).unwrap();
	let schema: Value = serde_json::from_str(&contents).expect("output must be valid JSON");

	// A schemars object schema for the struct exposes its fields under `properties`.
	let props = schema.get("properties").and_then(Value::as_object).expect("schema should have a properties object");
	assert!(props.contains_key("host"), "schema should describe the `host` field");
	assert!(props.contains_key("port"), "schema should describe the `port` field");
	// The flattened nested struct should appear too.
	assert!(props.contains_key("logging"), "schema should describe the flattened `logging` field");
}
