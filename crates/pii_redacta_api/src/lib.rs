//! PII Redacta API Library
//!
//! REST API for PII detection and redaction.

pub mod auth;
pub mod config;
pub mod extractors;
pub mod handlers;
pub mod jobs;
pub mod jwt;
pub mod metrics;
pub mod middleware;

#[cfg(test)]
mod security_test;

use axum::{
    extract::State,
    middleware::{from_fn, from_fn_with_state, map_response, Next},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use handlers::JobQueue;
use jwt::JwtConfig;
use metrics::AppMetrics;
use pii_redacta_core::db::redis::RedisPool;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

/// Application state for auth-enabled routes
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<pii_redacta_core::db::Database>,
    pub jwt_config: JwtConfig,
    /// Base64-encoded server secret for API key HMAC (separate from JWT secret)
    pub api_key_secret: String,
    /// In-memory rate limiter for login/register endpoints
    pub rate_limiter: Arc<middleware::rate_limit::InMemoryRateLimiter>,
    /// In-memory job queue
    pub job_queue: Arc<JobQueue>,
    /// Application metrics
    pub metrics: Arc<AppMetrics>,
    /// Optional Redis connection pool
    pub redis: Option<Arc<RedisPool>>,
}

/// Create the API router (MVP version without auth)
pub fn create_app() -> Router {
    let job_queue = Arc::new(JobQueue::new());

    Router::new()
        .route("/health", get(handlers::health::health))
        .route("/metrics", get(handlers::metrics::metrics))
        .route("/api/v1/detect", post(handlers::detection::detect))
        .route("/api/v1/upload", post(handlers::upload::upload))
        .route("/api/v1/jobs/:job_id", get(handlers::jobs::get_job_status))
        .layer(create_cors_layer(None))
        .layer(map_response(middleware::security::security_headers))
        .layer(RequestBodyLimitLayer::new(
            middleware::security::DEFAULT_MAX_BODY_SIZE,
        ))
        .layer(from_fn(middleware::security::limit_body_size))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(true)
                        .level(Level::INFO),
                )
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .with_state(job_queue)
}

/// Create router with portal auth endpoints
///
/// Includes full middleware stack:
/// - JWT authentication on protected routes
/// - Request ID propagation
/// - Structured logging/tracing
/// - CORS support
/// - Security headers
/// - Body size limiting
pub async fn create_app_with_auth(
    db: Arc<pii_redacta_core::db::Database>,
    jwt_secret: &str,
    api_key_secret: &str,
    cors_origins: Option<Vec<String>>,
    redis_url: Option<&str>,
) -> Result<(Router, AppState), jwt::JwtError> {
    let jwt_config = JwtConfig::new(jwt_secret, 24)?;
    let rate_limiter = Arc::new(middleware::rate_limit::InMemoryRateLimiter::new());
    let job_queue = Arc::new(JobQueue::new());
    let metrics = Arc::new(AppMetrics::new());

    // Try to connect to Redis; log warning and continue without it on failure
    let redis = match redis_url {
        Some(url) => match RedisPool::new(url).await {
            Ok(pool) => {
                tracing::info!("Redis connection established");
                Some(Arc::new(pool))
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to connect to Redis, continuing without it");
                None
            }
        },
        None => None,
    };

    let state = AppState {
        db,
        jwt_config: jwt_config.clone(),
        api_key_secret: api_key_secret.to_string(),
        rate_limiter,
        job_queue,
        metrics,
        redis,
    };

    // Protected routes — require valid JWT token
    let protected = Router::new()
        // Detection (authenticated)
        .route(
            "/api/v1/detect",
            post(handlers::detection::detect_authenticated),
        )
        // Upload & Jobs (authenticated)
        .route(
            "/api/v1/upload",
            post(handlers::upload::upload_authenticated),
        )
        .route(
            "/api/v1/jobs/:job_id",
            get(handlers::jobs::get_job_status_authenticated),
        )
        // Auth (authenticated)
        .route("/api/v1/auth/me", get(handlers::auth::me))
        .route(
            "/api/v1/auth/change-password",
            post(handlers::auth::change_password),
        )
        // Users
        .route(
            "/api/v1/users/me",
            get(handlers::auth::me).put(handlers::auth::update_user),
        )
        .route(
            "/api/v1/users/profile",
            get(handlers::auth::me)
                .put(handlers::auth::update_user)
                .patch(handlers::auth::update_user),
        )
        .route(
            "/api/v1/users/me/password",
            put(handlers::auth::change_password),
        )
        .route(
            "/api/v1/users/me/preferences",
            get(handlers::auth::get_preferences)
                .put(handlers::auth::update_preferences)
                .patch(handlers::auth::update_preferences),
        )
        // Dashboard
        .route(
            "/api/v1/dashboard/stats",
            get(handlers::usage::get_dashboard_stats),
        )
        // API Keys
        .route(
            "/api/v1/api-keys",
            get(handlers::api_keys::list_api_keys).post(handlers::api_keys::create_api_key),
        )
        .route(
            "/api/v1/api-keys/:id",
            delete(handlers::api_keys::delete_api_key),
        )
        .route(
            "/api/v1/api-keys/:id/revoke",
            post(handlers::api_keys::revoke_api_key),
        )
        // Usage
        .route("/api/v1/usage/stats", get(handlers::usage::get_usage_stats))
        .route("/api/v1/usage/daily", get(handlers::usage::get_daily_usage))
        .route(
            "/api/v1/usage/summary",
            get(handlers::usage::get_usage_summary),
        )
        // Subscription
        .route(
            "/api/v1/subscription",
            get(handlers::subscription::get_subscription),
        )
        // Metrics (authenticated)
        .route("/metrics", get(handlers::metrics::metrics_authenticated))
        .route_layer(from_fn(move |req, next| {
            let config = jwt_config.clone();
            auth::jwt_auth_middleware(config, req, next)
        }));

    // Rate-limited auth routes (10 requests/minute per IP)
    let rate_limited_auth = Router::new()
        .route("/api/v1/auth/register", post(handlers::auth::register))
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route_layer(from_fn_with_state(
            state.clone(),
            login_rate_limit_middleware,
        ));

    // Public routes — no auth required
    let public = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/health/deep", get(handlers::health::health_deep))
        .route("/api/v1/auth/logout", post(handlers::auth::logout))
        .route("/api/v1/tiers", get(handlers::subscription::list_tiers))
        .merge(rate_limited_auth);

    // Merge public and protected, apply shared middleware
    let app = public
        .merge(protected)
        .with_state(state.clone())
        .layer(create_cors_layer(cors_origins))
        .layer(map_response(middleware::security::security_headers))
        .layer(RequestBodyLimitLayer::new(
            middleware::security::DEFAULT_MAX_BODY_SIZE,
        ))
        .layer(from_fn(middleware::security::limit_body_size))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(true)
                        .level(Level::INFO),
                )
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    Ok((app, state))
}

