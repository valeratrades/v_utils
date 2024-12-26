pub fn format_eyre_chain_for_user(e: eyre::Report) -> String {
	let chain = e.chain().rev().collect::<Vec<_>>();
	let mut s = String::new();
	for (i, e) in chain.into_iter().enumerate() {
		if i > 0 {
			s.push('\n');
		}
		s.push_str("-> ");
		s.push_str(&e.to_string());
	}
	s
}

/// Constructs `eyre::Report` with capped size
pub fn report_msg(s: String) -> eyre::Report {
	let truncated_message = truncate_msg(&s);

	eyre::Report::msg(truncated_message)
}

/// Useful for putting random potentially large things into logs without thinking
#[track_caller]
#[function_name::named]
pub fn truncate_msg(s: &str) -> String {
	let lines: Vec<&str> = s.lines().collect();
	let total_lines = lines.len();

	if total_lines > 50 {
		let first_25 = &lines[..25];
		let last_25 = &lines[total_lines - 25..];
		let truncation_message = format!("------------------------- // truncated at {} by `{}`\n", std::panic::Location::caller(), function_name!());
		let concat_message = format!("{}\n{truncation_message}{}", first_25.join("\n"), last_25.join("\n"));

		concat_message
	} else {
		s.to_owned()
	}
}
