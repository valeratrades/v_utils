#![feature(error_generic_member_access)]
use v_utils_macros::wrap_err;

// Struct case: #[wrap_err] injects backtrace + spantrace and generates new()
#[wrap_err]
#[derive(Debug, thiserror::Error)]
#[error("leaf struct error: {msg}")]
pub struct LeafStructError {
	msg: String,
}

// Enum case: #[leaf] variants get fields injected + new_variant_name() constructors generated
#[wrap_err]
#[derive(Debug, thiserror::Error)]
pub enum MyError {
	#[leaf]
	#[error("bad value: {val}")]
	BadValue { val: String },

	#[leaf]
	#[error("missing field: {field}")]
	MissingField { field: &'static str },

	#[error(transparent)]
	Io(#[from] std::io::Error),
}

fn main() {
	let e = LeafStructError::new("oops".into());
	println!("{e}");

	let e2 = MyError::new_bad_value("x".into());
	println!("{e2}");

	let e3 = MyError::new_missing_field("name");
	println!("{e3}");
}
