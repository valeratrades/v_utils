use std::str::FromStr;

use chrono::Duration;
use eyre::{Result, eyre};
use serde::{Deserialize, Deserializer, de::Error as SerdeError};

#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct Timeframe {
	pub designator: TimeframeDesignator,
	pub n: usize,
}

#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum TimeframeDesignator {
	#[default]
	Seconds,
	Minutes,
	Hours,
	Days,
	Weeks,
	Months,
	Quarters,
	Years,
}
impl TimeframeDesignator {
	pub fn as_seconds(&self) -> usize {
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

	/// My prefered definition matches that of Binance.
	pub fn as_str(&self) -> &'static str {
		self.as_str_binance()
	}

	pub fn as_str_binance(&self) -> &'static str {
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

	pub fn as_str_bybit(&self) -> &'static str {
		match self {
			TimeframeDesignator::Minutes => "",
			TimeframeDesignator::Days => "D",
			TimeframeDesignator::Weeks => "W",
			TimeframeDesignator::Months => "M",
			_ => panic!("Invalid timeframe designator for Bybit: {:?}", self),
		}
	}
}

impl FromStr for TimeframeDesignator {
	type Err = eyre::Report;

	/// All characters could be in any casee, except for m:minutes and M:months
	fn from_str(s: &str) -> Result<Self> {
		match s {
			"s" => Ok(TimeframeDesignator::Seconds),
			"m" => Ok(TimeframeDesignator::Minutes),
			"h" => Ok(TimeframeDesignator::Hours),
			"H" => Ok(TimeframeDesignator::Hours),
			"d" => Ok(TimeframeDesignator::Days),
			"D" => Ok(TimeframeDesignator::Days),
			"w" => Ok(TimeframeDesignator::Weeks),
			"W" => Ok(TimeframeDesignator::Weeks),
			"M" => Ok(TimeframeDesignator::Months),
			"q" => Ok(TimeframeDesignator::Quarters),
			"Q" => Ok(TimeframeDesignator::Quarters),
			"y" => Ok(TimeframeDesignator::Years),
			"Y" => Ok(TimeframeDesignator::Years),
			_ => Err(eyre::eyre!("Invalid timeframe designator: {}", s)),
		}
	}
}

impl Timeframe {
	pub fn as_seconds(&self) -> usize {
		self.n * self.designator.as_seconds()
	}

	pub fn duration(&self) -> Duration {
		Duration::seconds(self.as_seconds() as i64)
	}

	pub fn from_seconds() -> Self {
		unimplemented!()
	}

	pub fn display(&self) -> String {
		format!("{}{}", self.n, self.designator.as_str())
	}

	pub fn format_binance(&self) -> Result<String> {
		let tf_string = format!("{}{}", self.n, self.designator.as_str_binance());
		let valid_values = vec![
			"1s", "5s", "15s", "30s", "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "3d", "1w", "1M",
		];
		if !valid_values.contains(&tf_string.as_str()) {
			return Err(eyre!("The Timeframe '{}' does not match exactly any of the values accepted by Binance API", tf_string));
		}

		Ok(tf_string)
	}

	pub fn fmt_binance(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		core::fmt::Display::fmt(&self.format_binance().unwrap(), f)
	}

	pub fn format_bybit(&self) -> Result<String> {
		let tf_string = match self.n {
			1 if self.designator != TimeframeDesignator::Minutes => self.designator.as_str_bybit().to_string(),
			_ => format!("{}{}", self.n, self.designator.as_str_bybit()),
		};
		let valid_values = vec!["1", "3", "5", "15", "30", "60", "120", "240", "360", "720", "D", "W", "M"];
		if !valid_values.contains(&tf_string.as_str()) {
			return Err(eyre!("The Timeframe does not match exactly any of the values accepted by Bybit API: {}", tf_string));
		}

		Ok(tf_string)
	}
}
impl std::fmt::Display for Timeframe {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.display().fmt(f)
	}
}

fn parse_timeframe(s: &str) -> Result<Timeframe> {
	let (n_str, designator_str) = match s.char_indices().next_back() {
		Some((idx, _)) => s.split_at(idx),
		None => return Err(eyre!("Timeframe string is empty. Expected a string representing a timeframe like '5s' or '3M'")),
	};

	let n = if n_str.is_empty() {
		1
	} else {
		n_str
			.parse::<usize>()
			.map_err(|_| eyre!("Invalid number in timeframe '{}'. Expected a string representing a timeframe like '5s' or '3M'", n_str))?
	};

	let designator = TimeframeDesignator::from_str(designator_str).map_err(|_| {
		eyre!(
			"Invalid or missing timeframe designator '{}'. Expected a string representing a timeframe like '5s' or '3M'",
			designator_str
		)
	})?;

	Ok(Timeframe { designator, n })
}

impl FromStr for Timeframe {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		parse_timeframe(s)
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
		Timeframe::from_str(*s).unwrap()
	}
}

impl<'de> Deserialize<'de> for Timeframe {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>, {
		let s = String::deserialize(deserializer)?;
		parse_timeframe(&s).map_err(|e| SerdeError::custom(e.to_string()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn to_str() {
		let tf = Timeframe {
			designator: TimeframeDesignator::Seconds,
			n: 5,
		};
		assert_eq!(tf.display(), "5s".to_owned());
	}
	#[test]
	fn deserialize() {
		let json_str = "\"5s\"";
		let tf: Timeframe = serde_json::from_str(json_str).unwrap();
		assert_eq!(tf.designator, TimeframeDesignator::Seconds);
		assert_eq!(tf.n, 5);
	}

	#[test]
	fn deser_quarters() {
		let json_str = "\"3Q\"";
		let tf: Timeframe = serde_json::from_str(json_str).unwrap();
		assert_eq!(tf.designator, TimeframeDesignator::Quarters);
		assert_eq!(tf.n, 3);
	}
}
