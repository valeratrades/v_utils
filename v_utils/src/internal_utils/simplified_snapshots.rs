//! Simplified snapshot plotting for internal testing.
//!
//! For external use, prefer the `snapshot_fonts` crate which provides more sophisticated
//! rendering with proper font support, multiple output formats, and better customization.
//! This module exists only for testing v_utils internals, since snapshot_fonts depends on v_utils.

struct PlotData {
	scale: f64,
	offset: f64,
	blocks: [char; 9],
}
impl PlotData {
	fn new(min_val: f64, max_val: f64, height: usize) -> Self {
		let data_range = max_val - min_val;
		let plot_range = (height * 8) as f64;
		let scale = plot_range / data_range;
		let offset = min_val * scale;
		let blocks = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
		//let braille = ['⠀', '⡀', '⡄', '⡆', '⢀', '⢠', '⣀', '⣆', '⣠', '⣤', '⣦', '⣴', '⣶', '⣷', '⣾', '⣿'];
		PlotData { scale, offset, blocks }
	}

	fn get_block_index(&self, val: f64, i: usize) -> usize {
		let scaled_val = val * self.scale - self.offset;
		(scaled_val - i as f64 * 8.0).clamp(0.0, 8.0) as usize
	}

	/// Raise by the smallest step (▁)
	fn raise_plot(&mut self) {
		self.offset -= 1.0;
	}
}

static SINGLE_PLOT_WIDTH: usize = 90;
static SINGLE_PLOT_HEIGHT: usize = 12;

#[derive(Clone, Debug, Default)]
pub struct SnapshotP {
	prices: Vec<f64>,
	secondary_pane: Option<Vec<Option<f64>>>,
	width: usize,
	height: usize,
}
/// Very not DRY
#[allow(unused)]
impl SnapshotP {
	pub fn build<T: Into<f64> + Copy>(prices: &[T]) -> Self {
		SnapshotP {
			prices: prices.iter().map(|x| (*x).into()).collect(),
			secondary_pane: None,
			width: SINGLE_PLOT_WIDTH,
			height: SINGLE_PLOT_HEIGHT,
		}
	}

	/// Height is always 2/5 that of the main pane
	pub fn secondary_pane_optional<T: Into<f64> + Copy>(self, secondary_pane: Vec<Option<T>>) -> Self {
		SnapshotP {
			secondary_pane: Some(secondary_pane.iter().map(|x| x.map(|x| x.into())).collect()),
			..self
		}
	}

	/// Height is always 2/5 that of the main pane
	pub fn secondary_pane<T: Into<f64> + Copy>(self, secondary_pane: Vec<T>) -> Self {
		SnapshotP {
			secondary_pane: Some(secondary_pane.iter().map(|x| Some((*x).into())).collect()),
			..self
		}
	}

	/// Default width is `90`
	pub fn width(self, width: usize) -> Self {
		SnapshotP { width, ..self }
	}

	/// Set height of the main pane. Secondary pane's height is automatically determined. Default height is `20`
	pub fn height_main_pane(self, height: usize) -> Self {
		SnapshotP { height, ..self }
	}

	/// # Panics
	/// Meant to be used only in tests, so if any input params are incorrect we panic.
	pub fn draw(self) -> String {
		let main_section = Self::plot_p(self.prices, self.width, self.height); // main must be plot_p, because first and last on it can never be empty.
		let mut out = main_section;
		if let Some(secondary_pane) = self.secondary_pane {
			let separator = "─".repeat(self.width);
			let secondary_section = Self::plot_p_optional(secondary_pane, self.width, (self.height * 3) / 5);
			out.push_str(&format!("\n{separator}\n{secondary_section}"));
		}
		out
	}

