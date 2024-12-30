#![allow(dead_code)]
use chrono::{DateTime, Duration, Utc};
use eyre::Result;

use crate::trades::Timeframe;

#[derive(Clone, Debug, Default, derive_new::new, Copy)]
pub struct Ohlc {
	pub open: f64,
	pub high: f64,
	pub low: f64,
	pub close: f64,
}

pub fn p_to_ohlc(p: &[(f64, DateTime<Utc>)], timeframe: &Timeframe) -> Result<Vec<Ohlc>> {
	if p.is_empty() {
		return Ok(Vec::new());
	}

	let duration = timeframe.duration();
	let mut ohlc_data = Vec::new();
	let mut current_ohlc = Ohlc::new(p[0].0, p[0].0, p[0].0, p[0].0);
	let mut current_start = p[0].1;

	for &(price, timestamp) in p.iter() {
		if timestamp >= current_start + duration {
			ohlc_data.push(current_ohlc);
			let duration_nanos = duration.num_nanoseconds().unwrap_or(0);
			current_start = timestamp - Duration::nanoseconds(timestamp.timestamp_nanos_opt().unwrap() % duration_nanos);
			current_ohlc = Ohlc::new(price, price, price, price);
		} else {
			current_ohlc.high = current_ohlc.high.max(price);
			current_ohlc.low = current_ohlc.low.min(price);
			current_ohlc.close = price;
		}
	}

	if !ohlc_data.is_empty() && current_ohlc.open != ohlc_data.last().unwrap().open {
		ohlc_data.push(current_ohlc);
	}

	Ok(ohlc_data)
}

/// take a price-series, and imagine that entries are constantly spaced
pub fn mock_p_to_ohlc(p: &[f64], step: usize) -> Vec<Ohlc> {
	let mut ohlc_data = Vec::new();

	for chunk in p.chunks(step) {
		if chunk.is_empty() {
			continue;
		}

		let ohlc = Ohlc {
			open: chunk[0],
			high: *chunk.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
			low: *chunk.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
			close: *chunk.last().unwrap(),
		};

		ohlc_data.push(ohlc);
	}

	ohlc_data
}

/// Timestamp is often [unsafely converted](crate::trades::guess_timestamp_unsafe) from a string
#[derive(Clone, Debug, Default, derive_new::new, Copy)]
pub struct Kline {
	pub open_time: DateTime<Utc>,
	pub ohlc: Ohlc,
	/// later on I'm likely to graduate to having everything normalized to USDT, or, even better, to actual inflation-adjusted USD dollars, but for now mark this as explicitly `quote`-denominated
	pub volume_quote: f64,
	pub trades: Option<usize>,
	pub taker_buy_volume_quote: Option<usize>,
	pub close_time: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
	use insta::{assert_debug_snapshot, assert_snapshot};

	use super::*;
	use crate::{distributions::laplace_random_walk, utils::SnapshotP};

	#[test]
	fn test_p_to_ohlc() {
		#[rustfmt::skip]
		let closes = laplace_random_walk(100.0, 100, 0.1, 0.0, Some(3));
		let ohlcs = mock_p_to_ohlc(&closes, 26);
		assert_debug_snapshot!((ohlcs.len(), ohlcs), @r###"
  (
      4,
      [
          Ohlc {
              open: 100.0,
              high: 100.09962855165924,
              low: 99.7703007260083,
              close: 99.85086488455471,
          },
          Ohlc {
              open: 99.70248401927323,
              high: 99.86895910516857,
              low: 99.45002911569662,
              close: 99.45002911569662,
          },
          Ohlc {
              open: 99.49265485898609,
              high: 99.89704516160468,
              low: 99.28621063445716,
              close: 99.75342310895584,
          },
          Ohlc {
              open: 99.88496041323388,
              high: 99.8979719681815,
              low: 99.42581828238964,
              close: 99.6898759173783,
          },
      ],
  )
  "###);
	}

	#[test]
	fn test_snapshot_plotting_ohlc() {
		let closes = laplace_random_walk(100.0, 1000, 0.1, 0.0, Some(1));
		let ohlcs = mock_p_to_ohlc(&closes, 10);
		let ohlc_closes = ohlcs.iter().map(|o| o.close).collect::<Vec<f64>>();
		let indicator = laplace_random_walk(100.0, 100, 0.1, 0.0, Some(2)).into_iter().map(Some).collect();
		let plot = SnapshotP::build(&ohlc_closes).secondary_pane_optional(indicator).draw();

		assert_snapshot!(plot, @r###"
                                             ▃       █▆                                     101.83
                 ▂▆                         ▁█       ██                                    ▄      
                ▁██                   ▂     ██     ▆ ██▂ ▅                                 █      
             ▃ ▇███▄▁      ▂     ▆▇▅▅▂█▃   ▄██▃▆▁  █ ███ █         ▅▇▂           ▅▁    ▂▄  █      
             █▂██████   ▃  █    ▃███████   ██████▃██████ █▇▅▂      ███▇▆      ▂  ██▄   ██▃▅█      
            ▃████████▄  █  █    ████████▅  ██████████████████ ▂▅   █████▂     █  ███  ▃█████      
            ██████████ ▇█▂ █▃▁ ▁█████████  ██████████████████▂██▄▆▂██████▆▇   █▁ ███▄▂██████      
            ██████████████▅███▂██████████ ▅████████████████████████████████ ▆▇██▃███████████      
           ▂█████████████████████████████▇█████████████████████████████████▇████████████████      
   ▁▇      █████████████████████████████████████████████████████████████████████████████████      
   ██   █▃ █████████████████████████████████████████████████████████████████████████████████      
  ███▁▅▄██▁█████████████████████████████████████████████████████████████████████████████████99.89
  ──────────────────────────────────────────────────────────────────────────────────────────
                ▂▃▂█▆▇█▂▆▄▄▄▆▅  ▁  ▆▁▂   ▂▁                                                 100.25
  ▅▁▁ ▁▃     ▇▄▄███████████████▇█▇▆███▄▃▄██▄ ▆▃▂                                                  
  ███▇████▄ ▃███████████████████████████████▇███▇                                                 
  █████████▇█████████████████████████████████████▇▄▁▅▂   ▂▂▃                                      
  ████████████████████████████████████████████████████▆▇▆███ ▁              ▄▆▅▃▇ ▅▁▅▂▃▆▆▇▅▅      
  ████████████████████████████████████████████████████████████▆█▅    ▂▄     █████▇██████████      
  ████████████████████████████████████████████████████████████████▅▅███▅▁▅▃▆████████████████99.03
  "###);
	}
}
