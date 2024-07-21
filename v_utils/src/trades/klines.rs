#![allow(dead_code)]
use crate::trades::Timeframe;
use crate::utils::snapshot_plot_p;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};

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
			current_start = timestamp - Duration::nanoseconds(timestamp.timestamp_nanos() % duration_nanos);
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

pub fn ohlc_snapshot(ohlcs: &[Ohlc], indicator: &[f64]) -> Result<String> {
	assert_eq!(ohlcs.len(), indicator.len());

	let closes = ohlcs.iter().map(|o| o.close).collect::<Vec<f64>>();

	let price_plot = snapshot_plot_p(&closes, 90, 12);
	let indicator_plot = snapshot_plot_p(indicator, 90, 12);

	let combined_plot = price_plot
		.lines()
		.zip(indicator_plot.lines())
		.map(|(price_line, indicator_line)| format!("{}│{}", price_line, indicator_line))
		.collect::<Vec<String>>()
		.join("\n");

	Ok(combined_plot)
}

//? add oi, lsr, etc?
#[derive(Clone, Debug, Default, derive_new::new, Copy)]
pub struct Kline {
	pub ohlc: Ohlc,
	pub timestamp: DateTime<Utc>,
	pub volume: f64,
}

#[cfg(test)]
mod tests {
	use super::*;
	use insta::{assert_debug_snapshot, assert_snapshot};

