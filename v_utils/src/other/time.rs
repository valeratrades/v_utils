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
		match self.0 {
			0..=59 => write!(f, "{:02}", self.0),
			60..=3599 => write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60),
			_ => write!(f, "{}:{:02}:{:02}", self.0 / 3600, (self.0 % 3600) / 60, self.0 % 60),
		}
	}
}
impl AsRef<u32> for Timelike {
	fn as_ref(&self) -> &u32 {
		&self.0
	}
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
		let time = String::deserialize(deserializer)?;
		let units = time_to_units(&time).map_err(|e| de::Error::custom(e.to_string()))?;

		Ok(Timelike(units))
	}
}

fn time_to_units(time: &str) -> Result<u32> {
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
		let time: Timelike = serde_json::from_str(r#""12:34""#).unwrap();
		assert_eq!(time.inner(), 754);

		let time: Timelike = serde_json::from_str(r#""12:34:56""#).unwrap();
		assert_eq!(time.inner(), 45296);

		let time: Timelike = serde_json::from_str(r#""34:56""#).unwrap();
		assert_eq!(time.inner(), 2096);

		assert_err!(serde_json::from_str::<Timelike>(r#""12:34:56:78""#));
	}
	#[test]
	fn test_time_ser() {
		let time = Timelike(30);
		assert_eq!(time.to_string(), "30");

		let time = Timelike(2096);
		assert_eq!(time.to_string(), "34:56");

		let time = Timelike(3600);
		assert_eq!(time.to_string(), "1:00:00");

		let time = Timelike(0);
		assert_eq!(&json!(time).to_string(), "\"00\"");

		let time = Timelike(45296);
		assert_eq!(&json!(time).to_string(), "\"12:34:56\"");
	}
}
