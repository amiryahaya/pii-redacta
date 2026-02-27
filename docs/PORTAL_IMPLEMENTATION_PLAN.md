# Portal Implementation Plan

**Phase:** Post-MVP (Sprints 9-16)  
**Focus:** User Management, API Keys, Admin Portal  
**Tech Stack:** PostgreSQL + Leptos + Axum

---

## Sprint Overview

```
Sprint 9-10:  Database & Authentication Core
Sprint 11-12: API Key Management
Sprint 13-14: Admin Portal UI (Leptos)
Sprint 15-16: Enterprise Features (SSO, Teams, Audit)
```

---

## Sprint 9: Database Foundation & User Model

### TDD Deliverables

**Database Migration Tests:**
```rust
// tests/migration_test.rs
#[tokio::test]
async fn test_users_table_created() {
    let pool = setup_test_db().await;
    let result = sqlx::query("SELECT * FROM users LIMIT 1")
        .fetch_optional(&pool)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_user_insert_and_retrieve() {
    let pool = setup_test_db().await;
    let user = create_test_user(&pool).await;
    
    let retrieved = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(user.email, retrieved.email);
}
```

**Password Hashing Tests:**
```rust
#[test]
fn test_password_hashing() {
    let password = "SecurePass123!";
    let hash = hash_password(password).unwrap();
    
    assert_ne!(hash, password);
    assert!(verify_password(password, &hash).unwrap());
    assert!(!verify_password("WrongPass", &hash).unwrap());
}
```

### Implementation

1. **Set up SQLx with PostgreSQL**
   ```toml
   [dependencies]
   sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "migrate"] }
   ```

2. **Create migrations**
   ```sql
   -- migrations/001_create_users.sql
   CREATE TABLE users (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       email VARCHAR(255) UNIQUE NOT NULL,
       password_hash VARCHAR(255) NOT NULL,
       email_verified BOOLEAN DEFAULT FALSE,
       created_at TIMESTAMPTZ DEFAULT NOW(),
       updated_at TIMESTAMPTZ DEFAULT NOW()
   );
   ```

3. **User model with SQLx**
   ```rust
   #[derive(sqlx::FromRow)]
   pub struct User {
       pub id: Uuid,
       pub email: String,
       pub password_hash: String,
       pub email_verified: bool,
       pub created_at: DateTime<Utc>,
   }
   ```

### Sprint 9 Commit
```
feat(portal): sprint 9 - database foundation and user model

- Add PostgreSQL with SQLx integration
- Create users table with migrations
- Implement Argon2 password hashing
- Add user repository pattern
- Test coverage: 100%

Tests: 10 new tests passing
```

---

## Sprint 10: Authentication API

### TDD Deliverables

**Registration Tests:**
```rust
#[tokio::test]
async fn test_register_success() {
    let app = create_app().await;
    let response = register(&app, "user@example.com", "password123").await;
    
    assert_eq!(response.status(), StatusCode::CREATED);
    assert!(response.json::<RegisterResponse>().success);
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let app = create_app().await;
    register(&app, "user@example.com", "password123").await;
    let response = register(&app, "user@example.com", "password123").await;
    
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
```

**Login/JWT Tests:**
```rust
#[tokio::test]
async fn test_login_success() {
    let app = create_app().await;
    register(&app, "user@example.com", "password123").await;
    
    let response = login(&app, "user@example.com", "password123").await;
    assert_eq!(response.status(), StatusCode::OK);
    
    let token = response.json::<LoginResponse>().access_token;
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_protected_route_with_valid_token() {
    let app = create_app().await;
    let token = get_auth_token(&app).await;
    
    let response = app
        .get("/api/v1/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

### Endpoints

```
POST /api/v1/auth/register
POST /api/v1/auth/login
POST /api/v1/auth/refresh
POST /api/v1/auth/forgot-password
POST /api/v1/auth/reset-password
GET  /api/v1/me
```

### Sprint 10 Commit
```
feat(portal): sprint 10 - authentication api

- Add registration endpoint with validation
- Add login with JWT access/refresh tokens
- Add password reset flow
- Add /me endpoint for current user
- Rate limiting on auth endpoints

