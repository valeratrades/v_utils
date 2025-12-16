pub mod eyre;
pub use eyre::*;

#[deprecated(since = "0.8.0", note = "will be removed in next major version; use the `snapshot_fonts` crate instead")]
pub mod snapshots;
#[allow(deprecated)]
pub use snapshots::*;

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
#[cfg(all(feature = "tracing", feature = "xdg"))]
#[macro_export]
macro_rules! clientside {
	() => {
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::xdg(env!("CARGO_PKG_NAME")).stderr_errors(true));
	};
	($fname:expr) => {
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::xdg(env!("CARGO_PKG_NAME")).fname($fname).stderr_errors(true));
	};
}

/// Fallback when xdg is not available - logs to stdout
#[cfg(all(feature = "tracing", not(feature = "xdg")))]
#[macro_export]
macro_rules! clientside {
	() => {
		eprintln!("[v_utils] Warning: `xdg` feature not enabled, logging to stdout instead of file. Add `xdg` feature to v_utils dependency to enable file logging.");
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::Stdout);
	};
	($fname:expr) => {
		eprintln!("[v_utils] Warning: `xdg` feature not enabled, logging to stdout instead of file. Add `xdg` feature to v_utils dependency to enable file logging.");
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::Stdout);
	};
}

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
