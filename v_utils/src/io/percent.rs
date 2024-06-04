use anyhow::{Error, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Default, derive_new::new, Serialize, PartialEq)]
pub struct Percent(pub f64);
impl<'de> Deserialize<'de> for Percent {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		FromStr::from_str(&s).map_err(de::Error::custom)
	}
}
impl FromStr for Percent {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self> {
		let stripped = s.trim_end_matches("%");

		let percent = if let Some(u) = stripped.parse::<usize>().ok() {
			u as f64 / 100.
		} else if let Some(f) = stripped.parse::<f64>().ok() {
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

impl std::fmt::Display for Percent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
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
	fn usize() {
		let p = Percent::from_str("50").unwrap();
		assert_eq!(p.0, 0.5);

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
}
