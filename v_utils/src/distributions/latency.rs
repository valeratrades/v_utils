//! Simulated arrival latency, log-normal fitted to a percentile triple.
//!
//! Log-normal because network round-trips are classically so, and because in ln-space its quantile
//! function is *linear* in z (`ln q = μ + σ·z`), making a three-point fit closed-form. A caller
//! that models a FIFO stream (e.g. a websocket lane) uses [`LatencySampler::arrival`], which is
//! monotonic per sampler — jitter never reorders within one stream.

use std::{
	hash::{Hash, Hasher},
	time::Duration,
};

use rand::{SeedableRng, rngs::StdRng};
use rand_distr::{Distribution, LogNormal};

/// Standard-normal quantiles at the 68/95/99.7 σ-band convention: `Φ⁻¹(0.68)`, `Φ⁻¹(0.95)`,
/// `Φ⁻¹(0.997)`.
const Z_ANCHORS: [f64; 3] = [0.467699, 1.644854, 2.747781];

/// A log-normal latency model fitted to three percentiles, seeded deterministically. Build a
/// per-stream [`LatencySampler`] with [`LatencyConfig::sampler`]: the seed is hashed with the
/// stream's key, so identical config ⇒ identical stream, distinct keys ⇒ distinct streams.
#[derive(Clone, Copy, Debug)]
pub struct LatencyConfig {
	pub p68: Duration,
	pub p95: Duration,
	pub p997: Duration,
	pub seed: u64,
}

impl LatencyConfig {
	pub fn sampler(&self, seed_key: &str) -> LatencySampler {
		let mut h = std::collections::hash_map::DefaultHasher::new();
		self.seed.hash(&mut h);
		seed_key.hash(&mut h);
		let (mu, sigma) = fit_ln([self.p68.as_nanos() as f64, self.p95.as_nanos() as f64, self.p997.as_nanos() as f64]);
		LatencySampler {
			dist: LogNormal::new(mu, sigma).expect("strictly increasing quantiles ⇒ sigma > 0 and finite"),
			rng: StdRng::seed_from_u64(h.finish()),
			prev_arrival: i64::MIN,
		}
	}
}

/// Monotonic per-stream arrival sampler over nanosecond timestamps.
#[derive(Debug)]
pub struct LatencySampler {
	dist: LogNormal<f64>,
	rng: StdRng,
	prev_arrival: i64,
}

impl LatencySampler {
	/// Effective arrival = `max(prev_arrival, ts_event + sample)` — FIFO within the stream.
	pub fn arrival(&mut self, ts_event: i64) -> i64 {
		let sample = self.dist.sample(&mut self.rng) as i64;
		let a = (ts_event + sample).max(self.prev_arrival);
		self.prev_arrival = a;
		a
	}
}

/// Least-squares line `ln q = μ + σ·z` through the three `(z, ln q)` anchors — closed-form. Panics
/// on a non-increasing or non-positive triple: misconfiguration must die at setup, not skew a run.
fn fit_ln(quantiles: [f64; 3]) -> (f64, f64) {
	assert!(
		0.0 < quantiles[0] && quantiles[0] < quantiles[1] && quantiles[1] < quantiles[2],
		"latency percentiles must be positive and strictly increasing, got {quantiles:?} (ns)"
	);
	let y = quantiles.map(f64::ln);
	let z_mean = Z_ANCHORS.iter().sum::<f64>() / 3.0;
	let y_mean = y.iter().sum::<f64>() / 3.0;
	let (mut num, mut den) = (0.0, 0.0);
	for i in 0..3 {
		num += (Z_ANCHORS[i] - z_mean) * (y[i] - y_mean);
		den += (Z_ANCHORS[i] - z_mean).powi(2);
	}
	let sigma = num / den;
	(y_mean - sigma * z_mean, sigma)
}

#[cfg(test)]
mod tests {
	use super::*;

	// Exact percentiles generated from a known (μ, σ) fit back to that (μ, σ): the anchors are
	// exactly collinear in ln-space, so least-squares is exact up to float error.
	#[test]
	fn fit_round_trips_exact_quantiles() {
		let check = |mu: f64, sigma: f64| {
			let qs = Z_ANCHORS.map(|z| (mu + sigma * z).exp());
			let (mu_hat, sigma_hat) = fit_ln(qs);
			assert!((mu_hat - mu).abs() < 1e-9, "μ: {mu_hat} vs {mu}");
			assert!((sigma_hat - sigma).abs() < 1e-9, "σ: {sigma_hat} vs {sigma}");
		};
		check((30e6f64).ln(), 0.5);
		check((1e6f64).ln(), 0.05);
		check((200e6f64).ln(), 1.2);
	}

	// A FIFO stream's simulated arrivals never go backwards even when jitter would reorder them.
	#[test]
	fn arrivals_are_monotonic() {
		let cfg = LatencyConfig {
			p68: Duration::from_millis(25),
			p95: Duration::from_millis(60),
			p997: Duration::from_millis(150),
			seed: 42,
		};
		let mut s = cfg.sampler("BTCUSDT:trades");
		let mut prev = i64::MIN;
		for ts in (0..10_000).map(|i| i * 1_000_000) {
			let a = s.arrival(ts);
			assert!(a >= prev, "arrival went backwards: {a} < {prev}");
			prev = a;
		}
	}
}