	#[test]
	fn test_p_to_ohlc() {
		#[rustfmt::skip]
		let closes = vec![100.0, 99.98034123405445, 100.0347290174959, 100.48839759605941, 99.62133401559197, 101.38574519793993, 101.6684245335374, 101.65966766136323, 101.70648749936485, 102.6232010411682, 102.97313350013474, 101.55631004207399, 100.25871594663444, 100.52272804857675, 100.58314893786022, 100.64283254607244, 100.73433531354823, 100.69221517631237, 100.09351720527273, 100.67293466664673, 100.64235444424168, 100.37334762199043, 101.05505250560705, 101.96492364175322, 102.2552341902445, 102.4643771874453, 103.00400856658018, 103.0770079705397, 103.02995640665938, 102.38206280914957, 101.44333626880916, 101.01280839314724, 100.9499248204719, 101.78576790776899, 102.10434545937888, 102.41886658150547, 101.8961177804279, 101.91029272363858, 104.75134118777744, 104.6278560056506, 104.58452393952936, 104.21408906771778, 103.83574406047777, 103.88493636600897, 103.59095001733286, 102.99965993528096, 103.08175530600438, 102.23148201587901, 102.38348765012664, 102.68463685169142, 102.78148763710935, 102.48123981286992, 102.87908213769386, 101.54193253304851, 102.05643181896018, 103.26123912359945, 103.69839088086984, 103.83468348905919, 104.04304962479134, 104.95516117788536, 104.92389865980158, 105.35315115800985, 104.7544940516362, 105.36401129198312, 105.37857194360474, 106.45390037633943, 105.00661272503059, 105.82631191045223, 106.28603604450699, 106.66008635913374, 105.11486352514159, 105.34500651042048, 105.23385387405953, 104.85123641027657, 105.39713078569835, 105.55530324795174, 105.79159364234994, 105.92782737092307, 108.05899313915141, 107.89735278459993, 108.43341001175129, 108.32542181864629, 108.33872576814629, 108.33443914321589, 108.55780426988207, 108.4253892576315, 107.50736654802179, 107.62402763087272, 107.51398114643504, 107.47638374795653, 107.55541974293325, 107.94972268681686, 108.00694173462705, 108.7869334128387, 107.90069882793894, 107.5365360328119, 106.69100048255488, 106.63267206807168, 107.03367790159332, 106.33479734000295, 106.585157352886];
		let ohlcs = mock_p_to_ohlc(&closes, 5);
		assert_debug_snapshot!(ohlcs, @r###"
  [
      Ohlc {
          open: 100.0,
          high: 100.48839759605941,
          low: 99.62133401559197,
          close: 99.62133401559197,
      },
      Ohlc {
          open: 101.38574519793993,
          high: 102.6232010411682,
          low: 101.38574519793993,
          close: 102.6232010411682,
      },
      Ohlc {
          open: 102.97313350013474,
          high: 102.97313350013474,
          low: 100.25871594663444,
          close: 100.58314893786022,
      },
      Ohlc {
          open: 100.64283254607244,
          high: 100.73433531354823,
          low: 100.09351720527273,
          close: 100.67293466664673,
      },
      Ohlc {
          open: 100.64235444424168,
          high: 102.2552341902445,
          low: 100.37334762199043,
          close: 102.2552341902445,
      },
      Ohlc {
          open: 102.4643771874453,
          high: 103.0770079705397,
          low: 102.38206280914957,
          close: 102.38206280914957,
      },
      Ohlc {
          open: 101.44333626880916,
          high: 102.10434545937888,
          low: 100.9499248204719,
          close: 102.10434545937888,
      },
      Ohlc {
          open: 102.41886658150547,
          high: 104.75134118777744,
          low: 101.8961177804279,
          close: 104.6278560056506,
      },
      Ohlc {
          open: 104.58452393952936,
          high: 104.58452393952936,
          low: 103.59095001733286,
          close: 103.59095001733286,
      },
      Ohlc {
          open: 102.99965993528096,
          high: 103.08175530600438,
          low: 102.23148201587901,
          close: 102.68463685169142,
      },
      Ohlc {
          open: 102.78148763710935,
          high: 102.87908213769386,
          low: 101.54193253304851,
          close: 102.05643181896018,
      },
      Ohlc {
          open: 103.26123912359945,
          high: 104.95516117788536,
          low: 103.26123912359945,
          close: 104.95516117788536,
      },
      Ohlc {
          open: 104.92389865980158,
          high: 105.37857194360474,
          low: 104.7544940516362,
          close: 105.37857194360474,
      },
      Ohlc {
          open: 106.45390037633943,
          high: 106.66008635913374,
          low: 105.00661272503059,
          close: 106.66008635913374,
      },
      Ohlc {
          open: 105.11486352514159,
          high: 105.39713078569835,
          low: 104.85123641027657,
          close: 105.39713078569835,
      },
      Ohlc {
          open: 105.55530324795174,
          high: 108.05899313915141,
          low: 105.55530324795174,
          close: 107.89735278459993,
      },
      Ohlc {
          open: 108.43341001175129,
          high: 108.55780426988207,
          low: 108.32542181864629,
          close: 108.55780426988207,
      },
      Ohlc {
          open: 108.4253892576315,
          high: 108.4253892576315,
          low: 107.47638374795653,
          close: 107.47638374795653,
      },
      Ohlc {
          open: 107.55541974293325,
          high: 108.7869334128387,
          low: 107.55541974293325,
          close: 107.90069882793894,
      },
      Ohlc {
          open: 107.5365360328119,
          high: 107.5365360328119,
          low: 106.33479734000295,
          close: 106.33479734000295,
      },
      Ohlc {
          open: 106.585157352886,
          high: 106.585157352886,
          low: 106.585157352886,
          close: 106.585157352886,
      },
  ]
  "###);
	}

	#[test]
	fn test_ohlc_snapshot_laplace() {
		#[rustfmt::skip]
		let closes = vec![100.0, 99.98034123405445, 100.0347290174959, 100.48839759605941, 99.62133401559197, 101.38574519793993, 101.6684245335374, 101.65966766136323, 101.70648749936485, 102.6232010411682, 102.97313350013474, 101.55631004207399, 100.25871594663444, 100.52272804857675, 100.58314893786022, 100.64283254607244, 100.73433531354823, 100.69221517631237, 100.09351720527273, 100.67293466664673, 100.64235444424168, 100.37334762199043, 101.05505250560705, 101.96492364175322, 102.2552341902445, 102.4643771874453, 103.00400856658018, 103.0770079705397, 103.02995640665938, 102.38206280914957, 101.44333626880916, 101.01280839314724, 100.9499248204719, 101.78576790776899, 102.10434545937888, 102.41886658150547, 101.8961177804279, 101.91029272363858, 104.75134118777744, 104.6278560056506, 104.58452393952936, 104.21408906771778, 103.83574406047777, 103.88493636600897, 103.59095001733286, 102.99965993528096, 103.08175530600438, 102.23148201587901, 102.38348765012664, 102.68463685169142, 102.78148763710935, 102.48123981286992, 102.87908213769386, 101.54193253304851, 102.05643181896018, 103.26123912359945, 103.69839088086984, 103.83468348905919, 104.04304962479134, 104.95516117788536, 104.92389865980158, 105.35315115800985, 104.7544940516362, 105.36401129198312, 105.37857194360474, 106.45390037633943, 105.00661272503059, 105.82631191045223, 106.28603604450699, 106.66008635913374, 105.11486352514159, 105.34500651042048, 105.23385387405953, 104.85123641027657, 105.39713078569835, 105.55530324795174, 105.79159364234994, 105.92782737092307, 108.05899313915141, 107.89735278459993, 108.43341001175129, 108.32542181864629, 108.33872576814629, 108.33443914321589, 108.55780426988207, 108.4253892576315, 107.50736654802179, 107.62402763087272, 107.51398114643504, 107.47638374795653, 107.55541974293325, 107.94972268681686, 108.00694173462705, 108.7869334128387, 107.90069882793894, 107.5365360328119, 106.69100048255488, 106.63267206807168, 107.03367790159332, 106.33479734000295, 106.585157352886];
		let ohlcs = mock_p_to_ohlc(&closes, 5);
		let ohlc_closes = ohlcs.iter().map(|o| o.close).collect::<Vec<f64>>();

		assert_snapshot!(snapshot_plot_p(&ohlc_closes, 90, 12), @r###"
                                                                   ▁▁▁▁████     ▁▁▁▁        
                                                                   ████████▅▅▅▅▅████        
                                                          ▄▄▄▄     █████████████████▁▁▁▁▃▃▃▃
                                                          ████     █████████████████████████
                                                  ▂▂▂▂▆▆▆▆████▇▇▇▇▇█████████████████████████
                                ▆▆▆▆▆             ██████████████████████████████████████████
                                █████▃▃▃▃         ██████████████████████████████████████████
       ▁▁▁▁                     █████████▁▁▁▁     ██████████████████████████████████████████
       ████         ▅▅▅▅▆▆▆▆▃▃▃▃█████████████▃▃▃▃▃██████████████████████████████████████████
       ████         ████████████████████████████████████████████████████████████████████████
       ████▃▃▃▃▄▄▄▄▄████████████████████████████████████████████████████████████████████████
  ▁▁▁▁▁█████████████████████████████████████████████████████████████████████████████████████
  "###);
	}
}
