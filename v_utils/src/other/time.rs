use eyre::{eyre, Result, WrapErr};
use serde::{de, Deserialize, Serialize};

/// Meant to work with %H:%M and %H:%M:%S and %M:%S
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct Timelike(pub u32);
impl Timelike {
	pub fn inner(&self) -> u32 {
		self.0
	}
}

impl std::fmt::Display for Timelike {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self.0 {
			0..=59 => format!("{:02}", self.0),
			60..=3599 => format!("{:02}:{:02}", self.0 / 60, self.0 % 60),
			_ => format!("{}:{:02}:{:02}", self.0 / 3600, (self.0 % 3600) / 60, self.0 % 60),
		};
		f.pad(&s)
	}
}
impl AsRef<u32> for Timelike {
	fn as_ref(&self) -> &u32 {
		&self.0
	}
}
impl std::ops::Deref for Timelike {
	type Target = u32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TimelikeHelper {
	String(String),
	Number(u32),
}

impl Serialize for Timelike {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer, {
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> Deserialize<'de> for Timelike {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: de::Deserializer<'de>, {
		let helper = TimelikeHelper::deserialize(deserializer)?;
		match helper {
			TimelikeHelper::String(time) => time_to_units(&time).map_err(|e| de::Error::custom(e.to_string())).map(Timelike),
			TimelikeHelper::Number(units) => Ok(Timelike(units)),
		}
	}
}

fn time_to_units(time: &str) -> Result<u32> {
	// If there are no colons, try to parse as seconds directly
	if !time.contains(':') {
		return time.parse().wrap_err_with(|| eyre!("Invalid time format: Could not parse '{}' as seconds", time));
	}

	let mut split = time.split(':');

	let first = split
		.next()
		.ok_or_else(|| eyre!("Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'", time))?
		.parse::<u32>()
		.wrap_err_with(|| eyre!("Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'", time))?;

	let second = split
		.next()
		.ok_or_else(|| eyre!("Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'", time))?
		.parse::<u32>()
		.wrap_err_with(|| eyre!("Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'", time))?;

	let units = match split.next() {
		Some(third) => {
			let third = third
				.parse::<u32>()
				.wrap_err_with(|| eyre!("Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'", time))?;
			first * 3600 + second * 60 + third
		}
		None => first * 60 + second,
	};

	if split.next().is_some() {
		return Err(eyre!("Invalid time format: Expected one of %H:%M, %H:%M:%S, or %M:%S, got '{}'", time));
	}

	Ok(units)
}

#[cfg(test)]
mod tests {
	use claim::assert_err;
	use serde_json::json;

	use super::*;

	#[test]
	fn test_time_de() {
		assert_eq!(serde_json::from_str::<Timelike>(r#""12:34""#).unwrap().inner(), 754);
		assert_eq!(serde_json::from_str::<Timelike>(r#""12:34:56""#).unwrap().inner(), 45296);
		assert_eq!(serde_json::from_str::<Timelike>(r#""34:56""#).unwrap().inner(), 2096);
		assert_eq!(serde_json::from_str::<Timelike>("754").unwrap().inner(), 754);
		assert_eq!(serde_json::from_str::<Timelike>(r#""34""#).unwrap().inner(), 34);
		assert_err!(serde_json::from_str::<Timelike>(r#""12:34:56:78""#));
	}

	#[test]
	fn test_time_ser() {
		assert_eq!(Timelike(30).to_string(), "30");
		assert_eq!(Timelike(2096).to_string(), "34:56");
		assert_eq!(Timelike(3600).to_string(), "1:00:00");
		assert_eq!(&json!(Timelike(0)).to_string(), "\"00\"");
		assert_eq!(&json!(Timelike(45296)).to_string(), "\"12:34:56\"");
	}
}
