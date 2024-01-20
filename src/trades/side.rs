use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
	Buy,
	Sell,
}
impl FromStr for Side {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_uppercase().as_str() {
			"BUY" => Ok(Side::Buy),
			"SELL" => Ok(Side::Sell),
			_ => Err(format!("Invalid side: {}", s)),
		}
	}
}
impl Side {
	pub fn to_str(&self) -> &'static str {
		match self {
			Side::Buy => "BUY",
			Side::Sell => "SELL",
		}
	}
}
impl std::fmt::Display for Side {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.to_str().fmt(f)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_side_from_str() {
		assert_eq!(Side::from_str("BUY").unwrap(), Side::Buy);
		assert_eq!(Side::from_str("Sell").unwrap(), Side::Sell);
		assert!(Side::from_str("foo").is_err());
	}

	#[test]
	fn test_side_to_str() {
		assert_eq!(Side::Buy.to_str(), "BUY");
		assert_eq!(Side::Sell.to_str(), "SELL");
	}
}
