use std::io::{self, Write};

/// Result of a confirmation prompt
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConfirmResult {
	Yes,
	No,
	All,
	/// User chose to change the suggestion; contains the edited value.
	Change(String),
}

/// Builder for confirmation prompts.
///
/// # Examples
/// ```rust,ignore
/// use v_utils::io::{confirmation, ConfirmResult};
///
/// // Simple yes/no
/// let result = confirmation("Proceed?").flush().await;
///
/// // With "all" option
/// let result = confirmation("Process this file?").all().flush().await;
///
/// // With "change" option for custom input
/// match confirmation("Use this value?")
///     .change("default suggestion")
///     .flush().await
/// {
///     ConfirmResult::Yes => println!("Using default"),
///     ConfirmResult::Change(edited) => println!("Using: {edited}"),
///     ConfirmResult::No => println!("Aborted"),
///     _ => {}
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Confirmation<'a> {
	message: &'a str,
	all: bool,
	change: Option<&'a str>,
}
impl<'a> Confirmation<'a> {
	/// Add the "all" option (`a/A`) to apply to all remaining items.
	pub fn all(mut self) -> Self {
		self.all = true;
		self
	}

	/// Add the "change" option (`c/C`) to allow editing the suggestion.
	///
	/// When selected, provides an inline TUI for editing, similar to cargo's build progress.
	pub fn change(mut self, suggestion: &'a str) -> Self {
		self.change = Some(suggestion);
		self
	}

	fn format_prompt(&self) -> String {
		let mut options = vec!["Y", "n"];
		if self.all {
			options.push("a");
		}
		if self.change.is_some() {
			options.push("c (change)");
		}
		format!("{} [{}] ", self.message, options.join("/"))
	}

	/// Execute the confirmation prompt.
	#[cfg(feature = "async-io")]
	pub async fn flush(self) -> ConfirmResult {
		let all = self.all;
		let change = self.change.map(|s| s.to_owned());
		let prompt = self.format_prompt();

		tokio::task::spawn_blocking(move || run_confirmation_blocking(&prompt, all, change.as_deref()))
			.await
			.expect("confirmation task panicked")
	}

	/// Execute the confirmation prompt (blocking, for non-async contexts).
	#[must_use]
	pub fn flush_blocking(self) -> ConfirmResult {
		run_confirmation_blocking(&self.format_prompt(), self.all, self.change)
	}
}

/// Entry point for building a confirmation prompt.
pub fn confirmation(message: &str) -> Confirmation<'_> {
	Confirmation { message, all: false, change: None }
}

fn run_confirmation_blocking(prompt: &str, all: bool, change: Option<&str>) -> ConfirmResult {
	let stdin = io::stdin();
	let mut stdout = io::stdout();

	print!("{prompt}");
	stdout.flush().unwrap();

	let mut input = String::new();
	while {
		input.clear();
		stdin.read_line(&mut input).expect("Failed to read line") > 0
	} {
		match input.trim().to_ascii_lowercase().as_str() {
			"y" | "yes" | "" => return ConfirmResult::Yes,
			"n" | "no" => {
				eprintln!("Aborted by user.");
				return ConfirmResult::No;
			}
			"a" | "all" if all => return ConfirmResult::All,
			"c" | "change" if change.is_some() => {
				if let Some(edited) = read_inline_edit(change.unwrap()) {
					return ConfirmResult::Change(edited);
				}
				// User cancelled, re-prompt
				print!("{prompt}");
				stdout.flush().unwrap();
			}
			_ => {
				print!("Invalid option. {prompt}");
				stdout.flush().unwrap();
			}
		}
	}
	// EOF on stdin
	eprintln!("Aborted by user.");
	ConfirmResult::No
}

/// Read user edit inline, cargo-style (same line updates).
/// Returns Some(edited_value) on Enter, None on Escape/Ctrl-C.
fn read_inline_edit(initial: &str) -> Option<String> {
	use std::io::Read;

	let mut stdout = io::stdout();
	let stdin = io::stdin();

	print!("\r\x1b[K> {initial}");
	stdout.flush().unwrap();

	let mut value = initial.to_string();
	let mut stdin_handle = stdin.lock();

	#[cfg(unix)]
	let original_termios = {
		use std::os::fd::AsRawFd;
		let fd = std::io::stdin().as_raw_fd();
		let mut termios = std::mem::MaybeUninit::uninit();
		unsafe {
			libc::tcgetattr(fd, termios.as_mut_ptr());
			let original = termios.assume_init();
			let mut raw = original;
			raw.c_lflag &= !(libc::ICANON | libc::ECHO);
			libc::tcsetattr(fd, libc::TCSANOW, &raw);
			Some((fd, original))
		}
	};

	#[cfg(not(unix))]
	let original_termios: Option<(i32, ())> = None;

	let mut byte = [0u8; 1];
	let mut result = None;
	while stdin_handle.read_exact(&mut byte).is_ok() {
		match byte[0] {
			b'\n' | b'\r' => {
				println!();
				result = Some(value);
				break;
			}
			0x1b | 0x03 => {
				print!("\r\x1b[K");
				stdout.flush().unwrap();
				break;
			}
			0x7f | 0x08 =>
				if !value.is_empty() {
					value.pop();
					print!("\r\x1b[K> {value}");
					stdout.flush().unwrap();
				},
			c if c.is_ascii_graphic() || c == b' ' => {
				value.push(c as char);
				print!("\r\x1b[K> {value}");
				stdout.flush().unwrap();
			}
			_ => {}
		}
	}

	#[cfg(unix)]
	if let Some((fd, original)) = original_termios {
		unsafe {
			libc::tcsetattr(fd, libc::TCSANOW, &original);
		}
	}

	result
}
