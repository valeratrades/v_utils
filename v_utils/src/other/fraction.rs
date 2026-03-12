use std::str::FromStr;

use derive_more::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, From, Into, Mul, MulAssign, Neg, Sub, SubAssign};
use eyre::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize, de};

use crate::utils;

/// A fraction newtype over f64. Stores the value as a plain ratio (numerator/denominator).
///
/// Parses from:
/// - `"3/1"` -> 3.0
/// - `"1/3"` -> 0.333...
/// - `"2.5"` -> 2.5
/// - `"0.5"` -> 0.5
#[derive(Add, AddAssign, Clone, Copy, Debug, Default, Deref, DerefMut, Div, DivAssign, From, Into, Mul, MulAssign, Neg, PartialEq, PartialOrd, Sub, SubAssign, derive_new::new)]
#[mul(forward)]
#[div(forward)]
pub struct Fraction(pub f64);

impl FromStr for Fraction {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		if let Some((num, den)) = s.split_once('/') {
			let num: f64 = num.trim().parse().map_err(|_| eyre::eyre!("invalid numerator in \"{s}\""))?;
			let den: f64 = den.trim().parse().map_err(|_| eyre::eyre!("invalid denominator in \"{s}\""))?;
			if den == 0.0 {
				bail!("denominator cannot be zero in \"{s}\"");
			}
			Ok(Fraction(num / den))
		} else {
			let v: f64 = s.parse().map_err(|_| eyre::eyre!("failed to parse \"{s}\" as fraction"))?;
			Ok(Fraction(v))
		}
	}
}

impl std::fmt::Display for Fraction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match f.precision() {
			Some(p) => format!("{:.*}", p, self.0),
			None => utils::format_significant_digits(self.0, 3),
		};

		if let Some(w) = f.width() {
			match f.align() {
				Some(std::fmt::Alignment::Left) => write!(f, "{:<width$}", s, width = w),
				Some(std::fmt::Alignment::Right) => write!(f, "{:>width$}", s, width = w),
				Some(std::fmt::Alignment::Center) => write!(f, "{:^width$}", s, width = w),
				None => write!(f, "{:width$}", s, width = w),
			}
		} else {
			write!(f, "{s}")
		}
	}
}

impl Serialize for Fraction {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer, {
		self.0.serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for Fraction {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>, {
		struct FractionVisitor;

		impl de::Visitor<'_> for FractionVisitor {
			type Value = Fraction;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a float, an integer, or a string fraction like '3/1'")
			}

			fn visit_f64<E>(self, value: f64) -> Result<Fraction, E>
			where
				E: de::Error, {
				Ok(Fraction(value))
			}

			fn visit_u64<E>(self, value: u64) -> Result<Fraction, E>
			where
				E: de::Error, {
				Ok(Fraction(value as f64))
			}

			fn visit_i64<E>(self, value: i64) -> Result<Fraction, E>
			where
				E: de::Error, {
				Ok(Fraction(value as f64))
			}

			fn visit_str<E>(self, value: &str) -> Result<Fraction, E>
			where
				E: de::Error, {
				Fraction::from_str(value).map_err(de::Error::custom)
			}
		}

		deserializer.deserialize_any(FractionVisitor)
	}
}

impl PartialEq<f64> for Fraction {
	fn eq(&self, other: &f64) -> bool {
		self.0 == *other
	}
}
impl PartialOrd<f64> for Fraction {
	fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(other)
	}
}

impl From<f32> for Fraction {
	fn from(f: f32) -> Self {
		Fraction(f as f64)
	}
}
impl From<i32> for Fraction {
	fn from(i: i32) -> Self {
		Fraction(i as f64)
	}
}
impl From<i64> for Fraction {
	fn from(i: i64) -> Self {
		Fraction(i as f64)
	}
}
impl From<u32> for Fraction {
	fn from(i: u32) -> Self {
		Fraction(i as f64)
	}
}
impl From<u64> for Fraction {
	fn from(i: u64) -> Self {
		Fraction(i as f64)
	}
}
impl From<isize> for Fraction {
	fn from(i: isize) -> Self {
		Fraction(i as f64)
	}
}
impl From<usize> for Fraction {
	fn from(i: usize) -> Self {
		Fraction(i as f64)
	}
}
impl From<&str> for Fraction {
	fn from(s: &str) -> Self {
		Fraction::from_str(s).unwrap()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_fraction_form() {
		let f = Fraction::from_str("3/1").unwrap();
		assert_eq!(f.0, 3.0);

		let f = Fraction::from_str("1/3").unwrap();
		assert!((f.0 - 1.0 / 3.0).abs() < f64::EPSILON);
	}

	#[test]
	fn parse_plain_float() {
		let f = Fraction::from_str("2.5").unwrap();
		assert_eq!(f.0, 2.5);
	}

	#[test]
	fn parse_plain_int() {
		let f = Fraction::from_str("3").unwrap();
		assert_eq!(f.0, 3.0);
	}

	#[test]
	fn zero_denominator() {
		assert!(Fraction::from_str("1/0").is_err());
	}

	#[test]
	fn json_float() {
		let f: Fraction = serde_json::from_str("2.5").unwrap();
		assert_eq!(f.0, 2.5);
	}

	#[test]
	fn json_int() {
		let f: Fraction = serde_json::from_str("3").unwrap();
		assert_eq!(f.0, 3.0);
	}

	#[test]
	fn json_string() {
		let f: Fraction = serde_json::from_str(r#""3/1""#).unwrap();
		assert_eq!(f.0, 3.0);
	}

	#[test]
	fn operators() {
		let a = Fraction::from_str("3/1").unwrap();
		let b = Fraction::from_str("1/1").unwrap();
		assert_eq!(a + b, Fraction(4.0));
		assert_eq!(a - b, Fraction(2.0));
		assert_eq!(a * b, Fraction(3.0));
		assert_eq!(a / b, Fraction(3.0));
	}

	#[test]
	fn compare_f64() {
		let f = Fraction::from_str("2.5").unwrap();
		assert!(f > 2.0);
		assert_eq!(f, 2.5);
		assert!(f < 3.0);
	}

	#[test]
	fn negative() {
		let f = Fraction::from_str("-3/2").unwrap();
		assert_eq!(f.0, -1.5);
	}
}
