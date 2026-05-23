// Scaffold: many types and fields are stubs; suppress noise until handlers are filled in.
#![allow(dead_code)]

mod auth;
mod config;
mod db;
mod error;
mod extract;
mod handlers;
mod response;
mod router;
mod scanner;
mod state;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_file("nexus.toml").expect("Failed to load nexus.toml");

    let pool = db::build_pool("nexus.db");

    let state = state::AppState::new(pool, config);
    let app = router::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");
    println!("subsonic-nexus listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("Server error");
}
