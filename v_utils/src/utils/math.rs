//! Mathematical functions and interpolation utilities.
//!
//! Provides essential mathematical operations including linear and quadratic
//! interpolation functions commonly used in financial data processing.

/// Macro for approximate floating-point equality comparison.
///
/// Compares two floating-point values with a specified epsilon tolerance.
///
/// # Usage
///
/// ```rust
/// use v_utils::approx_eq;
///
/// let a = 0.1 + 0.2;
/// let b = 0.3;
/// assert!(approx_eq!(f64, a, b, epsilon = 1e-10));
/// ```
#[macro_export]
macro_rules! approx_eq {
	($type:ty, $left:expr, $right:expr,epsilon = $epsilon:expr) => {{
		let left_val: $type = $left;
		let right_val: $type = $right;
		(left_val - right_val).abs() < $epsilon
	}};
	($type:ty, $left:expr, $right:expr,epsilon = $epsilon:expr,ulps = $ulps:expr) => {{
		let left_val: $type = $left;
		let right_val: $type = $right;
		(left_val - right_val).abs() < $epsilon
	}};
}

/// Calculates the interpolation weight between `x1` and `x2` for a value `x`.
///
/// The returned weight `w` satisfies `y = (1 - w) * y1 + w * y2` when
/// interpolating ordinates that correspond to abscissas `x1` and `x2`.
///
/// # Panics
///
/// Panics if `x1` and `x2` are too close (within machine epsilon).
#[inline]
#[must_use]
pub fn linear_weight(x1: f64, x2: f64, x: f64) -> f64 {
	const EPSILON: f64 = f64::EPSILON * 2.0;
	let diff = (x2 - x1).abs();
	assert!(
		diff >= EPSILON,
		"`x1` ({x1}) and `x2` ({x2}) are too close for stable interpolation (diff: {diff}, min: {EPSILON})"
	);
	(x - x1) / (x2 - x1)
}

/// Performs linear interpolation using a weight factor.
///
/// Given ordinates `y1` and `y2` and a weight `x1_diff`, computes:
/// `y1 + x1_diff * (y2 - y1)`.
#[inline]
#[must_use]
pub fn linear_weighting(y1: f64, y2: f64, x1_diff: f64) -> f64 {
	x1_diff.mul_add(y2 - y1, y1)
}

/// Finds the position for interpolation in a sorted array.
///
/// Returns the index of the largest element in `xs` that is less than `x`,
/// clamped to the valid range `[0, xs.len() - 1]`.
#[inline]
#[must_use]
pub fn pos_search(x: f64, xs: &[f64]) -> usize {
	if xs.is_empty() {
		return 0;
	}

	let n_elem = xs.len();
	let pos = xs.partition_point(|&val| val < x);
	std::cmp::min(std::cmp::max(pos.saturating_sub(1), 0), n_elem - 1)
}

/// Evaluates the quadratic Lagrange polynomial defined by three points.
///
/// Given points `(x0, y0)`, `(x1, y1)`, `(x2, y2)` returns *P(x)* where
/// *P* is the unique polynomial of degree ≤ 2 passing through the three points.
///
/// # Panics
///
/// Panics if any two abscissas are too close (within machine epsilon).
#[inline]
#[must_use]
pub fn quad_polynomial(x: f64, x0: f64, x1: f64, x2: f64, y0: f64, y1: f64, y2: f64) -> f64 {
	const EPSILON: f64 = f64::EPSILON * 2.0;

	let diff_01 = (x0 - x1).abs();
	let diff_02 = (x0 - x2).abs();
	let diff_12 = (x1 - x2).abs();

	assert!(
		diff_01 >= EPSILON && diff_02 >= EPSILON && diff_12 >= EPSILON,
		"Abscissas are too close for stable interpolation: x0={x0}, x1={x1}, x2={x2} (diffs: {diff_01:.2e}, {diff_02:.2e}, {diff_12:.2e}, min: {EPSILON})"
	);

	y0 * (x - x1) * (x - x2) / ((x0 - x1) * (x0 - x2)) + y1 * (x - x0) * (x - x2) / ((x1 - x0) * (x1 - x2)) + y2 * (x - x0) * (x - x1) / ((x2 - x0) * (x2 - x1))
}

