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
    /// API key configuration
    #[serde(default)]
    pub api_key: ApiKeyConfig,
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
    /// Comma-separated list of trusted proxy IP addresses (S12-3a).
    /// When set, X-Forwarded-For is only trusted from these IPs.
    /// Example: "10.0.0.1,10.0.0.2"
    #[serde(default)]
    pub trusted_proxies: String,
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
            trusted_proxies: String::new(),
        }
    }
}

impl ServerConfig {
    /// Parse the trusted_proxies string into a Vec<IpAddr> (S12-3a).
    pub fn parse_trusted_proxies(&self) -> Vec<std::net::IpAddr> {
        if self.trusted_proxies.is_empty() {
            return Vec::new();
        }
        self.trusted_proxies
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return None;
                }
                match trimmed.parse::<std::net::IpAddr>() {
                    Ok(ip) => Some(ip),
                    Err(_) => {
                        tracing::warn!("Invalid trusted proxy IP: {}", trimmed);
                        None
                    }
                }
            })
            .collect()
    }
}

/// Database configuration (S9-R3-15: custom Debug redacts credentials)
#[derive(Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    #[serde(default = "default_database_url")]
    pub url: String,
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

impl std::fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Redact credentials from the URL (S9-R3-15)
        let redacted_url = if let Some(at_pos) = self.url.find('@') {
            if let Some(scheme_end) = self.url.find("://") {
                format!(
                    "{}://***:***{}",
                    &self.url[..scheme_end],
                    &self.url[at_pos..]
                )
            } else {
                "[redacted]".to_string()
            }
        } else {
            self.url.clone()
        };
        f.debug_struct("DatabaseConfig")
            .field("url", &redacted_url)
            .field("max_connections", &self.max_connections)
            .finish()
    }
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

/// Redis configuration (S9-R4-01: custom Debug redacts credentials)
#[derive(Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    #[serde(default = "default_redis_url")]
    pub url: String,
}

impl std::fmt::Debug for RedisConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let redacted_url = if let Some(at_pos) = self.url.find('@') {
            if let Some(scheme_end) = self.url.find("://") {
                format!("{}://***{}", &self.url[..scheme_end], &self.url[at_pos..])
            } else {
                "[redacted]".to_string()
            }
        } else {
            self.url.clone()
        };
        f.debug_struct("RedisConfig")
            .field("url", &redacted_url)
            .finish()
    }
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

/// JWT configuration (S9-R3-09: custom Debug redacts secret)
#[derive(Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT secret (minimum 32 bytes)
    #[serde(default = "default_jwt_secret")]
    pub secret: String,
    /// Token expiration time in hours
    #[serde(default = "default_jwt_expiration")]
    pub expiration_hours: i64,
}

impl std::fmt::Debug for JwtConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtConfig")
            .field("secret", &"[redacted]")
            .field("expiration_hours", &self.expiration_hours)
            .finish()
    }
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

/// API key HMAC configuration (S9-R3-09: custom Debug redacts secret)
#[derive(Clone, Deserialize)]
pub struct ApiKeyConfig {
    /// Base64-encoded server secret for API key HMAC (minimum 32 bytes decoded)
    /// Generate with: openssl rand -base64 32
    #[serde(default = "default_api_key_secret")]
    pub secret: String,
}

impl std::fmt::Debug for ApiKeyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiKeyConfig")
            .field("secret", &"[redacted]")
            .finish()
    }
}

fn default_api_key_secret() -> String {
    // Default for development only — MUST be overridden in production
    "dGVzdC1zZWNyZXQtMzItYnl0ZXMtbG9uZy1rZXktZm9yLWhtYWM=".to_string()
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            secret: default_api_key_secret(),
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

        // S9-R3-10: Warn if JWT secret is still the default
        if self.jwt.secret == default_jwt_secret() {
            tracing::warn!(
                "JWT secret is set to the default value. Set PII_REDACTA_JWT_SECRET in production."
            );
        }

        // S9-R3-14: Validate API key secret is non-empty and looks like base64
        if self.api_key.secret.is_empty() {
            return Err(ConfigError::Validation(
                "API key secret cannot be empty".to_string(),
            ));
        }
        // Base64-encoded 32 bytes = at least 44 characters
        if self.api_key.secret.len() < 44 {
            return Err(ConfigError::Validation(
                "API key secret is too short (must be base64-encoded 32+ bytes)".to_string(),
            ));
        }

        // S9-R3-10: Warn if API key secret is still the default
        if self.api_key.secret == default_api_key_secret() {
            tracing::warn!(
                "API key secret is set to the default value. Set PII_REDACTA_API_KEY_SECRET in production."
            );
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

    #[test]
    fn test_validate_api_key_secret() {
        // Too short
        let config = Config {
            api_key: ApiKeyConfig {
                secret: "short".to_string(),
            },
            ..Config::default()
        };
        assert!(config.validate().is_err());

        // Empty
        let config = Config {
            api_key: ApiKeyConfig {
                secret: String::new(),
            },
            ..Config::default()
        };
        assert!(config.validate().is_err());

        // Valid (default secret is 52 chars, meets minimum)
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_debug_redacts_secrets() {
        let config = Config::default();
        let debug_output = format!("{:?}", config);
        // Secrets should be redacted in debug output
        assert!(debug_output.contains("[redacted]"));
        assert!(!debug_output.contains(&default_jwt_secret()));
        assert!(!debug_output.contains(&default_api_key_secret()));
    }

    #[test]
    fn test_config_parses_trusted_proxies() {
        let config = ServerConfig {
            trusted_proxies: "10.0.0.1,192.168.1.1".to_string(),
            ..Default::default()
        };
        let proxies = config.parse_trusted_proxies();
        assert_eq!(proxies.len(), 2);
        assert_eq!(proxies[0], "10.0.0.1".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(
            proxies[1],
            "192.168.1.1".parse::<std::net::IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_config_parses_empty_trusted_proxies() {
        let config = ServerConfig::default();
        let proxies = config.parse_trusted_proxies();
        assert!(proxies.is_empty());
    }

    #[test]
    fn test_config_parses_trusted_proxies_with_spaces() {
        let config = ServerConfig {
            trusted_proxies: " 10.0.0.1 , 192.168.1.1 ".to_string(),
            ..Default::default()
        };
        let proxies = config.parse_trusted_proxies();
        assert_eq!(proxies.len(), 2);
    }

    #[test]
    fn test_database_debug_redacts_credentials() {
        let db_config = DatabaseConfig {
            url: "postgres://user:password@localhost/db".to_string(),
            max_connections: 5,
        };
        let debug_output = format!("{:?}", db_config);
        assert!(!debug_output.contains("password"));
        assert!(debug_output.contains("***"));
    }
}
