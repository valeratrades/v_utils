use std::{
	ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
	str::FromStr,
};

use eyre::{Error, Result};
use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Copy, Clone, Debug, Default, derive_new::new, PartialEq)]
pub struct Percent(pub f64);
impl Percent {
	pub fn inner(self) -> f64 {
		self.0
	}
}

impl<'de> Deserialize<'de> for Percent {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>, {
		struct PercentVisitor;

		impl<'de> de::Visitor<'de> for PercentVisitor {
			type Value = Percent;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a float, an integer, or a string representing a percentage")
			}

			fn visit_f64<E>(self, value: f64) -> Result<Percent, E>
			where
				E: de::Error, {
				Ok(Percent(value))
			}

			fn visit_u64<E>(self, value: u64) -> Result<Percent, E>
			where
				E: de::Error, {
				Ok(Percent(value as f64 / 100.0))
			}

			fn visit_str<E>(self, value: &str) -> Result<Percent, E>
			where
				E: de::Error, {
				Percent::from_str(value).map_err(de::Error::custom)
			}
		}

		deserializer.deserialize_any(PercentVisitor)
	}
}
impl FromStr for Percent {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		let stripped = s.trim_end_matches("%");

		let percent = if let Ok(u) = stripped.parse::<isize>() {
			u as f64 / 100.
		} else if let Ok(f) = stripped.parse::<f64>() {
			match s.ends_with("%") {
				true => f / 100.,
				false => f,
			}
		} else {
			return Err(Error::msg("Failed to parse \"{s}\" to percent"));
		};

		Ok(Percent(percent))
	}
}

//? still not sure if I like `"xx%"` other the default derive (that is `0.xx`)
impl Serialize for Percent {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer, {
		let percent_number = self.0 * 100.;
		let s = match percent_number.fract() == 0. {
			true => format!("{}%", percent_number as isize),
			false => format!("{}%", percent_number),
		};
		s.serialize(serializer)
	}
}

impl std::fmt::Display for Percent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let percent_number = self.0 * 100.;
		match percent_number.fract() == 0. {
			true => write!(f, "{}%", percent_number as isize),
			false => write!(f, "{}%", percent_number),
		}
	}
}

impl PartialEq<f64> for Percent {
	fn eq(&self, other: &f64) -> bool {
		self.0 == *other
	}
}
impl PartialOrd<f64> for Percent {
	fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(other)
	}
}
impl std::ops::Deref for Percent {
	type Target = f64;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl From<f64> for Percent {
	fn from(f: f64) -> Self {
		Percent(f)
	}
}
impl From<f32> for Percent {
	fn from(f: f32) -> Self {
		Percent(f as f64)
	}
}
impl From<isize> for Percent {
	fn from(i: isize) -> Self {
		Percent(i as f64 / 100.)
	}
}
impl From<usize> for Percent {
	fn from(i: usize) -> Self {
		Percent(i as f64 / 100.)
	}
}
impl From<i32> for Percent {
	fn from(i: i32) -> Self {
		Percent(i as f64 / 100.)
	}
}
impl From<i64> for Percent {
	fn from(i: i64) -> Self {
		Percent(i as f64 / 100.)
	}
}
impl From<u32> for Percent {
	fn from(i: u32) -> Self {
		Percent(i as f64 / 100.)
	}
}
impl From<u64> for Percent {
	fn from(i: u64) -> Self {
		Percent(i as f64 / 100.)
	}
}

// Operators {{{
// // Add
impl Add for Percent {
	type Output = Percent;

	fn add(self, other: Percent) -> Percent {
		Percent(self.0 + other.0)
	}
}

impl Add<f64> for Percent {
	type Output = Percent;

	fn add(self, other: f64) -> Percent {
		Percent(self.0 + other)
	}
}

impl AddAssign for Percent {
	fn add_assign(&mut self, other: Percent) {
		self.0 += other.0;
	}
}

impl AddAssign<f64> for Percent {
	fn add_assign(&mut self, other: f64) {
		self.0 += other;
	}
}
//

// // Mul
impl Mul for Percent {
	type Output = Percent;

