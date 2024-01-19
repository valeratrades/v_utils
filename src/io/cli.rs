use std::io::{self, Write};

/// Confirm with user before proceeding.
///```rust
///use v_utils::io::confirm;
///if confirm("Gonna open a new 12.147047$ SELL order on ADAUSDT") {
///		println!("Opening order...");
///}
///```
pub fn confirm(message: &str) -> bool {
	let stdin = io::stdin();
	let mut stdout = io::stdout();

	print!("{}. Proceed? [Y/n] ", message);
	stdout.flush().unwrap();

	let mut input = String::new();
	stdin.read_line(&mut input).expect("Failed to read line");

	let input = input.trim().to_lowercase();
	input == "y" || input == "yes"
}
