//! Framework-neutral lightweight-charts interop boilerplate. A thin JS core (`lwc_core.js`, bundled
//! as a wasm-bindgen snippet) owns the chart instance and dispatches to an app-supplied
//! `draw(chart, data, viewSpec)` module served by the consumer at `draw_url`. Callers wire their own
//! reactive effect around [`mount`]; no leptos/dioxus dependency lives here.

#[cfg(target_arch = "wasm32")]
mod imp {
	use wasm_bindgen::prelude::*;

	#[wasm_bindgen(module = "/src/lwc/lwc_core.js")]
	extern "C" {
		#[wasm_bindgen(catch, js_name = mount)]
		async fn mount_js(el: web_sys::HtmlElement, draw_url: &str, data_json: &str, view_spec: &str) -> Result<JsValue, JsValue>;
	}

	/// Render `data_json` into a (reused) chart on `el`, dispatching to the JS `draw` module served at
	/// `draw_url`. `view_spec` is an opaque JSON blob forwarded verbatim to `draw`. Returns a banner
	/// string on a chart-side error, else `None`.
	pub async fn mount(el: web_sys::HtmlElement, draw_url: &str, data_json: &str, view_spec: &str) -> Option<String> {
		match mount_js(el, draw_url, data_json, view_spec).await {
			Ok(v) => v.as_string(),
			Err(e) => Some(format!("⚠ chart mount failed — {e:?}")),
		}
	}
}

#[cfg(target_arch = "wasm32")]
pub use imp::mount;
