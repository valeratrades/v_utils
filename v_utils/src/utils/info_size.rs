use std::str::FromStr;

use eyre::{Result, bail, eyre};
use serde::{Deserialize, Deserializer, Serialize, de::Error as SerdeError};
use strum::{EnumIter, IntoEnumIterator as _};

/// Information size, stored internally as number of bits
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct InfoSize(pub u64);

/// Whether the unit represents bits or bytes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfoSizeKind {
	Bit,
	Byte,
}

impl InfoSize {
	/// Create from a number and unit
	pub const fn from_parts(n: u64, unit: InfoSizeUnit) -> Self {
		Self(n * unit.as_bits())
	}

	// getters {{{1
	/// Get the raw number of bits
	pub const fn bits(&self) -> u64 {
		self.0
	}

	/// Get the number of bytes (integer division)
	pub const fn bytes(&self) -> u64 {
		self.0 / 8
	}

	// SI decimal units - bits
	pub const fn kilobits(&self) -> f64 {
		self.0 as f64 / 1_000.0
	}

	pub const fn megabits(&self) -> f64 {
		self.0 as f64 / 1_000_000.0
	}

	pub const fn gigabits(&self) -> f64 {
		self.0 as f64 / 1_000_000_000.0
	}

	pub const fn terabits(&self) -> f64 {
		self.0 as f64 / 1_000_000_000_000.0
	}

	pub const fn petabits(&self) -> f64 {
		self.0 as f64 / 1_000_000_000_000_000.0
	}

	// IEC binary units - bits
	pub const fn kibibits(&self) -> f64 {
		self.0 as f64 / 1_024.0
	}

	pub const fn mebibits(&self) -> f64 {
		self.0 as f64 / 1_048_576.0
	}

	pub const fn gibibits(&self) -> f64 {
		self.0 as f64 / 1_073_741_824.0
	}

	pub const fn tebibits(&self) -> f64 {
		self.0 as f64 / 1_099_511_627_776.0
	}

	pub const fn pebibits(&self) -> f64 {
		self.0 as f64 / 1_125_899_906_842_624.0
	}

	// SI decimal units - bytes
	pub const fn kilobytes(&self) -> f64 {
		self.0 as f64 / 8_000.0
	}

	pub const fn megabytes(&self) -> f64 {
		self.0 as f64 / 8_000_000.0
	}

	pub const fn gigabytes(&self) -> f64 {
		self.0 as f64 / 8_000_000_000.0
	}

	pub const fn terabytes(&self) -> f64 {
		self.0 as f64 / 8_000_000_000_000.0
	}

	pub const fn petabytes(&self) -> f64 {
		self.0 as f64 / 8_000_000_000_000_000.0
	}

	// IEC binary units - bytes
	pub const fn kibibytes(&self) -> f64 {
		self.0 as f64 / 8_192.0
	}

	pub const fn mebibytes(&self) -> f64 {
		self.0 as f64 / 8_388_608.0
	}

	pub const fn gibibytes(&self) -> f64 {
		self.0 as f64 / 8_589_934_592.0
	}

	pub const fn tebibytes(&self) -> f64 {
		self.0 as f64 / 8_796_093_022_208.0
	}

	pub const fn pebibytes(&self) -> f64 {
		self.0 as f64 / 9_007_199_254_740_992.0
	}

	//,}}}1

	/// Find the most appropriate unit for display
	pub fn unit(&self) -> InfoSizeUnit {
		// Prefer bytes over bits for display, and find the largest unit that divides evenly
		InfoSizeUnit::iter()
			.rev()
			.find(|u| u.kind() == InfoSizeKind::Byte && self.0 % u.as_bits() == 0)
			.or_else(|| InfoSizeUnit::iter().rev().find(|u| u.kind() == InfoSizeKind::Bit && self.0 % u.as_bits() == 0))
			.unwrap_or(InfoSizeUnit::Bit)
	}
}

impl FromStr for InfoSize {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		if s.is_empty() {
			bail!("InfoSize string is empty. Expected a string like '100MB' or '1GiB'");
		}

		// Find where the numeric part ends and the unit begins
		let split_point = s.chars().position(|c| c.is_ascii_alphabetic());

		let (n_str, unit_str) = match split_point {
			Some(pos) => s.split_at(pos),
			None => bail!("InfoSize string '{}' has no unit. Expected a unit like 'B', 'KB', 'MiB', etc.", s),
		};

		let unit = InfoSizeUnit::from_str(unit_str)?;

		let n = if n_str.is_empty() {
			1
		} else {
			n_str
				.parse::<u64>()
				.map_err(|_| eyre!("Invalid number in InfoSize string '{}'. Expected a `u64` number.", n_str))?
		};

