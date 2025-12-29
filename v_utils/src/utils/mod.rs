pub mod eyre;
pub use eyre::*;

pub mod format;
pub use format::*;

pub mod serde;
pub use serde::*;

/// Macro for logging to both stdout and tracing info
/// Usage: log!("message") or log!("format {}", arg)
#[macro_export]
macro_rules! log {
	($($arg:tt)*) => {{
		println!($($arg)*);
		tracing::info!($($arg)*);
	}};
}

/// Macro for logging to both stderr and tracing debug
/// Usage: elog!("message") or elog!("format {}", arg)
#[macro_export]
macro_rules! elog {
	($($arg:tt)*) => {{
		eprintln!($($arg)*);
		tracing::debug!($($arg)*);
	}};
}

#[cfg(feature = "tracing")]
pub mod tracing;
#[cfg(feature = "tracing")]
pub use tracing::*;

/// # HACK
/// Assumes that `color_eyre` is in scope
///
/// Optionally pass `option_env!("LOG_DIRECTIVES")` as the first argument to embed compile-time log directives.
#[cfg(all(feature = "tracing", feature = "xdg"))]
#[macro_export]
macro_rules! clientside {
	() => {
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(
			v_utils::utils::LogDestination::xdg(env!("CARGO_PKG_NAME"))
				.stderr_errors(true)
				.compiled_directives(option_env!("LOG_DIRECTIVES")),
		);
	};
	($fname:expr) => {
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(
			v_utils::utils::LogDestination::xdg(env!("CARGO_PKG_NAME"))
				.fname($fname)
				.stderr_errors(true)
				.compiled_directives(option_env!("LOG_DIRECTIVES")),
		);
	};
}

/// Fallback when xdg is not available - logs to stdout
#[cfg(all(feature = "tracing", not(feature = "xdg")))]
#[macro_export]
macro_rules! clientside {
	() => {
		eprintln!("[v_utils] Warning: `xdg` feature not enabled, logging to stdout instead of file. Add `xdg` feature to v_utils dependency to enable file logging.");
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::default().compiled_directives(option_env!("LOG_DIRECTIVES")));
	};
	($fname:expr) => {
		eprintln!("[v_utils] Warning: `xdg` feature not enabled, logging to stdout instead of file. Add `xdg` feature to v_utils dependency to enable file logging.");
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::default().compiled_directives(option_env!("LOG_DIRECTIVES")));
	};
}

/// **Warning**: Consider using `strum` crate instead - this macro is likely redundant for most use cases.
#[macro_export]
macro_rules! define_str_enum {
  ($(#[$meta:meta])* $vis:vis enum $name:ident {
    $($(#[$variant_meta:meta])* $variant:ident => $str:expr),* $(,)?
  }) => {
    $(#[$meta])*
    $vis enum $name {
      $($(#[$variant_meta])* $variant),*
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          $(Self::$variant => write!(f, "{}", $str)),*
        }
      }
    }

    impl std::str::FromStr for $name {
      type Err = eyre::Report;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          $($str => Ok(Self::$variant)),*,
          _ => eyre::bail!("Invalid {} string: {}", stringify!($name).to_lowercase(), s),
        }
      }
    }
  };
}
