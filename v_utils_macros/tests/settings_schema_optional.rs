//! Graceful-degradation half of the schema feature: a `#[derive(Settings)]` struct that does
//! NOT derive `schemars::JsonSchema` must still compile, and `write_schema()` must return an
//! informative error rather than silently doing nothing. Kept in its own file because the
//! `Settings` derive emits module-level items (`SettingsFlags` etc.) that collide if two such
//! structs share a module.

use v_utils_macros::Settings;

#[derive(Clone, Debug, Default, v_utils_macros::MyConfigPrimitives, Settings)]
#[allow(unused)]
struct NoSchemaConfig {
	#[serde(default)]
	host: String,
}

#[test]
fn write_schema_errs_without_jsonschema_derive() {
	let err = NoSchemaConfig::write_schema().expect_err("no JsonSchema impl, so this must error");
	let msg = err.to_string();
	assert!(msg.contains("JsonSchema"), "error should mention the missing derive, got: {msg}");
}
