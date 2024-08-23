use eyre::{Error, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Default, derive_new::new, PartialEq)]
pub struct Percent(pub f64);
impl<'de> Deserialize<'de> for Percent {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct PercentVisitor;

		impl<'de> de::Visitor<'de> for PercentVisitor {
			type Value = Percent;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a float, an integer, or a string representing a percentage")
			}

			fn visit_f64<E>(self, value: f64) -> Result<Percent, E>
			where
				E: de::Error,
			{
				Ok(Percent(value))
			}

			fn visit_u64<E>(self, value: u64) -> Result<Percent, E>
			where
				E: de::Error,
			{
				Ok(Percent(value as f64 / 100.0))
			}

			fn visit_str<E>(self, value: &str) -> Result<Percent, E>
			where
				E: de::Error,
			{
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
		S: serde::Serializer,
	{
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
		write!(f, "{}", self.0)
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
impl From<Percent> for f64 {
	fn from(percent: Percent) -> f64 {
		percent.0
	}
}

impl Percent {
	pub fn inner(self) -> f64 {
		self.0
	}
}

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
