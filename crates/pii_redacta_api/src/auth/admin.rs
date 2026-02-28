//! Admin verification middleware (S12-2b)
//!
//! Runs after JWT authentication to verify the user is actually an admin
//! by checking the database, with optional Redis caching.

use crate::extractors::{AdminUser, AuthUser};
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

/// Middleware that verifies admin status from the database.
///
/// Must run after `jwt_auth_middleware_with_state` which inserts `AuthUser`.
/// On success, inserts `AdminUser` into request extensions.
pub async fn admin_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Get AuthUser from extensions (set by JWT middleware)
    let auth_user = request
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or_else(|| {
            let body = serde_json::json!({
                "error": { "code": 401, "message": "Authentication required." }
            });
            (StatusCode::UNAUTHORIZED, Json(body)).into_response()
        })?;

    let user_id = auth_user.user_id;

    // Check Redis cache first (only positive results are cached — M5)
    let is_admin = if let Some(ref redis) = state.redis {
        let cache_key = format!("admin:{}", user_id);
        match redis.get_i64(&cache_key).await {
            Ok(Some(1)) => true,
            _ => {
                // Cache miss, negative, or error — query DB
                let db_admin = check_admin_in_db(&state, user_id).await;
                // Only cache positive admin status to avoid stale denial after promotion
                if db_admin {
                    let redis = redis.clone();
                    let cache_key_owned = cache_key;
                    tokio::spawn(async move {
                        let _ = redis.set_with_expiry(&cache_key_owned, 1i64, 60).await;
                    });
                }
                db_admin
            }
        }
    } else {
        check_admin_in_db(&state, user_id).await
    };

    if !is_admin {
        let body = serde_json::json!({
            "error": { "code": 403, "message": "Forbidden. Admin access required." }
        });
        return Err((StatusCode::FORBIDDEN, Json(body)).into_response());
    }

    // Insert AdminUser into extensions
    request.extensions_mut().insert(AdminUser {
        user_id: auth_user.user_id,
        email: auth_user.email,
    });

    Ok(next.run(request).await)
}

/// Query database for admin status.
async fn check_admin_in_db(state: &AppState, user_id: uuid::Uuid) -> bool {
    sqlx::query_scalar::<_, bool>("SELECT is_admin FROM users WHERE id = $1 AND deleted_at IS NULL")
        .bind(user_id)
        .fetch_optional(state.db.pool())
        .await
        .ok()
        .flatten()
        .unwrap_or(false)
}
