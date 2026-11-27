use axum::{
    Router,
    response::{Html, IntoResponse, Json},
    routing::get,
};
use serde::Serialize;
use std::path::Path;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    //routing to example index.html file
    let app = Router::new()
        .route("/api/example", get(example_api_route))
        //pictures can be accessed in html by using pictures/"name"
        .nest_service("/pictures", ServeDir::new("data/photos"))
        .fallback_service(ServeDir::new(require_client_dir()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn require_client_dir() -> &'static Path {
    let path = Path::new("client/");
    if !path.is_dir() {
        panic!("client directory does not exist, cannot display website")
    }
    path
}

async fn example_api_route() -> Json<Round> {
    let example_round = Round {
        image_id: 12345,
        image_path: "/pictures/example_image.jpg".to_string(),
    };
    Json(example_round)
}

#[derive(Serialize)]
struct Round {
    image_id: u32,
    image_path: String,
}

async fn start_round() -> impl IntoResponse {}

async fn submit_guess() -> impl IntoResponse {}

async fn calculate_score() -> impl IntoResponse {}