		Ok(InfoSize(n * unit.as_bits()))
	}
}

impl std::fmt::Display for InfoSize {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let unit = self.unit();
		let n = self.0 / unit.as_bits();
		let s = format!("{n}{unit}");

		crate::fmt_with_width!(f, &s)
	}
}

impl<'de> Deserialize<'de> for InfoSize {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>, {
		let s = String::deserialize(deserializer)?;
		Self::from_str(&s).map_err(|e| SerdeError::custom(e.to_string()))
	}
}

impl Serialize for InfoSize {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer, {
		serializer.serialize_str(&self.to_string())
	}
}

impl From<&str> for InfoSize {
	fn from(s: &str) -> Self {
		InfoSize::from_str(s).unwrap()
	}
}

impl From<&&str> for InfoSize {
	fn from(s: &&str) -> Self {
		InfoSize::from_str(s).unwrap()
	}
}

/// SI (decimal) vs IEC (binary) prefix
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InfoSizePrefix {
	#[default]
	None,
	/// SI prefix: K=1000, M=1000^2, etc.
	Kilo,
	Mega,
	Giga,
	Tera,
	Peta,
	/// IEC prefix: Ki=1024, Mi=1024^2, etc.
	Kibi,
	Mebi,
	Gibi,
	Tebi,
	Pebi,
}

#[derive(Clone, Copy, Debug, Default, EnumIter, PartialEq, Eq)]
pub enum InfoSizeUnit {
	// Bits (lowercase b)
	Bit,
	Kilobit,
	Megabit,
	Gigabit,
	Terabit,
	Petabit,
	Kibibit,
	Mebibit,
	Gibibit,
	Tebibit,
	Pebibit,
	// Bytes (uppercase B)
	#[default]
	Byte,
	Kilobyte,
	Megabyte,
	Gigabyte,
	Terabyte,
	Petabyte,
	Kibibyte,
	Mebibyte,
	Gibibyte,
	Tebibyte,
	Pebibyte,
}

impl InfoSizeUnit {
	/// Returns the number of bits this unit represents
	pub const fn as_bits(&self) -> u64 {
		match self {
			// Bits
			InfoSizeUnit::Bit => 1,
			InfoSizeUnit::Kilobit => 1_000,
			InfoSizeUnit::Megabit => 1_000_000,
			InfoSizeUnit::Gigabit => 1_000_000_000,
			InfoSizeUnit::Terabit => 1_000_000_000_000,
			InfoSizeUnit::Petabit => 1_000_000_000_000_000,
			InfoSizeUnit::Kibibit => 1_024,
			InfoSizeUnit::Mebibit => 1_048_576,
			InfoSizeUnit::Gibibit => 1_073_741_824,
			InfoSizeUnit::Tebibit => 1_099_511_627_776,
			InfoSizeUnit::Pebibit => 1_125_899_906_842_624,
			// Bytes (8 bits each)
			InfoSizeUnit::Byte => 8,
			InfoSizeUnit::Kilobyte => 8_000,
			InfoSizeUnit::Megabyte => 8_000_000,
			InfoSizeUnit::Gigabyte => 8_000_000_000,
			InfoSizeUnit::Terabyte => 8_000_000_000_000,
			InfoSizeUnit::Petabyte => 8_000_000_000_000_000,
			InfoSizeUnit::Kibibyte => 8_192,
			InfoSizeUnit::Mebibyte => 8_388_608,
			InfoSizeUnit::Gibibyte => 8_589_934_592,
			InfoSizeUnit::Tebibyte => 8_796_093_022_208,
			InfoSizeUnit::Pebibyte => 9_007_199_254_740_992,
		}
	}

