//! PII Redacta API Server
//!
//! REST API for PII detection and redaction.

use pii_redacta_api::create_app;

#[tokio::main]
async fn main() {
    let app = create_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!(
        "PII Redacta API listening on {}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}
