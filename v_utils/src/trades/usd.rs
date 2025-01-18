use derive_more::{
	derive::{Display, FromStr},
	Add, AddAssign, Deref, DerefMut, Div, DivAssign, From, Into, Mul, MulAssign, Neg, Sub, SubAssign,
};
use serde::{Deserialize, Serialize};

#[derive(
	Clone,
	Debug,
	Default,
	derive_new::new,
	Copy,
	PartialEq,
	PartialOrd,
	Deref,
	DerefMut,
	Add,
	AddAssign,
	Sub,
	SubAssign,
	Mul,
	MulAssign,
	Div,
	DivAssign,
	Neg,
	From,
	Into,
	Display,
	FromStr,
	Serialize,
	Deserialize,
)]
#[mul(forward)]
#[div(forward)]
/// A struct representing USD (in future, inflation-adjusted) value. That's it. Just a newtype. But extremely powerful.
pub struct Usd(pub f64);

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
}
