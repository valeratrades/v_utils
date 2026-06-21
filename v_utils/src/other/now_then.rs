use crate::{LargeNumber, Percent};

#[derive(bon::Builder, Clone, Copy, Debug, Default, serde::Deserialize, PartialEq, serde::Serialize)]
pub struct NowThen {
	#[builder(into)]
	pub now: LargeNumber,
	#[builder(into)]
	pub then: LargeNumber,
	pub duration: Option<std::time::Duration>,
}
impl NowThen {
	pub fn new(now: f64, then: f64) -> Self {
		Self {
			now: LargeNumber::new(now),
			then: LargeNumber::new(then),
			duration: None,
		}
	}

	pub fn from_now_diff(now: f64, diff: f64) -> Self {
		Self::new(now, now + diff)
	}
}

impl std::fmt::Display for NowThen {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let diff = LargeNumber::with_precision(self.now.value - self.then.value, 0.005);

		// the first number's suffix is redundant when it shares the diff's magnitude
		let now_str = self.now.format(self.now.magnitude() == diff.magnitude());
		let sign = if diff.value >= 0.0 { "+" } else { "" };
		let s = format!("{now_str}{sign}{}", diff.format(false));

		crate::fmt_with_width!(f, s)
	}
}

impl std::fmt::LowerExp for NowThen {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let diff = Percent::from((self.now.value - self.then.value) / self.then.value);
		write!(f, "{:e}{diff:+}", self.now.value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn display_1() {
		let s = format!(
			"{nt1}\n{nt2}\n{nt3}",
			nt1 = NowThen::new(69420.0, 67000.0),
			nt2 = NowThen::new(0.517563, 0.498),
			nt3 = NowThen::new(0.527563, 0.498),
		);
		insta::assert_snapshot!(s, @"
		69+2.42K
		0.52+0.0196
		0.53+0.0296
		");
	}

	#[test]
	fn lower_exp() {
		let nt = NowThen::new(69420.0, 67000.0);
		insta::assert_snapshot!(format!("{:e}", nt), @"6.942e4+3.6%");
	}
}
