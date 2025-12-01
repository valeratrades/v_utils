// Test to verify #[settings(skip)] works at runtime
use v_utils_macros::Settings;

#[derive(Clone, Debug, v_utils_macros::MyConfigPrimitives, Settings)]
pub struct TestConfig {
	pub visible_field: String,
	#[settings(skip)]
	pub skipped_field: String,
}

fn main() {
	// Verify that SettingsFlags does NOT have skipped_field
	let flags = SettingsFlags {
		config: None,
		visible_field: Some("test".to_string()),
		// Note: skipped_field is NOT here - that's the test!
	};

	// If this compiles, the test passes
	println!("✓ Test passed: skipped_field is not in SettingsFlags");

	// Additional compile-time check
	let _: Option<String> = flags.visible_field;
	// This line would fail to compile if skipped_field was in SettingsFlags:
	// let _: Option<String> = flags.skipped_field; // ❌ Would not compile
}
