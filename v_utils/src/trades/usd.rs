use serde::{Deserialize, Serialize};

#[derive(
	Clone,
	Debug,
	Default,
	derive_new::new,
	Copy,
	PartialEq,
	PartialOrd,
	derive_more::Deref,
	derive_more::DerefMut,
	derive_more::Add,
	derive_more::AddAssign,
	derive_more::Sub,
	derive_more::SubAssign,
	derive_more::Mul,
	derive_more::MulAssign,
	derive_more::Div,
	derive_more::DivAssign,
	derive_more::Neg,
	derive_more::From,
	derive_more::Into,
	derive_more::FromStr,
	Serialize,
	Deserialize,
)]
#[mul(forward)]
#[div(forward)]
/// A struct representing USD (in future, inflation-adjusted) value. That's it. Just a newtype. But extremely powerful.
pub struct Usd(pub f64);

impl std::ops::Mul<f64> for Usd {
	type Output = Self;

	fn mul(self, rhs: f64) -> Self::Output {
		Self(self.0 * rhs)
	}
}

impl std::fmt::Display for Usd {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let s = match f.precision() {
			Some(p) => format!("{:.*}", p, self.0),
			None =>
				if self.0.fract() != 0. {
					format!("{:.2}", self.0)
				} else {
					format!("{}", self.0)
				},
		};

		crate::fmt_with_width!(f, s)
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn operators() {
		use super::Usd;
		let usd = Usd(1.0);
		assert_eq!(usd, Usd(1.0));
		assert_eq!(usd + Usd(1.0), Usd(2.0));
		assert_eq!(usd - Usd(1.0), Usd(0.0));
		assert_eq!(usd * Usd(2.0), Usd(2.0));
		assert_eq!(usd / Usd(2.0), Usd(0.5));
		assert_eq!(-usd, Usd(-1.0));
	}

	#[test]
	fn precision_and_alignment() {
		use super::Usd;
		let usd = Usd(1.0);
		assert_eq!(format!("{}", usd), "1");
		assert_eq!(format!("{:.2}", usd), "1.00");
		assert_eq!(format!("|{:10}|", usd), "|1         |");
		assert_eq!(format!("|{:^10.2}|", usd), "|   1.00   |");
	}
}
