use rand::{SeedableRng, rngs::StdRng};
//Q: do I actually need rand_distr, or would rand::distr be enough?
use rand_distr::{Distribution, Normal};

/// Generates a random walk using Normal distribution.
///
/// # Arguments
/// - `std_dev`: Standard deviation of the normal distribution
/// - `drift`: Mean of the normal distribution
pub fn normal_random_walk(start: f64, num_steps: usize, std_dev: f64, drift: f64, seed: Option<u64>) -> Vec<f64> {
	let mut rng = match seed {
		Some(s) => StdRng::seed_from_u64(s),
		None => StdRng::from_os_rng(),
	};

	let normal = Normal::new(drift, std_dev).unwrap();

	let steps: Vec<f64> = (1..num_steps).map(|_| normal.sample(&mut rng)).collect();

	let walk: Vec<f64> = steps
		.iter()
		.scan(start, |state, &x| {
			*state += x;
			Some(*state)
		})
		.collect();

	std::iter::once(start).chain(walk).collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_normal_random_walk() {
		let start = 0.0;
		let num_steps = 1000;
		let std_dev = 0.1;
		let drift = 0.0;
		let seed = Some(42);

		let walk = normal_random_walk(start, num_steps, std_dev, drift, seed);
		let plot = crate::utils::SnapshotP::build(&walk).draw();

		insta::assert_snapshot!(plot, @r###"
                                                                       ▄                 ▂▄ 1.52
                                                                   ▂ ▆ █▁              ▄ ███     
                                                            ▁ ▃    █▃█▅██           ▂▄▄█▇███     
                                                            █▅█  █▇███████▅      ▄▂█████████     
                              ▇ ▄                        ▇█▆███ ▃███████████▁    ███████████     
  ▁          ▁▆              ▅█ █                       ▅██████▂█████████████▄▆▄ ███████████     
  █▆        ▁██          ▃█▅▇██▂█▂▂     ▂            ▇▆▇████████████████████████▂███████████     
  ███▄▁▅   ▇███▅      ▂▂███████████▆▂▂ ▅█▃        ▅▄▂███████████████████████████████████████     
  ██████ ▆▁█████▅ ▆ ▇▁████████████████▄███      ▂▅██████████████████████████████████████████     
  ███████████████▆█▃██████████████████████▇  ▃▆ ████████████████████████████████████████████     
  █████████████████████████████████████████▇ ██▆████████████████████████████████████████████     
  ██████████████████████████████████████████▄███████████████████████████████████████████████-1.54
  "###);
	}
}
