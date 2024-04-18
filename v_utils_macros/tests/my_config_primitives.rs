use std::path::PathBuf;
use v_utils_macros::MyConfigPrimitives;

#[derive(Clone, Debug, MyConfigPrimitives)]
pub struct Test {
	alpaca_key: String,
	alpaca_secret: String,
	a_random_non_string: i32,
	path: PathBuf,
}

fn main() {
	let toml_str = r#"
	alpaca_key = "PKTJYTJNKYSBHAZYT3CO"
alpaca_secret = { env = "ALPACA_API_SECRET" }
a_random_non_string = 1
path = "~/.config/a_test_path"
"#;

	let t: Test = toml::from_str(toml_str).expect("Failed to deserialize");

	assert_eq!(t.alpaca_key, "PKTJYTJNKYSBHAZYT3CO");
	assert_eq!(t.path, PathBuf::from("/home/v/.config/a_test_path"));
}