Tests: 15 new tests passing
```

---

## Sprint 11: API Key Management Backend

### TDD Deliverables

**Key Generation Tests:**
```rust
#[test]
fn test_api_key_generation() {
    let key = ApiKey::generate();
    
    assert!(key.plaintext.starts_with("pii_live_"));
    assert_eq!(key.hash.len(), 64); // SHA-256 hex
    assert!(!key.plaintext.is_empty());
}

#[tokio::test]
async fn test_api_key_storage() {
    let pool = setup_test_db().await;
    let user = create_test_user(&pool).await;
    
    let key = ApiKey::generate();
    let stored = store_api_key(&pool, user.id, &key).await.unwrap();
    
    assert_eq!(stored.user_id, user.id);
    assert_eq!(stored.key_hash, key.hash);
    // Plaintext should NOT be in database
    let raw_db: String = sqlx::query_scalar("SELECT key_hash FROM api_keys WHERE id = $1")
        .bind(stored.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_ne!(raw_db, key.plaintext);
}
```

**Permission Tests:**
```rust
#[tokio::test]
async fn test_api_key_permissions() {
    let pool = setup_test_db().await;
    let key = create_api_key_with_permissions(&pool, &["detect:read", "upload:write"]).await;
    
    assert!(key.has_permission("detect:read"));
    assert!(key.has_permission("upload:write"));
    assert!(!key.has_permission("admin"));
}
```

### Database Schema

```sql
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(8) NOT NULL,
    permissions JSONB NOT NULL DEFAULT '["detect:read"]',
    rate_limit_per_minute INTEGER DEFAULT 100,
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Endpoints

```
GET    /api/v1/api-keys           # List user's keys
POST   /api/v1/api-keys           # Create new key
DELETE /api/v1/api-keys/:id       # Revoke key
PATCH  /api/v1/api-keys/:id       # Update name/permissions
```

### Sprint 11 Commit
```
feat(portal): sprint 11 - api key management backend

- Add API key generation with secure hashing
- Add key permissions system
- Add rate limiting per key
- Add key lifecycle (create/revoke/update)
- Test coverage: 100%

Tests: 12 new tests passing
```

---

## Sprint 12: API Key Middleware & Integration

### TDD Deliverables

**Authentication Middleware Tests:**
```rust
#[tokio::test]
async fn test_api_key_authentication() {
    let app = create_app().await;
    let (_, key_plaintext) = create_api_key(&app).await;
    
    let response = app
        .post("/api/v1/detect")
        .header("X-API-Key", &key_plaintext)
        .json(&json!({"text": "test@example.com"}))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_api_key_rejected() {
    let app = create_app().await;
    
    let response = app
        .post("/api/v1/detect")
        .header("X-API-Key", "invalid_key_12345")
        .json(&json!({"text": "test"}))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_rate_limit_enforced() {
    let app = create_app().await;
    let (_, key) = create_api_key_with_limit(&app, 2).await;
    
    // First 2 requests succeed
    for _ in 0..2 {
        let resp = make_request(&app, &key).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    // 3rd request rate limited
    let resp = make_request(&app, &key).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}
```

### Implementation

1. **API Key Middleware**
   ```rust
   pub async fn api_key_auth(
       State(pool): State<PgPool>,
       headers: HeaderMap,
       mut request: Request<Body>,
       next: Next,
   ) -> Response<Body> {
       let api_key = headers
           .get("X-API-Key")
           .and_then(|v| v.to_str().ok())
           .ok_or_else(|| unauthorized())?;
       
       let key_data = validate_api_key(&pool, api_key).await?;
       
       // Add user info to request extensions
       request.extensions_mut().insert(key_data);
       
       next.run(request).await
   }
   ```

2. **Rate Limiting**
   ```rust
   use redis::AsyncCommands;
   
   pub async fn check_rate_limit(
       redis: &mut redis::aio::Connection,
       key_id: &str,
       limit: i32,
   ) -> Result<bool> {
       let current: i32 = redis
           .get(format!("rate_limit:{}", key_id))
           .await
           .unwrap_or(0);
       
       if current >= limit {
           return Ok(false);
       }
       
       redis.incr(format!("rate_limit:{}", key_id), 1).await?;
       redis.expire(format!("rate_limit:{}", key_id), 60).await?;
       
       Ok(true)
   }
   ```

### Sprint 12 Commit
```
feat(portal): sprint 12 - api key middleware and rate limiting

- Add API key authentication middleware
- Add Redis-based rate limiting per key
- Integrate API keys with existing endpoints
- Add rate limit headers (X-RateLimit-Remaining)

Tests: 15 new tests passing
```

---

## Sprint 13-14: Admin Portal UI (Leptos)

### TDD Deliverables

**Component Tests:**
```rust
#[wasm_bindgen_test]
fn test_login_form_validation() {
    mount_to_body(|| view! { <LoginForm /> });
    
    // Fill invalid email
    let email_input = document().query_selector("#email").unwrap().unwrap();
    email_input.set_value("invalid-email");
    
    // Submit
    let submit = document().query_selector("button[type=submit]").unwrap().unwrap();
    submit.click();
    
    // Check error message
    let error = document().query_selector(".error").unwrap().unwrap();
    assert!(error.text_content().unwrap().contains("Invalid email"));
}

#[wasm_bindgen_test]
async fn test_api_keys_list() {
    let mock_keys = vec![
        ApiKey { id: "1", name: "Production", ... },
        ApiKey { id: "2", name: "Development", ... },
    ];
    
    provide_context(MockApiService::new(mock_keys));
    
    mount_to_body(|| view! { <ApiKeysPage /> });
    
    let keys = document().query_selector_all(".api-key-card").unwrap();
    assert_eq!(keys.length(), 2);
}
```

### Pages to Build

1. **Login Page**
   - Email/password form
   - Error handling
   - Redirect after auth

2. **Dashboard**
   - Usage statistics
   - Recent activity
   - Quick actions

3. **API Keys Page**
   - List all keys
   - Create new key (with copy-to-clipboard)
   - Revoke key
   - Edit permissions

4. **Settings Page**
   - Profile info
   - Change password
   - Email preferences

### Sprint 13-14 Commit
```
feat(portal): sprint 13-14 - admin portal ui with leptos

- Add Leptos frontend framework
- Create login page with validation
- Create dashboard with usage stats
- Create API keys management page
- Add responsive design with Tailwind

Tests: 20 new tests passing
```

---

## Sprint 15-16: Enterprise Features

### Features

1. **Organization/Team Support**
   ```sql
   CREATE TABLE organizations (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name VARCHAR(255) NOT NULL,
       plan VARCHAR(50) NOT NULL DEFAULT 'free',
       max_users INTEGER DEFAULT 5,
       max_api_keys INTEGER DEFAULT 10
   );
   
   CREATE TABLE organization_members (
       organization_id UUID REFERENCES organizations(id),
       user_id UUID REFERENCES users(id),
       role VARCHAR(50) NOT NULL, -- 'owner', 'admin', 'member'
       PRIMARY KEY (organization_id, user_id)
   );
   ```

2. **Audit Logging**
   ```rust
   pub async fn log_audit_event(
       pool: &PgPool,
       event: AuditEvent,
   ) -> Result<()> {
       sqlx::query(
           "INSERT INTO audit_logs (user_id, action, resource, metadata) VALUES ($1, $2, $3, $4)"
       )
       .bind(event.user_id)
       .bind(event.action)
       .bind(event.resource)
       .bind(event.metadata)
       .execute(pool)
       .await
   }
   ```

3. **SSO Integration**
   - SAML 2.0 support
   - OIDC (Google, Microsoft, Okta)

4. **Advanced Rate Limiting**
   - Per-organization limits
   - Burst allowances
   - Geographic restrictions

### Sprint 15-16 Commit
```
feat(portal): sprint 15-16 - enterprise features

- Add organization/team support
- Add comprehensive audit logging
- Add SSO integration (SAML/OIDC)
- Add advanced rate limiting
- Add usage analytics

Tests: 25 new tests passing
```

---

## Summary Timeline

| Sprint | Focus | Deliverables | Tests |
|--------|-------|--------------|-------|
| 9 | Database | PostgreSQL + User model | 10 |
| 10 | Auth API | Register/Login/JWT | 15 |
| 11 | API Keys | Key generation backend | 12 |
| 12 | Middleware | Auth middleware + Rate limit | 15 |
| 13-14 | Portal UI | Leptos frontend | 20 |
| 15-16 | Enterprise | SSO, Teams, Audit | 25 |
| **Total** | | | **97** |

---

**Total Post-MVP Tests:** 97 new tests  
**Combined MVP + Portal:** 187 tests  
**Estimated Duration:** 16 weeks (4 months)

---

**Ready to start Sprint 9: Database Foundation?**