	fn mul(self, other: Percent) -> Percent {
		Percent(self.0 * other.0)
	}
}

impl Mul<f64> for Percent {
	type Output = Percent;

	fn mul(self, other: f64) -> Percent {
		Percent(self.0 * other)
	}
}

impl MulAssign for Percent {
	fn mul_assign(&mut self, other: Percent) {
		self.0 *= other.0;
	}
}

impl MulAssign<f64> for Percent {
	fn mul_assign(&mut self, other: f64) {
		self.0 *= other;
	}
}
//

// // Sub
impl Sub for Percent {
	type Output = Percent;

	fn sub(self, other: Percent) -> Percent {
		Percent(self.0 - other.0)
	}
}

impl Sub<f64> for Percent {
	type Output = Percent;

	fn sub(self, other: f64) -> Percent {
		Percent(self.0 - other)
	}
}

impl SubAssign for Percent {
	fn sub_assign(&mut self, other: Percent) {
		self.0 -= other.0;
	}
}

impl SubAssign<f64> for Percent {
	fn sub_assign(&mut self, other: f64) {
		self.0 -= other;
	}
}
//

// // Div
impl Div for Percent {
	type Output = Percent;

	fn div(self, other: Percent) -> Percent {
		Percent(self.0 / other.0)
	}
}

impl Div<f64> for Percent {
	type Output = Percent;

	fn div(self, other: f64) -> Percent {
		Percent(self.0 / other)
	}
}

impl DivAssign for Percent {
	fn div_assign(&mut self, other: Percent) {
		self.0 /= other.0;
	}
}

impl DivAssign<f64> for Percent {
	fn div_assign(&mut self, other: f64) {
		self.0 /= other;
	}
}
//

// // Rem
impl Rem for Percent {
	type Output = Percent;

	fn rem(self, other: Percent) -> Percent {
		Percent(self.0 % other.0)
	}
}

impl Rem<f64> for Percent {
	type Output = Percent;

	fn rem(self, other: f64) -> Percent {
		Percent(self.0 % other)
	}
}

impl Rem<Percent> for f64 {
	type Output = f64;

	fn rem(self, other: Percent) -> f64 {
		self % other.0
	}
}

impl RemAssign for Percent {
	fn rem_assign(&mut self, other: Percent) {
		self.0 %= other.0;
	}
}

impl RemAssign<f64> for Percent {
	fn rem_assign(&mut self, other: f64) {
		self.0 %= other;
	}
}
//

// // Neg
impl Neg for Percent {
	type Output = Percent;

	fn neg(self) -> Percent {
		Percent(-self.0)
	}
}
//

//,}}}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn isize() {
		let p = Percent::from_str("-50").unwrap();
		assert_eq!(p.0, -0.5);

		let p = Percent::from_str("50%").unwrap();
		assert_eq!(p.0, 0.5);
	}

	#[test]
	fn float() {
		let p = Percent::from_str("0.5").unwrap();
		assert_eq!(p.0, 0.5);

		let p = Percent::from_str("0.5%").unwrap();
		assert_eq!(p.0, 0.005);
	}

	#[test]
	fn json() {
		let float_json = r#"0.5"#;
		let p: Percent = serde_json::from_str(float_json).unwrap();
		assert_eq!(p.0, 0.5);

		let isize_json = r#"50"#;
		let p: Percent = serde_json::from_str(isize_json).unwrap();
		assert_eq!(p.0, 0.5);

		let string_json = r#""50%""#;
		let p: Percent = serde_json::from_str(string_json).unwrap();
		assert_eq!(p.0, 0.5);
	}

	#[test]
	fn compare() {
		let p = Percent::from_str("50%").unwrap();
		assert!(p < 0.51);
		assert_eq!(p, 0.5);
		assert!(p > 0.49);
	}
}
