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
			let _ = writeln!(file, "{}", line);
		}
		let _ = file.flush();

		eprintln!("[log-guardian] Trimmed log file: removed {} lines, {} remaining", lines_to_skip, remaining_lines.len());
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
	let mut setup = |make_writer: Box<dyn Fn() -> Box<dyn Write> + Send + Sync>, stderr_errors: bool, log_dir: Option<PathBuf>| {
		//TODO: 	console_error_panic_hook::set_once(); // for wasm32 targets exclusively.
		//let tokio_console_artifacts_filter = EnvFilter::new("tokio[trace]=off,runtime[trace]=off");
		//TEST: if `with_ansi(false)` removes the need for `AnsiEsc` completely
		let formatting_layer = tracing_subscriber::fmt::layer().json().pretty().with_writer(make_writer).with_ansi(false).with_file(true).with_line_number(true)/*.with_filter(tokio_console_artifacts_filter)*/;

		let env_filter = filter_with_directives(&mut logs_during_init, log_dir.as_deref());

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

	match log_destination {
		LogDestination::File { path, stderr_errors } => {
			destination_is_path(path, stderr_errors, setup);
		}
		LogDestination::Stdout => {
			setup(Box::new(|| Box::new(std::io::stdout())), false, None);
		}
		#[cfg(all(not(target_arch = "wasm32"), feature = "xdg"))]
		LogDestination::Xdg { dname, fname, stderr_errors } => {
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
pub enum LogDestination {
	#[default]
	Stdout,
	File {
		path: PathBuf,
		stderr_errors: bool,
	},
	#[cfg(all(not(target_arch = "wasm32"), feature = "xdg"))]
	Xdg {
		dname: String,
		fname: Option<String>,
		stderr_errors: bool,
	},
}

impl LogDestination {
	/// Helper for creating [File](LogDestination::File) variant
	pub fn file<P: Into<PathBuf>>(path: P) -> Self {
		LogDestination::File {
			path: path.into(),
			stderr_errors: false,
		}
	}

	/// Helper for creating [XdgDataHome](LogDestination::Xdg) variant
	#[cfg(all(not(target_arch = "wasm32"), feature = "xdg"))]
	pub fn xdg<S: Into<String>>(name: S) -> Self {
		LogDestination::Xdg {
			dname: name.into(),
			fname: None,
			stderr_errors: false,
		}
	}

	/// Set custom filename for Xdg variant (creates `{fname}.log`)
	#[cfg(all(not(target_arch = "wasm32"), feature = "xdg"))]
	pub fn fname<S: Into<String>>(self, fname: S) -> Self {
		match self {
			LogDestination::Xdg { dname, stderr_errors, .. } => LogDestination::Xdg {
				dname,
				fname: Some(fname.into()),
				stderr_errors,
			},
			_ => self,
		}
	}

	/// Enable/disable ERROR level logging to stderr
	pub fn stderr_errors(self, enabled: bool) -> Self {
		match self {
			LogDestination::File { path, .. } => LogDestination::File { path, stderr_errors: enabled },
			#[cfg(all(not(target_arch = "wasm32"), feature = "xdg"))]
			LogDestination::Xdg { dname, fname, .. } => LogDestination::Xdg {
				dname,
				fname,
				stderr_errors: enabled,
			},
			other => other,
		}
	}
}
impl From<&str> for LogDestination {
	fn from(s: &str) -> Self {
		if s == "stdout" {
			LogDestination::Stdout
		} else {
			LogDestination::File {
				path: s.into(),
				stderr_errors: false,
			}
		}
	}
}

impl From<PathBuf> for LogDestination {
	fn from(path: PathBuf) -> Self {
		LogDestination::File { path, stderr_errors: false }
	}
}

const CARGO_DIRECTIVES_PATH: &str = ".cargo/log_directives";
const DIRECTIVES_FILENAME: &str = "_log_directives";

fn filter_with_directives(logs_during_init: &mut Vec<Box<dyn FnOnce()>>, log_dir: Option<&std::path::Path>) -> EnvFilter {
	static DEFAULT_DIRECTIVES: &str = "debug,hyper=info,hyper_util=info";

	let log_dir_path = log_dir.map(|d| d.join(DIRECTIVES_FILENAME));

	// Try .cargo/log_directives first, then _log_directives in log directory
	let (directives, source): (Cow<'_, str>, Option<String>) = if let Ok(s) = std::fs::read_to_string(CARGO_DIRECTIVES_PATH) {
		(Cow::Owned(s.trim().to_owned()), Some(CARGO_DIRECTIVES_PATH.to_owned()))
	} else if let Some(ref p) = log_dir_path {
		if let Ok(s) = std::fs::read_to_string(p) {
			(Cow::Owned(s.trim().to_owned()), Some(p.display().to_string()))
		} else {
			(Cow::Borrowed(DEFAULT_DIRECTIVES), None)
		}
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