	pub const fn as_str(&self) -> &'static str {
		match self {
			InfoSizeUnit::Bit => "b",
			InfoSizeUnit::Kilobit => "Kb",
			InfoSizeUnit::Megabit => "Mb",
			InfoSizeUnit::Gigabit => "Gb",
			InfoSizeUnit::Terabit => "Tb",
			InfoSizeUnit::Petabit => "Pb",
			InfoSizeUnit::Kibibit => "Kib",
			InfoSizeUnit::Mebibit => "Mib",
			InfoSizeUnit::Gibibit => "Gib",
			InfoSizeUnit::Tebibit => "Tib",
			InfoSizeUnit::Pebibit => "Pib",
			InfoSizeUnit::Byte => "B",
			InfoSizeUnit::Kilobyte => "KB",
			InfoSizeUnit::Megabyte => "MB",
			InfoSizeUnit::Gigabyte => "GB",
			InfoSizeUnit::Terabyte => "TB",
			InfoSizeUnit::Petabyte => "PB",
			InfoSizeUnit::Kibibyte => "KiB",
			InfoSizeUnit::Mebibyte => "MiB",
			InfoSizeUnit::Gibibyte => "GiB",
			InfoSizeUnit::Tebibyte => "TiB",
			InfoSizeUnit::Pebibyte => "PiB",
		}
	}

	pub const fn kind(&self) -> InfoSizeKind {
		match self {
			InfoSizeUnit::Bit
			| InfoSizeUnit::Kilobit
			| InfoSizeUnit::Megabit
			| InfoSizeUnit::Gigabit
			| InfoSizeUnit::Terabit
			| InfoSizeUnit::Petabit
			| InfoSizeUnit::Kibibit
			| InfoSizeUnit::Mebibit
			| InfoSizeUnit::Gibibit
			| InfoSizeUnit::Tebibit
			| InfoSizeUnit::Pebibit => InfoSizeKind::Bit,
			_ => InfoSizeKind::Byte,
		}
	}
}

impl std::fmt::Display for InfoSizeUnit {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl FromStr for InfoSizeUnit {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		// Reject invalid lowercase patterns: first letter must be uppercase for prefixed units
		// Valid: B, b, KB, Kb, KiB, Kib, MB, Mb, MiB, Mib, etc.
		// Invalid: kb, kB, kib, kiB, mb, mB, etc.
		if s.len() > 1 {
			let first_char = s.chars().next().unwrap();
			if first_char.is_ascii_lowercase() && first_char != 'b' {
				bail!("Invalid unit '{}': prefix must be uppercase (e.g., 'KB' not 'kB')", s);
			}
		}

