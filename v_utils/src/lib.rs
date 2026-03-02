#![allow(clippy::get_first)]
#![allow(clippy::len_zero)]
#![allow(clippy::tabs_in_doc_comments)]
#![feature(stmt_expr_attributes)]
#![feature(specialization)]
#![allow(incomplete_features)]

#[cfg(all(feature = "assert-wasm-compat", feature = "async-io"))]
compile_error!("Feature `async-io` is not compatible with wasm.");

#[cfg(all(feature = "assert-wasm-compat", feature = "full"))]
compile_error!("Feature `full` is not compatible with wasm (pulls in console-subscriber with mio).");

#[cfg(all(feature = "assert-wasm-compat", feature = "xdg"))]
compile_error!("Feature `xdg` is not compatible with wasm.");

// of course it's included unconditionally - the crate itself is called "v_utils"
pub mod utils;

#[cfg(feature = "io")]
pub mod io;
pub mod other;
#[cfg(feature = "lite")]
pub mod prelude;
#[cfg(feature = "trades")]
pub mod trades;
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
	#[cfg(feature = "cli")]
	pub extern crate toml;

	#[cfg(feature = "xdg")]
	pub extern crate xdg;

	#[cfg(all(feature = "io", not(target_arch = "wasm32")))]
	pub use crate::io::xdg::{home_dir, xdg_cache_fallback, xdg_config_fallback, xdg_data_fallback, xdg_runtime_fallback, xdg_state_fallback};

	#[cfg(feature = "cli")]
	#[derive(Debug, thiserror::Error)]
	pub enum SettingsError {
		#[error("Found multiple config files:\n{}\n\nPlease keep only one. Pick a location, merge all settings into it, then delete the rest.", .paths.iter().map(|p| format!("  - {}", p.display())).collect::<Vec<_>>().join("\n"))]
		MultipleConfigs { paths: Vec<std::path::PathBuf> },
		/// NB: no `#[from]`/`#[source]` — these are terminal error messages, not chain links.
		/// With `#[from]`, thiserror sets `source()` to the inner type, which causes
		/// `format_eyre_chain_for_user` to print the same message twice (once as root, once as wrapper).
		#[error("{0}")]
		Parse(crate::__internal::config::ConfigError),
		#[error("{0}")]
		Other(crate::__internal::eyre::Report),
	}
	#[cfg(feature = "cli")]
	impl From<crate::__internal::config::ConfigError> for SettingsError {
		fn from(e: crate::__internal::config::ConfigError) -> Self {
			Self::Parse(e)
		}
	}
	#[cfg(feature = "cli")]
	impl From<crate::__internal::eyre::Report> for SettingsError {
		fn from(e: crate::__internal::eyre::Report) -> Self {
			Self::Other(e)
		}
	}

	#[cfg(feature = "cli")]
	impl crate::utils::SysexitCode for SettingsError {
		fn sysexit(&self) -> crate::utils::Sysexit {
			crate::utils::Sysexit::Config
		}
	}
}
#[cfg(feature = "distributions")]
pub mod distributions;
#[cfg(test)]
pub(crate) mod internal_utils;

//Q: I like the idea of having a prelude, but atm it just leads to possibility of mismatching def paths, client imports v_utils and something else relying on a different version of v_utils

pub use other::*;

#[cfg(feature = "macros")]
pub extern crate v_utils_macros as macros;
