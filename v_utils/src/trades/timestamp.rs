use chrono::{DateTime, TimeZone, Utc};
use eyre::{bail, Result};

/// Doesn't support negative timestamps
pub fn guess_timestamp_unsafe(timestamp: String) -> Result<DateTime<Utc>> {
	// Try parsing as ISO 8601 format
	if let Ok(dt) = timestamp.parse::<DateTime<Utc>>() {
		return Ok(dt);
	}

	// Try guessing the denominator
	if let Ok(num) = timestamp.parse::<u64>() {
		let len = timestamp.len();
		let nanos = match len {
			10 => num * 1_000_000_000,
			13 => num * 1_000_000,
			16 => num * 1_000,
			19 => num,
			_ => bail!("Invalid timestamp length for guessing: {len}\nTimestamp: {timestamp}"),
		};
		return Ok(Utc.timestamp_nanos(nanos as i64));
	}

	bail!("Couldn't parse timestamp: {}", timestamp)
}
