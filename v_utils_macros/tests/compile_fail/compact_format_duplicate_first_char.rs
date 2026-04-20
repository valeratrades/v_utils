use v_utils_macros::CompactFormatNamed;

// Fields 'alpha' and 'amplitude' both start with 'a'.
// Compact format encodes each field by its first character, so duplicates must be a compile error.
#[derive(CompactFormatNamed, Debug)] //~ ERROR: proc-macro derive panicked
struct Ambiguous {
	alpha: f64,
	amplitude: f64,
}
