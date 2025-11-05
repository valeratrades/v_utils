use std::{str::FromStr, time::Duration};

use eyre::{Result, bail, eyre};
use serde::{Deserialize, Deserializer, Serialize, de::Error as SerdeError};
use strum::{EnumIter, IntoEnumIterator as _};

#[derive(Clone, Copy, Debug, Default, EnumIter, PartialEq)]
pub enum TimeframeDesignator {
	Seconds,
	#[default]
	Minutes,
	Hours,
	Days,
	Weeks,
	Months,
	Quarters,
	Years,
}
impl TimeframeDesignator {
	pub const fn as_seconds(&self) -> u32 {
		match self {
			TimeframeDesignator::Seconds => 1,
			TimeframeDesignator::Minutes => 60,
			TimeframeDesignator::Hours => 60 * 60,
			TimeframeDesignator::Days => 24 * 60 * 60,
			TimeframeDesignator::Weeks => 7 * 24 * 60 * 60,
			TimeframeDesignator::Months => 30 * 24 * 60 * 60,       //NB: is approximate
			TimeframeDesignator::Quarters => 30 * 24 * 60 * 60 * 3, //NB: is approximate
			TimeframeDesignator::Years => 30 * 24 * 60 * 60 * 12,   //NB: is approximate
		}
	}

	//Q: not sure if it's better to keep this on its own or move inside the Display impl - is having this be `&'static str` worth something?
	pub const fn as_str(&self) -> &'static str {
		match self {
			TimeframeDesignator::Seconds => "s",
			TimeframeDesignator::Minutes => "m",
			TimeframeDesignator::Hours => "h",
			TimeframeDesignator::Days => "d",
			TimeframeDesignator::Weeks => "w",
			TimeframeDesignator::Months => "M",
			TimeframeDesignator::Quarters => "Q",
			TimeframeDesignator::Years => "y",
		}
	}
}
impl std::fmt::Display for TimeframeDesignator {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl FromStr for TimeframeDesignator {
	type Err = eyre::Report;

