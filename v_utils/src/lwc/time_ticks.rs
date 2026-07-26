//! Auto-increasing time-axis label precision. Mirror of the price axis's `stepDecimals`: as ticks get
//! denser we pick a finer label format so adjacent ticks never collapse to the same string. Pure and
//! dep-free (manual UTC breakdown, no `jiff`) so it's unit-testable on native and reusable by any Rust
//! lightweight-charts consumer.

/// ~pixels we want each label to breathe in; all label-density policy lives in this one const.
const TARGET_LABEL_PX: f64 = 70.0;

/// Format a tick at `unix_secs` given the chart's visible `span_secs` over `width_px` pixels, picking a
/// precision fine enough that neighbouring ticks disambiguate.
pub fn format_time_tick(unix_secs: f64, span_secs: f64, width_px: f64) -> String {
	let secs_per_label = if width_px > 0.0 { span_secs * TARGET_LABEL_PX / width_px } else { span_secs };
	let (y, mo, d, h, mi, s, frac) = breakdown(unix_secs);
	match Rung::pick(secs_per_label) {
		Rung::Date => format!("{y:04}-{mo:02}-{d:02}"),
		Rung::HourZero => format!("{mo:02}-{d:02} {h:02}:00"),
		Rung::HourMin => format!("{h:02}:{mi:02}"),
		Rung::Sec => format!("{h:02}:{mi:02}:{s:02}"),
		// ponytail: secs_per_label is an *estimate* (LWC hides real adjacent-tick deltas). Coarse
		// power-of-ten buckets keep it robust; if a duplicate ever slips, upgrade to a stateful
		// last-label dedup inside this fn.
		Rung::SubSec(dec) => {
			let secf = s as f64 + frac;
			format!("{mi:02}:{secf:0w$.dec$}", w = 3 + dec as usize, dec = dec as usize)
		}
	}
}
enum Rung {
	Date,     // YYYY-MM-DD
	HourZero, // MM-DD HH:00
	HourMin,  // HH:MM
	Sec,      // HH:MM:SS
	SubSec(u32),
}

impl Rung {
	fn pick(secs_per_label: f64) -> Self {
		if secs_per_label >= 86400.0 {
			Rung::Date
		} else if secs_per_label >= 3600.0 {
			Rung::HourZero
		} else if secs_per_label >= 60.0 {
			Rung::HourMin
		} else if secs_per_label >= 1.0 {
			Rung::Sec
		} else {
			// mirror of price-side stepDecimals: one decimal per order of magnitude below 1s.
			let d = (-secs_per_label.log10()).ceil().clamp(1.0, 9.0) as u32;
			Rung::SubSec(d)
		}
	}
}

/// UTC breakdown → (year, month, day, hour, min, sec, frac). Hinnant `civil_from_days` for the date.
fn breakdown(unix_secs: f64) -> (i64, u32, u32, u32, u32, u32, f64) {
	let whole = unix_secs.floor() as i64;
	let frac = unix_secs - whole as f64;
	let days = whole.div_euclid(86400);
	let tod = whole.rem_euclid(86400);
	let (h, mi, s) = ((tod / 3600) as u32, ((tod % 3600) / 60) as u32, (tod % 60) as u32);

	let z = days + 719468;
	let era = if z >= 0 { z } else { z - 146096 } / 146097;
	let doe = z - era * 146097;
	let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
	let year = yoe + era * 400;
	let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
	let mp = (5 * doy + 2) / 153;
	let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
	let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
	(year + i64::from(month <= 2), month, day, h, mi, s, frac)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn adjacent_ticks_distinct_and_known_formats() {
		// 1490000000 = 2017-03-20 08:53:20 UTC. Each span/width lands secs_per_label on a distinct rung.
		assert_eq!(format_time_tick(1490000000.0, 20.0 * 86400.0, 1000.0), "2017-03-20");
		assert_eq!(format_time_tick(1490000000.0, 30.0 * 3600.0, 1000.0), "03-20 08:00");
		assert_eq!(format_time_tick(1490000000.0, 3600.0, 1000.0), "08:53");
		assert_eq!(format_time_tick(1490000000.0, 60.0, 1000.0), "08:53:20");
		assert_eq!(format_time_tick(1490000000.4, 5.0, 1000.0), "53:20.4");

		// Densest realistic zoom: adjacent ticks one resolution-step apart must never repeat a label.
		for &(span, width) in &[(20.0 * 86400.0, 1000.0), (30.0 * 3600.0, 1000.0), (3600.0, 1000.0), (60.0, 1000.0), (5.0, 1000.0), (0.5, 1000.0)] {
			let step = span * TARGET_LABEL_PX / width;
			let base = 1490000000.0;
			let a = format_time_tick(base, span, width);
			let b = format_time_tick(base + step, span, width);
			assert_ne!(a, b, "duplicate label at span={span} width={width}: {a:?}");
		}
	}
}