	fn plot_p_optional(prices: Vec<Option<f64>>, width: usize, height: usize) -> String {
		if prices.is_empty() {
			return " ".repeat(width).repeat(height);
		}
		let non_empty_prices = prices.iter().filter_map(|x| *x).collect::<Vec<f64>>();

		let min_val = non_empty_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
		let max_val = non_empty_prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

		let min_step = (max_val - min_val) / 100.0;
		let f_len = min_step.to_string().split('.').collect::<Vec<&str>>()[1].chars().take_while(|&c| c == '0').count() + 1;
		let max_str = format!("{max_val:.f_len$}").trim_end_matches(".0").to_string();
		let min_str = format!("{min_val:.f_len$}").trim_end_matches(".0").to_string();
		let side_panel_width = max_str.len().max(min_str.len());
		let mut side_panel = String::with_capacity(height * side_panel_width);
		for i in 0..height {
			if i == 0 {
				side_panel.push_str(&format!("{max_str}\n"));
			} else if i == height - 1 {
				side_panel.push_str(&format!("{min_str}\n"));
			} else {
				side_panel.push_str(&format!("{:>side_panel_width$}\n", " "));
			}
		}
		side_panel.pop(); // remove last newline

		if (max_val - min_val).abs() < f64::EPSILON {
			return " ".repeat(width).repeat(height);
		}

		let mut plot_data = PlotData::new(min_val, max_val, height);
		plot_data.raise_plot(); // here we always want to reise to be able to distinguish between empty and non-empty prices

		let mut plot = Vec::with_capacity(height);
		for i in (0..height).rev() {
			let row: String = (0..width)
				.map(|j| {
					let index = (j as f64 * prices.len() as f64 / width as f64) as usize;
					match prices[index] {
						Some(val) => {
							let block_index = plot_data.get_block_index(val, i);
							plot_data.blocks[block_index]
						}
						None => ' ',
					}
				})
				.collect();
			plot.push(row);
		}

		join_str_blocks_v(plot.join("\n"), side_panel)
	}

	fn plot_p(prices: Vec<f64>, width: usize, height: usize) -> String {
		if prices.is_empty() {
			panic!("prices are empty");
		}

		let min_val = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
		let max_val = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
		if (max_val - min_val).abs() < f64::EPSILON {
			return " ".repeat(width).repeat(height);
		}

		let min_step = (max_val - min_val) / 100.0;
		let f_len = min_step.to_string().split('.').collect::<Vec<&str>>()[1].chars().take_while(|&c| c == '0').count() + 1;
		let max_str = format!("{max_val:.f_len$}").trim_end_matches(".0").to_string();
		let min_str = format!("{min_val:.f_len$}").trim_end_matches(".0").to_string();
		let side_panel_width = max_str.len().max(min_str.len());
		let mut side_panel = String::with_capacity(height * side_panel_width);
		for i in 0..height {
			if i == 0 {
				side_panel.push_str(&format!("{max_str}\n"));
			} else if i == height - 1 {
				side_panel.push_str(&format!("{min_str}\n"));
			} else {
				side_panel.push_str(&format!("{:>side_panel_width$}\n", " "));
			}
		}
		side_panel.pop(); // remove last newline

		let mut plot_data = PlotData::new(min_val, max_val, height);

		// Check if we need to raise the plot
		let first_block = plot_data.get_block_index(prices[0], height - 1);
		let last_block = plot_data.get_block_index(prices[prices.len() - 1], height - 1);
		if first_block == 0 || last_block == 0 {
			plot_data.raise_plot();
		}

		let mut plot = Vec::with_capacity(height);

		for i in (0..height).rev() {
			let row: String = (0..width)
				.map(|j| {
					let index = (j as f64 * prices.len() as f64 / width as f64) as usize;
					let val = prices[index];
					let block_index = plot_data.get_block_index(val, i);
					plot_data.blocks[block_index]
				})
				.collect();
			plot.push(row);
		}

		join_str_blocks_v(plot.join("\n"), side_panel)
	}
}

fn join_str_blocks_v(left: String, right: String) -> String {
	assert_eq!(left.split('\n').count(), right.split('\n').count());
	left.lines().zip(right.lines()).map(|(l, r)| format!("{l}{r}")).collect::<Vec<String>>().join("\n")
}

/// # Panics
/// if ordinals on orders are outside of prices or not ascending.
///
/// # Blocker
/// Until better fonts, distinctions between price formats, multiple order lines at a time & order types, actual timeframes, are all extremely problematic; so their implementation is postponed.
///
/// # Architecture
/// Uses [SnapshotP] to build the plot, for finer control use it instead.
pub fn snapshot_plot_orders<T: Into<f64> + Copy>(prices: &[T], orders: &[(usize, Option<T>)]) -> String {
	let prices = prices.iter().map(|x| (*x).into()).collect::<Vec<f64>>();
	let orders = orders.iter().map(|(i, x)| (*i, x.map(|x| x.into()))).collect::<Vec<(usize, Option<f64>)>>();
	assert!(orders.iter().all(|(i, _)| *i < prices.len()));
	assert!(orders.windows(2).all(|w| w[0].0 < w[1].0));

	let mut order_points = Vec::with_capacity(prices.len());
	let mut last_order: (usize, Option<f64>) = (0, None);
	for (i, order) in orders.iter() {
		order_points.extend((last_order.0..*i).map(|_| last_order.1));
		last_order = (*i, *order);
	}
	order_points.extend((last_order.0..prices.len()).map(|_| last_order.1));

	SnapshotP::build(&prices).secondary_pane_optional(order_points).draw()
}

