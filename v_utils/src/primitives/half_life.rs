use std::{str::FromStr, time::Duration};

use eyre::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize, de::Error as SerdeError};

use super::Timeframe;

/// Written like a [`Timeframe`] ("1w"); never zero.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HalfLife(Timeframe);
impl HalfLife {
	/// The share still left after `elapsed`, in `(0, 1]`.
	pub fn left(&self, elapsed: Duration) -> f64 {
		let left = 0.5f64.powf(elapsed.as_secs_f64() / self.0.duration().as_secs_f64());
		assert!(left.is_finite(), "{elapsed:?} over a half-life of {self} came out {left}");
		left
	}
}
impl FromStr for HalfLife {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		let tf = Timeframe::from_str(s)?;
		if tf.0 == 0 {
			bail!("a half-life is positive. Got: '{s}'");
		}
		Ok(Self(tf))
	}
}
impl std::fmt::Display for HalfLife {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}
impl<'de> Deserialize<'de> for HalfLife {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>, {
		let s = String::deserialize(deserializer)?;
		Self::from_str(&s).map_err(|e| SerdeError::custom(e.to_string()))
	}
}
impl Serialize for HalfLife {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer, {
		self.0.serialize(serializer)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn halves_per_half_life() {
		let week = HalfLife::from_str("1w").unwrap();
		let days = |n: u64| Duration::from_secs(n * 24 * 3600);
		assert_eq!(week.left(Duration::ZERO), 1.0);
		assert_eq!(week.left(days(7)), 0.5);
		assert_eq!(week.left(days(21)), 0.125);
		assert!(HalfLife::from_str("0d").is_err());
	}
}
