use crate::Percent;

#[derive(Debug, Clone, Default, Copy, serde::Deserialize, serde::Serialize)]
pub struct NowThen {
	pub now: f64,
	pub then: f64,
	pub duration: Option<std::time::Duration>,
}
impl NowThen {
	pub fn new(now: f64, then: f64) -> Self {
		Self { now, then, duration: None }
	}

	pub fn from_now_diff(now: f64, diff: f64) -> Self {
		Self::new(now, now + diff)
	}

	pub fn add_duration(mut self, duration: std::time::Duration) -> Self {
		self.duration = Some(duration);
		self
	}
}

impl std::fmt::Display for NowThen {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let diff = self.now - self.then;

		let (now_f, now_suffix) = format_number_compactly(self.now, 0.03);
		let (diff_f, diff_suffix) = format_number_compactly(diff, 0.005);
		let now_suffix = if now_suffix == diff_suffix { "" } else { now_suffix };

		let diff_str = format!("{:+}{}", diff_f, diff_suffix);
		let now_str = format!("{}{}", now_f, now_suffix);
		let s = format!("{}{}", now_str, diff_str);

		crate::fmt_with_width!(f, s)
	}
}

impl std::fmt::LowerExp for NowThen {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let diff = Percent::from((self.now - self.then) / self.then);
		write!(f, "{:e}{:+}", self.now, diff)
	}
}

fn format_number_compactly(mut n: f64, precision: f64) -> (f64, &'static str) {
	assert!(precision >= 0.0, "Precision can't be negative, the hell? {:?}", precision);
	let mut thousands = 0;
	while n.abs() >= 1000.0 {
		n /= 1000.0;
		thousands += 1;
	}

	let sure_n_digits = precision.log(0.1).ceil() as usize + 1;
	let mut n_str = {
		let mut temp_str = "".to_string();
		let mut countdown = sure_n_digits + 2; // the whole block is to cut out what we definitely can cut out, so might as well have a buffer
		for c in n.to_string().chars() {
			temp_str.push(c);
			if c != '.' {
				countdown -= 1;
			}
			if countdown == 0 {
				break;
			}
		}
		temp_str
	};

	// format, then subtract one, and try format again; if within precision from original, commit.
	loop {
		if !n_str.contains('.') {
			break;
		}
		let n_precision = n_str.split('.').last().unwrap().len();
		let try_round_one_more = format!("{:.*}", n_precision - 1, n);
		if ((n - try_round_one_more.parse::<f64>().unwrap()) / n).abs() > precision {
			break;
		} else {
			n_str = try_round_one_more;
		}
	}
	let mut n = n_str.parse::<f64>().unwrap();

	if n.abs() >= 1000.0 {
		n /= 1000.0;
		thousands += 1;
	}

	fn suffix_from_n_thousands(n: usize) -> &'static str {
		match n {
			0 => "",
			1 => "K",
			2 => "M",
			3 => "B",
			4 => "T",
			5 => "Q",
			_ => panic!("Number is too large, calm down"),
		}
	}
	(n, suffix_from_n_thousands(thousands))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn display_1() {
		let nt = NowThen::new(69420.0, 67000.0);
		insta::assert_snapshot!(nt.to_string(), @"69+2.42K");

		let nt = NowThen::new(0.517563, 0.498);
		insta::assert_snapshot!(nt.to_string(), @"0.52+0.0196");

		let nt = NowThen::new(0.527563, 0.498);
		insta::assert_snapshot!(nt.to_string(), @"0.53+0.0296");
	}

	#[test]
	fn lower_exp() {
		let nt = NowThen::new(69420.0, 67000.0);
		insta::assert_snapshot!(format!("{:e}", nt), @"6.942e4+3.6%");
	}
}
