//! Integration test for `v_utils::clientside!()` logging.
//!
//! `clientside!()` installs a *global* tracing subscriber that writes to
//! the real stdout/stderr file descriptors and to a file under
//! `XDG_STATE_HOME`. Two reasons we cannot drive this from a `#[test]`:
//!   1. `tracing_subscriber::registry().init()` is one-shot per process,
//!   2. libtest's stdout/stderr capture only intercepts the `print!`/
//!      `println!` macros via `OUTPUT_CAPTURE`, not direct
//!      `io::Stdout::write` calls — which is what tracing-subscriber does.
//! So we spawn `examples/logging_emit.rs` as a fresh subprocess, point
//! `XDG_STATE_HOME` at a tempdir, capture its stdout/stderr, then read
//! the log file out of the tempdir.

use std::{path::PathBuf, process::Command};

const PKG: &str = "v_utils";
const EXAMPLE: &str = "logging_emit";

/// Stitches stdout, stderr and the on-disk log file into one snapshot
/// payload, in that order with labelled separators.
fn run_and_collect() -> String {
	let tmp = tempfile::tempdir().expect("create tempdir");
	let xdg_state_home = tmp.path();

	// Pre-create the per-package state dir and seed `_log_directives`
	// to lock the runtime filter at info+ regardless of what `option_env!`
	// or `.cargo/log_directives` may want to splice in.
	let log_dir = xdg_state_home.join(PKG);
	std::fs::create_dir_all(&log_dir).expect("mkdir log_dir");
	std::fs::write(log_dir.join("_log_directives"), "info\n").expect("write _log_directives");

	let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

	// Pre-build the example so the run-phase's stderr only contains
	// what the example itself emitted (no "Compiling …" / rustc warnings).
	let build_status = Command::new(&cargo)
		.args(["build", "--quiet", "--example", EXAMPLE, "--features", "xdg"])
		.current_dir(&manifest_dir)
		.status()
		.expect("cargo build --example");
	assert!(build_status.success(), "cargo build failed");

	// Workspace-relative target dir; honours CARGO_TARGET_DIR if set.
	let target_dir = std::env::var_os("CARGO_TARGET_DIR")
		.map(PathBuf::from)
		.unwrap_or_else(|| manifest_dir.parent().expect("workspace root").join("target"));
	let binary = target_dir.join("debug").join("examples").join(EXAMPLE);

	let output = Command::new(&binary)
		.current_dir(&manifest_dir)
		.env("XDG_STATE_HOME", xdg_state_home)
		// Strip directives that could leak from the dev shell.
		.env_remove("RUST_LOG")
		.env_remove("LOG_DIRECTIVES")
		.env_remove("__IS_INTEGRATION_TEST")
		.output()
		.expect("spawn example binary");

	assert!(
		output.status.success(),
		"example exited non-zero: {}\nstderr:\n{}",
		output.status,
		String::from_utf8_lossy(&output.stderr)
	);

	let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
	let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
	let file = std::fs::read_to_string(log_dir.join(".log")).expect("read log file");

	let combined = format!("===== STDOUT =====\n{stdout}===== STDERR =====\n{stderr}===== FILE =====\n{file}");
	// `tempfile::tempdir()` honours $TMPDIR (nix-shell sets it under
	// `/tmp/nix-shell.XXX/`), so the path is fully dynamic — substitute it
	// as a literal string instead of relying on a regex pattern.
	combined.replace(&xdg_state_home.display().to_string(), "<TMPDIR>")
}

#[test]
fn clientside_writes_stdout_stderr_and_file() {
	let combined = run_and_collect();

	insta::with_settings!({
		filters => vec![
			// ISO-8601 timestamps emitted by tracing-subscriber
			(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z", "<TIMESTAMP>"),
			// ANSI escape sequences (stderr layer enables ANSI)
			(r"\x1b\[[0-9;]*m", ""),
			// Thread ids
			(r"ThreadId\(\d+\)", "ThreadId(N)"),
		],
	}, {
		insta::assert_snapshot!(combined);
	});
}
