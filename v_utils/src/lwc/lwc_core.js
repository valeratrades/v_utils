// Framework-neutral lightweight-charts core. Owns the chart instance (one per host element, reused
// across mounts so the visible time range survives redraws) and dispatches to an app-supplied
// `draw(chart, data, viewSpec, lib)` module fetched at runtime. The chart↔host boilerplate lives here;
// "what we chart" lives entirely in the app's draw module.

// Loaded lazily inside mount() via the app's import map — a static top-level import would become a
// hard dependency of the wasm glue, so a slow/failed CDN would stall the whole app before first paint.
let _lib;
const lib = () => (_lib ??= import('lightweight-charts'));

const charts = new WeakMap();
const draws = new Map();

// Returns null on success, or a banner string on any chart-side failure — the caller renders it and
// stays alive (under panic=abort a thrown/rejected promise crossing into wasm nukes the whole app).
export async function mount(el, drawUrl, dataJson, viewSpec) {
  try {
    const lwc = await lib();
    let chart = charts.get(el);
    if (!chart) {
      chart = lwc.createChart(el, { autoSize: true });
      charts.set(el, chart);
    }
    let mod = draws.get(drawUrl);
    if (!mod) { mod = await import(drawUrl); draws.set(drawUrl, mod); }
    await mod.draw(chart, JSON.parse(dataJson), JSON.parse(viewSpec), lwc);
    return null;
  } catch (e) {
    return `⚠ chart error — ${(e && e.message) || e}`;
  }
}
