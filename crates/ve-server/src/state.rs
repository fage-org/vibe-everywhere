//! Application State
//!
//! Shared state for the server including database, WebSocket hub, and configuration.

use std::sync::Arc;

use crate::config::Config;
use crate::db::DbPool;
use crate::hub::Hub;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db: DbPool,

    /// WebSocket hub for real-time communication
    pub hub: Arc<Hub>,

    /// Server configuration
    pub config: Arc<Config>,
}

impl AppState {
    /// Create a new application state
    pub fn new(db: DbPool, hub: Hub, config: Config) -> Self {
        Self {
            db,
            hub: Arc::new(hub),
            config: Arc::new(config),
        }
    }
}
