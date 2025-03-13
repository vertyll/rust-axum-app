use std::net::SocketAddr;
use axum::Router;
use axum::routing::get;

#[tokio::main]
async fn main() {
    let routes_hello = Router::new().route("/hello", get(|| async { "Hello, World!" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, routes_hello).await.unwrap();
}