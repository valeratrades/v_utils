use std::{
	borrow::Cow,
	fs::File,
	io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
	path::{Path, PathBuf},
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

/// Entries older than this are dropped from the log file.
const LOG_MAX_AGE_SECS: i64 = 7 * 24 * 60 * 60;
/// Backstop for the age cap, which cannot bound a process that logs in a hot loop.
const LOG_MAX_SIZE_BYTES: u64 = 512 * 1024 * 1024;
/// How often the guardian re-checks the log file (1 minute)
const LOG_CHECK_INTERVAL: Duration = Duration::from_secs(60);
const CARGO_DIRECTIVES_PATH: &str = ".cargo/log_directives";
const DIRECTIVES_FILENAME: &str = "_log_directives";
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

// OTLP export layer (logs + traces over HTTP). Active only when
// OTEL_EXPORTER_OTLP_ENDPOINT is set, so non-cluster runs stay untouched. HTTP
// (reqwest-blocking) is deliberate: init runs before any tokio runtime exists,
// and the gRPC exporter would panic for lack of a reactor.
#[cfg(feature = "otlp")]
use std::sync::OnceLock;
use std::{
	collections::BTreeMap,
	env::{args_os, current_dir, current_exe, vars_os},
};

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
		//[x]TODO!!!: check out [tracing appender](https://docs.rs/tracing-appender/latest/tracing_appender/) - seems very useful for long-running processes. Probably should add it here + config for it in the same place as directives conf
		// Unbounded growth is handled instead by the guardian thread below (age + size cap),
		// which keeps the one-file-per-app layout that `LogDestination::Xdg` and every consumer
		// tailing `$XDG_STATE_HOME/<app>/.log` depend on — `tracing-appender`'s rotation only
		// ever writes `.log.<date>`. The runtime-configurable half of this is still open.

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
			.with(otlp_layer())
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
#[cfg(feature = "otlp")]
static OTLP_PROVIDERS: OnceLock<(opentelemetry_sdk::trace::SdkTracerProvider, opentelemetry_sdk::logs::SdkLoggerProvider)> = OnceLock::new();

#[cfg(feature = "otlp")]
fn otlp_layer<S>() -> Option<Box<dyn tracing_subscriber::Layer<S> + Send + Sync>>
where
	S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + Send + Sync, {
	use opentelemetry::trace::TracerProvider as _;
	use opentelemetry_otlp::{LogExporter, SpanExporter};
	use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider, trace::SdkTracerProvider};
	use tracing_subscriber::Layer as _;

	std::env::var_os("OTEL_EXPORTER_OTLP_ENDPOINT")?;
	// Resource picks up service.name from OTEL_SERVICE_NAME / OTEL_RESOURCE_ATTRIBUTES.
	let resource = Resource::builder().build();
	let span_exporter = SpanExporter::builder().with_http().build().expect("OTLP span exporter builds from OTEL_* env");
	let tracer_provider = SdkTracerProvider::builder().with_batch_exporter(span_exporter).with_resource(resource.clone()).build();
	let traces_layer = tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("v_utils"));
	let log_exporter = LogExporter::builder().with_http().build().expect("OTLP log exporter builds from OTEL_* env");
	let logger_provider = SdkLoggerProvider::builder().with_batch_exporter(log_exporter).with_resource(resource).build();
	let logs_layer = opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);
	// Providers own the batch-export threads; keep them alive for the process.
	let _ = OTLP_PROVIDERS.set((tracer_provider, logger_provider));
	Some(traces_layer.and_then(logs_layer).boxed())
}

#[cfg(not(feature = "otlp"))]
fn otlp_layer<S>() -> Option<Box<dyn tracing_subscriber::Layer<S> + Send + Sync>>
where
	S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + Send + Sync, {
	None
}

/// Wrapper to allow Arc<Mutex<File>> to implement Write safely
#[derive(Clone)]
struct SharedFileWriter {
	file: Arc<Mutex<File>>,
	path: Arc<PathBuf>,
	needs_trim: Arc<AtomicBool>,
}

impl SharedFileWriter {
	fn do_trim(&self, file: &mut File) {
		let Some(drop_bytes) = drop_offset(self.path.as_ref(), now_unix_secs() - LOG_MAX_AGE_SECS) else {
			return;
		};
		match shift_file_back(file, self.path.as_ref(), drop_bytes) {
			Ok(()) => eprintln!("[log-guardian] Trimmed {drop_bytes} bytes of aged-out entries from {}", self.path.display()),
			// A failed shift leaves the file as it was; the guardian retries on the next tick.
			Err(e) => eprintln!("[log-guardian] Failed to trim {}: {e}", self.path.display()),
		}
	}
}

fn now_unix_secs() -> i64 {
	std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("system clock is after 1970").as_secs() as i64
}

