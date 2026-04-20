#![feature(error_generic_member_access)]
use v_utils_macros::wrap_err;

// Struct case: injects backtrace + spantrace, generates new()
#[wrap_err]
#[derive(Debug, thiserror::Error)]
#[error("leaf struct error: {msg}")]
pub struct LeafStructError {
	msg: String,
}

// Enum case: named-field leaf, unit leaf, foreign wrap, and plain variant
#[wrap_err]
#[derive(Debug, thiserror::Error)]
pub enum MyError {
	#[leaf]
	#[error("bad value: {val}")]
	BadValue { val: String },

	#[leaf]
	#[error("unit variant, no user fields")]
	UnitVariant,

	// Foreign: generates From<std::io::Error> capturing backtrace + spantrace
	#[foreign]
	Io(std::io::Error),

	// Foreign with explicit error format (preserved as-is)
	#[foreign]
	#[error("parse failed: {source}")]
	Parse(std::num::ParseIntError),
}

#[test]
fn test() {
	// Struct: new() auto-captures
	let e = LeafStructError::new("oops".into());
	println!("{e}");

	// Enum named leaf: new_bad_value()
	let e2 = MyError::new_bad_value("x".into());
	println!("{e2}");

	// Enum unit leaf: new_unit_variant() — no args
	let e3 = MyError::new_unit_variant();
	println!("{e3}");

	// Foreign: From impl auto-captures at ? site
	let e4: MyError = std::io::Error::other("disk full").into();
	println!("{e4}");

	// Foreign with explicit format
	let e5: MyError = "not_a_number".parse::<i32>().unwrap_err().into();
	println!("{e5}");
}
