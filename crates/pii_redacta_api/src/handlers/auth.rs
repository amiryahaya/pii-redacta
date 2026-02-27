//! Authentication handlers for PII Redacta API
//!
//! Provides endpoints for user registration, login, and profile management.

use crate::extractors::AuthUser;
use crate::jwt::generate_token;
use crate::AppState;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Auth error types
#[derive(Debug, thiserror::Error)]
pub enum AuthHandlerError {
    #[error("Email already exists")]
    EmailExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid current password")]
    InvalidCurrentPassword,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Password hashing error")]
    PasswordHashError,
    #[error("JWT error: {0}")]
    Jwt(#[from] crate::jwt::JwtError),
    #[error("Validation error: {0}")]
    Validation(String),
}

impl IntoResponse for AuthHandlerError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AuthHandlerError::EmailExists => (
                StatusCode::CONFLICT,
                "An account with this email already exists",
            ),
            AuthHandlerError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password")
            }
            AuthHandlerError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AuthHandlerError::InvalidCurrentPassword => {
                (StatusCode::BAD_REQUEST, "Current password is incorrect")
            }
            AuthHandlerError::PasswordHashError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Authentication error")
            }
            AuthHandlerError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred",
            ),
        };

        let body = serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

/// User registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub company_name: Option<String>,
}

/// User login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Update user profile request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
}

/// Change password request
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Auth response with user and token
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

/// User response structure
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    #[serde(rename = "emailNotificationsEnabled")]
    pub email_notifications_enabled: bool,
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Validate email format
fn validate_email(email: &str) -> Result<(), &'static str> {
    if email.len() < 5 {
        return Err("Email must be at least 5 characters");
    }
    if !email.contains('@') {
        return Err("Email must contain @");
    }
    // More strict validation using regex pattern
    let email_regex =
        regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !email_regex.is_match(email) {
        return Err("Invalid email format");
    }
    Ok(())
}

/// Validate password strength
fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long");
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("Password must contain at least one uppercase letter");
    }
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err("Password must contain at least one lowercase letter");
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Password must contain at least one number");
    }
    if !password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
    {
        return Err(
            "Password must contain at least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)",
        );
    }
    Ok(())
}

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AuthHandlerError> {
    // Validate email
    if let Err(e) = validate_email(&req.email) {
        return Err(AuthHandlerError::Validation(e.to_string()));
    }

    // Validate password
    if let Err(e) = validate_password(&req.password) {
        return Err(AuthHandlerError::Validation(e.to_string()));
    }

    // Validate display_name length if provided
    if let Some(ref name) = req.display_name {
        if name.len() > 100 {
            return Err(AuthHandlerError::Validation(
                "Display name must be less than 100 characters".to_string(),
            ));
        }
    }

    // Validate company_name length if provided
    if let Some(ref company) = req.company_name {
        if company.len() > 100 {
            return Err(AuthHandlerError::Validation(
                "Company name must be less than 100 characters".to_string(),
            ));
        }
    }

    // Check if email already exists
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1 AND deleted_at IS NULL")
        .bind(&req.email)
        .fetch_optional(state.db.pool())
        .await?;

    if existing.is_some() {
        return Err(AuthHandlerError::EmailExists);
    }

    // Hash password with Argon2id
    let password_hash = hash_password(&req.password)?;

    // Create user
    let user_id = Uuid::new_v4();
    let user = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, company_name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
        RETURNING id, email, password_hash, display_name, company_name, email_notifications_enabled, is_admin, created_at
        "#,
    )
    .bind(user_id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.display_name)
    .bind(&req.company_name)
    .fetch_one(state.db.pool())
    .await?;

    // Create trial subscription (14 days)
    let trial_tier = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM tiers WHERE name = 'trial'")
        .fetch_one(state.db.pool())
        .await?;

    sqlx::query(
        r#"
        INSERT INTO subscriptions (user_id, tier_id, status, current_period_start, current_period_end, created_at, updated_at)
        VALUES ($1, $2, 'trial', NOW(), NOW() + INTERVAL '14 days', NOW(), NOW())
        "#,
    )
    .bind(user_id)
    .bind(trial_tier.0)
    .execute(state.db.pool())
    .await?;

    // Generate JWT token
    let token = generate_token(user.id, &user.email, user.is_admin, &state.jwt_config)?;

    Ok(Json(AuthResponse {
        user: user.into(),
        token,
    }))
}

/// Login existing user
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AuthHandlerError> {
    // Validate email
    if let Err(e) = validate_email(&req.email) {
        return Err(AuthHandlerError::Validation(e.to_string()));
    }

    // Find user by email
    let user = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, password_hash, display_name, company_name, 
               email_notifications_enabled, is_admin, created_at
        FROM users 
        WHERE email = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(&req.email)
    .fetch_optional(state.db.pool())
    .await?;

    let user = match user {
        Some(u) => u,
        None => return Err(AuthHandlerError::InvalidCredentials),
    };

    // Verify password
    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AuthHandlerError::InvalidCredentials);
    }

    // Update last login
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(state.db.pool())
        .await?;

    // Generate JWT token
    let token = generate_token(user.id, &user.email, user.is_admin, &state.jwt_config)?;

    Ok(Json(AuthResponse {
        user: user.into(),
        token,
    }))
}

