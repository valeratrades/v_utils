//! Regression: a default expression on a field whose type `MyConfigPrimitives` rewrites in
//! its synthesized `Helper` (`String`/`SecretString` → `PrivateValue`, `PathBuf` →
//! `ExpandedPath`) must generate a `__default_<field>()` returning the *Helper* type, not the
//! declared type. Previously the fn returned the declared type while the Helper field was the
//! rewritten one, so serde's `#[serde(default = "..")]` failed with a type mismatch
//! (`expected PrivateValue, found String`) far from the source.

use std::path::PathBuf;

use secrecy::{ExposeSecret, SecretString};
use v_utils_macros::MyConfigPrimitives;

#[derive(Clone, Debug, MyConfigPrimitives)]
struct Cfg {
	#[settings(default = "fallback-prefix".to_string())]
	prefix: String,
	#[settings(default = PathBuf::from("/tmp/x"))]
	path: PathBuf,
	#[settings(default = Some("opt".to_string()))]
	maybe: Option<String>,
	#[settings(default = SecretString::new("sekret".to_string().into_boxed_str()))]
	secret: SecretString,
	// non-rewritten type: default must keep working unchanged
	#[settings(default = 7u8)]
	retries: u8,
}

#[test]
fn defaults_on_rewritten_types() {
	let c: Cfg = toml::from_str("").expect("empty config deserializes via defaults");
	assert_eq!(c.prefix, "fallback-prefix");
	assert_eq!(c.path, PathBuf::from("/tmp/x"));
	assert_eq!(c.maybe, Some("opt".to_string()));
	assert_eq!(c.secret.expose_secret(), "sekret");
	assert_eq!(c.retries, 7);
}