//TODO!!!!!: `plot_xy()`, that would have explicit axis markings in the expected locations (for now make strictly positive to avoid logic for dynamically positioning the axis).

#[cfg(all(test, feature = "distributions"))]
mod tests {
	use insta::assert_snapshot;
	use rand::{Rng, SeedableRng, rngs::StdRng};

	use super::*;
	use crate::distributions::laplace_random_walk;

	#[test]
	fn test_snapshot_plot_p() {
		let data = laplace_random_walk(100.0, 1000, 0.1, 0.0, Some(42));
		let plot = SnapshotP::build(&data).draw();

		assert_snapshot!(plot, @r"
		                                                                    ▂▃▄▃                  103.50
		                                                                 ▃  █████▆▁▆▇▄                  
		                                                                ▅█▅▆██████████▃       ▃▆▄▄      
		                                                              ▄▄███████████████▅▅▆▂  ▂████      
		                                                            ▅▅█████████████████████▅▇█████      
		                                                           ███████████████████████████████      
		                   ▂                ▂        ▅▄▁▄         ▁███████████████████████████████      
		                 ▆██▃▁         ▂▁  ▅█▇▄   ▁ █████▁ ▅    ▃▅████████████████████████████████      
		▂▃  ▃           ▄█████▇     ▆▆▇██▇▆████▆▅▆█▇██████▇█▇ ▂▁██████████████████████████████████      
		██▃▅█▇▆ ▃       ███████▇ ▇█▅█████████████████████████▆████████████████████████████████████      
		█████████▇▃ ▁  ▇████████▄█████████████████████████████████████████████████████████████████      
		███████████▇█▇▇███████████████████████████████████████████████████████████████████████████98.73
		");
	}

	#[test]
	fn test_snapshot_plot_orders() {
		let prices = laplace_random_walk(100.0, 1000, 0.1, 0.0, Some(42));
		let n_orders = 10;
		let mut orders_left_to_select = 10;
		let mut order_ordinals = Vec::with_capacity(n_orders);
		for i in 0..prices.len() {
			let target_probability = orders_left_to_select as f64 / (prices.len() - i) as f64;
			let mut rng = StdRng::seed_from_u64(i as u64);
			if rng.random_range(0.0..1.0) < target_probability {
				order_ordinals.push(i);
				orders_left_to_select -= 1;
			}
		}
		let order_prices = laplace_random_walk(100.0, n_orders, 1.0, 0.0, Some(4));
		let mut orders = Vec::with_capacity(n_orders);
		for (i, o) in order_ordinals.iter().enumerate() {
			let order = match i == 6 || i == 7 {
				true => None,
				_ => Some(order_prices[i]),
			};
			orders.push((*o, order));
		}
		let plot = snapshot_plot_orders(&prices, &orders);
		insta::assert_snapshot!(plot, @r"
		                                                                    ▂▃▄▃                  103.50
		                                                                 ▃  █████▆▁▆▇▄                  
		                                                                ▅█▅▆██████████▃       ▃▆▄▄      
		                                                              ▄▄███████████████▅▅▆▂  ▂████      
		                                                            ▅▅█████████████████████▅▇█████      
		                                                           ███████████████████████████████      
		                   ▂                ▂        ▅▄▁▄         ▁███████████████████████████████      
		                 ▆██▃▁         ▂▁  ▅█▇▄   ▁ █████▁ ▅    ▃▅████████████████████████████████      
		▂▃  ▃           ▄█████▇     ▆▆▇██▇▆████▆▅▆█▇██████▇█▇ ▂▁██████████████████████████████████      
		██▃▅█▇▆ ▃       ███████▇ ▇█▅█████████████████████████▆████████████████████████████████████      
		█████████▇▃ ▁  ▇████████▄█████████████████████████████████████████████████████████████████      
		███████████▇█▇▇███████████████████████████████████████████████████████████████████████████98.73
		──────────────────────────────────────────────────────────────────────────────────────────
		                             ▅▅▅▅▅▅▅▅▅▅▅▅▅▅▅██████████████                                101.80
		               ▄▄▄▄▄▄▄▄▄▄▄▄▄▄█████████████████████████████                                      
		               ███████████████████████████████████████████                                      
		        ▇▇▇▇▇▇▇███████████████████████████████████████████                                      
		        ██████████████████████████████████████████████████                                      
		        ██████████████████████████████████████████████████                                      
		        ██████████████████████████████████████████████████   ▂▂▂▂▂▂▂▂▂▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁97.82
		");
	}
}
