use std::fmt::Write;

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

pub fn exit_on_error<T, E: Into<eyre::Report>>(r: Result<T, E>) -> T {
	match r {
		Ok(t) => t,
		Err(e) => {
			println!("{}", format_eyre_chain_for_user(e.into()));
			std::process::exit(7);
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
		format!("{}{truncation_message}{}", start_cut, end_cut)
	} else {
		s.to_owned()
	}
}
