use std::path::PathBuf;

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
	whoami: String,
	a_random_non_string: i32,
	path: PathBuf,
	#[private_value]
	port: Port,
	#[private_value]
	test_private_value_works_with_non_strings: usize,
}

fn main() {
	let toml_str = r#"
	alpaca_key = "PKTJYTJNKYSBHAZYT3CO"
whoami = { env = "USER" }
a_random_non_string = 1
path = "~/.config/a_test_path"
port = "8080"
test_private_value_works_with_non_strings = 1234
"#;

	let t: Test = toml::from_str(toml_str).expect("Failed to deserialize");

	// variables change, so assert properties
	assert_eq!(t.alpaca_key, "PKTJYTJNKYSBHAZYT3CO");
	assert_eq!(t.path, PathBuf::from(format!("{}/.config/a_test_path", std::env::var("HOME").unwrap())));
	assert_eq!(t.whoami, std::env::var("USER").unwrap());
	assert_eq!(t.a_random_non_string, 1);
	assert_eq!(t.port, Port(8080));
}
