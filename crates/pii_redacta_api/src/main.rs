//! PII Redacta API Server
//!
//! REST API for PII detection and redaction.

use pii_redacta_api::{config::Config, create_app_with_auth, init_tracing};
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for structured logging
    init_tracing();

    info!("Starting PII Redacta API server...");

    // Load and validate configuration
    let config = match Config::from_env() {
        Ok(cfg) => {
            if let Err(e) = cfg.validate() {
                error!(error = %e, "Configuration validation failed");
                return Err(e.into());
            }
            info!(config = ?cfg, "Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            error!(error = %e, "Failed to load configuration");
            return Err(e.into());
        }
    };

    // Initialize database connection
    let db = match pii_redacta_core::db::Database::new(&config.database.url).await {
        Ok(db) => {
            info!("Database connection established successfully");
            Arc::new(db)
        }
        Err(e) => {
            error!(error = %e, "Failed to connect to database");
            return Err(e.into());
        }
    };

    // Run database migrations (ignore errors if tables already exist)
    match db.migrate().await {
        Ok(_) => info!("Database migrations completed successfully"),
        Err(e) => {
            // Check if error is about existing tables (not a critical error)
            let err_str = e.to_string();
            if err_str.contains("already exists") {
                info!("Database migrations already applied, skipping");
            } else {
                error!(error = %e, "Failed to run database migrations");
                return Err(e.into());
            }
        }
    }

    // Create application router
    let app =
        match create_app_with_auth(db.clone(), &config.jwt.secret, Some(config.cors_origins())) {
            Ok(router) => router,
            Err(e) => {
                error!(error = %e, "Failed to create application router");
                return Err(e.into());
            }
        };

    // Bind to address
    let addr = config.server_addr()?;
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            let actual_addr = listener.local_addr()?;
            info!(
                host = %actual_addr.ip(),
                port = %actual_addr.port(),
                "API server listening"
            );
            listener
        }
        Err(e) => {
            error!(address = %addr, error = %e, "Failed to bind to address");
            return Err(e.into());
        }
    };

    // Setup graceful shutdown handler
    let shutdown_signal = create_shutdown_signal();

    // Start server with graceful shutdown
    info!("Server ready to accept connections");

    match axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
    {
        Ok(_) => {
            info!("Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "Server error");
            Err(e.into())
        }
    }
}

/// Create a shutdown signal handler
///
/// Handles SIGTERM and SIGINT signals for graceful shutdown
fn create_shutdown_signal() -> impl std::future::Future<Output = ()> {
    let ctrl_c = async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C signal, initiating graceful shutdown...");
            }
            Err(e) => {
                warn!(error = %e, "Failed to listen for Ctrl+C signal");
            }
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                if let Some(()) = signal.recv().await {
                    info!("Received SIGTERM signal, initiating graceful shutdown...");
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to listen for SIGTERM signal");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    async move {
        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }
}
