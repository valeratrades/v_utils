#[derive(Clone, Debug, Default)]
/// In reality fields can hardly ever even fill 8 bytes, but there is hardly a price to having a safety margin here.
pub struct Pair {
	pub base: [u8; 16],
	pub quote: [u8; 16],
}
impl Pair {
	//HACK: suboptimal implementation
	pub fn new<S: Into<String>>(base: S, quote: S) -> Self {
		let base = base.into().to_uppercase();
		let quote = quote.into().to_uppercase();
		let mut base_bytes = [0; 16];
		let mut quote_bytes = [0; 16];
		base_bytes[..base.len()].copy_from_slice(base.as_bytes());
		quote_bytes[..quote.len()].copy_from_slice(quote.as_bytes());
		Self {
			base: base_bytes,
			quote: quote_bytes,
		}
	}
}

impl std::fmt::Display for Pair {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let base = std::str::from_utf8(&self.base).unwrap();
		let quote = std::str::from_utf8(&self.quote).unwrap();
		write!(f, "{}{}", base, quote)
	}
}
