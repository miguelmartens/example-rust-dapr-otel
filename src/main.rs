//! Example Rust app with Dapr state store and OpenTelemetry.

mod config;
mod server;
mod telemetry;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = config::Config::load();

    // Initialize tracing (JSON logs to stdout)
    // When OTEL is configured, tracing spans will be exported via the global tracer provider
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?),
        )
        .init();

    // Initialize OpenTelemetry
    let shutdown_telemetry = telemetry::init(
        cfg.otel_exporter_endpoint.as_deref(),
        &cfg.otel_service_name,
    );

    // Wait for Dapr sidecar when DAPR_GRPC_PORT is set
    if std::env::var("DAPR_GRPC_PORT").is_ok() {
        wait_for_dapr().await;
    }

    // Create state client: Dapr or in-memory fallback
    let state_client: Arc<dyn server::StateClient> = match create_dapr_client().await {
        Ok(client) => Arc::new(server::DaprStateClient::new(client)),
        Err(e) => {
            info!(
                "Dapr unavailable, using in-memory store for local dev: {}",
                e
            );
            Arc::new(server::MemStore::new())
        }
    };

    let app_state = server::AppState {
        state_client,
        store_name: cfg.store_name.clone(),
    };

    let app = server::router(app_state).layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let listener = TcpListener::bind(addr).await?;

    info!(port = cfg.port, store = cfg.store_name, "server starting");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("server stopped");
    shutdown_telemetry();

    Ok(())
}

/// Poll Dapr outbound health until ready or timeout.
async fn wait_for_dapr() {
    let port = std::env::var("DAPR_HTTP_PORT").unwrap_or_else(|_| "3500".to_string());
    let url = format!("http://127.0.0.1:{}/v1.0/healthz/outbound", port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);

    while tokio::time::Instant::now() < deadline {
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                info!("Dapr sidecar ready");
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn create_dapr_client(
) -> Result<dapr::Client<dapr::client::TonicClient>, Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("DAPR_GRPC_PORT").unwrap_or_else(|_| "3500".to_string());

    let client = dapr::Client::<dapr::client::TonicClient>::connect_with_port(
        "https://127.0.0.1".to_string(),
        port,
    )
    .await
    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

    Ok(client)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutting down");
}
