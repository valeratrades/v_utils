use std::time::Instant;

use bon::Builder;

use crate::{other::Timelike, print_rolling};

/// Single-line terminal progress bar using `print_rolling!`.
///
///```rust
///use v_utils::io::ProgressBar;
///
///let mut pb = ProgressBar::builder().total(100).build();
///for i in 0..=100 {
///    pb.progress(i);
///}
///```
#[derive(Builder, Clone, Debug)]
pub struct ProgressBar {
	total: usize,
	/// Bar width in characters (default: 40)
	#[builder(default = 40)]
	width: usize,
	/// Fill character (default: '█')
	#[builder(default = '█')]
	fill: char,
	/// Empty character (default: '░')
	#[builder(default = '░')]
	empty: char,
	/// Optional prefix shown before the bar
	#[builder(default)]
	prefix: String,
	#[builder(skip)]
	started: Option<Instant>,
}

impl ProgressBar {
	pub fn new(total: usize) -> Self {
		Self::builder().total(total).build()
	}

	pub fn progress(&mut self, i: usize) {
		let started = *self.started.get_or_insert_with(Instant::now);
		let ratio = if self.total == 0 { 1.0 } else { (i as f64 / self.total as f64).min(1.0) };
		let filled = (ratio * self.width as f64) as usize;
		let empty = self.width - filled;
		let pct = (ratio * 100.0) as u32;

		let eta = if i > 0 {
			let elapsed = started.elapsed().as_secs_f64();
			let remaining = elapsed * (self.total.saturating_sub(i)) as f64 / i as f64;
			format!(" ETA {}", Timelike(remaining.ceil() as u32))
		} else {
			String::new()
		};

		let prefix = if self.prefix.is_empty() { String::new() } else { format!("{} ", self.prefix) };

		print_rolling!(
			"{prefix}▕{}{}▏ {pct}%{eta}",
			str::repeat(&self.fill.to_string(), filled),
			str::repeat(&self.empty.to_string(), empty)
		);

		if i >= self.total {
			eprintln!();
		}
	}
}
