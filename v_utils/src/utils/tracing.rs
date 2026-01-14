use std::{
	borrow::Cow,
	fs::File,
	io::{BufRead, BufReader, Seek, SeekFrom, Write},
	path::PathBuf,
	sync::{
		Arc, Mutex,
		atomic::{AtomicBool, Ordering},
	},
	thread,
	time::Duration,
};

use tracing::{info, warn};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, prelude::*};

/// Maximum log file size before trimming (20GB)
const LOG_MAX_SIZE_BYTES: u64 = 20 * 1024 * 1024 * 1024;
/// How often to check log file size (1 minute)
const LOG_CHECK_INTERVAL: Duration = Duration::from_secs(60);

/// Wrapper to allow Arc<Mutex<File>> to implement Write safely
#[derive(Clone)]
struct SharedFileWriter {
	file: Arc<Mutex<File>>,
	path: Arc<PathBuf>,
	needs_trim: Arc<AtomicBool>,
}

impl SharedFileWriter {
	fn do_trim(&self, file: &mut File) {
		// Read the file content (open new handle for reading)
		let lines: Vec<String> = match std::fs::File::open(self.path.as_ref()) {
			Ok(f) => BufReader::new(f).lines().filter_map(|l| l.ok()).collect(),
			Err(_) => return,
		};

		let total_lines = lines.len();
		if total_lines < 1000 {
			return;
		}

		// Keep the last 75% of lines
		let lines_to_skip = total_lines / 4;
		let remaining_lines = &lines[lines_to_skip..];

		// Truncate and rewrite the file
		if file.set_len(0).is_err() {
			return;
		}
		if file.seek(SeekFrom::Start(0)).is_err() {
			return;
		}

		for line in remaining_lines {
			let _ = writeln!(file, "{line}");
		}
		let _ = file.flush();

		eprintln!("[log-guardian] Trimmed log file: removed {lines_to_skip} lines, {} remaining", remaining_lines.len());
	}
}

impl Write for SharedFileWriter {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let mut file = self.file.lock().unwrap();

		// Check if guardian signaled we need to trim
		if self.needs_trim.swap(false, Ordering::Relaxed) {
			self.do_trim(&mut file);
		}

		file.write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.file.lock().unwrap().flush()
	}
}

/// Spawns a guardian thread that monitors log file size and signals when trim is needed.
fn spawn_log_guardian(path: Arc<PathBuf>, needs_trim: Arc<AtomicBool>) {
	thread::spawn(move || {
		loop {
			thread::sleep(LOG_CHECK_INTERVAL);

			let file_size = match std::fs::metadata(path.as_ref()) {
				Ok(m) => m.len(),
				Err(_) => continue,
			};

			if file_size > LOG_MAX_SIZE_BYTES {
				needs_trim.store(true, Ordering::Relaxed);
			}
		}
	});
}

