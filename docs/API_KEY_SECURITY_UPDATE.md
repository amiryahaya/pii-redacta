# API Key Security Enhancement

**Status:** Security Update Required  
**Priority:** High  
**Current:** SHA-256 | **Recommended:** HMAC-SHA256 + Secret

---

## Why Move Away from SHA-256?

### SHA-256 Problems for API Keys

```
┌─────────────────────────────────────────────────────────────┐
│  SHA-256 RISKS                                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Rainbow Table Attack                                    │
│     - Attacker pre-computes SHA-256 of common keys         │
│     - If DB leaks, keys can be cracked quickly             │
│                                                             │
│  2. No Server Secret                                        │
│     - Hash is deterministic (same input = same output)     │
│     - No way to invalidate all keys if DB leaked           │
│                                                             │
│  3. Fast Computation                                        │
│     - Attackers can brute-force billions of attempts/sec   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Recommended: HMAC-SHA256 with Server Secret

### Why HMAC-SHA256?

| Feature | SHA-256 | HMAC-SHA256 |
|---------|---------|-------------|
| Rainbow table resistant | ❌ No | ✅ Yes (needs secret) |
| Can rotate secrets | ❌ No | ✅ Yes |
| Brute force resistant | ❌ No | ✅ Yes |
| Verification speed | ⚡ Fast | ⚡ Fast |
| Industry standard | ✅ Yes | ✅ Yes (Stripe, AWS) |

### Implementation

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use rand::Rng;
use base64::{Engine as _, engine::general_purpose};

// Type alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

pub struct ApiKey {
    pub prefix: String,        // "pii_live" - identifies key type
    pub identifier: String,    // "abc123" - first 8 chars for UI
    pub plaintext: String,     // Full key shown ONCE to user
    pub hash: String,          // HMAC hash stored in DB
}

impl ApiKey {
    /// Generate a new API key
    pub fn generate(key_secret: &[u8]) -> Self {
        let prefix = "pii_live";
        
        // 32 bytes = 256 bits of entropy
        let random_bytes: Vec<u8> = (0..32)
            .map(|_| rand::thread_rng().gen())
            .collect();
        
        let random_part = general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);
        let identifier = &random_part[..8]; // First 8 chars for identification
        
        let plaintext = format!("{}_{}_{}", prefix, identifier, random_part);
        
        // Compute HMAC-SHA256
        let mut mac = HmacSha256::new_from_slice(key_secret)
            .expect("HMAC can take key of any size");
        mac.update(plaintext.as_bytes());
        let hash_bytes = mac.finalize().into_bytes();
        let hash = general_purpose::STANDARD.encode(hash_bytes);
        
        Self {
            prefix: prefix.to_string(),
            identifier: identifier.to_string(),
            plaintext,
            hash,
        }
    }
    
    /// Verify an API key against stored hash
    pub fn verify(provided_key: &str, stored_hash: &str, key_secret: &[u8]) -> bool {
        let mut mac = HmacSha256::new_from_slice(key_secret)
            .expect("HMAC can take key of any size");
        mac.update(provided_key.as_bytes());
        let computed_hash = general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        
        // Constant-time comparison to prevent timing attacks
        computed_hash == stored_hash
    }
}

// Usage in your application
lazy_static::lazy_static! {
    // Load from environment variable or secure key management
    static ref API_KEY_SECRET: Vec<u8> = std::env::var("API_KEY_SECRET")
        .expect("API_KEY_SECRET must be set")
        .into_bytes();
}
```

### Database Schema Update

```sql
-- Updated api_keys table
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    
    -- HMAC-SHA256 hash (NOT plain SHA-256)
    key_hash VARCHAR(255) NOT NULL,
    
    -- First 8 chars for UI identification
    key_identifier VARCHAR(8) NOT NULL,
    
    -- Key type/prefix
    key_prefix VARCHAR(20) NOT NULL DEFAULT 'pii_live',
    
    -- Permissions and limits
    permissions JSONB NOT NULL DEFAULT '["detect:read"]',
    rate_limit_per_minute INTEGER DEFAULT 100,
    
    -- Lifecycle
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    
    UNIQUE(user_id, key_identifier)
);

-- Index for fast lookup
CREATE INDEX idx_api_keys_lookup ON api_keys(key_prefix, key_identifier);
```

---

## Alternative: BLAKE3 (Performance-Focused)

If you need even faster verification (millions of API calls/sec):

```rust
use blake3::Hasher;

pub fn hash_api_key(key: &str, secret: &[u8]) -> String {
    let mut hasher = Hasher::new_keyed(secret.try_into().unwrap());
    hasher.update(key.as_bytes());
    hasher.finalize().to_hex().to_string()
}
```

**BLAKE3 vs HMAC-SHA256:**
- BLAKE3: ~2x faster, modern design
- HMAC-SHA256: More widely audited, FIPS compliant

**Recommendation:** Use HMAC-SHA256 for enterprise/FIPS compliance, BLAKE3 for performance.

---

## Secret Rotation Strategy

```rust
/// Rotate API key secret (invalidate all old keys)
pub fn rotate_secret() {
    // 1. Generate new secret
    let new_secret = generate_secure_random(32);
    
    // 2. Store new secret in secure key management (AWS KMS, HashiCorp Vault)
    store_secret("api_key_secret_v2", &new_secret);
    
    // 3. Update application to use new secret
    // Old keys will fail verification - users must regenerate
    
    // 4. After grace period, delete old secret
}
```

---

## Migration Plan from SHA-256

```rust
// Migration: Mark all existing SHA-256 keys as requiring rotation
pub async fn migrate_api_keys(pool: &PgPool) -> Result<()> {
    // 1. Add new columns
    sqlx::query(r#"
        ALTER TABLE api_keys 
        ADD COLUMN key_identifier VARCHAR(8),
        ADD COLUMN key_version INTEGER DEFAULT 1
    "#).execute(pool).await?;
    
    // 2. Mark existing SHA-256 keys for rotation
    sqlx::query(r#"
        UPDATE api_keys 
        SET key_version = 0,
            is_active = FALSE
        WHERE key_version IS NULL
    "#).execute(pool).await?;
    
    // 3. Send emails to users to regenerate keys
    // 4. After 30 days, delete SHA-256 keys
    
    Ok(())
}
```

---

## Updated Cargo.toml

```toml
[dependencies]
# Replace: sha2 = "0.10"
# With:
hmac = "0.12"
sha2 = "0.10"
blake3 = { version = "1.5", optional = true }
base64 = "0.22"
```

---

## Security Checklist

- [ ] Use HMAC-SHA256 with 32-byte server secret
- [ ] Store secret in environment variable or KMS
- [ ] Never log API keys (even in error messages)
- [ ] Use constant-time comparison for verification
- [ ] Implement secret rotation capability
- [ ] Show key ONLY ONCE to user on creation
- [ ] Support key expiration dates
- [ ] Audit all key usage

---

**Ready to implement the security update?**
