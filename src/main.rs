//! Caxton HTTP Server

use caxton::create_app;

/// Application entrypoint
#[tokio::main]
async fn main() {
    let app = create_app();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
