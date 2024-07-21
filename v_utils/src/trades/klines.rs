#![allow(dead_code)]
use anyhow::Result;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Default, derive_new::new, Copy)]
pub struct Ohlc {
	pub open: f64,
	pub high: f64,
	pub low: f64,
	pub close: f64,
}

pub fn p_to_ohlc(p: &[f64]) -> Result<Vec<Ohlc>> {
	if p.len() % 4 != 0 {
		return Err(anyhow::anyhow!("p_to_ohlc: prices length not multiple of 4"));
	}

	Ok(p.chunks(4)
		.map(|p| Ohlc {
			open: p[0],
			high: p[1],
			low: p[2],
			close: p[3],
		})
		.collect())
}

pub fn p_to_ohlc_force(prices: &[f64]) -> Vec<Ohlc> {
	let offset = prices.len() % 4;
	let cut_prices = &prices[offset..];
	p_to_ohlc(cut_prices).unwrap()
}

pub fn ohlc_snapshot(ohlcs: &[Ohlc], indicator: &[f64]) -> Result<String> {
	assert_eq!(ohlcs.len(), indicator.len());

	let closes = ohlcs.iter().map(|o| o.close).collect::<Vec<f64>>();

	//- take the largest %-wise outlier out of closes and indicator

	todo!()
}

//? add oi, lsr, etc?
#[derive(Clone, Debug, Default, derive_new::new, Copy)]
pub struct Kline {
	pub ohlc: Ohlc,
	pub timestamp: DateTime<Utc>,
	pub volume: f64,
}
