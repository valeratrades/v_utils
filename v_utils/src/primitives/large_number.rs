use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A number carrying the relative precision at which it should be rendered compactly (`1.2K`, `-0.0196`, …).
///
/// `precision` is display config, not data: serde treats `LargeNumber` transparently as its `f64` value.
///```rust
/// use v_utils::LargeNumber;
/// assert_eq!(LargeNumber::new(69420.0).format(false).as_str(), "69K");
/// assert_eq!(LargeNumber::with_precision(2420.0, 0.005).format(false).as_str(), "2.42K");
/// assert_eq!(LargeNumber::new(69420.0).format(true).as_str(), "69"); // skip SI suffix
///```
#[derive(Clone, Copy, Debug, derive_more::Deref, derive_more::DerefMut, PartialEq)]
pub struct LargeNumber {
	#[deref]
	#[deref_mut]
	pub value: f64,
	pub precision: f64,
}
impl LargeNumber {
	pub const DEFAULT_PRECISION: f64 = 0.03;

	pub const fn new(value: f64) -> Self {
		Self {
			value,
			precision: Self::DEFAULT_PRECISION,
		}
	}

	pub const fn with_precision(value: f64, precision: f64) -> Self {
		Self { value, precision }
	}

	/// Number of thousands-groupings used when rendered, i.e. which SI suffix applies (0 → none, 1 → K, …).
	pub const fn magnitude(&self) -> u32 {
		self.compute().3
	}

	/// Pure, const-evaluable compact rendering. `skip_si` omits the metric suffix while keeping the scaled-down number.
	pub const fn format(&self, skip_si: bool) -> Compact {
		let (neg, scaled, dp, thousands) = self.compute();

		// scaled is the rounded mantissa * 10^dp; lay its digits out with a decimal point dp places from the right.
		let mut digits = [0u8; 24];
		let mut nd = 0;
		let mut s = scaled;
		loop {
			digits[nd] = (s % 10) as u8;
			s /= 10;
			nd += 1;
			if s == 0 {
				break;
			}
		}
		while nd < dp as usize + 1 {
			digits[nd] = 0;
			nd += 1;
		}

		let mut buf = [0u8; 24];
		let mut len = 0;
		if neg {
			buf[len] = b'-';
			len += 1;
		}
		let mut i = nd;
		while i > dp as usize {
			i -= 1;
			buf[len] = b'0' + digits[i];
			len += 1;
		}
		if dp > 0 {
			buf[len] = b'.';
			len += 1;
			let mut j = dp as usize;
			while j > 0 {
				j -= 1;
				buf[len] = b'0' + digits[j];
				len += 1;
			}
		}
		if !skip_si && let Some(suffix) = suffix_byte(thousands) {
			buf[len] = suffix;
			len += 1;
		}

		Compact { buf, len: len as u8 }
	}

	/// Returns `(negative, rounded_mantissa_scaled_by_10^dp, dp, thousands)`.
	const fn compute(&self) -> (bool, u64, u32, u32) {
		assert!(self.precision >= 0.0, "Precision can't be negative");
		assert!(self.value.is_finite(), "LargeNumber must be finite");
		const MAX_DP: u32 = 15;

		let neg = self.value < 0.0;
		let mut m = if neg { -self.value } else { self.value };
		let mut thousands = 0;
		while m >= 1000.0 {
			m /= 1000.0;
			thousands += 1;
			assert!(thousands <= 5, "Number is too large, calm down");
		}

		loop {
			// smallest dp whose rounding stays within the relative precision
			let mut dp = 0;
			loop {
				let scale = pow10(dp);
				let rounded = ((m * scale + 0.5) as u64) as f64 / scale;
				let err = if m == 0.0 { 0.0 } else { fabs((rounded - m) / m) };
				if err <= self.precision || dp >= MAX_DP {
					break;
				}
				dp += 1;
			}

			let scale = pow10(dp);
			let scaled = (m * scale + 0.5) as u64;
			// rounding can bump the mantissa over a thousands boundary (e.g. 999.6 → 1.0K)
			if scaled as f64 / scale >= 1000.0 {
				m = scaled as f64 / scale / 1000.0;
				thousands += 1;
				assert!(thousands <= 5, "Number is too large, calm down");
				continue;
			}
			return (neg, scaled, dp, thousands);
		}
	}
}

/// Stack-allocated rendered form of a [`LargeNumber`]; ASCII, hence freely `as_str`-able.
#[derive(Clone, Copy, Debug)]
pub struct Compact {
	buf: [u8; 24],
	len: u8,
}
impl Compact {
	pub fn as_str(&self) -> &str {
		std::str::from_utf8(&self.buf[..self.len as usize]).expect("only ASCII digits/suffixes are ever written")
	}
}

const fn pow10(n: u32) -> f64 {
	let mut p = 1.0;
	let mut i = 0;
	while i < n {
		p *= 10.0;
		i += 1;
	}
	p
}

const fn fabs(x: f64) -> f64 {
	if x < 0.0 { -x } else { x }
}

const fn suffix_byte(thousands: u32) -> Option<u8> {
	match thousands {
		0 => None,
		1 => Some(b'K'),
		2 => Some(b'M'),
		3 => Some(b'B'),
		4 => Some(b'T'),
		5 => Some(b'Q'),
		_ => panic!("Number is too large, calm down"),
	}
}

impl Default for LargeNumber {
	fn default() -> Self {
		Self::new(0.0)
	}
}

impl From<f64> for LargeNumber {
	fn from(value: f64) -> Self {
		Self::new(value)
	}
}

impl std::fmt::Display for LargeNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		crate::fmt_with_width!(f, self.format(false).as_str())
	}
}

impl Serialize for LargeNumber {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		self.value.serialize(serializer)
	}
}
impl<'de> Deserialize<'de> for LargeNumber {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		f64::deserialize(deserializer).map(Self::new)
	}
}
impl std::ops::Deref for Compact {
	type Target = str;

	fn deref(&self) -> &str {
		self.as_str()
	}
}
impl AsRef<str> for Compact {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}
impl std::fmt::Display for Compact {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}
