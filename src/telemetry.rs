//! OpenTelemetry trace and metric initialization.

use opentelemetry::global;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator, resource::Resource, runtime::Tokio, trace::TracerProvider,
};
use tracing::info;

/// Initialize OpenTelemetry trace and metric providers.
/// When `endpoint` is None or empty, uses no-op providers.
/// Returns a shutdown function to flush and close exporters.
pub fn init(endpoint: Option<&str>, service_name: &str) -> Box<dyn FnOnce() + Send> {
    let endpoint = endpoint.and_then(|s| {
        let s = s.trim();
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    });

    if endpoint.is_none() {
        info!("OTEL_EXPORTER_OTLP_ENDPOINT not set, using no-op telemetry");
        return Box::new(|| {});
    }

    let endpoint = endpoint.unwrap();
    let service_name = service_name.to_string();

    // Build full endpoint URLs (OTLP HTTP uses /v1/traces and /v1/metrics)
    let traces_endpoint = format!("{}/v1/traces", ensure_http(&endpoint));
    let metrics_endpoint = format!("{}/v1/metrics", ensure_http(&endpoint));

    let resource = Resource::new([opentelemetry::KeyValue::new(
        "service.name",
        service_name.clone(),
    )]);

    // Initialize tracer (global owns it; use shutdown_tracer_provider for cleanup)
    let tracer_initialized = match init_tracer(&traces_endpoint, resource.clone()) {
        Ok(tp) => {
            global::set_tracer_provider(tp);
            global::set_text_map_propagator(TraceContextPropagator::new());
            true
        }
        Err(e) => {
            tracing::warn!("failed to init tracer, using no-op: {}", e);
            false
        }
    };

    // Initialize meter (metrics) - global owns it; provider will flush on drop
    let meter_initialized = match init_meter(&metrics_endpoint, resource) {
        Ok(mp) => {
            global::set_meter_provider(mp);
            true
        }
        Err(e) => {
            tracing::warn!("failed to init meter, using no-op: {}", e);
            false
        }
    };

    Box::new(move || {
        if tracer_initialized {
            global::shutdown_tracer_provider();
        }
        // Meter provider has no global shutdown; SdkMeterProvider flushes on drop when global is replaced
        let _ = meter_initialized;
    })
}

fn init_tracer(
    endpoint: &str,
    resource: Resource,
) -> Result<TracerProvider, Box<dyn std::error::Error + Send + Sync>> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .with_protocol(Protocol::HttpBinary)
        .build()?;

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, Tokio)
        .with_resource(resource)
        .build();

    Ok(provider)
}

fn init_meter(
    endpoint: &str,
    resource: Resource,
) -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, Box<dyn std::error::Error + Send + Sync>>
{
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .with_protocol(Protocol::HttpBinary)
        .build()?;

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(
        exporter,
        opentelemetry_sdk::runtime::Tokio,
    )
    .with_interval(std::time::Duration::from_secs(10))
    .build();

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_resource(resource)
        .with_reader(reader)
        .build();

    Ok(provider)
}

fn ensure_http(endpoint: &str) -> String {
    let endpoint = endpoint.trim();
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.to_string()
    } else {
        format!("http://{}", endpoint)
    }
}