/// Create CORS layer based on configuration
///
/// Note: `allow_credentials(true)` is incompatible with wildcard `allow_origin(Any)` per
/// the CORS spec. When origins are `*`, credentials are disabled. (S9-1)
fn create_cors_layer(origins: Option<Vec<String>>) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::HeaderName::from_static("x-request-id"),
        ])
        .max_age(std::time::Duration::from_secs(3600));

    // Configure allowed origins — wildcard disables credentials
    match origins {
        Some(origins) if origins.iter().any(|o| o == "*") => cors.allow_origin(Any),
        Some(origins) => {
            let allowed_origins: Vec<axum::http::HeaderValue> =
                origins.into_iter().filter_map(|o| o.parse().ok()).collect();
            cors.allow_origin(allowed_origins).allow_credentials(true)
        }
        None => {
            // Default: allow common development ports with credentials
            cors.allow_origin([
                "http://localhost:3000".parse().unwrap(),
                "http://localhost:5173".parse().unwrap(),
                "http://127.0.0.1:3000".parse().unwrap(),
                "http://127.0.0.1:5173".parse().unwrap(),
            ])
            .allow_credentials(true)
        }
    }
}

/// Rate limit middleware for login/register endpoints.
/// Allows 10 requests per minute per IP address.
///
/// When Redis is available, uses distributed rate limiting; otherwise
/// falls back to the in-memory rate limiter.
async fn login_rate_limit_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> axum::response::Response {
    // Extract IP from X-Forwarded-For header or ConnectInfo.
    // Skip rate limiting when IP is indeterminate (e.g., in tests without ConnectInfo).
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            req.extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
        });

    if let Some(ref ip) = ip {
        let allowed = if let Some(ref redis) = state.redis {
            // Try Redis-based rate limiting
            let key = format!("rl:{}", ip);
            match redis.incr_with_expiry(&key, 60).await {
                Ok(count) => count <= 10,
                Err(e) => {
                    tracing::warn!(error = %e, "Redis rate limit failed, falling back to in-memory");
                    state.rate_limiter.check_ip(ip, 10, 60)
                }
            }
        } else {
            state.rate_limiter.check_ip(ip, 10, 60)
        };

        if !allowed {
            let body = serde_json::json!({
                "error": {
                    "code": 429,
                    "message": "Too many requests. Please try again later.",
                }
            });
            return (axum::http::StatusCode::TOO_MANY_REQUESTS, axum::Json(body)).into_response();
        }
    }

    next.run(req).await
}

/// Initialize tracing subscriber for structured logging
pub fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,pii_redacta_api=debug")),
        )
        .with(fmt::layer().json())
        .init();
}

/// Initialize tracing subscriber with pretty formatting (for development)
pub fn init_tracing_pretty() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,pii_redacta_api=debug")),
        )
        .with(fmt::layer().pretty())
        .init();
}
