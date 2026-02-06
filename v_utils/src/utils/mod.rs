pub mod eyre;
pub use eyre::*;

pub mod format;
pub use format::*;

pub mod info_size;
pub use info_size::*;

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

/// Sets up error handling (color_eyre, miette) and tracing subscriber for client-side applications.
///
/// HACK: Assumes that `color_eyre` and `miette` are in scope.
///
/// # Behavior
/// - Installs color_eyre panic/error hooks (with span trace capture disabled)
/// - Installs miette diagnostic hook (with terminal links and 3 context lines)
/// - Initializes tracing subscriber with:
///   - JSON formatted logs to `~/.local/state/{pkg_name}/{pkg_name}.log`
///   - WARN and ERROR also printed to stderr
///   - Log directives from `LOG_DIRECTIVES` env var (set by build.rs)
///
/// # Integration Test Mode
/// When `__IS_INTEGRATION_TEST` env var is set, logs to stdout with debug level instead.
/// This allows integration tests to capture and inspect log output.
///
/// # Usage
/// ```ignore
/// // Default log filename ({pkg_name}.log)
/// v_utils::clientside!();
///
/// // Custom log filename from Option<String> (e.g., from CLI arg)
/// v_utils::clientside!(extract_log_to());
/// ```
#[cfg(all(feature = "tracing", feature = "xdg"))]
#[macro_export]
macro_rules! clientside {
	() => {
		v_utils::clientside!(None::<String>);
	};
	($fname:expr) => {
		//color_eyre::config::HookBuilder::default().capture_span_trace_by_default(false).install().unwrap(); // thought would allow for nice interop with miette, but in reality I lose my colored traces
		color_eyre::install().unwrap();
		miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().terminal_links(true).build()))).expect("miette hook already set");
		if std::env::var("__IS_INTEGRATION_TEST").is_ok() {
			// SAFETY: Called at program start before any other threads are spawned
			unsafe { std::env::set_var("LOG_DIRECTIVES", concat!("debug,", env!("CARGO_PKG_NAME"), "=debug")) };
			v_utils::utils::init_subscriber(v_utils::utils::LogDestination::default());
		} else {
			let mut dest = v_utils::utils::LogDestination::xdg(env!("CARGO_PKG_NAME"))
				.stderr_errors(true)
				.compiled_directives(option_env!("LOG_DIRECTIVES"));
			if let Some(fname) = $fname {
				dest = dest.fname(fname);
			}
			v_utils::utils::init_subscriber(dest);
		}
	};
}

/// Fallback when xdg is not available - logs to stdout
#[cfg(all(feature = "tracing", not(feature = "xdg")))]
#[macro_export]
macro_rules! clientside {
	() => {
		v_utils::clientside!(None::<String>);
	};
	($fname:expr) => {
		let _ = $fname; // silence unused warning
		eprintln!("[v_utils] Warning: `xdg` feature not enabled, logging to stdout instead of file. Add `xdg` feature to v_utils dependency to enable file logging.");
		color_eyre::install().unwrap();
		miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().terminal_links(true).context_lines(3).build()))).expect("miette hook already set");
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
