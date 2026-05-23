//! Shared application state threaded through all request handlers.

use crate::config::Config;
use crate::db::Pool;

/// Cheap-to-clone state injected into every axum handler via `State<AppState>`.
///
/// `Pool` is an `Arc`-backed r2d2 pool, so cloning is cheap.
/// `Config` is a small TOML-derived struct, also cheap to clone.
#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub config: Config,
}

impl AppState {
    pub fn new(pool: Pool, config: Config) -> Self {
        Self { pool, config }
    }
}
