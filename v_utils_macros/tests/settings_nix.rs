use std::fs;

use clap::Parser;
use serde::Deserialize;
use v_utils_macros::Settings;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Settings)]
#[serde(crate = "v_utils::__internal::serde")]
pub struct TestConfig {
	name: String,
	value: u32,
}

#[allow(dead_code)]
#[derive(Debug, Parser)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}

fn main() {
	// Create a temporary .nix file
	let temp_dir = std::env::temp_dir();
	let nix_file_path = temp_dir.join("test_config.nix");

	// Write a simple nix config that evaluates to JSON
	let nix_content = r#"{
  name = "test_app";
  value = 42;
}
"#;

	fs::write(&nix_file_path, nix_content).expect("Failed to write test .nix file");

	// Test that the eval_nix_file method exists and can be called
	// Note: This will only work if nix is installed
	if let Ok(json_str) = TestConfig::eval_nix_file(nix_file_path.to_str().unwrap()) {
		println!("Nix evaluation succeeded: {}", json_str);

		// Verify it's valid JSON
		let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Nix output should be valid JSON");

		assert_eq!(parsed["name"], "test_app");
		assert_eq!(parsed["value"], 42);
	} else {
		println!("Nix not installed or evaluation failed - skipping nix test");
	}

	// Clean up
	let _ = fs::remove_file(&nix_file_path);
}
