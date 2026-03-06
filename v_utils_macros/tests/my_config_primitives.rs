use std::path::PathBuf;

use secrecy::SecretString;
use v_utils_macros::MyConfigPrimitives;

#[derive(Clone, Debug, PartialEq)]
pub struct Port(u16);

impl std::str::FromStr for Port {
	type Err = std::num::ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Port(s.parse()?))
	}
}

#[allow(dead_code)]
#[derive(Clone, Debug, MyConfigPrimitives)]
pub struct Test {
	alpaca_key: String,
	alpaca_secret: SecretString,
	whoami: String,
	a_random_non_string: i32,
	path: PathBuf,
	#[private_value]
	port: Port,
	#[private_value]
	test_private_value_works_with_non_strings: usize,
	optional_string: Option<String>,
	optional_secret: Option<SecretString>,
	#[serde(default)]
	string_with_default: String,
	#[serde(default = "__default_num_of_retries")]
	pub num_of_retries: u8,
	#[primitives(skip)]
	skipped_string: String,
	#[private_value]
	optional_port: Option<Port>,
}
fn main() {
	let toml_str = r#"
	alpaca_key = "PKTJYTJNKYSBHAZYT3CO"
	alpaca_secret = { env = "HOME" }
whoami = { env = "USER" }
a_random_non_string = 1
path = "~/.config/a_test_path"
port = "8080"
test_private_value_works_with_non_strings = 1234
optional_string = { env = "USER" }
optional_secret = { env = "USER" }
skipped_string = "this should not be wrapped in PrivateValue"
optional_port = "9090"
"#;

	let t: Test = toml::from_str(toml_str).expect("Failed to deserialize");

	// variables change, so assert properties
	assert_eq!(t.alpaca_key, "PKTJYTJNKYSBHAZYT3CO");
	assert_eq!(secrecy::ExposeSecret::expose_secret(&t.alpaca_secret), &std::env::var("HOME").unwrap());
	assert_eq!(t.path, PathBuf::from(format!("{}/.config/a_test_path", std::env::var("HOME").unwrap())));
	assert_eq!(t.whoami, std::env::var("USER").unwrap());
	assert_eq!(t.a_random_non_string, 1);
	assert_eq!(t.port, Port(8080));
	assert_eq!(t.optional_string, Some(std::env::var("USER").unwrap()));
	assert_eq!(
		t.optional_secret.as_ref().map(secrecy::ExposeSecret::expose_secret),
		Some(std::env::var("USER").unwrap().as_str())
	);
	assert_eq!(t.string_with_default, ""); // Test that serde(default) works - empty string is the default for String
	assert_eq!(t.num_of_retries, 3); // Test that custom default function works
	assert_eq!(t.skipped_string, "this should not be wrapped in PrivateValue"); // Test that #[primitives(skip)] works
	assert_eq!(t.optional_port, Some(Port(9090))); // Test that Option<T> with #[private_value] works

	// Test that SecretString fields show [REDACTED] in debug output (handled by secrecy crate)
	let debug_output = format!("{t:?}");
	assert!(debug_output.contains("[REDACTED]"), "SecretString should show [REDACTED] in debug output, got: {debug_output}");

	// Test that Option<T> with #[private_value] becomes None when env var is missing
	let toml_with_missing_env = r#"
	alpaca_key = "PKTJYTJNKYSBHAZYT3CO"
	alpaca_secret = { env = "HOME" }
whoami = { env = "USER" }
a_random_non_string = 1
path = "~/.config/a_test_path"
port = "8080"
test_private_value_works_with_non_strings = 1234
optional_string = { env = "USER" }
optional_secret = { env = "USER" }
skipped_string = "test"
optional_port = { env = "THIS_ENV_VAR_DEFINITELY_DOES_NOT_EXIST_12345" }
"#;

	let t2: Test = toml::from_str(toml_with_missing_env).expect("Failed to deserialize with missing env var");
	assert_eq!(t2.optional_port, None, "Option<T> with #[private_value] should be None when env var is missing");
}
fn __default_num_of_retries() -> u8 {
	3
}
