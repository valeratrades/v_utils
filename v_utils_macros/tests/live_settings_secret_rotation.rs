//! A `{ file = … }` secret is read at build, so `LiveSettings` must rebuild when that file changes, not only when the config does.

use std::time::Duration;

use v_utils_macros::{LiveSettings, MyConfigPrimitives, Settings};

#[derive(Clone, Debug, Default, LiveSettings, MyConfigPrimitives, Settings)]
#[settings(config_name = "v_utils_secret_rotation")]
struct AppConfig {
	#[serde(default)]
	token: String,
}

#[test]
fn rotated_secret_file_is_picked_up() {
	let tmp = tempfile::tempdir().unwrap();
	// SAFETY: the only `#[test]` in this binary.
	unsafe {
		std::env::set_var("XDG_CONFIG_HOME", tmp.path());
		std::env::set_var("HOME", tmp.path());
	}
	let secret = tmp.path().join("token");
	std::fs::write(&secret, "old\n").unwrap();
	std::fs::write(tmp.path().join("v_utils_secret_rotation.toml"), format!("token = {{ file = \"{}\" }}\n", secret.display())).unwrap();

	let live = LiveSettings::new(SettingsFlags::default(), Duration::ZERO).unwrap();
	assert_eq!(live.config().unwrap().token, "old");

	std::thread::sleep(Duration::from_millis(10));
	std::fs::write(&secret, "new\n").unwrap();
	assert_eq!(live.config().unwrap().token, "new");
}
