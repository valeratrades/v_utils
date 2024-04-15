use serde::Deserialize;
use v_utils_macros::PrivateValues;

#[derive(Clone, Debug, PrivateValues)]
pub struct Spy {
	alpaca_key: String,
	alpaca_secret: String,
	a_random_non_string: i32,
}

fn main() {
	let toml_str = r#"alpaca_key = "PKTJYTJNKYSBHAZYT3CO"
alpaca_secret = { env = "ALPACA_API_SECRET" }
a_random_non_string = 1"#;
	let _: Spy = toml::from_str(toml_str).expect("Failed to deserialize");
}