/// Performs quadratic interpolation for the point `x` given sorted abscissas `xs` and ordinates `ys`.
///
/// # Panics
///
/// Panics if `xs.len() < 3` or `xs.len() != ys.len()`.
#[must_use]
pub fn quadratic_interpolation(x: f64, xs: &[f64], ys: &[f64]) -> f64 {
	let n_elem = xs.len();
	let epsilon = 1e-8;

	assert!(n_elem >= 3, "Need at least 3 points for quadratic interpolation");
	assert_eq!(xs.len(), ys.len(), "xs and ys must have the same length");

	if x <= xs[0] {
		return ys[0];
	}

	if x >= xs[n_elem - 1] {
		return ys[n_elem - 1];
	}

	let pos = pos_search(x, xs);

	if (xs[pos] - x).abs() < epsilon {
		return ys[pos];
	}

	if pos == 0 {
		return quad_polynomial(x, xs[0], xs[1], xs[2], ys[0], ys[1], ys[2]);
	}

	if pos == n_elem - 2 {
		return quad_polynomial(x, xs[n_elem - 3], xs[n_elem - 2], xs[n_elem - 1], ys[n_elem - 3], ys[n_elem - 2], ys[n_elem - 1]);
	}

	let w = linear_weight(xs[pos], xs[pos + 1], x);

	linear_weighting(
		quad_polynomial(x, xs[pos - 1], xs[pos], xs[pos + 1], ys[pos - 1], ys[pos], ys[pos + 1]),
		quad_polynomial(x, xs[pos], xs[pos + 1], xs[pos + 2], ys[pos], ys[pos + 1], ys[pos + 2]),
		w,
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_linear_weight_valid_cases() {
		let cases = [(0.0, 10.0, 5.0, 0.5), (1.0, 3.0, 2.0, 0.5), (0.0, 1.0, 0.25, 0.25), (0.0, 1.0, 0.75, 0.75)];
		for (x1, x2, x, expected) in cases {
			let result = linear_weight(x1, x2, x);
			assert!(approx_eq!(f64, result, expected, epsilon = 1e-10), "Expected {expected}, was {result}");
		}
	}

	#[test]
	#[should_panic(expected = "too close for stable interpolation")]
	fn test_linear_weight_zero_divisor() {
		let _ = linear_weight(1.0, 1.0, 0.5);
	}

	#[test]
	#[should_panic(expected = "too close for stable interpolation")]
	fn test_linear_weight_near_equal_values() {
		let _ = linear_weight(1.0, 1.0 + f64::EPSILON, 0.5);
	}

	#[test]
	fn test_linear_weight_with_small_differences() {
		let result = linear_weight(0.0, 1e-12, 5e-13);
		assert!(result.is_finite());
		assert!((result - 0.5).abs() < 1e-10);
	}

	#[test]
	fn test_linear_weight_just_above_epsilon() {
		let result = linear_weight(1.0, 1.0 + 1e-9, 1.0 + 5e-10);
		assert!(result.is_finite());
	}

	#[test]
	fn test_linear_weighting() {
		let cases = [(1.0, 3.0, 0.5, 2.0), (10.0, 20.0, 0.25, 12.5), (0.0, 10.0, 0.0, 0.0), (0.0, 10.0, 1.0, 10.0)];
		for (y1, y2, weight, expected) in cases {
			let result = linear_weighting(y1, y2, weight);
			assert!(approx_eq!(f64, result, expected, epsilon = 1e-10), "Expected {expected}, was {result}");
		}
	}

	#[test]
	fn test_pos_search() {
		let cases: &[(f64, &[f64], usize)] = &[
			(5.0, &[1.0, 2.0, 3.0, 4.0, 6.0, 7.0], 3),
			(1.5, &[1.0, 2.0, 3.0, 4.0], 0),
			(0.5, &[1.0, 2.0, 3.0, 4.0], 0),
			(4.5, &[1.0, 2.0, 3.0, 4.0], 3),
			(2.0, &[1.0, 2.0, 3.0, 4.0], 0),
		];
		for (x, xs, expected) in cases {
			assert_eq!(pos_search(*x, xs), *expected);
		}
	}

	#[test]
	fn test_pos_search_edge_cases() {
		assert_eq!(pos_search(5.0, &[10.0]), 0);
		assert_eq!(pos_search(3.0, &[1.0, 2.0, 3.0, 4.0]), 1);
		assert_eq!(pos_search(1.5, &[1.0, 2.0]), 0);
	}

	#[test]
	fn test_pos_search_empty_slice() {
		let empty: &[f64] = &[];
		assert_eq!(pos_search(42.0, empty), 0);
	}

	#[test]
	fn test_quad_polynomial_linear_case() {
		let result = quad_polynomial(1.5, 1.0, 2.0, 3.0, 1.0, 2.0, 3.0);
		assert!(approx_eq!(f64, result, 1.5, epsilon = 1e-10));
	}

	#[test]
	fn test_quad_polynomial_parabola() {
		let result = quad_polynomial(1.5, 0.0, 1.0, 2.0, 0.0, 1.0, 4.0);
		let expected = 1.5 * 1.5;
		assert!(approx_eq!(f64, result, expected, epsilon = 1e-10));
	}

	#[test]
	#[should_panic(expected = "too close for stable interpolation")]
	fn test_quad_polynomial_duplicate_x() {
		let _ = quad_polynomial(0.5, 1.0, 1.0, 2.0, 0.0, 1.0, 4.0);
	}

	#[test]
	#[should_panic(expected = "too close for stable interpolation")]
	fn test_quad_polynomial_near_equal_x_values() {
		let _ = quad_polynomial(0.5, 1.0, 1.0 + f64::EPSILON, 2.0, 0.0, 1.0, 4.0);
	}

	#[test]
	fn test_quad_polynomial_with_small_differences() {
		let result = quad_polynomial(5e-13, 0.0, 1e-12, 2e-12, 0.0, 1.0, 4.0);
		assert!(result.is_finite());
	}

	#[test]
	fn test_quad_polynomial_just_above_epsilon() {
		let result = quad_polynomial(0.5, 0.0, 1.0 + 1e-9, 2.0, 0.0, 1.0, 4.0);
		assert!(result.is_finite());
	}

	#[test]
	fn test_quadratic_interpolation_boundary_conditions() {
		let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
		let ys = vec![1.0, 4.0, 9.0, 16.0, 25.0];

		assert_eq!(quadratic_interpolation(0.5, &xs, &ys), ys[0]);
		assert_eq!(quadratic_interpolation(6.0, &xs, &ys), ys[4]);
	}

	#[test]
	fn test_quadratic_interpolation_exact_points() {
		let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
		let ys = vec![1.0, 4.0, 9.0, 16.0, 25.0];

		for (i, &x) in xs.iter().enumerate() {
			let result = quadratic_interpolation(x, &xs, &ys);
			assert!(approx_eq!(f64, result, ys[i], epsilon = 1e-6));
		}
	}

	#[test]
	fn test_quadratic_interpolation_intermediate_values() {
		let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
		let ys = vec![1.0, 4.0, 9.0, 16.0, 25.0];

		let result = quadratic_interpolation(2.5, &xs, &ys);
		let expected = 2.5 * 2.5;
		assert!((result - expected).abs() < 0.1);
	}

	#[test]
	#[should_panic(expected = "Need at least 3 points")]
	fn test_quadratic_interpolation_insufficient_points() {
		let _ = quadratic_interpolation(1.5, &[1.0, 2.0], &[1.0, 4.0]);
	}

	#[test]
	#[should_panic(expected = "xs and ys must have the same length")]
	fn test_quadratic_interpolation_mismatched_lengths() {
		let _ = quadratic_interpolation(1.5, &[1.0, 2.0, 3.0], &[1.0, 4.0]);
	}
}
