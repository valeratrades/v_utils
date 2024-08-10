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
		PlotData { scale, offset, blocks }
	}

	fn get_block_index(&self, val: f64, i: usize) -> usize {
		let scaled_val = val * self.scale - self.offset;
		(scaled_val - i as f64 * 8.0).clamp(0.0, 8.0) as usize
	}

	fn raise_plot(&mut self) {
		self.offset -= 1.0; // Raise by the smallest step (▁)
	}
}

/// Recommended width x height: 90 x 12
pub fn snapshot_plot_p<T: Into<f64> + Copy>(arr: &[T], width: usize, height: usize) -> String {
	if arr.is_empty() {
		return String::from("Empty array");
	}
	let arr = arr.iter().map(|x| (*x).into()).collect::<Vec<f64>>();

	let min_val = arr.iter().fold(f64::INFINITY, |a, &b| a.min(b));
	let max_val = arr.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

	if (max_val - min_val).abs() < f64::EPSILON {
		return " ".repeat(width).repeat(height);
	}

	let mut plot_data = PlotData::new(min_val, max_val, height);

	// Check if we need to raise the plot
	let first_block = plot_data.get_block_index(arr[0], height - 1);
	let last_block = plot_data.get_block_index(arr[arr.len() - 1], height - 1);
	if first_block == 0 || last_block == 0 {
		plot_data.raise_plot();
	}

	let mut plot = Vec::with_capacity(height);

	for i in (0..height).rev() {
		let row: String = (0..width)
			.map(|j| {
				let index = (j as f64 * arr.len() as f64 / width as f64) as usize;
				let val = arr[index];
				let block_index = plot_data.get_block_index(val, i);
				plot_data.blocks[block_index]
			})
			.collect();
		plot.push(row);
	}

	plot.join("\n")
}

#[cfg(test)]
mod tests {
	use super::*;
	use insta::assert_snapshot;

	#[test]
	fn test_snapshot_plot_p() {
		let data = crate::distributions::laplace_random_walk(100.0, 1000, 0.1, 0.0, Some(42));

		assert_snapshot!(snapshot_plot_p(&data, 90, 12), @r###"
                                                                      ▂▃▄▃                  
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
  ███████████▇█▇▇███████████████████████████████████████████████████████████████████████████
  "###);
	}
}
