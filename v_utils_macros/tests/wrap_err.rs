#![feature(error_generic_member_access)]
use v_utils_macros::wrap_err;

// Struct case: #[wrap_err] injects backtrace + spantrace into a named-field struct
#[wrap_err]
#[derive(Debug, thiserror::Error)]
#[error("leaf struct error: {msg}")]
pub struct LeafStructError {
	msg: String,
}

// Enum case: #[leaf] variants get backtrace + spantrace injected; others untouched
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
	let e = LeafStructError {
		msg: "oops".into(),
		backtrace: std::backtrace::Backtrace::capture(),
		spantrace: tracing_error::SpanTrace::capture(),
	};
	println!("{e}");

	let e2 = MyError::BadValue {
		val: "x".into(),
		backtrace: std::backtrace::Backtrace::capture(),
		spantrace: tracing_error::SpanTrace::capture(),
	};
	println!("{e2}");

	let e3 = MyError::MissingField {
		field: "name",
		backtrace: std::backtrace::Backtrace::capture(),
		spantrace: tracing_error::SpanTrace::capture(),
	};
	println!("{e3}");
}
