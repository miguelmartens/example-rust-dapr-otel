//! In-memory state store for local development without Dapr.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

use super::StateClient;

/// In-memory state store for local dev when Dapr is unavailable.
#[derive(Debug, Default)]
pub struct MemStore {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl MemStore {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl StateClient for MemStore {
    async fn get_state(
        &self,
        _store: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let data = self
            .data
            .read()
            .map_err(|e| format!("lock poisoned: {}", e))?;
        Ok(data.get(key).cloned())
    }

    async fn save_state(
        &self,
        _store: &str,
        key: &str,
        value: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self
            .data
            .write()
            .map_err(|e| format!("lock poisoned: {}", e))?;
        data.insert(key.to_string(), value);
        Ok(())
    }

    async fn delete_state(
        &self,
        _store: &str,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self
            .data
            .write()
            .map_err(|e| format!("lock poisoned: {}", e))?;
        data.remove(key);
        Ok(())
    }
}
