//! Application configuration
//!
//! Provides centralized configuration management for the API server.

use serde::Deserialize;
use std::net::SocketAddr;

/// Application configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,
    /// Redis configuration
    #[serde(default)]
    pub redis: RedisConfig,
    /// JWT configuration
    #[serde(default)]
    pub jwt: JwtConfig,
    /// CORS configuration
    #[serde(default)]
    pub cors: CorsConfig,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,
    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Maximum request body size in bytes
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_timeout() -> u64 {
    30
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            timeout_seconds: default_timeout(),
            max_body_size: default_max_body_size(),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    #[serde(default = "default_database_url")]
    pub url: String,
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_database_url() -> String {
    "postgres://localhost/pii_redacta".to_string()
}

fn default_max_connections() -> u32 {
    10
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_database_url(),
            max_connections: default_max_connections(),
        }
    }
}

/// Redis configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    #[serde(default = "default_redis_url")]
    pub url: String,
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: default_redis_url(),
        }
    }
}

/// JWT configuration
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT secret (minimum 32 bytes)
    #[serde(default = "default_jwt_secret")]
    pub secret: String,
    /// Token expiration time in hours
    #[serde(default = "default_jwt_expiration")]
    pub expiration_hours: i64,
}

fn default_jwt_secret() -> String {
    "your-secret-key-must-be-at-least-32-bytes-long".to_string()
}

fn default_jwt_expiration() -> i64 {
    24
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: default_jwt_secret(),
            expiration_hours: default_jwt_expiration(),
        }
    }
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins (comma-separated or "*" for all)
    #[serde(default = "default_cors_origins")]
    pub allowed_origins: String,
    /// Allowed methods (comma-separated)
    #[serde(default = "default_cors_methods")]
    pub allowed_methods: String,
    /// Allowed headers (comma-separated)
    #[serde(default = "default_cors_headers")]
    pub allowed_headers: String,
    /// Allow credentials
    #[serde(default = "default_allow_credentials")]
    pub allow_credentials: bool,
    /// Max age for preflight cache in seconds
    #[serde(default = "default_cors_max_age")]
    pub max_age: u64,
}

fn default_cors_origins() -> String {
    "http://localhost:3000,http://localhost:5173".to_string()
}

fn default_cors_methods() -> String {
    "GET,POST,PUT,DELETE,PATCH,OPTIONS".to_string()
}

fn default_cors_headers() -> String {
    "authorization,content-type,x-request-id".to_string()
}

fn default_allow_credentials() -> bool {
    true
}

fn default_cors_max_age() -> u64 {
    3600
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: default_cors_origins(),
            allowed_methods: default_cors_methods(),
            allowed_headers: default_cors_headers(),
            allow_credentials: default_allow_credentials(),
            max_age: default_cors_max_age(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute for authenticated users
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
    /// Requests per hour for unauthenticated IP-based limiting
    #[serde(default = "default_ip_requests_per_hour")]
    pub ip_requests_per_hour: u32,
}

fn default_requests_per_minute() -> u32 {
    60
}

fn default_ip_requests_per_hour() -> u32 {
    10
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: default_requests_per_minute(),
            ip_requests_per_hour: default_ip_requests_per_hour(),
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// Environment variables are prefixed with `PII_REDACTA_`.
    /// For example: `PII_REDACTA_SERVER_PORT` maps to `config.server.port`
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            // Override from environment variables
            .add_source(
                config::Environment::with_prefix("PII_REDACTA")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()?;

        settings.try_deserialize()
    }

    /// Get server socket address
    pub fn server_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.server.host, self.server.port).parse()
    }

    /// Get CORS allowed origins as a vector
    pub fn cors_origins(&self) -> Vec<String> {
        if self.cors.allowed_origins == "*" {
            vec!["*".to_string()]
        } else {
            self.cors
                .allowed_origins
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate JWT secret length
        if self.jwt.secret.len() < 32 {
            return Err(ConfigError::Validation(
                "JWT secret must be at least 32 bytes long".to_string(),
            ));
        }

        // Validate database URL
        if self.database.url.is_empty() {
            return Err(ConfigError::Validation(
                "Database URL cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration error: {0}")]
    Validation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.jwt.expiration_hours, 24);
    }

    #[test]
    fn test_cors_origins_parsing() {
        let config = CorsConfig {
            allowed_origins: "http://localhost:3000, http://localhost:5173".to_string(),
            ..Default::default()
        };
        let origins = config.allowed_origins.split(',').collect::<Vec<_>>();
        assert_eq!(origins.len(), 2);
    }

    #[test]
    fn test_validate_jwt_secret() {
        let config = Config {
            jwt: JwtConfig {
                secret: "short".to_string(),
                ..Default::default()
            },
            ..Config::default()
        };
        assert!(config.validate().is_err());

        let config = Config {
            jwt: JwtConfig {
                secret: "this-is-a-secure-secret-key-that-is-long-enough".to_string(),
                ..Default::default()
            },
            ..Config::default()
        };
        assert!(config.validate().is_ok());
    }
}
