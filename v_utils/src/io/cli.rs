use std::io::{self, Write};

/// Confirm with user before proceeding.
///```rust
///use v_utils::io::confirm;
///if confirm("Gonna open a new 12.147047$ SELL order on ADAUSDT") {
///		println!("Opening order...");
///}
///```
//? abort after 30s without response?
#[must_use]
pub fn confirm<T: AsRef<str>>(message: T) -> bool {
	let stdin = io::stdin();
	let mut stdout = io::stdout();

	print!("{}. Proceed? [Y/n] ", message.as_ref());
	stdout.flush().unwrap();

	let mut input = String::new();
	stdin.read_line(&mut input).expect("Failed to read line");

	let input = input.trim().to_lowercase();
	if input == "y" || input == "yes" {
		true
	} else {
		eprintln!("Aborted by user.");
		false
	}
}
