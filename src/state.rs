//! Shared application state threaded through all request handlers.

use crate::config::Config;
use crate::db::DbPool;

/// Cheap-to-clone state injected into every axum handler via `State<AppState>`.
///
/// `DbPool` is an `Arc`-backed bb8 pool, so cloning is cheap.
/// `Config` is a small TOML-derived struct, also cheap to clone.
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Config,
}

impl AppState {
    pub fn new(pool: DbPool, config: Config) -> Self {
        Self { pool, config }
    }
}
