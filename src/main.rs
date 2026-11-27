use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};
use std::path::Path;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    //routing to example index.html file
    let app = Router::new()
        .route_service("/", ServeFile::new("assets/index.html"))
        .fallback_service(ServeDir::new(require_client_dir()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn start_round() -> impl IntoResponse {}

async fn submit_guess() -> impl IntoResponse {}

async fn calculate_score() -> impl IntoResponse {}

fn require_client_dir() -> &'static Path {
    let path = Path::new("client/");
    if !path.is_dir() {
        panic!("client directory does not exist, cannot display website")
    }
    path
}
