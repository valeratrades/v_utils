use anyhow::{Context, Error, Result};
use serde::{de, Deserialize, Serialize};
use std::str::FromStr;

/// Meant to work with %H:%M and %H:%M:%S and %M:%S
#[derive(Clone, Debug, Default)]
pub struct Timelike(pub u32);
impl Timelike {
	pub fn into(self) -> u32 {
		self.0
	}
}

impl std::fmt::Display for Timelike {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.0 {
			0..=59 => write!(f, "{}", self.0),
			60..=3599 => write!(f, "{}:{}", self.0 / 60, self.0 % 60),
			_ => write!(f, "{}:{}:{}", self.0 / 3600, (self.0 % 3600) / 60, self.0 % 60),
		}
	}
}
impl AsRef<u32> for Timelike {
	fn as_ref(&self) -> &u32 {
		&self.0
	}
}

impl<'de> Deserialize<'de> for Timelike {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: de::Deserializer<'de>,
	{
		let time = String::deserialize(deserializer)?;
		let units = time_to_units(&time).map_err(|e| de::Error::custom(e.to_string()))?;

		Ok(Timelike(units))
	}
}

fn time_to_units(time: &str) -> Result<u32> {
	let mut split = time.split(':');

	let first = split
		.next()
		.ok_or(Error::msg(format!(
			"Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'",
			time
		)))?
		.parse::<u32>()
		.context(Error::msg(format!(
			"Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'",
			time
		)))?;

	let second = split
		.next()
		.ok_or(Error::msg(format!(
			"Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'",
			time
		)))?
		.parse::<u32>()
		.context(Error::msg(format!(
			"Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'",
			time
		)))?;

	let units = match split.next() {
		Some(third) => {
			let third = third.parse::<u32>().context(Error::msg(format!(
				"Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'",
				time
			)))?;
			first * 3600 + second * 60 + third
		}
		None => first * 60 + second,
	};

	if let Some(_) = split.next() {
		return Err(Error::msg(format!(
			"Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'",
			time
		)));
	}

	Ok(units)
}

#[cfg(test)]
mod tests {
	use super::*;
	use claim::assert_err;

	#[test]
	fn test_time_de() {
		let time: Timelike = serde_json::from_str(r#""12:34""#).unwrap();
		assert_eq!(time.into(), 754);

		let time: Timelike = serde_json::from_str(r#""12:34:56""#).unwrap();
		assert_eq!(time.into(), 45296);

		let time: Timelike = serde_json::from_str(r#""34:56""#).unwrap();
		assert_eq!(time.into(), 2096);

		assert_err!(serde_json::from_str::<Timelike>(r#""12:34:56:78""#));
	}
	#[test]
	fn test_time_display() {
		let time = Timelike(30);
		assert_eq!(time.to_string(), "30");

		let time = Timelike(45296);
		assert_eq!(time.to_string(), "12:34:56");

		let time = Timelike(2096);
		assert_eq!(time.to_string(), "34:56");
	}
}