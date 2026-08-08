use std::{str::FromStr, time::Duration};

use eyre::{Result, eyre};
use serde::{Deserialize, Deserializer, Serialize, de::Error as SerdeError};
use strum::EnumIter;

/// Ascending; `EnumIter` equivalence asserted in tests.
const TIMEFRAME_DESIGNATORS: [TimeframeDesignator; 9] = [
	TimeframeDesignator::Milliseconds,
	TimeframeDesignator::Seconds,
	TimeframeDesignator::Minutes,
	TimeframeDesignator::Hours,
	TimeframeDesignator::Days,
	TimeframeDesignator::Weeks,
	TimeframeDesignator::Months,
	TimeframeDesignator::Quarters,
	TimeframeDesignator::Years,
];
/// Common timeframes, as offered by exchanges. `MIN`/`MO` because screaming-snake would collide `m` with `M`.
pub const TF_1MS: Timeframe = Timeframe::from("1ms");
pub const TF_100MS: Timeframe = Timeframe::from("100ms");
pub const TF_1S: Timeframe = Timeframe::from("1s");
pub const TF_5S: Timeframe = Timeframe::from("5s");
pub const TF_15S: Timeframe = Timeframe::from("15s");
pub const TF_30S: Timeframe = Timeframe::from("30s");
pub const TF_1MIN: Timeframe = Timeframe::from("1m");
pub const TF_2MIN: Timeframe = Timeframe::from("2m");
pub const TF_3MIN: Timeframe = Timeframe::from("3m");
pub const TF_5MIN: Timeframe = Timeframe::from("5m");
pub const TF_15MIN: Timeframe = Timeframe::from("15m");
pub const TF_30MIN: Timeframe = Timeframe::from("30m");
pub const TF_1H: Timeframe = Timeframe::from("1h");
pub const TF_2H: Timeframe = Timeframe::from("2h");
pub const TF_4H: Timeframe = Timeframe::from("4h");
pub const TF_6H: Timeframe = Timeframe::from("6h");
pub const TF_8H: Timeframe = Timeframe::from("8h");
pub const TF_12H: Timeframe = Timeframe::from("12h");
pub const TF_1D: Timeframe = Timeframe::from("1d");
pub const TF_3D: Timeframe = Timeframe::from("3d");
pub const TF_5D: Timeframe = Timeframe::from("5d");
pub const TF_1W: Timeframe = Timeframe::from("1w");
pub const TF_1MO: Timeframe = Timeframe::from("1M");
pub const TF_3MO: Timeframe = Timeframe::from("3M");
pub const TF_1Q: Timeframe = Timeframe::from("1q");
pub const TF_1Y: Timeframe = Timeframe::from("1y");
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

	/// All characters could be in any case, except for m:minutes and M:months
	const fn from_ascii(b: &[u8]) -> Option<Self> {
		Some(match b {
			b"ms" => TimeframeDesignator::Milliseconds,
			b"s" => TimeframeDesignator::Seconds,
			b"m" | b"min" => TimeframeDesignator::Minutes,
			b"h" | b"H" => TimeframeDesignator::Hours,
			b"d" | b"D" => TimeframeDesignator::Days,
			b"w" | b"W" | b"wk" => TimeframeDesignator::Weeks,
			b"M" | b"mo" => TimeframeDesignator::Months,
			b"q" | b"Q" => TimeframeDesignator::Quarters,
			b"y" | b"Y" => TimeframeDesignator::Years,
			_ => return None,
		})
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

	fn from_str(s: &str) -> Result<Self> {
		Self::from_ascii(s.as_bytes()).ok_or_else(|| eyre!("Invalid timeframe designator: {s}"))
	}
}

