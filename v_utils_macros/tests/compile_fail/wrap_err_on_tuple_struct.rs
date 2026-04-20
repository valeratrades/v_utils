use v_utils_macros::wrap_err;

// wrap_err requires named fields — tuple structs are not supported.
#[wrap_err] //~ ERROR: custom attribute panicked
#[derive(Debug)]
pub struct TupleError(String);
