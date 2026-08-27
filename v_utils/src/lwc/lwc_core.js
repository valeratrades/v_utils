// Framework-neutral lightweight-charts core. Owns the chart instance (one per host element, reused
// across mounts so the visible time range survives redraws) and dispatches to an app-supplied
// `draw(chart, data, viewSpec, lib)` module fetched at runtime. The chart↔host boilerplate lives here;
// "what we chart" lives entirely in the app's draw module.

// The library rides in the wasm binary as a string (build.rs pins and fetches it), so a blob URL is
// what turns it back into a module — no import map, no network, nothing for the consumer to set up.
// Still lazy and memoized: 196 KB of parse work stays off first paint, and off pages with no chart.
let _lib;
const lib = (src) =>
  (_lib ??= (async () => {
    const url = URL.createObjectURL(new Blob([src], { type: 'text/javascript' }));
    try {
      return await import(url);
    } finally {
      URL.revokeObjectURL(url);
    }
  })());

const charts = new WeakMap();
const draws = new Map();

// Returns null on success, or a banner string on any chart-side failure — the caller renders it and
// stays alive (under panic=abort a thrown/rejected promise crossing into wasm nukes the whole app).
export async function mount(el, drawUrl, dataJson, viewSpec, fmt, libSrc) {
  try {
    const lwc = await lib(libSrc);
    let chart = charts.get(el);
    if (!chart) {
      chart = lwc.createChart(el, { autoSize: true, layout: { attributionLogo: false } });
      charts.set(el, chart);
      // Rust owns label policy; JS only feeds it live geometry (visible span + pixel width — only
      // obtainable from the chart). Falls back to the default label on pre-data (null range).
      const ts = chart.timeScale();
      ts.applyOptions({
        tickMarkFormatter: (time) => {
          const r = ts.getVisibleRange();
          if (!r) return String(time);
          return fmt(time, r.to - r.from, ts.width());
        },
      });
    }
    let mod = draws.get(drawUrl);
    if (!mod) { mod = await import(drawUrl); draws.set(drawUrl, mod); }
    await mod.draw(chart, JSON.parse(dataJson), JSON.parse(viewSpec), lwc);
    return null;
  } catch (e) {
    return `⚠ chart error — ${(e && e.message) || e}`;
  }
}
