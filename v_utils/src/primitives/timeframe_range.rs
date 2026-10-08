use std::{
	ops::{Bound, Range, RangeBounds, RangeInclusive},
	str::FromStr,
};

use eyre::{Result, bail, eyre};
use serde::{Deserialize, Deserializer, Serialize, de::Error as SerdeError};

use super::Timeframe;

/// `a..b` / `a..=b` over [`Timeframe`]s, written as Rust writes them: `"3s..=1w"`.
///
/// Only the bounded forms: an open end has no length to draw from. In code, prefer the std ranges
/// directly (`TF_1S..=TF_1W`); this exists to be parsed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeframeRange {
	Exclusive(Range<Timeframe>),
	Inclusive(RangeInclusive<Timeframe>),
}
impl TimeframeRange {
	fn is_empty(&self) -> bool {
		match self {
			Self::Exclusive(r) => r.is_empty(),
			Self::Inclusive(r) => r.is_empty(),
		}
	}
}
impl From<Range<Timeframe>> for TimeframeRange {
	fn from(r: Range<Timeframe>) -> Self {
		Self::Exclusive(r)
	}
}
impl From<RangeInclusive<Timeframe>> for TimeframeRange {
	fn from(r: RangeInclusive<Timeframe>) -> Self {
		Self::Inclusive(r)
	}
}
impl RangeBounds<Timeframe> for TimeframeRange {
	fn start_bound(&self) -> Bound<&Timeframe> {
		match self {
			Self::Exclusive(r) => r.start_bound(),
			Self::Inclusive(r) => r.start_bound(),
		}
	}

	fn end_bound(&self) -> Bound<&Timeframe> {
		match self {
			Self::Exclusive(r) => r.end_bound(),
			Self::Inclusive(r) => r.end_bound(),
		}
	}
}
#[cfg(feature = "distributions")]
impl rand::distr::uniform::SampleRange<Timeframe> for TimeframeRange {
	fn sample_single<R: rand::Rng + ?Sized>(self, rng: &mut R) -> Result<Timeframe, rand::distr::uniform::Error> {
		match self {
			Self::Exclusive(r) => r.sample_single(rng),
			Self::Inclusive(r) => r.sample_single(rng),
		}
	}

	fn is_empty(&self) -> bool {
		self.is_empty()
	}
}
/// Refuses an empty range: written out by hand, it is a typo rather than an intent.
impl FromStr for TimeframeRange {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		let (start, end, inclusive) = match s.split_once("..") {
			Some((start, end)) => match end.strip_prefix('=') {
				Some(end) => (start, end, true),
				None => (start, end, false),
			},
			None => bail!("expected `<start>..<end>` or `<start>..=<end>`. Got: '{s}'"),
		};
		let parse = |side: &str| Timeframe::from_str(side).map_err(|e| eyre!("{e}, in range '{s}'"));
		let range: Self = match inclusive {
			true => (parse(start)?..=parse(end)?).into(),
			false => (parse(start)?..parse(end)?).into(),
		};
		if range.is_empty() {
			bail!("range '{s}' is empty");
		}
		Ok(range)
	}
}
impl std::fmt::Display for TimeframeRange {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Exclusive(r) => write!(f, "{}..{}", r.start, r.end),
			Self::Inclusive(r) => write!(f, "{}..={}", r.start(), r.end()),
		}
	}
}
impl<'de> Deserialize<'de> for TimeframeRange {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>, {
		let s = String::deserialize(deserializer)?;
		Self::from_str(&s).map_err(|e| SerdeError::custom(e.to_string()))
	}
}
impl Serialize for TimeframeRange {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer, {
		serializer.serialize_str(&self.to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::primitives::{TF_1S, TF_1W, TF_5S};

	#[test]
	fn parses_as_rust_writes_ranges() {
		assert_eq!("3s..=1w".parse::<TimeframeRange>().unwrap(), (Timeframe::from("3s")..=TF_1W).into());
		assert_eq!("1s..5s".parse::<TimeframeRange>().unwrap(), (TF_1S..TF_5S).into());
		for s in ["3s..=1w", "1s..5s", "5s..=5s"] {
			assert_eq!(s.parse::<TimeframeRange>().unwrap().to_string(), s);
		}
	}

	#[test]
	fn refuses_open_empty_and_malformed() {
		for s in ["3s..", "..1w", "..", "5s..5s", "1w..=3s", "3s", "3x..1w"] {
			assert!(s.parse::<TimeframeRange>().is_err(), "{s}");
		}
	}

	#[cfg(feature = "distributions")]
	#[test]
	fn samples_within_bounds() {
		use rand::RngExt as _;
		let mut rng = rand::rng();
		for _ in 0..1000 {
			assert!((TF_1S..=TF_5S).contains(&rng.random_range(TF_1S..=TF_5S)));
			let tf = rng.random_range("1s..5s".parse::<TimeframeRange>().unwrap());
			assert!((TF_1S..TF_5S).contains(&tf));
		}
		assert_eq!(rng.random_range(TF_5S..=TF_5S), TF_5S);
	}
}
