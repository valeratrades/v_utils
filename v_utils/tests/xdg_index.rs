#![cfg(unix)]
//! `~/.<app>/` indexes every XDG base dir the macros create for an app.
//! One `#[test]`: scenarios mutate process-global env, so they run sequentially.

use std::{
	fs,
	path::{Path, PathBuf},
};

const APP: &str = env!("CARGO_PKG_NAME");

/// Fresh `HOME` + `XDG_*` under a tempdir.
fn sandbox() -> tempfile::TempDir {
	let tmp = tempfile::tempdir().unwrap();
	// SAFETY: the only `#[test]` in this binary, scenarios run sequentially.
	unsafe {
		std::env::set_var("HOME", tmp.path());
		for (var, sub) in [
			("XDG_CONFIG_HOME", "config"),
			("XDG_DATA_HOME", "data"),
			("XDG_CACHE_HOME", "cache"),
			("XDG_STATE_HOME", "state"),
			("XDG_RUNTIME_DIR", "runtime"),
		] {
			let dir = tmp.path().join(sub);
			fs::create_dir_all(&dir).unwrap();
			fs::set_permissions(&dir, std::os::unix::fs::PermissionsExt::from_mode(0o700)).unwrap(); // xdg crate refuses a runtime dir others can read
			std::env::set_var(var, dir);
		}
	}
	tmp
}

fn index_entry(home: &Path, entry: &str) -> PathBuf {
	home.join(format!(".{APP}")).join(entry)
}

#[test]
fn xdg_index() {
	// fresh: link created, points at the per-app base dir
	{
		let tmp = sandbox();
		let file = v_utils::xdg_cache_file!("a/b");
		let link = index_entry(tmp.path(), "cache");
		assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
		assert_eq!(fs::read_link(&link).unwrap(), tmp.path().join("cache").join(APP));
		assert!(file.starts_with(fs::read_link(&link).unwrap()));

		v_utils::xdg_cache_dir!("other"); // idempotent
		assert_eq!(fs::read_link(&link).unwrap(), tmp.path().join("cache").join(APP));
	}

	// legacy: real `~/.<app>/data` is moved to XDG, replaced by a link
	{
		let tmp = sandbox();
		let legacy = index_entry(tmp.path(), "data");
		fs::create_dir_all(&legacy).unwrap();
		fs::write(legacy.join("x"), "payload").unwrap();

		let dir = v_utils::xdg_data_dir!("");
		assert_eq!(fs::read_to_string(dir.join("x")).unwrap(), "payload");
		assert!(fs::symlink_metadata(&legacy).unwrap().is_symlink());
	}

	// stale: link into a wiped location is repointed
	{
		let tmp = sandbox();
		let link = index_entry(tmp.path(), "state");
		fs::create_dir_all(link.parent().unwrap()).unwrap();
		std::os::unix::fs::symlink(tmp.path().join("gone"), &link).unwrap();

		v_utils::xdg_state_dir!("");
		assert_eq!(fs::read_link(&link).unwrap(), tmp.path().join("state").join(APP));
	}

	// conflict: legacy and XDG both hold data -> refuse to merge
	{
		let tmp = sandbox();
		let legacy = index_entry(tmp.path(), "config");
		fs::create_dir_all(&legacy).unwrap();
		fs::write(legacy.join("old"), "").unwrap();
		let xdg = tmp.path().join("config").join(APP);
		fs::create_dir_all(&xdg).unwrap();
		fs::write(xdg.join("new"), "").unwrap();

		let res = std::panic::catch_unwind(|| v_utils::xdg_config_dir!(""));
		assert!(res.is_err());
		assert!(legacy.join("old").exists() && xdg.join("new").exists());
	}
}