/// Implemented over the number of milliseconds
///
/// `ConstParamTy` so a timeframe can stand in const-generic position: a retained window *is* a
/// timeframe, and spelling it there as a bare `u64` is what loses the unit.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, derive_more::Add, core::marker::ConstParamTy, derive_more::Sub)]
pub struct Timeframe(pub u64);
impl Timeframe {
	pub fn try_as_predefined(&self, predefined: &[&'static str]) -> Option<&'static str> {
		let interpreted = predefined.iter().map(|&s| Self::from_str(s).unwrap()).collect::<Vec<_>>();
		let idx = interpreted.iter().position(|x| x == self)?;
		Some(predefined[idx])
	}

	pub fn duration(&self) -> Duration {
		Duration::from_millis(self.0)
	}

	/// Allows for defining static arrays of Timeframes easily
	pub const fn from_naive(n: u64, designator: TimeframeDesignator) -> Self {
		Self(n * designator.as_millis())
	}

	const fn parse_ascii(b: &[u8]) -> Result<Self, &'static str> {
		if b.is_empty() {
			return Err("Timeframe string is empty. Expected a string representing a timeframe like '5s' or '3M'");
		}
		let (mut n, mut i) = (0u64, 0);
		while i < b.len() && b[i].is_ascii_digit() {
			n = match n.checked_mul(10) {
				Some(n) => n,
				None => return Err("Number in timeframe str overflows a `u64`"),
			};
			n += (b[i] - b'0') as u64;
			i += 1;
		}
		let (count, designator) = b.split_at(i);
		let designator = match designator {
			// Bybit has silent minutes. No other major exchange silents a different designator so this workaround is sufficient.
			b"" => TimeframeDesignator::Minutes,
			d => match TimeframeDesignator::from_ascii(d) {
				Some(d) => d,
				None => return Err(r#"Invalid timeframe designator. Expected one of ["ms", "s", "m", "min", "h", "H", "d", "D", "w", "W", "wk", "M", "mo", "q", "Q", "y", "Y"]"#),
			},
		};
		if count.is_empty() {
			n = 1;
		}
		match n.checked_mul(designator.as_millis()) {
			Some(millis) => Ok(Self(millis)),
			None => Err("Timeframe overflows a `u64` of milliseconds"),
		}
	}

	#[deprecated(since = "3.0.0", note = "Use `duration` instead")]
	pub fn seconds(&self) -> u64 {
		self.0 / 1_000
	}

	pub const fn designator(&self) -> TimeframeDesignator {
		assert!(self.0 != 0, "0-len timeframes are not representable");
		let mut i = TIMEFRAME_DESIGNATORS.len();
		while i > 0 {
			i -= 1;
			let d = TIMEFRAME_DESIGNATORS[i];
			if self.0 % d.as_millis() == 0 {
				return d;
			}
		}
		unreachable!()
	}
}
impl FromStr for Timeframe {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		Self::parse_ascii(s.as_bytes()).map_err(|e| eyre!("{e}. Got: '{s}'"))
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
// A `Timeframe` is (de)serialized as a string like "1m"/"5s", so its JSON Schema is a string.
#[cfg(feature = "schemars")]
impl schemars::JsonSchema for Timeframe {
	fn schema_name() -> std::borrow::Cow<'static, str> {
		"Timeframe".into()
	}

	fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
		schemars::json_schema!({
			"type": "string",
			"description": "A timeframe like \"1m\", \"5s\", \"4h\", \"1d\".",
		})
	}
}

/// Const, so the literal that *names* a timeframe is the same one that defines it, including in
/// const-generic position. Malformed input is then a compile error.
///
/// # Panics
const impl From<&str> for Timeframe {
	fn from(s: &str) -> Self {
		match Timeframe::parse_ascii(s.as_bytes()) {
			Ok(tf) => tf,
			// a const `panic!` takes a `&str` argument; inlining it into the format string would make it non-const
			Err(e) => panic!("{}", { e }),
		}
	}
}
/// # Panics
const impl From<&&str> for Timeframe {
	fn from(s: &&str) -> Self {
		Timeframe::from(*s)
	}
}

impl From<Duration> for Timeframe {
	fn from(d: Duration) -> Self {
		Timeframe(d.as_millis() as u64)
	}
}
impl std::ops::Div for Timeframe {
	type Output = u64;

	fn div(self, rhs: Timeframe) -> u64 {
		assert_eq!(self.0 % rhs.0, 0, "{self} is not a whole multiple of {rhs}");
		self.0 / rhs.0
	}
}
impl std::ops::Div<u64> for Timeframe {
	type Output = Timeframe;

	fn div(self, rhs: u64) -> Timeframe {
		assert_eq!(self.0 % rhs, 0, "{self} does not split into {rhs} whole parts");
		Timeframe(self.0 / rhs)
	}
}
impl std::ops::Mul<u64> for Timeframe {
	type Output = Timeframe;

	fn mul(self, rhs: u64) -> Timeframe {
		Timeframe(self.0 * rhs)
	}
}

#[cfg(test)]
mod timeframe_tests {
	use strum::IntoEnumIterator as _;

	use super::*;

	const _: () = assert!(Timeframe::from("15m").0 == Timeframe::from_naive(15, TimeframeDesignator::Minutes).0);

	#[test]
	fn designators_array_matches_enum() {
		assert!(TimeframeDesignator::iter().eq(TIMEFRAME_DESIGNATORS));
	}

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