/// Get current user info
pub async fn me(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<UserResponse>, AuthHandlerError> {
    let user = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, password_hash, display_name, company_name, 
               email_notifications_enabled, is_admin, created_at
        FROM users 
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?;

    match user {
        Some(u) => Ok(Json(u.into())),
        None => Err(AuthHandlerError::UserNotFound),
    }
}

/// Change user password
pub async fn change_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AuthHandlerError> {
    // Validate new password
    if let Err(e) = validate_password(&req.new_password) {
        return Err(AuthHandlerError::Validation(e.to_string()));
    }

    // Get current password hash
    let row = sqlx::query_as::<_, (String,)>("SELECT password_hash FROM users WHERE id = $1")
        .bind(auth_user.user_id)
        .fetch_optional(state.db.pool())
        .await?;

    let (current_hash,) = match row {
        Some(r) => r,
        None => return Err(AuthHandlerError::UserNotFound),
    };

    // Verify current password
    if !verify_password(&req.current_password, &current_hash)? {
        return Err(AuthHandlerError::InvalidCurrentPassword);
    }

    // Hash new password
    let new_hash = hash_password(&req.new_password)?;

    // Update password
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&new_hash)
        .bind(auth_user.user_id)
        .execute(state.db.pool())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Update user profile
pub async fn update_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AuthHandlerError> {
    // Validate display_name length if provided
    if let Some(ref name) = req.display_name {
        if name.len() > 100 {
            return Err(AuthHandlerError::Validation(
                "Display name must be less than 100 characters".to_string(),
            ));
        }
    }

    // Validate company_name length if provided
    if let Some(ref company) = req.company_name {
        if company.len() > 100 {
            return Err(AuthHandlerError::Validation(
                "Company name must be less than 100 characters".to_string(),
            ));
        }
    }

    let user = sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE users
        SET display_name = COALESCE($1, display_name),
            company_name = COALESCE($2, company_name),
            updated_at = NOW()
        WHERE id = $3 AND deleted_at IS NULL
        RETURNING id, email, password_hash, display_name, company_name,
                  email_notifications_enabled, is_admin, created_at
        "#,
    )
    .bind(&req.display_name)
    .bind(&req.company_name)
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?;

    match user {
        Some(u) => Ok(Json(u.into())),
        None => Err(AuthHandlerError::UserNotFound),
    }
}

/// User preferences
#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreferences {
    #[serde(rename = "emailQuotaAlert")]
    pub email_quota_alert: bool,
    #[serde(rename = "emailSecurityAlert")]
    pub email_security_alert: bool,
    #[serde(rename = "emailMarketing")]
    pub email_marketing: bool,
    #[serde(rename = "emailMonthlyReport")]
    pub email_monthly_report: bool,
}

/// Get user notification preferences
pub async fn get_preferences(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<UserPreferences>, AuthHandlerError> {
    let row = sqlx::query_as::<_, (bool,)>(
        "SELECT email_notifications_enabled FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?;

    match row {
        Some((enabled,)) => Ok(Json(UserPreferences {
            email_quota_alert: enabled,
            email_security_alert: enabled,
            email_marketing: false,
            email_monthly_report: enabled,
        })),
        None => Err(AuthHandlerError::UserNotFound),
    }
}

/// Update user notification preferences
pub async fn update_preferences(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UserPreferences>,
) -> Result<Json<UserPreferences>, AuthHandlerError> {
    let any_enabled = req.email_quota_alert || req.email_security_alert || req.email_monthly_report;

    sqlx::query(
        r#"
        UPDATE users
        SET email_notifications_enabled = $1, updated_at = NOW()
        WHERE id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(any_enabled)
    .bind(auth_user.user_id)
    .execute(state.db.pool())
    .await?;

    Ok(Json(req))
}

/// Logout user (client-side token deletion, but we can track server-side if needed)
pub async fn logout() -> StatusCode {
    // In a stateless JWT system, logout is handled client-side by deleting the token
    // Optionally, we could add the token to a blacklist in Redis
    StatusCode::NO_CONTENT
}

/// Hash a password using Argon2id
fn hash_password(password: &str) -> Result<String, AuthHandlerError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AuthHandlerError::PasswordHashError)
}

/// Verify a password against a hash
fn verify_password(password: &str, hash: &str) -> Result<bool, AuthHandlerError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthHandlerError::PasswordHashError)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// User row from database
#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    display_name: Option<String>,
    company_name: Option<String>,
    email_notifications_enabled: bool,
    is_admin: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<UserRow> for UserResponse {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id.to_string(),
            email: row.email,
            display_name: row.display_name,
            company_name: row.company_name,
            email_notifications_enabled: row.email_notifications_enabled,
            is_admin: row.is_admin,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation_valid() {
        // Valid: has uppercase, lowercase, number, and special char
        assert!(validate_password("Test123!").is_ok());
        assert!(validate_password("MyP@ssw0rd").is_ok());
        assert!(validate_password("C0mpl3x!Pass").is_ok());
    }

    #[test]
    fn test_password_validation_too_short() {
        assert!(validate_password("T1!").is_err());
        assert!(validate_password("T1!aaaa").is_err()); // 7 chars
    }

    #[test]
    fn test_password_validation_no_uppercase() {
        assert!(validate_password("test123!").is_err());
    }

    #[test]
    fn test_password_validation_no_lowercase() {
        assert!(validate_password("TEST123!").is_err());
    }

    #[test]
    fn test_password_validation_no_number() {
        assert!(validate_password("TestPass!").is_err());
    }

    #[test]
    fn test_password_validation_no_special() {
        assert!(validate_password("Test1234").is_err());
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name@domain.co.uk").is_ok());
        assert!(validate_email("a@b.co").is_ok());

        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("test@").is_err());
        assert!(validate_email("a@b").is_err()); // TLD too short
    }
}
