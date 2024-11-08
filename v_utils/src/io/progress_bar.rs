use std::time::{SystemTime, UNIX_EPOCH};

///```rust
///use v_utils::io::ProgressBar;
///
///let mut pb = ProgressBar::new(100);
///for i in 0..100 {
///    pb.progress(i);
///}
///
#[derive(Clone, Debug)]
pub struct ProgressBar {
	bar_width: f64,
	timestamp_ms: u128,
	total: f64,
}
impl ProgressBar {
	pub fn new(total: usize) -> Self {
		let bar_width: f64 = 133.0;
		let timestamp_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
		let total = total as f64;
		ProgressBar { bar_width, timestamp_ms, total }
	}

	pub fn progress(&mut self, i: usize) {
		const CLEAR: &str = "\x1B[2J\x1B[1;1H";
		let scalar: f64 = self.bar_width / self.total;
		let display_i = (i as f64 * scalar) as usize;
		let display_total = (self.total * scalar) as usize;

		println!("{}", CLEAR);
		println!("[{}{}]", "*".repeat(display_i), " ".repeat(display_total - display_i));

		let since_timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() - self.timestamp_ms;
		let progress_left_scalar = (self.total - i as f64) / i as f64;
		let left_s = (since_timestamp_ms as f64 * progress_left_scalar / 1000.0) as usize;
		println!("Time left: ≈ {}s", left_s);
	}
}
