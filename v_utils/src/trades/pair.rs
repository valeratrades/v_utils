use derive_more::{Deref, DerefMut};
use eyre::Report;

#[derive(Clone, Default, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct Asset(pub [u8; 16]);
impl Asset {
	pub fn new<S: AsRef<str>>(s: S) -> Self {
		let mut bytes = [0; 16];
		bytes[..s.as_ref().len()].copy_from_slice(s.as_ref().as_bytes());
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
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

impl std::str::FromStr for Pair {
	type Err = Report;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let delimiters = [',', '-', '_', '/'];

		for delimiter in delimiters {
			if s.contains(delimiter) {
				let parts: Vec<_> = s.split(delimiter).map(str::trim).filter(|s| !s.is_empty()).collect();
				if parts.len() == 2 {
					return Ok(Self::new(parts[0], parts[1]));
				}
				return Err(eyre::eyre!("Invalid pair format: {}", s));
			}
		}

		if !s.is_empty() && s.len() % 2 == 0 {
			let mid = s.len() / 2;
			let (base, quote) = s.split_at(mid);
			if !base.is_empty() && !quote.is_empty() {
				return Ok(Self::new(base, quote));
			}
		}

		Err(eyre::eyre!("Invalid pair format: {}", s))
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
		assert_eq!("BTC - USD".parse::<Pair>().unwrap(), Pair::new("BTC", "USD"));
		assert_eq!("BTCUSD".parse::<Pair>().unwrap(), Pair::new("BTC", "USD"));

		assert!("".parse::<Pair>().is_err());
		assert!("BTC".parse::<Pair>().is_err());
		assert!(dbg!("BTC-".parse::<Pair>()).is_err());
		assert!("-USD".parse::<Pair>().is_err());
	}
}