	/// All characters could be in any casee, except for m:minutes and M:months
	fn from_str(s: &str) -> Result<Self> {
		match s {
			"s" => Ok(TimeframeDesignator::Seconds),
			"m" => Ok(TimeframeDesignator::Minutes),
			"min" => Ok(TimeframeDesignator::Minutes),
			"h" => Ok(TimeframeDesignator::Hours),
			"H" => Ok(TimeframeDesignator::Hours),
			"d" => Ok(TimeframeDesignator::Days),
			"D" => Ok(TimeframeDesignator::Days),
			"w" => Ok(TimeframeDesignator::Weeks),
			"W" => Ok(TimeframeDesignator::Weeks),
			"wk" => Ok(TimeframeDesignator::Weeks),
			"M" => Ok(TimeframeDesignator::Months),
			"mo" => Ok(TimeframeDesignator::Months),
			"q" => Ok(TimeframeDesignator::Quarters),
			"Q" => Ok(TimeframeDesignator::Quarters),
			"y" => Ok(TimeframeDesignator::Years),
			"Y" => Ok(TimeframeDesignator::Years),
			_ => bail!("Invalid timeframe designator: {}", s),
		}
	}
}

/// Implemented over the number of seconds
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timeframe(pub u32);
impl Timeframe {
	pub fn try_as_predefined(&self, predefined: &[&'static str]) -> Option<&'static str> {
		let interpreted = predefined.iter().map(|&s| Self::from_str(s).unwrap()).collect::<Vec<_>>();
		let idx = interpreted.iter().position(|x| x == self)?;
		Some(predefined[idx])
	}

	pub fn duration(&self) -> Duration {
		Duration::from_secs(self.0 as u64)
	}

	pub fn signed_duration(&self) -> jiff::SignedDuration {
		jiff::SignedDuration::from_secs(self.0 as i64)
	}

	/// Allows for defining static arrays of Timeframes easily
	pub const fn from_naive(n: u32, designator: TimeframeDesignator) -> Self {
		Self(n * designator.as_seconds())
	}

	#[deprecated(note = "Use `duration` instead")]
	pub fn seconds(&self) -> u32 {
		self.0
	}

	pub fn designator(&self) -> TimeframeDesignator {
		TimeframeDesignator::iter()
			.rev()
			.find(|d| self.0 % d.as_seconds() == 0)
			.expect("This can only fails if we were to allow creation of 0-len timeframes")
	}
}
impl FromStr for Timeframe {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		// Find where the numeric part ends and the designator begins
		let split_point = s.chars().position(|c| c.is_ascii_alphabetic());

		let (n_str, designator_str) = match split_point {
			Some(pos) => s.split_at(pos),
			None => (s, "m"), // Bybit has silent minutes. No other major exchange silents a different designator so this workaround is sufficient.
		};

		if s.is_empty() {
			bail!("Timeframe string is empty. Expected a string representing a timeframe like '5s' or '3M'");
		}

		let allowed_designators = ["s", "m", "min", "h", "H", "d", "D", "w", "W", "wk", "M", "mo", "q", "Q", "y", "Y"];
		let designator = TimeframeDesignator::from_str(designator_str)
			.map_err(|_| eyre!(r#"Invalid timeframe designator '{designator_str}'. Expected one of the following: [{:?}]"#, allowed_designators))?;

		let n = if n_str.is_empty() {
			1
		} else {
			n_str.parse::<u32>().map_err(|_| eyre!("Invalid number in timeframe str '{n_str}'. Expected a `u32` number."))?
		};

		let total_seconds = n * designator.as_seconds();

		Ok(Timeframe(total_seconds))
	}
}
impl std::fmt::Display for Timeframe {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let designator = self.designator();
		let n = self.0 / designator.as_seconds();
		let s = format!("{n}{designator}");

		crate::fmt_with_width!(f, &s)
	}
}
impl<'de> Deserialize<'de> for Timeframe {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>, {
		let s = String::deserialize(deserializer)?;
		Self::from_str(&s).map_err(|e| SerdeError::custom(e.to_string()))
	}
}
impl Serialize for Timeframe {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer, {
		serializer.serialize_str(&self.to_string())
	}
}

/// # Panics
impl From<&str> for Timeframe {
	fn from(s: &str) -> Self {
		Timeframe::from_str(s).unwrap()
	}
}
/// # Panics
impl From<&&str> for Timeframe {
	fn from(s: &&str) -> Self {
		Timeframe::from_str(s).unwrap()
	}
}

#[cfg(test)]
mod timeframe_tests {
	use super::*;

	#[test]
	fn to_str() {
		let tf = Timeframe(5);
		assert_eq!(tf.to_string(), "5s".to_owned());
	}

	#[test]
	fn deserialize() {
		let tf: Timeframe = serde_json::from_str("\"5s\"").unwrap();
		assert_eq!(tf, Timeframe(5));
	}

	#[test]
	fn parse_weird() {
		let tf = Timeframe::from_str("5min").unwrap();
		assert_eq!(tf, Timeframe(5 * 60));

		let tf = Timeframe::from_str("1wk").unwrap();
		assert_eq!(tf.designator(), TimeframeDesignator::Weeks);

		let tf = Timeframe::from_str("mo").unwrap();
		assert_eq!(tf.designator(), TimeframeDesignator::Months);
	}

	#[test]
	fn predicated() {
		static TFS_BINANCE: [&str; 19] = [
			"1s", "5s", "15s", "30s", "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "3d", "1w", "1M",
		];
		static TFS_BYBIT: [&str; 13] = ["1", "3", "5", "15", "30", "60", "120", "240", "360", "720", "D", "W", "M"];
		static TFS_MEXC: [&str; 9] = ["1m", "5m", "15m", "30m", "60m", "4h", "1d", "1W", "1M"];
		static TFS_YAHOO: [&str; 12] = ["1m", "2m", "5m", "15m", "30m", "60m", "1h", "1d", "5d", "1wk", "1mo", "3mo"];

		assert_eq!(Timeframe::from("1h").try_as_predefined(&TFS_BINANCE), Some("1h"));
		assert_eq!(Timeframe::from("1h").try_as_predefined(&TFS_BYBIT), Some("60"));
		assert_eq!(Timeframe::from("1h").try_as_predefined(&TFS_MEXC), Some("60m"));
		assert_eq!(Timeframe::from("3M").try_as_predefined(&TFS_YAHOO), Some("3mo"));
	}
}
