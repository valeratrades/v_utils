use std::env;
use std::fs;
use std::path::Path;

fn main() {
	if env::var("NO_SLOW_TESTS").is_ok() {
		println!("cargo:rustc-cfg=slow_tests");
	}

	// Inform Cargo that if the environment variable changes, the build script should rerun.
	println!("cargo:rerun-if-env-changed=SLOW_TESTS");

	// Re-run if this build.rs script changes
	let build_script_path = Path::new("build.rs");
	if let Ok(metadata) = fs::metadata(build_script_path) {
		if let Ok(modified) = metadata.modified() {
			println!("cargo:rerun-if-changed={:?}", modified);
		}
	}
}
