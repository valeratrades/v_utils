use eyre::{Result, bail};
use jiff::Timestamp;

/// Doesn't support negative timestamps
pub fn guess_timestamp_unsafe(timestamp: String) -> Result<Timestamp> {
	// Try parsing as ISO 8601 format
	if let Ok(dt) = timestamp.parse::<Timestamp>() {
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
		return Ok(Timestamp::from_nanosecond(nanos as i128).unwrap());
	}

	bail!("Couldn't parse timestamp: {}", timestamp)
}
