use derive_more::{Deref, DerefMut};
use eyre::Report;

#[derive(Clone, Default, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct Asset(pub [u8; 16]);
impl Asset {
	pub fn new<S: AsRef<str>>(s: S) -> Self {
		let s = s.as_ref().to_uppercase();
		let mut bytes = [0; 16];
		bytes[..s.len()].copy_from_slice(s.as_bytes());
		Self(bytes)
	}
}
impl std::fmt::Display for Asset {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let s = std::str::from_utf8(&self.0).unwrap().trim_end_matches('\0');
		write!(f, "{s}")
	}
}
impl std::fmt::Debug for Asset {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let s = std::str::from_utf8(&self.0).unwrap().trim_end_matches('\0');
		write!(f, "{s}")
	}
}
impl From<&str> for Asset {
	fn from(s: &str) -> Self {
		Self::new(s)
	}
}
impl From<String> for Asset {
	fn from(s: String) -> Self {
		Self::new(s)
	}
}

#[derive(Clone, Debug, Default, Copy, PartialEq, Eq, Hash)]
pub struct Pair {
	base: Asset,
	quote: Asset,
}
impl Pair {
	pub fn new<S: Into<Asset>>(base: S, quote: S) -> Self {
		Self {
			base: base.into(),
			quote: quote.into(),
		}
	}
}
impl<A: Into<Asset>> From<(A, A)> for Pair {
	fn from((base, quote): (A, A)) -> Self {
		Self::new(base, quote)
	}
}
impl std::fmt::Display for Pair {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}{}", self.base, self.quote)
	}
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid pair format '{provided_str}'. Expected two assets separated by one of: [{}]", allowed_delimiters.join(" "))]
pub struct InvalidPairError {
	provided_str: String,
	allowed_delimiters: Vec<String>,
}
impl InvalidPairError {
	pub fn new<S: Into<String>>(provided_str: &str, allowed_delimiters: impl IntoIterator<Item = S>) -> Self {
		Self {
			provided_str: provided_str.to_owned(),
			allowed_delimiters: allowed_delimiters.into_iter().map(Into::into).collect(),
		}
	}
}

impl std::str::FromStr for Pair {
	type Err = Report;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let delimiters = [',', '-', '_', '/'];
		let recognized_quotes = ["USD", "USDT", "USDC", "BTC", "ETH"];

		for delimiter in delimiters {
			if s.contains(delimiter) {
				let parts: Vec<_> = s.split(delimiter).map(str::trim).filter(|s| !s.is_empty()).collect();
				if parts.len() == 2 {
					return Ok(Self::new(parts[0], parts[1]));
				}
				return Err(InvalidPairError::new(s, delimiters.iter().map(|c| c.to_string())).into());
			}
		}

		if let Some(quote) = recognized_quotes.iter().find(|q| s.ends_with(*q)) {
			let base_len = s.len() - quote.len();
			if base_len > 0 {
				let base = &s[..base_len];
				return Ok(Self::new(base, *quote));
			}
		}

		Err(InvalidPairError::new(s, delimiters.iter().map(|c| c.to_string())).into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_pairs() {
		assert_eq!("BTC-USD".parse::<Pair>().unwrap(), Pair::new("BTC", "USD"));
		assert_eq!("ETH,USD".parse::<Pair>().unwrap(), Pair::new("ETH", "USD"));
		assert_eq!("SOL_USDT".parse::<Pair>().unwrap(), Pair::new("SOL", "USDT"));
		assert_eq!("XRP/USDC".parse::<Pair>().unwrap(), Pair::new("XRP", "USDC"));
		assert_eq!("btc - usd".parse::<Pair>().unwrap(), Pair::new("BTC", "USD"));
		assert_eq!("DOGEUSDT".parse::<Pair>().unwrap(), Pair::new("DOGE", "USDT"));

		assert!("something".parse::<Pair>().is_err());
		assert!("".parse::<Pair>().is_err());
		assert!("BTC".parse::<Pair>().is_err());
		assert!("BTC-".parse::<Pair>().is_err());
		assert!("-USD".parse::<Pair>().is_err());
	}
}
