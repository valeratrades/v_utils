//! Terminal charts: proportional bars, and time-axis gantts that a live caller re-renders with the axis end at "now".

/// `frac` of `width` cells, to an eighth of a cell; not padded.
pub fn bar(frac: f64, width: usize) -> String {
	assert!((0.0..=1.0 + 1e-9).contains(&frac), "bar fraction out of range: {frac}");
	let eighths = (frac * (width * 8) as f64).round() as usize;
	let mut s = "█".repeat(eighths / 8);
	if eighths % 8 != 0 {
		s.push([' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'][eighths % 8]);
	}
	s
}

/// Times are seconds from the chart's origin.
#[derive(Clone, Debug)]
pub struct Row {
	pub label: String,
	pub depth: usize,
	pub start: f64,
	pub end: f64,
	pub glyph: char,
}

#[derive(Clone, Debug)]
pub struct Marker {
	pub at: f64,
	pub glyph: char,
	pub name: String,
}

/// Rows drawn in the order given; a row past `axis_end` is clamped and ends in `→`. Markers become columns (a blank glyph
/// draws none), then a legend.
pub fn gantt(rows: &[Row], markers: &[Marker], width: usize, axis_end: f64) -> String {
	assert!(axis_end > 0.0, "gantt axis must span time, got {axis_end}");
	let labels: Vec<String> = rows.iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.label)).collect();
	let lw = labels.iter().map(|l| l.chars().count()).max().unwrap_or(0).min(width * 2 / 5);
	let bw = width.saturating_sub(lw + 11);
	assert!(bw >= 10, "width {width} leaves {bw} cells for bars");
	let cell = |t: f64| ((t / axis_end * bw as f64).round().max(0.0) as usize).min(bw);

	let (scale, unit) = if axis_end < 1.0 { (1e3, "ms") } else { (1.0, "s") };
	let span = axis_end * scale;
	let step = (-3..=6)
		.flat_map(|e| [1.0, 2.0, 5.0].map(|m| m * 10f64.powi(e)))
		.find(|s| span / s <= (bw / 8) as f64)
		.expect("1e6 steps cover any span a terminal fits");
	let decimals = (-step.log10()).ceil().max(0.0) as usize;
	let mut axis = vec![' '; bw];
	for i in 0..=(span / step + 1e-9) as usize {
		let v = i as f64 * step;
		let at = cell(v / scale);
		let label: Vec<char> = format!("|{v:.decimals$}{unit}").chars().collect();
		if at + label.len() <= bw {
			axis[at..at + label.len()].copy_from_slice(&label);
		}
	}
	let mut out = vec![format!("{}{}", " ".repeat(lw + 11), axis.into_iter().collect::<String>())];

	let marker_cols: Vec<(usize, char)> = markers.iter().filter(|m| m.at <= axis_end && !m.glyph.is_whitespace()).map(|m| (cell(m.at).min(bw - 1), m.glyph)).collect();
	for (row, label) in rows.iter().zip(&labels) {
		let mut cells = vec![' '; bw];
		for &(c, g) in &marker_cols {
			cells[c] = g;
		}
		let a = cell(row.start).min(bw - 1);
		for c in &mut cells[a..cell(row.end).max(a + 1)] {
			*c = row.glyph;
		}
		if row.end > axis_end {
			cells[bw - 1] = '→';
		}
		let label: String = label.chars().take(lw).collect();
		out.push(format!("{label:<lw$} {:>8}  {}", duration(row.end - row.start), cells.into_iter().collect::<String>()));
	}

	if !markers.is_empty() {
		out.push(String::new());
		let nw = markers.iter().map(|m| m.name.chars().count()).max().expect("non-empty");
		let mut sorted: Vec<&Marker> = markers.iter().collect();
		sorted.sort_by(|a, b| a.at.total_cmp(&b.at));
		out.extend(sorted.iter().map(|m| format!("{} {:<nw$} {:>8}", m.glyph, m.name, duration(m.at))));
	}
	out.iter().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n")
}

fn duration(secs: f64) -> String {
	match secs {
		s if s < 1.0 => format!("{:.1}ms", s * 1e3),
		s if s < 10.0 => format!("{s:.2}s"),
		s => format!("{s:.1}s"),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn representative() {
		let row = |label: &str, depth, start, end, glyph| Row {
			label: label.into(),
			depth,
			start,
			end,
			glyph,
		};
		let rows = [
			row("exec → first log line", 0, 0.0, 0.42, '█'),
			row("load config", 0, 0.42, 1.9, '█'),
			row("plugin foo", 1, 0.5, 0.503, '█'),
			row("a label long enough that the cap has to cut it short", 1, 1.0, 1.6, '▒'),
			row("connect to a server that never settles", 0, 1.2, 9.0, '░'),
		];
		let markers = [
			Marker {
				at: 1.95,
				glyph: '┊',
				name: "prompt usable".into(),
			},
			Marker {
				at: 0.8,
				glyph: '╎',
				name: "VeryLazy".into(),
			},
		];
		insta::assert_snapshot!(gantt(&rows, &markers, 110, 2.5), @"
		                                                       |0.0s      |0.5s      |1.0s      |1.5s      |2.0s
		exec → first log line                         420.0ms  █████████         ╎                        ┊
		load config                                     1.48s           █████████████████████████████████ ┊
		  plugin foo                                    3.0ms             █      ╎                        ┊
		  a label long enough that the cap has to cu  600.0ms                    ╎   ▒▒▒▒▒▒▒▒▒▒▒▒▒        ┊
		connect to a server that never settles          7.80s                    ╎       ░░░░░░░░░░░░░░░░░░░░░░░░░░░░→

		╎ VeryLazy       800.0ms
		┊ prompt usable    1.95s
		");
	}
}
