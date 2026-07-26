//! Framework-neutral lightweight-charts interop boilerplate. A thin JS core (`lwc_core.js`, bundled
//! as a wasm-bindgen snippet) owns the chart instance and dispatches to an app-supplied
//! `draw(chart, data, viewSpec)` module served by the consumer at `draw_url`. Callers wire their own
//! reactive effect around [`mount`]; no leptos/dioxus dependency lives here.

mod time_ticks;
pub use time_ticks::format_time_tick;

#[cfg(target_arch = "wasm32")]
mod imp {
	use std::cell::OnceCell;

	use wasm_bindgen::prelude::*;

	#[wasm_bindgen(module = "/src/lwc/lwc_core.js")]
	extern "C" {
		#[wasm_bindgen(catch, js_name = mount)]
		async fn mount_js(el: web_sys::HtmlElement, draw_url: &str, data_json: &str, view_spec: &str, fmt: &js_sys::Function) -> Result<JsValue, JsValue>;
	}

	thread_local! {
		// Leaked once: stateless formatter, single-threaded wasm — cheaper than reallocating the closure
		// each mount and it must outlive every tickMarkFormatter call it's handed to.
		static TICK_FMT: OnceCell<js_sys::Function> = const { OnceCell::new() };
	}

	fn tick_fmt() -> js_sys::Function {
		TICK_FMT.with(|c| {
			c.get_or_init(|| {
				let cb = Closure::<dyn Fn(f64, f64, f64) -> String>::new(super::format_time_tick);
				cb.into_js_value().unchecked_into()
			})
			.clone()
		})
	}

	/// Render `data_json` into a (reused) chart on `el`, dispatching to the JS `draw` module served at
	/// `draw_url`. `view_spec` is an opaque JSON blob forwarded verbatim to `draw`. Returns a banner
	/// string on a chart-side error, else `None`.
	pub async fn mount(el: web_sys::HtmlElement, draw_url: &str, data_json: &str, view_spec: &str) -> Option<String> {
		match mount_js(el, draw_url, data_json, view_spec, &tick_fmt()).await {
			Ok(v) => v.as_string(),
			Err(e) => Some(format!("⚠ chart mount failed — {e:?}")),
		}
	}
}

#[cfg(target_arch = "wasm32")]
pub use imp::mount;
