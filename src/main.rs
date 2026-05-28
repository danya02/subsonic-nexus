// Scaffold: many types and fields are stubs; suppress noise until handlers are filled in.
#![allow(dead_code)]

mod auth;
mod config;
mod db;
mod error;
mod extract;
mod handlers;
mod nexus;
mod response;
mod router;
mod scanner;
mod state;

#[tokio::main]
async fn main() {
    // Default to INFO for this crate and tower_http when RUST_LOG is not set.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "subsonic_nexus=info,tower_http=info".into()),
        )
        .init();

    let config = config::Config::from_file("nexus.toml").expect("Failed to load nexus.toml");

    let pool = db::build_pool("nexus.db").await;

    let state = state::AppState::new(pool, config);
    let app = router::build_router(state.clone());

    // Background scan task: runs immediately on startup, then repeats every
    // `scan_interval_secs` seconds. Set to 0 to disable periodic rescanning.
    let scan_pool = state.pool.clone();
    let scan_servers = state.config.servers.clone();
    let scan_interval_secs = state.config.nexus.scan_interval_secs;
    tokio::spawn(async move {
        if scan_interval_secs == 0 {
            scanner::run_full_scan(&scan_pool, &scan_servers).await;
        } else {
            let period = std::time::Duration::from_secs(scan_interval_secs);
            let mut ticker = tokio::time::interval(period);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                ticker.tick().await; // first tick fires immediately
                scanner::run_full_scan(&scan_pool, &scan_servers).await;
            }
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");
    tracing::info!("subsonic-nexus listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("Server error");
}
