use std::io::{self, Write};

#[cfg(feature = "break-wasm")]
use tokio::io::{AsyncBufReadExt, BufReader};

/// Result of a confirm_all prompt
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmAllResult {
	Yes,
	No,
	All,
}

/// Confirm with user before proceeding (blocking version).
///```rust
///use v_utils::io::confirm_blocking;
///if confirm_blocking("Gonna open a new 12.147047$ SELL order on ADAUSDT") {
///		println!("Opening order...");
///}
///```
//? abort after 30s without response?
#[must_use]
pub fn confirm_blocking<T: AsRef<str>>(message: T) -> bool {
	let stdin = io::stdin();
	let mut stdout = io::stdout();

	print!("{} [Y/n] ", message.as_ref());
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

/// Confirm with user before proceeding (async version).
///```rust,ignore
///use v_utils::io::confirm;
///if confirm("Gonna open a new 12.147047$ SELL order on ADAUSDT").await {
///		println!("Opening order...");
///}
///```
#[cfg(feature = "break-wasm")]
pub async fn confirm<T: AsRef<str>>(message: T) -> bool {
	let mut stdout = io::stdout();

	print!("{} [Y/n] ", message.as_ref());
	stdout.flush().unwrap();

	let stdin = tokio::io::stdin();
	let mut reader = BufReader::new(stdin);
	let mut input = String::new();
	reader.read_line(&mut input).await.expect("Failed to read line");

	let input = input.trim().to_lowercase();
	if input == "y" || input == "yes" {
		true
	} else {
		eprintln!("Aborted by user.");
		false
	}
}

/// Confirm with user before proceeding, with an [A]ll option (blocking version).
///```rust
///use v_utils::io::{confirm_all_blocking, ConfirmAllResult};
///match confirm_all_blocking("Process this file?") {
///		ConfirmAllResult::Yes => println!("Processing..."),
///		ConfirmAllResult::No => println!("Skipping..."),
///		ConfirmAllResult::All => println!("Processing all remaining..."),
///}
///```
#[must_use]
pub fn confirm_all_blocking<T: AsRef<str>>(message: T) -> ConfirmAllResult {
	let stdin = io::stdin();
	let mut stdout = io::stdout();

	print!("{} [Y/n/a] ", message.as_ref());
	stdout.flush().unwrap();

	let mut input = String::new();
	stdin.read_line(&mut input).expect("Failed to read line");

	let input = input.trim().to_lowercase();
	match input.as_str() {
		"y" | "yes" => ConfirmAllResult::Yes,
		"a" | "all" => ConfirmAllResult::All,
		_ => {
			eprintln!("Aborted by user.");
			ConfirmAllResult::No
		}
	}
}

/// Confirm with user before proceeding, with an [A]ll option (async version).
///```rust,ignore
///use v_utils::io::{confirm_all, ConfirmAllResult};
///match confirm_all("Process this file?").await {
///		ConfirmAllResult::Yes => println!("Processing..."),
///		ConfirmAllResult::No => println!("Skipping..."),
///		ConfirmAllResult::All => println!("Processing all remaining..."),
///}
///```
#[cfg(feature = "break-wasm")]
pub async fn confirm_all<T: AsRef<str>>(message: T) -> ConfirmAllResult {
	let mut stdout = io::stdout();

	print!("{} [Y/n/a] ", message.as_ref());
	stdout.flush().unwrap();

	let stdin = tokio::io::stdin();
	let mut reader = BufReader::new(stdin);
	let mut input = String::new();
	reader.read_line(&mut input).await.expect("Failed to read line");

	let input = input.trim().to_lowercase();
	match input.as_str() {
		"y" | "yes" => ConfirmAllResult::Yes,
		"a" | "all" => ConfirmAllResult::All,
		_ => {
			eprintln!("Aborted by user.");
			ConfirmAllResult::No
		}
	}
}
