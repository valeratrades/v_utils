use rand::{rngs::StdRng, SeedableRng};
use rand_distr::{Distribution, Normal};

/// Generates a random walk using Laplace distribution.
///
/// # Arguments
/// - `scale`: Var(laplace) = $2 * scale^2$
/// - `drift`: `mu` parameter, $start + drift$ is point of peak probability density.
pub fn laplace_random_walk(start: f64, num_steps: usize, scale: f64, drift: f64, seed: Option<u64>) -> Vec<f64> {
	let mut rng = match seed {
		Some(s) => StdRng::seed_from_u64(s),
		None => StdRng::from_entropy(),
	};

	let normal = Normal::new(0.0, 1.0).unwrap();

	let steps: Vec<f64> = (0..num_steps)
		.map(|_| {
			let u: f64 = normal.sample(&mut rng);
			let v: f64 = normal.sample(&mut rng);
			drift + scale * (u.abs() - v.abs())
		})
		.collect();

	let walk: Vec<f64> = steps
		.iter()
		.scan(start, |state, &x| {
			*state += x;
			Some(*state)
		})
		.collect();

	std::iter::once(start).chain(walk).collect()
}

mod tests {
	#[test]
	fn test_laplace_random_walk() {
		let start = 100.0;
		let num_steps = 1000;
		let scale = 0.1;
		let drift = 0.0;
		let seed = Some(42);

		let walk = super::laplace_random_walk(start, num_steps, scale, drift, seed);
		let plot = crate::utils::snapshot_plot_p(&walk, 90, 12);

		insta::assert_snapshot!(plot, @r###"
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
