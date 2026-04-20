#![feature(error_generic_member_access)]
use v_utils_macros::wrap_err;

// Struct case: injects backtrace + spantrace, generates new()
#[wrap_err]
#[derive(Debug, thiserror::Error)]
#[error("leaf struct error: {msg}")]
pub struct LeafStructError {
	msg: String,
}

// Enum case: named-field leaf, unit leaf, and non-leaf variants
#[wrap_err]
#[derive(Debug, thiserror::Error)]
pub enum MyError {
	#[leaf]
	#[error("bad value: {val}")]
	BadValue { val: String },

	#[leaf]
	#[error("unit variant, no user fields")]
	UnitVariant,

	#[error(transparent)]
	Io(#[from] std::io::Error),
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
}
