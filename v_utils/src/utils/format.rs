#[macro_export]
macro_rules! fmt_with_width {
	($f:expr, $s:expr) => {{
		if $f.fill() != ' ' && $f.fill() != '\0' {
			unimplemented!("Specifying fill is not supported. Rust is letting us down, impossible to implement, call `to_string()` and use its implementation.");
		}
		if let Some(w) = $f.width() {
			match $f.align() {
				Some(std::fmt::Alignment::Left) => write!($f, "{:<width$}", $s, width = w),
				Some(std::fmt::Alignment::Right) => write!($f, "{:>width$}", $s, width = w),
				Some(std::fmt::Alignment::Center) => write!($f, "{:^width$}", $s, width = w),
				_ => write!($f, "{:width$}", $s, width = w),
			}
		} else {
			write!($f, "{}", $s)
		}
	}};
}

/// formats _up to_ specified number of significant digits
///```rust
/// use v_utils::utils::format_significant_digits;
/// assert_eq!(format_significant_digits(0.000123456789, 3), "0.000123");
///```
pub fn format_significant_digits(n: f64, sig_digits: usize) -> String {
	if n == 0.0 {
		return "0".to_string();
	}

	let full = format!("{:.12}", n.abs());
	if !full.contains('.') {
		return format!("{n:.1}");
	}

	let first_sig = full.chars().position(|c| c != '0' && c != '.').unwrap_or(0);

	let mut sig_count = 0;
	let mut last_pos = full.len();
	for (i, c) in full.chars().enumerate().skip(first_sig) {
		if c != '.' {
			sig_count += 1;
			if sig_count == sig_digits {
				last_pos = i + 1;
				break;
			}
		}
	}

	let precision = full[..last_pos].split('.').nth(1).unwrap_or("").len();
	format!("{:.*}", precision, n)
}