/// # Panics (iff ` Some(path)` && `path`'s parent dir doesn't exist || `path` is not writable)
/// Set "TEST_LOG=1" to redirect to stdout
pub fn init_subscriber(log_destination: LogDestination) {
	let mut logs_during_init: Vec<Box<dyn FnOnce()>> = Vec::new();
	let compiled_directives = log_destination.compiled_directives;

	let mut setup = |make_writer: Box<dyn Fn() -> Box<dyn Write> + Send + Sync>, stderr_errors: bool, log_dir: Option<PathBuf>| {
		//TODO: 	console_error_panic_hook::set_once(); // for wasm32 targets exclusively.
		//let tokio_console_artifacts_filter = EnvFilter::new("tokio[trace]=off,runtime[trace]=off");
		//TEST: if `with_ansi(false)` removes the need for `AnsiEsc` completely
		let formatting_layer = tracing_subscriber::fmt::layer().json().pretty().with_writer(make_writer).with_ansi(false).with_file(true).with_line_number(true)/*.with_filter(tokio_console_artifacts_filter)*/;

		let env_filter = filter_with_directives(&mut logs_during_init, log_dir.as_deref(), compiled_directives);

		let error_layer = ErrorLayer::default();

		// freaks out if it's built into a binary, and then two instances of it are created.
		//TODO: figure out how to limit this to debug builds \
		//#[feature("tokio_full")]
		//let console_layer = console_subscriber::spawn::<Registry>(); // does nothing unless `RUST_LOG=tokio=trace,runtime=trace`. But how do I make it not write to file for them?
		//
		//TODO!!!: check out [tracing appender](https://docs.rs/tracing-appender/latest/tracing_appender/) - seems very useful for long-running processes. Probably should add it here + config for it in the same place as directives conf

		use tracing_subscriber::filter::LevelFilter;

		// Conditionally create stderr layer (WARN and ERROR go to stderr)
		let stderr_layer = if stderr_errors {
			Some(tracing_subscriber::fmt::layer().with_writer(std::io::stderr).with_ansi(true).with_filter(LevelFilter::WARN))
		} else {
			None
		};

		tracing_subscriber::registry()
			//.with(console_layer)
			.with(env_filter)
			.with(formatting_layer)
			.with(stderr_layer)
			.with(error_layer)
			.init();
		//tracing_subscriber::registry()
		//  .with(tracing_subscriber::layer::Layer::and_then(formatting_layer, error_layer).with_filter(env_filter))
		//  .with(console_layer)
		//  .init();
	};

	fn destination_is_path<F, P>(path: P, stderr_errors: bool, setup: F)
	where
		P: Into<PathBuf> + Sized,
		F: FnOnce(Box<dyn Fn() -> Box<dyn Write> + Send + Sync>, bool, Option<PathBuf>), {
		let path = path.into();
		let log_dir = path.parent().map(|p| p.to_path_buf());

		// Open the file once and share it via Arc<Mutex<>>
		let file = std::fs::OpenOptions::new()
			.create(true)
			.write(true)
			.truncate(true)
			.open(&path)
			.unwrap_or_else(|_| panic!("Couldn't open {} for writing. If its parent directory doesn't exist, create it manually first", path.display()));

		let file_arc = Arc::new(Mutex::new(file));
		let path_arc = Arc::new(path);
		let needs_trim = Arc::new(AtomicBool::new(false));

		// Spawn guardian thread to monitor log file size
		spawn_log_guardian(Arc::clone(&path_arc), Arc::clone(&needs_trim));

		let shared_writer = SharedFileWriter {
			file: file_arc,
			path: path_arc,
			needs_trim,
		};

		setup(
			Box::new(move || {
				// Clone the wrapper, which clones the Arc (not the file handle)
				Box::new(shared_writer.clone()) as Box<dyn Write>
			}),
			stderr_errors,
			log_dir,
		);
	}

	let stderr_errors = log_destination.stderr_errors;
	match log_destination.kind {
		LogDestinationKind::File { path } => {
			destination_is_path(path, stderr_errors, setup);
		}
		LogDestinationKind::Stdout => {
			setup(Box::new(|| Box::new(std::io::stdout())), false, None);
		}
		#[cfg(all(not(target_arch = "wasm32"), feature = "xdg"))]
		LogDestinationKind::Xdg { dname, fname } => {
			let associated_state_home = xdg::BaseDirectories::with_prefix(dname).create_state_directory("").unwrap();
			let filename = fname
				.as_ref()
				.map(|s| if s.ends_with(".log") { s.to_string() } else { format!("{s}.log") })
				.unwrap_or_else(|| ".log".to_string());
			let log_path = associated_state_home.join(filename);
			destination_is_path(log_path, stderr_errors, setup);
		}
	};

	for log in logs_during_init {
		log();
	}
	info!("Starting ...");

	trace_the_init(); //? Should I make this a trace?
}

#[derive(Clone, Debug, Default)]
pub struct LogDestination {
	pub kind: LogDestinationKind,
	pub stderr_errors: bool,
	/// Compile-time embedded directives (set via build.rs). Takes priority over file-based directives.
	pub compiled_directives: Option<&'static str>,
}
impl LogDestination {
	/// Helper for creating File variant
	pub fn file<P: Into<PathBuf>>(path: P) -> Self {
		LogDestination {
			kind: LogDestinationKind::File { path: path.into() },
			stderr_errors: false,
			compiled_directives: None,
		}
	}

	/// Helper for creating Xdg variant
	#[cfg(all(not(target_arch = "wasm32"), feature = "xdg"))]
	pub fn xdg<S: Into<String>>(name: S) -> Self {
		LogDestination {
			kind: LogDestinationKind::Xdg { dname: name.into(), fname: None },
			stderr_errors: false,
			compiled_directives: None,
		}
	}

	/// Set custom filename for Xdg variant (creates `{fname}.log`)
	#[cfg(all(not(target_arch = "wasm32"), feature = "xdg"))]
	pub fn fname<S: Into<String>>(mut self, fname: S) -> Self {
		if let LogDestinationKind::Xdg { dname, .. } = self.kind {
			self.kind = LogDestinationKind::Xdg { dname, fname: Some(fname.into()) };
		}
		self
	}

	/// Enable/disable ERROR level logging to stderr
	pub fn stderr_errors(mut self, enabled: bool) -> Self {
		self.stderr_errors = enabled;
		self
	}

	/// Set compile-time embedded directives (takes priority over file-based directives).
	/// Typically used with `option_env!("LOG_DIRECTIVES")` in the downstream crate.
	pub fn compiled_directives(mut self, directives: Option<&'static str>) -> Self {
		self.compiled_directives = directives;
		self
	}
}

