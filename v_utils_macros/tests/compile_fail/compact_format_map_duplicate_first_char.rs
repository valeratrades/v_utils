use v_utils_macros::CompactFormatMap;

// Fields 'take_profit' and 'trailing_stop' both start with 't'.
// CompactFormatMap encodes each field by its first character, so duplicates must be a compile error.
#[derive(CompactFormatMap, Debug)] //~ ERROR: proc-macro derive panicked
pub struct Ambiguous {
	take_profit: f64,
	trailing_stop: f64,
}