/// Unix seconds of the event that a `.pretty()` line opens, or `None` if the line is a
/// continuation (`    at …` / `    in …`), a blank, or otherwise not an event header.
///
/// Shape being matched, from the `fmt` layer configured in [`init_subscriber`]:
/// `␣␣2026-08-12T21:00:52.158805Z␣␣WARN␣…`
fn event_start_secs(line: &str) -> Option<i64> {
	let stamp = line.strip_prefix("  ")?.get(..19)?;
	let b = stamp.as_bytes();
	if b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
		return None;
	}
	let num = |r: std::ops::Range<usize>| stamp[r].parse::<i64>().ok();
	let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
	let (h, mi, s) = (num(11..13)?, num(14..16)?, num(17..19)?);

	// Hinnant `days_from_civil` — inverse of the `civil_from_days` in `lwc::time_ticks`.
	let y = y - i64::from(m <= 2);
	let era = if y >= 0 { y } else { y - 399 } / 400;
	let yoe = y - era * 400;
	let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
	let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
	let days = era * 146097 + doe - 719468;

	Some(days * 86400 + h * 3600 + mi * 60 + s)
}

/// Byte offset of the first event at or after `cutoff_secs`, i.e. how much of the head has aged
/// out. `None` when nothing needs dropping. Streams rather than reading the file in — these grow
/// to hundreds of MB, and the guardian runs on boxes where that is most of RAM.
fn drop_offset(path: &Path, cutoff_secs: i64) -> Option<u64> {
	let f = std::fs::File::open(path).ok()?;
	let len = f.metadata().ok()?.len();
	let mut reader = BufReader::new(f);
	let mut offset = 0u64;
	let mut saw_aged = false;
	let mut first_live = None;
	let mut line = String::new();
	//LOOP: bounded by EOF, which `read_line` only reports as it reaches it
	loop {
		line.clear();
		match reader.read_line(&mut line) {
			Ok(0) => break,
			Ok(n) => {
				// Only headers are cut points, so an event's continuations go with it.
				if let Some(secs) = event_start_secs(&line) {
					if secs >= cutoff_secs {
						first_live = Some(offset);
						break;
					}
					saw_aged = true;
				}
				offset += n as u64;
			}
			// Non-UTF8 in the log means we cannot find event boundaries past this point.
			Err(_) => break,
		}
	}
	let by_age = match first_live {
		Some(o) => o,
		None if saw_aged => len, // every event aged out
		None => 0,               // no events at all, or none old enough
	};
	let by_size = len.saturating_sub(LOG_MAX_SIZE_BYTES);
	let drop = by_age.max(by_size).min(len);
	(drop > 0).then_some(drop)
}

