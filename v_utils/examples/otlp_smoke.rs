//! Smoke test for the `otlp` feature: emits one span + one error log over OTLP.
//! Run with OTEL_EXPORTER_OTLP_ENDPOINT / OTEL_SERVICE_NAME set at a collector.
use tracing::{error, info, info_span};

fn main() {
	v_utils::utils::init_subscriber(v_utils::utils::LogDestination::default().stderr_errors(true));
	{
		let span = info_span!("otlp_smoke_span", kind = "test");
		let _g = span.enter();
		info!("otlp smoke: info line");
		error!(check = "delivery", "otlp smoke: error line");
	} // span closes here so the batch exporter can pick it up
	// let the batch exporters flush before the process exits
	std::thread::sleep(std::time::Duration::from_secs(7));
}
