use std::fmt::Write;

pub trait SysexitCode {
	fn sysexit(&self) -> Sysexit;
}
/// Exit codes from sysexits.h, plus a generic fallback.
///
/// `None` (1) is for errors that don't have a specific sysexit mapping —
/// use it as the catch-all in `SysexitCode` impls instead of guessing a code.
#[derive(Clone, Copy, Debug, Default)]
#[repr(i32)]
pub enum Sysexit {
	/// Generic error, no specific sysexit code applies
	#[default]
	None = 1,
	Usage = 64,
	DataErr = 65,
	NoInput = 66,
	NoUser = 67,
	NoHost = 68,
	Unavailable = 69,
	Software = 70,
	OsErr = 71,
	OsFile = 72,
	CantCreat = 73,
	IoErr = 74,
	TempFail = 75,
	Protocol = 76,
	NoPerm = 77,
	Config = 78,
}
impl From<Sysexit> for i32 {
	fn from(s: Sysexit) -> i32 {
		s as i32
	}
}

pub fn format_eyre_chain_for_user(e: eyre::Report) -> String {
	let mut s = String::new();

	fn write_chain(err: &dyn std::error::Error, s: &mut String) {
		if let Some(src) = err.source() {
			write_chain(src, s);
			s.push('\n');
			s.push_str("-> ");
			let _ = write!(s, "{err}");
		} else {
			s.push_str("\x1b[31mError\x1b[0m: ");
			let _ = write!(s, "{err}");
		}
	}

	write_chain(e.as_ref(), &mut s);
	s
}

mod sealed {
	use super::SysexitCode;

	pub trait ExitCode {
		fn exit_code(&self) -> i32;
	}
	impl<E> ExitCode for E {
		default fn exit_code(&self) -> i32 {
			7
		}
	}
	impl<E: SysexitCode> ExitCode for E {
		fn exit_code(&self) -> i32 {
			self.sysexit().into()
		}
	}
}
use sealed::ExitCode;

pub fn exit_on_error<T, E: Into<eyre::Report>>(r: Result<T, E>) -> T {
	match r {
		Ok(t) => t,
		Err(e) => {
			let code = e.exit_code();
			eprintln!("{}", format_eyre_chain_for_user(e.into()));
			std::process::exit(code);
		}
	}
}

/// Constructs `eyre::Report` with capped size
pub fn report_msg(s: String) -> eyre::Report {
	let truncated_message = truncate_msg(&s);

	eyre::Report::msg(truncated_message)
}

/// Useful for putting random potentially large things into logs without thinking
#[track_caller]
#[function_name::named]
pub fn truncate_msg<S: AsRef<str>>(s: S) -> String {
	const MAX_LINES: usize = 50;
	const CHARS_IN_A_LINE: usize = 150;
	let truncation_message = format!("\n------------------------- // truncated at {} by `{}`\n", std::panic::Location::caller(), function_name!());

	let s = s.as_ref();
	let lines: Vec<&str> = s.lines().collect();
	if lines.len() > MAX_LINES {
		let start_cut = &lines[..(MAX_LINES / 2)];
		let end_cut = &lines[lines.len() - (MAX_LINES / 2)..];
		format!("{}{truncation_message}{}", start_cut.join("\n"), end_cut.join("\n"))
	} else if s.chars().count() > MAX_LINES * CHARS_IN_A_LINE {
		let start_cut = &s[..(MAX_LINES * CHARS_IN_A_LINE / 2)];
		let end_cut = &s[s.len() - (MAX_LINES * CHARS_IN_A_LINE / 2)..];
		format!("{start_cut}{truncation_message}{end_cut}")
	} else {
		s.to_owned()
	}
}