/// Discards the first `drop_bytes` of the file by copying the tail over the head in place.
/// The read handle stays `drop_bytes` ahead of the write cursor throughout, so the regions
/// never overlap.
fn shift_file_back(file: &mut File, path: &Path, drop_bytes: u64) -> std::io::Result<()> {
	let mut src = std::fs::File::open(path)?;
	src.seek(SeekFrom::Start(drop_bytes))?;
	file.seek(SeekFrom::Start(0))?;

	let written = std::io::copy(&mut src, file)?;
	file.set_len(written)?;
	file.seek(SeekFrom::End(0))?;
	file.flush()
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

/// Spawns a guardian thread that signals when the log file has aged past [`LOG_MAX_AGE_SECS`]
/// or grown past [`LOG_MAX_SIZE_BYTES`]. The trim itself happens on the next write, where the
/// file handle is already locked.
fn spawn_log_guardian(path: Arc<PathBuf>, needs_trim: Arc<AtomicBool>) {
	thread::spawn(move || {
		//LOOP: daemon thread, lives as long as the process it logs for
		loop {
			thread::sleep(LOG_CHECK_INTERVAL);

			let Ok(meta) = std::fs::metadata(path.as_ref()) else { continue };
			let aged = std::fs::File::open(path.as_ref())
				.ok()
				.and_then(|f| {
					let mut first = String::new();
					BufReader::new(f).read_line(&mut first).ok()?;
					event_start_secs(&first)
				})
				.is_some_and(|oldest| oldest < now_unix_secs() - LOG_MAX_AGE_SECS);

			if aged || meta.len() > LOG_MAX_SIZE_BYTES {
				needs_trim.store(true, Ordering::Relaxed);
			}
		}
	});
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

fn normalize_directives(s: &str) -> String {
	s.lines().map(|l| l.trim()).filter(|l| !l.is_empty() && !l.starts_with('#')).collect::<Vec<_>>().join(",")
}

fn filter_with_directives(logs_during_init: &mut Vec<Box<dyn FnOnce()>>, log_dir: Option<&Path>, compiled_directives: Option<&'static str>) -> EnvFilter {
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
fn trace_the_init() {
	let args: Vec<_> = args_os().collect();
	let vars: BTreeMap<_, _> = vars_os().collect();
	tracing::trace!("Executed as {exe:?} in {dir:?}\n", exe = current_exe(), dir = current_dir(),);
	tracing::trace!("Arguments: {args:#?}\n", args = args);
	tracing::trace!("Environment: {vars:#?}\n", vars = vars);
}

/// Installs `color_eyre` with frame filters that strip async runtime noise
/// (tokio, mio, futures_util) and test harness / panic machinery from backtraces.
///
/// Compiled symbols embed crate disambiguator hashes (e.g. `std[d04b43a2428f6e7c]::panicking`),
/// so we strip `[...]` before matching.
#[macro_export]
macro_rules! install_color_eyre {
	() => {
		color_eyre::config::HookBuilder::default()
			.add_frame_filter(Box::new(|frames| {
				let strip_hashes = |name: &str| -> String {
					let mut out = String::with_capacity(name.len());
					let mut chars = name.chars();
					while let Some(c) = chars.next() {
						if c == '[' {
							for c2 in chars.by_ref() {
								if c2 == ']' {
									break;
								}
							}
						} else {
							out.push(c);
						}
					}
					out
				};
				frames.retain(|frame| {
					let Some(name) = frame.name.as_ref() else { return true };
					let name = strip_hashes(name);
					let name = name.as_str();
					// async runtime
					if name.starts_with("tokio::") || name.starts_with("<tokio::") {
						return false;
					}
					if name.starts_with("mio::") {
						return false;
					}
					if name.starts_with("futures_util::") {
						return false;
					}
					if name.contains("::future::") {
						return false;
					}
					if name.contains("core::pin::Pin") {
						return false;
					}
					// test harness & panic machinery below user code
					if name.starts_with("test::") {
						return false;
					}
					if name.contains("std::panicking") || name.contains("std::panic::") {
						return false;
					}
					if name.contains("std::thread") || name.contains("std::sys::") {
						return false;
					}
					if name.contains("core::ops::function") {
						return false;
					}
					if name.contains("core::panic::unwind_safe") {
						return false;
					}
					if name.starts_with("<alloc::boxed::Box<dyn core") {
						return false;
					}
					if name.starts_with("__rustc") {
						return false;
					}
					if name.contains("start_thread") || name.contains("clone3") {
						return false;
					}
					true
				});
			}))
			.install()
			.unwrap()
	};
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

	#[test]
	fn event_start_secs_reads_headers_and_skips_continuations() {
		assert_eq!(event_start_secs("  1970-01-01T00:00:00.000000Z  INFO x: y"), Some(0));
		assert_eq!(event_start_secs("  2026-08-12T21:00:52.158805Z  WARN x: y"), Some(1786568452));
		// Leap-day and end-of-era cases, where `days_from_civil`'s month shift is easiest to get wrong.
		assert_eq!(event_start_secs("  2024-02-29T00:00:00.0Z  INFO x: y"), Some(1709164800));
		assert_eq!(event_start_secs("  2000-03-01T00:00:00.0Z  INFO x: y"), Some(951868800));

		assert_eq!(event_start_secs("    at src/main.rs:60"), None);
		assert_eq!(event_start_secs("    in some::span with x: 1"), None);
		assert_eq!(event_start_secs(""), None);
		assert_eq!(event_start_secs("  not-a-timestamp here"), None);
	}

	#[test]
	fn drop_offset_cuts_at_the_first_live_event() {
		let dir = tempfile::tempdir().expect("create tempdir");
		let path = dir.path().join("t.log");
		let head = "  2026-08-01T00:00:00.0Z  INFO old: a\n    at src/a.rs:1\n\n";
		let tail = "  2026-08-12T00:00:00.0Z  INFO new: b\n    at src/b.rs:2\n\n";
		std::fs::write(&path, format!("{head}{tail}")).expect("write log");

		// Cutoff between the two events: the whole first event, continuation included, goes.
		let cutoff = event_start_secs("  2026-08-06T00:00:00.0Z  INFO x: y").expect("parses");
		assert_eq!(drop_offset(&path, cutoff), Some(head.len() as u64));

		// Nothing old enough to drop.
		assert_eq!(drop_offset(&path, 0), None);

		// Everything aged out.
		assert_eq!(drop_offset(&path, i64::MAX), Some((head.len() + tail.len()) as u64));
	}

	#[test]
	fn shift_file_back_keeps_the_tail_and_appends_after_it() {
		let dir = tempfile::tempdir().expect("create tempdir");
		let path = dir.path().join("t.log");
		std::fs::write(&path, "DROPME").expect("seed");
		// Larger than the 64KiB copy buffer, so the chunked loop is actually exercised.
		let keep = "k".repeat(200 * 1024);
		std::fs::write(&path, format!("DROPME{keep}")).expect("write log");

		let mut file = std::fs::OpenOptions::new().write(true).open(&path).expect("open for write");
		shift_file_back(&mut file, &path, 6).expect("shift succeeds");
		write!(file, "TAIL").expect("append after shift");
		drop(file);

		assert_eq!(std::fs::read_to_string(&path).expect("read back"), format!("{keep}TAIL"));
	}
}
