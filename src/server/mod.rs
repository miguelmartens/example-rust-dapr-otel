//! HTTP server and state management.

mod memstore;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

pub use memstore::MemStore;

/// Trait for state store operations (Dapr or in-memory).
#[async_trait::async_trait]
pub trait StateClient: Send + Sync {
    async fn get_state(
        &self,
        store: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>>;
    async fn save_state(
        &self,
        store: &str,
        key: &str,
        value: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete_state(
        &self,
        store: &str,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Dapr client wrapper implementing StateClient.
pub struct DaprStateClient {
    client: Mutex<dapr::Client<dapr::client::TonicClient>>,
}

impl DaprStateClient {
    pub fn new(client: dapr::Client<dapr::client::TonicClient>) -> Self {
        Self {
            client: Mutex::new(client),
        }
    }
}

#[async_trait::async_trait]
impl StateClient for DaprStateClient {
    async fn get_state(
        &self,
        store: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut client = self.client.lock().await;
        let response = client
            .get_state(store, key, None)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        let data = response.data;
        Ok(if data.is_empty() { None } else { Some(data) })
    }

    async fn save_state(
        &self,
        store: &str,
        key: &str,
        value: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut client = self.client.lock().await;
        client
            .save_state(store, key, value, None, None, None)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        Ok(())
    }

    async fn delete_state(
        &self,
        store: &str,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut client = self.client.lock().await;
        client
            .delete_state(store, key, None)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        Ok(())
    }
}

/// App state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub state_client: Arc<dyn StateClient>,
    pub store_name: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/livez", get(livez))
        .route("/readyz", get(readyz))
        .route("/health", get(health))
        .route("/api/v1/state/:key", get(get_state))
        .route("/api/v1/state/:key", post(save_state))
        .route("/api/v1/state/:key", delete(delete_state))
        .with_state(state)
}

async fn livez() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn readyz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn get_state(State(app): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    if key.is_empty() {
        return (StatusCode::BAD_REQUEST, Body::from("missing key")).into_response();
    }

    match app.state_client.get_state(&app.store_name, &key).await {
        Ok(Some(value)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/octet-stream")],
            Body::from(value),
        )
            .into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Body::from("not found")).into_response(),
        Err(e) => {
            error!("get state failed: key={} err={}", key, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from("internal error"),
            )
                .into_response()
        }
    }
}

async fn save_state(
    State(app): State<AppState>,
    Path(key): Path<String>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    if key.is_empty() {
        return (StatusCode::BAD_REQUEST, Body::from("missing key")).into_response();
    }

    match app
        .state_client
        .save_state(&app.store_name, &key, body.to_vec())
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            error!("save state failed: key={} err={}", key, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from("internal error"),
            )
                .into_response()
        }
    }
}

async fn delete_state(State(app): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    if key.is_empty() {
        return (StatusCode::BAD_REQUEST, Body::from("missing key")).into_response();
    }

    match app.state_client.delete_state(&app.store_name, &key).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            error!("delete state failed: key={} err={}", key, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from("internal error"),
            )
                .into_response()
        }
    }
}
