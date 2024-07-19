#![allow(dead_code)]
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Default, derive_new::new, Copy)]
struct Ohlc {
	open: f64,
	high: f64,
	low: f64,
	close: f64,
}

//? add oi, lsr, etc?
#[derive(Clone, Debug, Default, derive_new::new, Copy)]
struct Kline {
	ohlc: Ohlc,
	timestamp: DateTime<Utc>,
	volume: f64,
}
