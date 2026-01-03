use std::{str::FromStr, time::Duration};

use eyre::{Result, bail, eyre};
use serde::{Deserialize, Deserializer, Serialize, de::Error as SerdeError};
use strum::{EnumIter, IntoEnumIterator as _};

#[derive(Clone, Copy, Debug, Default, EnumIter, PartialEq)]
pub enum TimeframeDesignator {
	Milliseconds,
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
	pub const fn as_millis(&self) -> u64 {
		match self {
			TimeframeDesignator::Milliseconds => 1,
			TimeframeDesignator::Seconds => 1_000,
			TimeframeDesignator::Minutes => 60_000,
			TimeframeDesignator::Hours => 3_600_000,
			TimeframeDesignator::Days => 86_400_000,
			TimeframeDesignator::Weeks => 604_800_000,
			TimeframeDesignator::Months => 2_592_000_000,   //NB: is approximate (30 days)
			TimeframeDesignator::Quarters => 7_776_000_000, //NB: is approximate (90 days)
			TimeframeDesignator::Years => 31_536_000_000,   //NB: is approximate (365 days)
		}
	}

	//Q: not sure if it's better to keep this on its own or move inside the Display impl - is having this be `&'static str` worth something?
	pub const fn as_str(&self) -> &'static str {
		match self {
			TimeframeDesignator::Milliseconds => "ms",
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
			"ms" => Ok(TimeframeDesignator::Milliseconds),
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

/// Implemented over the number of milliseconds
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timeframe(pub u64);
impl Timeframe {
	pub fn try_as_predefined(&self, predefined: &[&'static str]) -> Option<&'static str> {
		let interpreted = predefined.iter().map(|&s| Self::from_str(s).unwrap()).collect::<Vec<_>>();
		let idx = interpreted.iter().position(|x| x == self)?;
		Some(predefined[idx])
	}

	pub fn duration(&self) -> Duration {
		Duration::from_millis(self.0 as u64)
	}

	pub fn signed_duration(&self) -> jiff::SignedDuration {
		jiff::SignedDuration::from_millis(self.0 as i64)
	}

	/// Allows for defining static arrays of Timeframes easily
	pub const fn from_naive(n: u64, designator: TimeframeDesignator) -> Self {
		Self(n * designator.as_millis())
	}

	#[deprecated(since = "v3.0.0", note = "Use `duration` instead")]
	pub fn seconds(&self) -> u64 {
		self.0 / 1_000
	}

	pub fn designator(&self) -> TimeframeDesignator {
		TimeframeDesignator::iter()
			.rev()
			.find(|d| self.0 % d.as_millis() == 0)
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

		let allowed_designators = ["ms", "s", "m", "min", "h", "H", "d", "D", "w", "W", "wk", "M", "mo", "q", "Q", "y", "Y"];
		let designator = TimeframeDesignator::from_str(designator_str)
			.map_err(|_| eyre!(r#"Invalid timeframe designator '{designator_str}'. Expected one of the following: [{:?}]"#, allowed_designators))?;

		let n = if n_str.is_empty() {
			1
		} else {
			n_str.parse::<u64>().map_err(|_| eyre!("Invalid number in timeframe str '{n_str}'. Expected a `u64` number."))?
		};

		let total_millis = n * designator.as_millis();

		Ok(Timeframe(total_millis))
	}
}
impl std::fmt::Display for Timeframe {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let designator = self.designator();
		let n = self.0 / designator.as_millis();
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

impl From<Duration> for Timeframe {
	fn from(d: Duration) -> Self {
		Timeframe(d.as_millis() as u64)
	}
}

#[cfg(test)]
mod timeframe_tests {
	use super::*;

	#[test]
	fn to_str() {
		let tf = Timeframe(5_000);
		assert_eq!(tf.to_string(), "5s".to_owned());
	}

	#[test]
	fn deserialize() {
		let tf: Timeframe = serde_json::from_str("\"5s\"").unwrap();
		assert_eq!(tf, Timeframe(5_000));
	}

	#[test]
	fn parse_weird() {
		let tf = Timeframe::from_str("5min").unwrap();
		assert_eq!(tf, Timeframe(5 * 60 * 1_000));

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

	#[test]
	fn milliseconds_support() {
		let tf = Timeframe::from_str("100ms").unwrap();
		assert_eq!(tf, Timeframe(100));
		assert_eq!(tf.to_string(), "100ms");

		let tf = Timeframe::from_str("500ms").unwrap();
		assert_eq!(tf.designator(), TimeframeDesignator::Milliseconds);
	}

	#[test]
	fn from_duration() {
		let d = Duration::from_millis(5000);
		let tf = Timeframe::from(d);
		assert_eq!(tf, Timeframe(5_000));
		assert_eq!(tf.to_string(), "5s");

		let d = Duration::from_millis(250);
		let tf = Timeframe::from(d);
		assert_eq!(tf, Timeframe(250));
		assert_eq!(tf.to_string(), "250ms");
	}
}
