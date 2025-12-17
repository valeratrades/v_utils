#![allow(clippy::get_first)]
#![allow(clippy::len_zero)]
#![allow(clippy::tabs_in_doc_comments)]
#![feature(stmt_expr_attributes)]

#[cfg(all(feature = "assert-wasm-compat", feature = "async-io"))]
compile_error!("Feature `async-io` is not compatible with wasm.");

#[cfg(all(feature = "assert-wasm-compat", feature = "full"))]
compile_error!("Feature `full` is not compatible with wasm (pulls in console-subscriber with mio).");

#[cfg(all(feature = "assert-wasm-compat", feature = "xdg"))]
compile_error!("Feature `xdg` is not compatible with wasm.");

// of course it's included unconditionally - the crate itself is called "v_utils"
pub mod utils;

#[cfg(test)]
pub(crate) mod internal_utils;

//Q: I like the idea of having a prelude, but atm it just leads to possibility of mismatching def paths, client imports v_utils and something else relying on a different version of v_utils
#[cfg(feature = "lite")]
pub mod prelude;

pub mod other;
pub use other::*;

#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "trades")]
pub mod trades;

#[cfg(feature = "macros")]
pub extern crate v_utils_macros as macros;

#[doc(hidden)]
pub mod __internal {
	pub extern crate eyre;
	pub extern crate serde;

	#[cfg(feature = "wasm")]
	pub extern crate console_error_panic_hook;
	#[cfg(feature = "wasm")]
	pub extern crate console_log;

	#[cfg(feature = "cli")]
	pub extern crate config;
	#[cfg(feature = "cli")]
	pub extern crate facet;
	#[cfg(feature = "cli")]
	pub extern crate facet_json;
	#[cfg(feature = "cli")]
	pub extern crate facet_toml;
	#[cfg(feature = "cli")]
	pub extern crate serde_json;

	#[cfg(feature = "xdg")]
	pub extern crate xdg;

	#[cfg(all(feature = "io", not(target_arch = "wasm32")))]
	pub use crate::io::xdg::{home_dir, xdg_cache_fallback, xdg_config_fallback, xdg_data_fallback, xdg_runtime_fallback, xdg_state_fallback};
}

#[cfg(feature = "distributions")]
pub mod distributions;
