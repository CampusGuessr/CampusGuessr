use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    //routing to example index.html file
    let app = Router::new()
        .route("/", get(handler))
        //        .route_service("/0", ServeFile::new("assets/index.html"))
        .fallback_service(ServeDir::new("assets/"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>hello,world<h1>")
}
