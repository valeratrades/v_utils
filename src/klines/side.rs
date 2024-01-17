#[derive(Debug, Clone, Copy)]
pub enum Side {
	Buy,
	Sell,
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
