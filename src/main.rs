// Scaffold: many types and fields are stubs; suppress noise until handlers are filled in.
#![allow(dead_code)]

mod auth;
mod extract;
mod error;
mod handlers;
mod response;
mod router;

#[tokio::main]
async fn main() {
    let app = router::build_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");
    println!("subsonic-nexus listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("Server error");
}
