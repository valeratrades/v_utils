//! Settings as a deployed binary resolves them from its environment. The fixture package is
//! `env-prefix-fixture`, so its vars are `ENV_PREFIX_FIXTURE_<FIELD>`, sections joined by `__`.

use std::{path::Path, process::Command};

#[track_caller]
fn check(env: &[(&str, &str)], expected_code: i32, expected_output: &str) {
	let root = Path::new(env!("CARGO_MANIFEST_DIR"));
	let tmp = tempfile::tempdir().unwrap();
	let out = Command::new(env!("CARGO"))
		.args(["run", "-q", "--manifest-path"])
		.arg(root.join("tests/env-prefix-fixture/Cargo.toml"))
		.env("CARGO_TARGET_DIR", Path::new(env!("CARGO_TARGET_TMPDIR")).join("env-prefix-fixture"))
		.env("HOME", tmp.path())
		.env("XDG_CONFIG_HOME", tmp.path())
		.env_remove("APP_ENV")
		.envs(env.iter().copied())
		.output()
		.unwrap();
	let output = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
	assert_eq!(out.status.code(), Some(expected_code), "{output}");
	assert!(output.contains(expected_output), "expected `{expected_output}` in:\n{output}");
}

#[test]
fn env_names_and_required_in() {
	check(
		&[("ENV_PREFIX_FIXTURE_HOST", "h"), ("ENV_PREFIX_FIXTURE_PORT", "80"), ("ENV_PREFIX_FIXTURE_DB__URL", "u")],
		0,
		r#"host=Some("h") port=80 db.url=Some("u")"#,
	);
	check(&[("ENV_PREFIX_FIXTURE__HOST", "h"), ("HOST", "h")], 0, "host=None");

	check(&[], 0, "host=None port=0 db.url=None");
	check(&[("APP_ENV", "production")], 78, "required with APP_ENV=production:\n  - db.url\n  - host\n");
	check(&[("APP_ENV", "staging")], 78, "required with APP_ENV=staging:\n  - db.url\n\n");
	check(
		&[("APP_ENV", "production"), ("ENV_PREFIX_FIXTURE_HOST", "h"), ("ENV_PREFIX_FIXTURE_DB__URL", "u")],
		0,
		r#"host=Some("h")"#,
	);
}
