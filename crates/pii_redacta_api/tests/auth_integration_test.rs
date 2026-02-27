//! Authentication Integration Tests
//!
//! These tests require PostgreSQL to be running.
//! Run with: cargo test --test auth_integration_test

use pii_redacta_api::jwt::JwtConfig;

/// Test JWT token generation and validation
#[test]
fn test_jwt_generation_and_validation() {
    let config = JwtConfig::new("test-secret-key-at-least-32-bytes-long-for-tests", 24)
        .expect("Should create config with valid secret");
    let user_id = uuid::Uuid::new_v4();

    // Generate token
    let token = pii_redacta_api::jwt::generate_token(user_id, "test@example.com", false, &config)
        .expect("Should generate token");

    assert!(!token.is_empty());

    // Validate token
    let claims =
        pii_redacta_api::jwt::validate_token(&token, &config).expect("Should validate token");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, "test@example.com");
    assert!(!claims.is_admin);
}

#[test]
fn test_jwt_invalid_token() {
    let config = JwtConfig::new("test-secret-key-at-least-32-bytes-long", 24)
        .expect("Should create config with valid secret");
    let result = pii_redacta_api::jwt::validate_token("invalid.token.here", &config);
    assert!(result.is_err());
}

#[test]
fn test_jwt_wrong_secret() {
    let config1 = JwtConfig::new("secret-one-32-bytes-long-for-testing", 24)
        .expect("Should create config with valid secret");
    let config2 = JwtConfig::new("secret-two-32-bytes-long-for-testing", 24)
        .expect("Should create config with valid secret");
    let user_id = uuid::Uuid::new_v4();

    let token = pii_redacta_api::jwt::generate_token(user_id, "test@example.com", false, &config1)
        .expect("Should generate token");

    // Validate with wrong secret should fail
    let result = pii_redacta_api::jwt::validate_token(&token, &config2);
    assert!(result.is_err());
}

/// Test JWT secret validation
#[test]
fn test_jwt_secret_validation() {
    // Short secret should fail
    let result = JwtConfig::new("short", 24);
    assert!(result.is_err());

    // 32 byte secret should succeed
    let result = JwtConfig::new("this-is-exactly-32-bytes-long!!!", 24);
    assert!(result.is_ok());
}

/// Test password hashing with Argon2id
#[test]
fn test_password_hashing() {
    use argon2::{
        password_hash::{
            rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        },
        Argon2,
    };

    let password = "my-secure-password-123";

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Should hash password")
        .to_string();

    assert!(!password_hash.is_empty());
    assert!(password_hash.starts_with("$argon2id$"));

    // Verify password
    let parsed_hash = PasswordHash::new(&password_hash).expect("Should parse hash");
    assert!(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok());

    // Wrong password should fail
    assert!(Argon2::default()
        .verify_password("wrong-password".as_bytes(), &parsed_hash)
        .is_err());
}

/// Test JWT config
#[test]
fn test_jwt_config() {
    let config = JwtConfig::new("my-secret-key-must-be-32-bytes-long", 48)
        .expect("Should create config with valid secret");
    assert_eq!(config.secret(), "my-secret-key-must-be-32-bytes-long");
}

/// Test token extraction from header
#[test]
fn test_extract_token_from_header() {
    assert_eq!(
        pii_redacta_api::jwt::extract_token_from_header("Bearer my-token-123"),
        Some("my-token-123")
    );
    assert_eq!(
        pii_redacta_api::jwt::extract_token_from_header("Bearer  spaced-token  "),
        Some("spaced-token")
    );
    assert_eq!(
        pii_redacta_api::jwt::extract_token_from_header("Basic dXNlcjpwYXNz"),
        None
    );
    assert_eq!(pii_redacta_api::jwt::extract_token_from_header(""), None);
}
