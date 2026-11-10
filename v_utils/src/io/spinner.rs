use std::{
	io::IsTerminal,
	sync::mpsc,
	thread::{self, JoinHandle},
	time::Duration,
};

/// Dots ticking on stderr for as long as it lives; cleared on drop. No-op when stderr is not a terminal.
///
/// Each frame leaves the cursor at column 0, so anything printed meanwhile overwrites the dots and the spinner resumes on the line below.
///```rust
///let _spinner = v_utils::io::Spinner::start();
///```
#[derive(Debug)]
pub struct Spinner {
	stop: mpsc::Sender<()>,
	ticker: Option<JoinHandle<()>>,
}

impl Spinner {
	pub fn start() -> Self {
		let (stop, stopped) = mpsc::channel();
		let ticker = std::io::stderr().is_terminal().then(|| {
			thread::spawn(move || {
				for frame in [".  ", ".. ", "...", " ..", "  .", "   "].iter().cycle() {
					eprint!("\r{frame}\r");
					if stopped.recv_timeout(Duration::from_millis(250)) != Err(mpsc::RecvTimeoutError::Timeout) {
						break;
					}
				}
				eprint!("\r   \r");
			})
		});
		Self { stop, ticker }
	}
}

impl Drop for Spinner {
	fn drop(&mut self) {
		if let Some(ticker) = self.ticker.take() {
			self.stop.send(()).expect("ticker holds the receiver until stopped");
			ticker.join().expect("ticker only prints");
		}
	}
}
