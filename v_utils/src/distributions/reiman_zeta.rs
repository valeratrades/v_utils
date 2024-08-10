use rand::distributions::{Distribution, WeightedIndex};
use rand::{rngs::StdRng, SeedableRng};

fn zeta(s: f64, n: usize) -> f64 {
	(1..=n).map(|k| 1.0 / (k as f64).powf(s)).sum()
}

pub struct ReimanZeta {
	pub alpha: f64,
	pub weights: Vec<f64>,
	pub normalization_constant: f64,
}

impl ReimanZeta {
	pub fn new(alpha: f64, max_k: usize) -> ReimanZeta {
		let normalization_constant = zeta(alpha, max_k);

		let mut weights = Vec::with_capacity(max_k);
		for k in 1..=max_k {
			weights.push((k as f64).powf(-alpha) / normalization_constant);
		}

		ReimanZeta {
			alpha,
			weights,
			normalization_constant,
		}
	}

	pub fn sample(&self, seed: Option<u64>) -> usize {
		let mut rng = match seed {
			Some(s) => StdRng::seed_from_u64(s),
			None => StdRng::from_entropy(),
		};

		let dist = WeightedIndex::new(&self.weights).unwrap();
		dist.sample(&mut rng) + 1
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::utils::snapshot_plot_p;

	#[test]
	fn test_reiman_zeta() {
		let alpha = 1.0;
		let max_k = 1000;
		let zeta = ReimanZeta::new(alpha, max_k);

		let mut samples = (0..1000).map(|i| zeta.sample(Some(i)) as u32).collect::<Vec<u32>>();
		samples.sort_by(|a, b| b.cmp(a));
		let plot = snapshot_plot_p(&samples, 90, 12);
		insta::assert_snapshot!(plot, @r###"
  █                                                                                         
  ██▄                                                                                       
  ████▄                                                                                     
  █████▇▂                                                                                   
  ███████▅                                                                                  
  ████████▇                                                                                 
  ██████████▄▂                                                                              
  ████████████▇▄▁                                                                           
  ███████████████▅▂▁                                                                        
  ██████████████████▇▄▃▁                                                                    
  ██████████████████████▇▆▅▄▃▂▁                                                             
  ██████████████████████████████▇▇▇▆▆▅▅▄▄▄▄▃▃▃▃▃▃▂▂▂▂▂▂▂▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁
  "###);
	}
}
