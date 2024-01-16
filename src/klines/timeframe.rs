use anyhow::Result;
use chrono::Duration;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

#[derive(Debug, Default, Clone)]
pub struct Timeframe {
	designator: TimeframeDesignator,
	n: usize,
}

// TimeframeDesignator {{{
#[derive(Debug, PartialEq, EnumString, Display, Serialize, Deserialize, Clone)]
pub enum TimeframeDesignator {
	Seconds,
	Minutes,
	Hours,
	Days,
	Weeks,
	Months,
}
impl TimeframeDesignator {
	pub fn as_seconds(&self) -> usize {
		match self {
			TimeframeDesignator::Seconds => 1,
			TimeframeDesignator::Minutes => 60,
			TimeframeDesignator::Hours => 60 * 60,
			TimeframeDesignator::Days => 24 * 60 * 60,
			TimeframeDesignator::Weeks => 7 * 24 * 60 * 60,
			TimeframeDesignator::Months => 30 * 24 * 60 * 60, //NB: is approximate
		}
	}

	/// All characters could be in any casee, except for m:minutes and M:months
	pub fn from_str(s: &str) -> Result<Self> {
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
			_ => Err(anyhow::anyhow!("Invalid timeframe designator: {}", s)),
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
		}
	}

	pub fn as_str_bybit(&self) -> &'static str {
		match self {
			TimeframeDesignator::Minutes => "",
			TimeframeDesignator::Days => "D",
			TimeframeDesignator::Weeks => "W",
			TimeframeDesignator::Months => "M",
			_ => panic!("Invalid timeframe designator for Bybit: {}", self),
		}
	}
}
//,}}}

// impl Timeframe {{{
impl Timeframe {
	pub fn as_seconds(&self) -> usize {
		self.n * self.designator.as_seconds()
	}

	pub fn duration(&self) -> Duration {
		Duration::seconds(self.as_seconds())
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
			anynow!("The Timeframe does not match exactly any of the values accepted by Binance API: {}", tf);
		}

		tf_string
	}

	pub fn format_bybit(&self) -> Result<String> {
		let tf_string = match self.n {
			1 if self.designator != TimeframeDesignator::Minutes => format!("{}", self.designator.as_str_bybit()),
			_ => format!("{}{}", self.n, self.designator.as_str_bybit()),
		};
		let valid_values = vec!["1", "3", "5", "15", "30", "60", "120", "240", "360", "720", "D", "W", "M"];
		if !valid_values.contains(&tf_string.as_str()) {
			anynow!("The Timeframe does not match exactly any of the values accepted by Bybit API: {}", tf);
		}

		tf_string
	}
}
//,}}}

// Serializer, Deserializer {{{
impl<'de> Deserialize<'de> for Timeframe {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(TimeframeVisitor)
	}
}

struct TimeframeVisitor;

impl<'de> Visitor<'de> for TimeframeVisitor {
	type Value = Timeframe;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a string representing a timeframe like '5s' or '3M'")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		let (n_str, designator_str) = value.split_at(len - 1);
		let n = match n_str {
			"" => 1,
			_ => n_str.parse::<usize>().map_err(E::custom)?,
		};
		let designator = TimeframeDesignator::from_str(designator_str).map_err(|_| E::custom("invalid or missing timeframe designator"))?;

		Ok(Timeframe { designator, n })
	}
}
//,}}}

// Tests {{{
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn timeframe_serialize() {
		let tf = Timeframe {
			designator: TimeframeDesignator::s,
			n: 5,
		};
		let s = serde_json::to_string(&tf).unwrap();
		assert_eq!(s, "5s".to_owned());
	}
	#[test]
	fn timeframe_deserialize() {
		let s = "5s";
		let tf: Timeframe = serde_json::from_str(s).unwrap();
		assert_eq!(tf.designator, TimeframeDesignator::s);
		assert_eq!(tf.n, 5);
	}
}
//,}}}