		match s {
			// Bits
			"b" => Ok(InfoSizeUnit::Bit),
			"Kb" => Ok(InfoSizeUnit::Kilobit),
			"Mb" => Ok(InfoSizeUnit::Megabit),
			"Gb" => Ok(InfoSizeUnit::Gigabit),
			"Tb" => Ok(InfoSizeUnit::Terabit),
			"Pb" => Ok(InfoSizeUnit::Petabit),
			"Kib" => Ok(InfoSizeUnit::Kibibit),
			"Mib" => Ok(InfoSizeUnit::Mebibit),
			"Gib" => Ok(InfoSizeUnit::Gibibit),
			"Tib" => Ok(InfoSizeUnit::Tebibit),
			"Pib" => Ok(InfoSizeUnit::Pebibit),
			// Bytes
			"B" => Ok(InfoSizeUnit::Byte),
			"KB" => Ok(InfoSizeUnit::Kilobyte),
			"MB" => Ok(InfoSizeUnit::Megabyte),
			"GB" => Ok(InfoSizeUnit::Gigabyte),
			"TB" => Ok(InfoSizeUnit::Terabyte),
			"PB" => Ok(InfoSizeUnit::Petabyte),
			"KiB" => Ok(InfoSizeUnit::Kibibyte),
			"MiB" => Ok(InfoSizeUnit::Mebibyte),
			"GiB" => Ok(InfoSizeUnit::Gibibyte),
			"TiB" => Ok(InfoSizeUnit::Tebibyte),
			"PiB" => Ok(InfoSizeUnit::Pebibyte),
			_ => bail!("Invalid info size unit: {}", s),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_bytes() {
		assert_eq!(InfoSize::from_str("1B").unwrap(), InfoSize(8));
		assert_eq!(InfoSize::from_str("1KB").unwrap(), InfoSize(8_000));
		assert_eq!(InfoSize::from_str("1MB").unwrap(), InfoSize(8_000_000));
		assert_eq!(InfoSize::from_str("1GB").unwrap(), InfoSize(8_000_000_000));
		assert_eq!(InfoSize::from_str("1TB").unwrap(), InfoSize(8_000_000_000_000));
		assert_eq!(InfoSize::from_str("1PB").unwrap(), InfoSize(8_000_000_000_000_000));
	}

	#[test]
	fn parse_binary_bytes() {
		assert_eq!(InfoSize::from_str("1KiB").unwrap(), InfoSize(8_192));
		assert_eq!(InfoSize::from_str("1MiB").unwrap(), InfoSize(8_388_608));
		assert_eq!(InfoSize::from_str("1GiB").unwrap(), InfoSize(8_589_934_592));
		assert_eq!(InfoSize::from_str("1TiB").unwrap(), InfoSize(8_796_093_022_208));
		assert_eq!(InfoSize::from_str("1PiB").unwrap(), InfoSize(9_007_199_254_740_992));
	}

	#[test]
	fn parse_bits() {
		assert_eq!(InfoSize::from_str("1b").unwrap(), InfoSize(1));
		assert_eq!(InfoSize::from_str("1Kb").unwrap(), InfoSize(1_000));
		assert_eq!(InfoSize::from_str("1Mb").unwrap(), InfoSize(1_000_000));
		assert_eq!(InfoSize::from_str("1Gb").unwrap(), InfoSize(1_000_000_000));
		assert_eq!(InfoSize::from_str("1Tb").unwrap(), InfoSize(1_000_000_000_000));
		assert_eq!(InfoSize::from_str("1Pb").unwrap(), InfoSize(1_000_000_000_000_000));
	}

	#[test]
	fn parse_binary_bits() {
		assert_eq!(InfoSize::from_str("1Kib").unwrap(), InfoSize(1_024));
		assert_eq!(InfoSize::from_str("1Mib").unwrap(), InfoSize(1_048_576));
		assert_eq!(InfoSize::from_str("1Gib").unwrap(), InfoSize(1_073_741_824));
		assert_eq!(InfoSize::from_str("1Tib").unwrap(), InfoSize(1_099_511_627_776));
		assert_eq!(InfoSize::from_str("1Pib").unwrap(), InfoSize(1_125_899_906_842_624));
	}

	#[test]
	fn parse_with_numbers() {
		assert_eq!(InfoSize::from_str("100MB").unwrap(), InfoSize(800_000_000));
		assert_eq!(InfoSize::from_str("512KiB").unwrap(), InfoSize(512 * 8_192));
		assert_eq!(InfoSize::from_str("8b").unwrap(), InfoSize(8));
	}

	#[test]
	fn reject_invalid_lowercase() {
		assert!(InfoSize::from_str("1kb").is_err());
		assert!(InfoSize::from_str("1kB").is_err());
		assert!(InfoSize::from_str("1mb").is_err());
		assert!(InfoSize::from_str("1mB").is_err());
		assert!(InfoSize::from_str("1gb").is_err());
		assert!(InfoSize::from_str("1gB").is_err());
		assert!(InfoSize::from_str("1kib").is_err());
		assert!(InfoSize::from_str("1kiB").is_err());
		assert!(InfoSize::from_str("1mib").is_err());
		assert!(InfoSize::from_str("1miB").is_err());
	}

	#[test]
	fn getters() {
		let size = InfoSize::from_str("1GB").unwrap();
		assert_eq!(size.bits(), 8_000_000_000);
		assert_eq!(size.bytes(), 1_000_000_000);
		assert_eq!(size.gigabytes(), 1.0);
		assert_eq!(size.megabytes(), 1_000.0);
		assert_eq!(size.kilobytes(), 1_000_000.0);

		let size = InfoSize::from_str("1GiB").unwrap();
		assert_eq!(size.gibibytes(), 1.0);
		assert_eq!(size.mebibytes(), 1024.0);
	}

	#[test]
	fn display() {
		assert_eq!(InfoSize(8).to_string(), "1B");
		assert_eq!(InfoSize(8_000).to_string(), "1KB");
		assert_eq!(InfoSize(8_000_000).to_string(), "1MB");
		assert_eq!(InfoSize(8_192).to_string(), "1KiB");
		assert_eq!(InfoSize(1).to_string(), "1b");
		// 1000 bits = 125 bytes, so bytes is preferred for display
		assert_eq!(InfoSize(1_000).to_string(), "125B");
	}

	#[test]
	fn serde_roundtrip() {
		let size = InfoSize::from_str("100MB").unwrap();
		let json = serde_json::to_string(&size).unwrap();
		assert_eq!(json, "\"100MB\"");
		let parsed: InfoSize = serde_json::from_str(&json).unwrap();
		assert_eq!(parsed, size);
	}

	#[test]
	fn from_parts() {
		assert_eq!(InfoSize::from_parts(100, InfoSizeUnit::Megabyte), InfoSize(800_000_000));
		assert_eq!(InfoSize::from_parts(1, InfoSizeUnit::Gibibyte), InfoSize(8_589_934_592));
	}

	#[test]
	fn ordering() {
		let small = InfoSize::from_str("1KB").unwrap();
		let large = InfoSize::from_str("1MB").unwrap();
		assert!(small < large);
	}

	#[test]
	fn implicit_count() {
		assert_eq!(InfoSize::from_str("KB").unwrap(), InfoSize(8_000));
		assert_eq!(InfoSize::from_str("MiB").unwrap(), InfoSize(8_388_608));
	}
}