#[derive(Clone, Debug, Default)]
pub enum LogDestinationKind {
	#[default]
	Stdout,
	File {
		path: PathBuf,
	},
	#[cfg(all(not(target_arch = "wasm32"), feature = "xdg"))]
	Xdg {
		dname: String,
		fname: Option<String>,
	},
}

impl From<&str> for LogDestination {
	fn from(s: &str) -> Self {
		if s == "stdout" { LogDestination::default() } else { LogDestination::file(s) }
	}
}

impl From<PathBuf> for LogDestination {
	fn from(path: PathBuf) -> Self {
		LogDestination::file(path)
	}
}

const CARGO_DIRECTIVES_PATH: &str = ".cargo/log_directives";
const DIRECTIVES_FILENAME: &str = "_log_directives";

fn normalize_directives(s: &str) -> String {
	s.lines().map(|l| l.trim()).filter(|l| !l.is_empty() && !l.starts_with('#')).collect::<Vec<_>>().join(",")
}

fn filter_with_directives(logs_during_init: &mut Vec<Box<dyn FnOnce()>>, log_dir: Option<&std::path::Path>, compiled_directives: Option<&'static str>) -> EnvFilter {
	static DEFAULT_DIRECTIVES: &str = "debug,hyper=info,hyper_util=info";

	let log_dir_path = log_dir.map(|d| d.join(DIRECTIVES_FILENAME));

	// Priority order:
	// 1. .cargo/log_directives file (for development - highest priority)
	// 2. _log_directives in log directory (for runtime override of installed binaries)
	// 3. Compiled-in directives (production defaults, embedded via build.rs)
	// 4. Hard-coded default directives

	let (directives, source): (Cow<'_, str>, Option<String>) = if let Ok(s) = std::fs::read_to_string(CARGO_DIRECTIVES_PATH) {
		(Cow::Owned(normalize_directives(&s)), Some(CARGO_DIRECTIVES_PATH.to_owned()))
	} else if let Some(ref p) = log_dir_path {
		if let Ok(s) = std::fs::read_to_string(p) {
			(Cow::Owned(normalize_directives(&s)), Some(p.display().to_string()))
		} else if let Some(compiled) = compiled_directives {
			(Cow::Owned(normalize_directives(compiled)), Some("compiled-in (LOG_DIRECTIVES)".to_owned()))
		} else {
			(Cow::Borrowed(DEFAULT_DIRECTIVES), None)
		}
	} else if let Some(compiled) = compiled_directives {
		(Cow::Owned(normalize_directives(compiled)), Some("compiled-in (LOG_DIRECTIVES)".to_owned()))
	} else {
		(Cow::Borrowed(DEFAULT_DIRECTIVES), None)
	};

	match source {
		Some(path) => {
			let directives_str = directives.clone().into_owned();
			logs_during_init.push(Box::new(move || info!("Using log directives from `{path}`:\n{directives_str}")));
		}
		None => {
			let cargo_path = CARGO_DIRECTIVES_PATH.to_owned();
			let log_dir_msg = log_dir_path.map(|p| p.display().to_string());
			logs_during_init.push(Box::new(move || match log_dir_msg {
				Some(p) => warn!("No log directives file found (checked `{cargo_path}` and `{p}`), using defaults"),
				None => warn!("No log directives file found at `{cargo_path}`, using defaults"),
			}));
		}
	}

	EnvFilter::builder()
		.parse(&directives)
		.unwrap_or_else(|_| panic!("Error parsing tracing directives:\n```\n{directives}\n```\n"))
}

use std::{
	collections::BTreeMap,
	env::{args_os, current_dir, current_exe, vars_os},
};
fn trace_the_init() {
	let args: Vec<_> = args_os().collect();
	let vars: BTreeMap<_, _> = vars_os().collect();
	tracing::trace!("Executed as {exe:?} in {dir:?}\n", exe = current_exe(), dir = current_dir(),);
	tracing::trace!("Arguments: {args:#?}\n", args = args);
	tracing::trace!("Environment: {vars:#?}\n", vars = vars);
}

#[cfg(test)]
mod tests {
	use tracing_subscriber::EnvFilter;

	use super::*;

	#[test]
	fn normalize_directives_handles_mixed_formats() {
		let input = r#"

debug,hyper=info,hyper_util=info
# this is a comment
warn
  trace
my_crate=debug

"#;
		let normalized = normalize_directives(input);
		assert_eq!(normalized, "debug,hyper=info,hyper_util=info,warn,trace,my_crate=debug");

		// Verify it actually parses
		EnvFilter::builder().parse(&normalized).expect("normalized directives should parse");
	}
}
