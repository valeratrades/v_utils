use std::str::FromStr;

use derive_more::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, From, Into, Mul, MulAssign, Neg, Sub, SubAssign};
use eyre::{eyre, Result};
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::utils;

#[derive(Clone, Debug, Default, derive_new::new, Copy, PartialEq, PartialOrd, Deref, DerefMut, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Neg, From, Into)]
#[mul(forward)]
#[div(forward)]
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

		impl de::Visitor<'_> for PercentVisitor {
			type Value = Percent;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a float, an integer, a string percentage, or '<number>x' format")
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
				if let Some(x_val) = value.strip_suffix('x') {
					return match x_val.parse::<f64>() {
						Ok(n) => Ok(Percent(n)),
						Err(_) => Err(de::Error::custom(format!("Invalid 'x' format: {value}"))),
					};
				}
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
			return Err(eyre!("Failed to parse \"{s}\" to percent"));
		};

		Ok(Percent(percent))
	}
}

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
		let mut s = match percent_number.fract() == 0. {
			true => format!("{}%", percent_number as isize),
			false => {
				let num_string = match f.precision() {
					Some(p) => format!("{:.*}", p, percent_number),
					None => utils::format_significant_digits(percent_number, 2),
				};
				format!("{}%", num_string)
			}
		};
		if f.sign_plus() {
			let sign = if self.0 >= 0. { "+" } else { "" };
			s = format!("{}{}", sign, s);
		}

		// these ones are default
		if f.fill() != ' ' && f.fill() != '\0' {
			unimplemented!("Specifying fill is not supported. Rust is letting us down, impossible to implement, call `to_string()` and use its implementation.");
		}
		if let Some(w) = f.width() {
			match f.align() {
				Some(std::fmt::Alignment::Left) => write!(f, "{:<width$}", s, width = w),
				Some(std::fmt::Alignment::Right) => write!(f, "{:>width$}", s, width = w),
				Some(std::fmt::Alignment::Center) => write!(f, "{:^width$}", s, width = w),
				None => write!(f, "{:width$}", s, width = w),
			}
		} else {
			write!(f, "{}", s)
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

// Froms {{{
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
impl From<&str> for Percent {
	fn from(s: &str) -> Self {
		Percent::from_str(s).unwrap()
	}
}
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

	#[test]
	fn allows_above_100() {
		let p = Percent::from_str("150%").unwrap();
		assert_eq!(p.0, 1.5);
	}

	#[test]
	fn allows_negative() {
		let p = Percent::from_str("-50%").unwrap();
		assert_eq!(p.0, -0.5);
	}

	#[test]
	fn x_format() {
		let json = r#""1.5x""#;
		let p: Percent = serde_json::from_str(json).unwrap();
		assert_eq!(p.0, 1.5);

		let json = r#""0.5x""#;
		let p: Percent = serde_json::from_str(json).unwrap();
		assert_eq!(p.0, 0.5);
	}

	#[test]
	fn operators() {
		let p = Percent::from_str("50%").unwrap();
		let p2 = Percent::from_str("50%").unwrap();
		assert_eq!(p + p2, Percent::from_str("100%").unwrap());
		assert_eq!(p - p2, Percent::from_str("0%").unwrap());
		assert_eq!(p * p2, Percent::from_str("25%").unwrap());
		assert_eq!(p / p2, Percent::from_str("100%").unwrap());
	}

	#[test]
	fn precision_and_alignment() {
		let p = Percent::from_str("0.123456").unwrap();
		assert_eq!(format!("{:.2}", p), "12.35%");
		assert_eq!(format!("{:.0}", p), "12%");

		//TODO!:
		assert_eq!(format!("|{:<10}|", p), "|12.3456%  |");
		assert_eq!(format!("|{:^15}|", p), "|   12.3456%    |");
	}
}
