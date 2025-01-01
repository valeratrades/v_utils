#[derive(Clone, Default, Copy, PartialEq, Eq, Hash)]
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
