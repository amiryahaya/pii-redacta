# PII Redacta - Implementation Plan

## Overview

This document outlines the implementation strategy for **PII Redacta**, a hybrid-security PII detection and redaction service.

**Security Model:**
- **Phase 1 (MVP):** Standard Mode - Simple API, server-side processing
- **Phase 2:** Zero-Knowledge Mode - Encrypted processing, client holds keys

---

## 1. Project Structure

```
pii-redacta/
├── Cargo.toml                    # Workspace root
├── Cargo.lock
├── README.md
├── ARCHITECTURE.md
├── ROADMAP.md
├── IMPLEMENTATION_PLAN.md
├── docker-compose.yml
├── Dockerfile
├── k8s/                          # Kubernetes manifests
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   └── secrets.yaml
├── proto/                        # Protocol buffer definitions
│   └── pii_redacta/
│       └── v1/
│           ├── detection.proto
│           ├── file.proto
│           └── common.proto
├── crates/                       # Workspace members
│   ├── pii_redacta_core/        # Core detection + tokenization
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── detection/
│   │       │   ├── mod.rs
│   │       │   ├── patterns.rs       # Rust NIF patterns
│   │       │   ├── gliner.rs         # GLiNER integration
│   │       │   ├── pipeline.rs       # Detection pipeline
│   │       │   ├── merger.rs         # Result merging
│   │       │   └── rules/
│   │       │       ├── mod.rs
│   │       │       ├── medical.rs
│   │       │       ├── financial.rs
│   │       │       └── legal.rs
│   │       ├── tokenization/
│   │       │   ├── mod.rs
│   │       │   ├── generator.rs      # Token generation
│   │       │   ├── map.rs            # Token map management
│   │       │   └── restorer.rs       # Token restoration
│   │       └── entity.rs
│   │
│   ├── pii_redacta_security/    # Crypto + secure memory
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── crypto/
│   │       │   ├── mod.rs
│   │       │   ├── aes_gcm.rs        # AES-256-GCM (ZK mode)
│   │       │   ├── ml_kem.rs         # ML-KEM-768 (ZK mode)
│   │       │   └── hkdf.rs           # Key derivation
│   │       └── memory/
│   │           ├── mod.rs
│   │           ├── arena.rs          # Secure memory arena
│   │           └── zeroization.rs    # Secure clearing
│   │
│   ├── pii_redacta_db/          # Database layer
│   │   ├── Cargo.toml
│   │   ├── migrations/
│   │   │   ├── 001_initial.sql
│   │   │   └── 002_add_indexes.sql
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── connection.rs
│   │       ├── models/
│   │       │   ├── mod.rs
│   │       │   ├── tenant.rs
│   │       │   ├── processing_job.rs
│   │       │   ├── redacted_file.rs
│   │       │   ├── processed_event.rs
│   │       │   └── audit_log.rs
│   │       └── repositories/
│   │           ├── mod.rs
│   │           ├── tenant_repo.rs
│   │           ├── job_repo.rs
│   │           └── file_repo.rs
│   │
│   ├── pii_redacta_api/         # REST API (Phase 1) + gRPC (Phase 2)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── main.rs
│   │       ├── rest/                 # Phase 1: REST only
│   │       │   ├── mod.rs
│   │       │   ├── router.rs
│   │       │   ├── handlers/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── detection.rs  # /detect, /redact, /restore
│   │       │   │   ├── file.rs       # File upload/download
│   │       │   │   └── job.rs        # Job status
│   │       │   └── middleware/
│   │       │       ├── mod.rs
│   │       │       ├── auth.rs
│   │       │       └── rate_limit.rs
│   │       ├── grpc/                 # Phase 2: gRPC
│   │       │   ├── mod.rs
│   │       │   ├── server.rs
│   │       │   └── detection_service.rs
│   │       └── proto/
│   │           └── pii_redacta.v1.rs  # Generated
│   │
│   └── pii_redacta_cli/         # CLI tool
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── tests/                        # Integration tests
│   ├── integration/
│   │   ├── detection_test.rs
│   │   ├── tokenization_test.rs
│   │   ├── api_test.rs
│   │   └── file_test.rs
│   └── fixtures/
│       └── sample_documents/
│
└── benches/                      # Benchmarks
    ├── detection_bench.rs
    └── tokenization_bench.rs
```

---

## 2. Phase 1: MVP (Standard Mode)

### 2.1 Security Architecture (Standard Mode)

In Standard Mode, the server processes plaintext with these safeguards:

```rust
// Standard Mode - Server-side processing with security
pub struct StandardProcessor {
    detector: DetectionPipeline,
    tokenizer: Tokenizer,
    secure_arena: SecureArena,
}

impl StandardProcessor {
    pub async fn process(&self, text: &str) -> Result<ProcessingResult, Error> {
        // 1. Copy to secure memory (mlock)
        let secure_text = self.secure_arena.store(text)?;
        
        // 2. Detect PII
        let entities = self.detector.detect(&secure_text).await?;
        
        // 3. Tokenize
        let (redacted, token_map) = self.tokenizer.tokenize(&secure_text, &entities);
        
        // 4. Zeroize immediately after processing
        self.secure_arena.zeroize();
        
        Ok(ProcessingResult {
            redacted_text: redacted,
            token_map,
            entities,
        })
    }
}
```

### 2.2 Core Modules

#### Detection Pipeline

```rust
// 4-tier pipeline for MVP
pub struct DetectionPipeline {
    pattern_matcher: PatternMatcher,      // Tier 1: <1ms
    gliner: Option<GlinerClient>,         // Tier 2: ~150ms (optional)
    rules_engine: RulesEngine,            // Tier 3: Domain rules
}

impl DetectionPipeline {
    pub async fn detect(&self, text: &str) -> Result<Vec<Entity>, Error> {
        // Tier 1: Pattern matching (always)
        let pattern_entities = self.pattern_matcher.detect(text);
        
        // Tier 2: GLiNER (if enabled)
        let gliner_entities = if let Some(gliner) = &self.gliner {
            gliner.detect(text).await.unwrap_or_default()
        } else {
            vec![]
        };
        
        // Merge and deduplicate
        let merged = self.merge_results(pattern_entities, gliner_entities);
        
        // Tier 3: Apply domain rules
        let filtered = self.rules_engine.apply(merged, text);
        
        Ok(filtered)
    }
}
```

#### Secure Memory Arena

```rust
pub struct SecureArena {
    buffer: Vec<u8>,
    locked: bool,
}

impl SecureArena {
    pub fn new(size: usize) -> Result<Self, ArenaError> {
        let mut buffer = vec![0u8; size];
        
        // Prevent swapping to disk
        let locked = unsafe {
            libc::mlock(buffer.as_mut_ptr(), buffer.len()) == 0
        };
        
        Ok(Self { buffer, locked })
    }
    
    pub fn store(&mut self, data: &str) -> Result<SecureText, ArenaError> {
        if data.len() > self.buffer.len() {
            return Err(ArenaError::BufferTooSmall);
        }
        
        self.buffer[..data.len()].copy_from_slice(data.as_bytes());
        Ok(SecureText { 
            ptr: self.buffer.as_ptr(), 
            len: data.len() 
        })
    }
    
    pub fn zeroize(&mut self) {
        self.buffer.zeroize();
    }
}

impl Drop for SecureArena {
    fn drop(&mut self) {
        self.zeroize();
        if self.locked {
            unsafe { 
                libc::munlock(self.buffer.as_ptr(), self.buffer.len()) 
            };
        }
    }
}
```

### 2.3 API Handlers (Standard Mode)

#### Detection Handler

```rust
// POST /api/v1/detect
pub async fn detect_handler(
    State(state): State<Arc<AppState>>,
    claims: JwtClaims,
    Json(req): Json<DetectRequest>,
) -> Result<Json<DetectResponse>, ApiError> {
    // Validate input
    if req.text.len() > 1_000_000 {
        return Err(ApiError::PayloadTooLarge);
    }
    
    // Process in secure memory
    let result = state.processor.process(&req.text).await?;
    
    // Audit log (metadata only, NO PII values)
    state.audit.log(AuditEvent {
        tenant_id: claims.tenant_id,
        user_id: claims.sub,
        action: "detect",
        entity_count: result.entities.len(),
        entity_types: extract_types(&result.entities),
        duration_ms: timer.elapsed_ms(),
        // NEVER: entity values, token mappings
    }).await?;
    
    Ok(Json(DetectResponse {
        redacted_text: result.redacted_text,
        token_map: result.token_map,        // Client can restore if needed
        entity_count: result.entities.len(),
        entity_types: extract_types(&result.entities),
    }))
}
```

#### File Processing Handler

```rust
// POST /api/v1/files/submit
pub async fn submit_file_handler(
    State(state): State<Arc<AppState>>,
    claims: JwtClaims,
    mut multipart: Multipart,
) -> Result<Json<SubmitResponse>, ApiError> {
    // Extract file from multipart
    let file = extract_file(&mut multipart).await?;
    
    // Upload to S3
    let s3_key = state.storage.upload(&file).await?;
    
    // Create processing job
    let job = state.db.create_job(CreateJobRequest {
        tenant_id: claims.tenant_id,
        user_id: claims.sub,
        source_type: "s3",
        source_reference: &s3_key,
    }).await?;
    
    // Queue background job
    state.worker.enqueue(FileProcessingJob {
        job_id: job.id,
        tenant_id: claims.tenant_id,
        s3_key,
        filename: file.filename,
    }).await?;
    
    Ok(Json(SubmitResponse {
        job_id: job.id,
        status: JobStatus::Pending,
        estimated_completion: job.estimated_completion(),
    }))
}
```

### 2.4 Database Models

```rust
// models/processing_job.rs
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProcessingJob {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub job_type: JobType,           // Text, Document, Batch
    pub status: JobStatus,           // Pending, Processing, Completed, Failed
    
    pub source_type: String,         // inline, s3
    pub source_reference: String,    // S3 key or content hash
    
    pub entity_count: i32,
    pub entity_types: Option<Vec<String>>,
    
    pub result_storage_key: Option<String>,  // S3 key for redacted file
    pub error_message: Option<String>,
    
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// models/audit_log.rs - NO PII VALUES
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub action: String,              // detect, redact, restore
    pub resource_type: String,       // job, file
    pub resource_id: Option<Uuid>,
    pub metadata: AuditMetadata,     // Metadata only
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    pub entity_count: u32,
    pub entity_types: Vec<String>,
    pub duration_ms: u64,
    pub detection_sources: Vec<String>,
    // NEVER: entity values, original text
}
```

---

## 3. Phase 2: Zero-Knowledge Mode

### 3.1 ZK Mode Architecture

```rust
// Zero-Knowledge Mode - Client holds keys
pub struct ZkProcessor {
    detector: DetectionPipeline,
    tokenizer: Tokenizer,
    kem_wrapper: KemWrapper,
    secure_arena: SecureArena,
}

impl ZkProcessor {
    pub async fn process_secure(
        &self,
        encrypted_text: &[u8],
        encrypted_dek: &[u8],
        ml_kem_pk: &[u8],
    ) -> Result<ZkProcessingResult, Error> {
        // 1. Decrypt DEK using server's ML-KEM private key
        let dek = self.kem_wrapper.decapsulate(encrypted_dek)?;
        
        // 2. Decrypt text using DEK
        let text = decrypt_aes_gcm(encrypted_text, &dek)?;
        
        // 3. Process in secure memory
        let secure_text = self.secure_arena.store(&text)?;
        let entities = self.detector.detect(&secure_text).await?;
        let (redacted, token_map) = self.tokenizer.tokenize(&secure_text, &entities);
        self.secure_arena.zeroize();
        
        // 4. Encrypt results
        let encrypted_redacted = encrypt_aes_gcm(&redacted, &dek)?;
        let encrypted_token_map = encrypt_aes_gcm(
            &serde_json::to_vec(&token_map)?, 
            &dek
        )?;
        
        // 5. Re-wrap DEK with client's KEM public key
        let (wrapped_dek, kem_ciphertext) = self.kem_wrapper.encapsulate(ml_kem_pk, &dek)?;
        
        Ok(ZkProcessingResult {
            encrypted_redacted_text: encrypted_redacted,
            encrypted_token_map,
            wrapped_dek,
            ml_kem_ciphertext: kem_ciphertext,
            entity_count: entities.len(),
        })
    }
}
```

### 3.2 ZK API Handler

```rust
// POST /api/v1/detect/secure
pub async fn detect_secure_handler(
    State(state): State<Arc<AppState>>,
    claims: JwtClaims,
    Json(req): Json<DetectSecureRequest>,
) -> Result<Json<DetectSecureResponse>, ApiError> {
    // Validate ZK mode enabled
    if !state.config.zk_mode_enabled {
        return Err(ApiError::ZkModeNotEnabled);
    }
    
    // Process in ZK mode
    let result = state.zk_processor.process_secure(
        &req.encrypted_text,
        &req.encrypted_dek,
        &req.ml_kem_public_key,
    ).await?;
    
    // Audit log (no PII, even encrypted)
    state.audit.log(AuditEvent {
        tenant_id: claims.tenant_id,
        user_id: claims.sub,
        action: "detect_secure",
        entity_count: result.entity_count,
        // ... metadata only
    }).await?;
    
    Ok(Json(DetectSecureResponse {
        encrypted_redacted_text: result.encrypted_redacted_text,
        encrypted_token_map: result.encrypted_token_map,
        wrapped_dek: result.wrapped_dek,
        ml_kem_ciphertext: result.ml_kem_ciphertext,
        entity_count: result.entity_count,
    }))
}
```

### 3.3 Database Changes for ZK Mode

```rust
// Add to processing_jobs table
pub struct ProcessingJob {
    // ... existing fields
    
    // ZK mode fields (optional)
    pub encrypted_dek: Option<Vec<u8>>,
    pub kem_ciphertext_ml: Option<Vec<u8>>,
    pub kem_ciphertext_kaz: Option<Vec<u8>>,
}
```

---

## 4. Configuration

### 4.1 Phase 1 (MVP) Configuration

```toml
# config.toml - Standard Mode

[server]
http_port = 8080
grpc_port = 50051
workers = 4

[database]
url = "postgres://localhost/pii_redacta"
pool_size = 20

[detection]
patterns_enabled = true
gliner_enabled = true
gliner_model_path = "/models/gliner_medium-v2.1.onnx"
presidio_enabled = false
slm_validation = false

[security]
mode = "standard"                    # "standard" for MVP
require_secure_memory = true         # mlock for all processing
max_text_size = "1MB"

[audit]
enabled = true
retention_days = 90
log_pii_values = false               # NEVER true in production

[storage]
type = "s3"                          # or "minio", "gcs"
bucket = "pii-redacta-files"
region = "us-east-1"
```

### 4.2 Phase 2 Configuration (with ZK Mode)

```toml
# config.toml - With Zero-Knowledge

[security]
mode = "hybrid"                      # "standard", "zero-knowledge", or "hybrid"
zk_mode_enabled = true               # Enable ZK endpoints
ml_kem_private_key_path = "/secrets/ml-kem.key"
ml_kem_public_key_path = "/secrets/ml-kem.pub"
require_kem_for_zk = true

[encryption]
aes_gcm_nonce_size = 12
aes_gcm_tag_size = 16
```

---

## 5. Testing Strategy

### 5.1 Security Testing

```rust
#[tokio::test]
async fn test_no_pii_in_logs() {
    let test_text = "john.doe@example.com";
    
    // Process
    let _ = processor.process(test_text).await;
    
    // Verify logs don't contain PII
    let logs = get_logs();
    assert!(!logs.contains("john.doe@example.com"));
    assert!(!logs.contains("john.doe"));
    assert!(logs.contains("entity_count=1"));  // OK: metadata only
}

#[tokio::test]
async fn test_secure_memory_zeroization() {
    let arena = SecureArena::new(1024).unwrap();
    
    // Store sensitive data
    let data = "sensitive PII data";
    arena.store(data).unwrap();
    
    // Zeroize
    arena.zeroize();
    
    // Verify buffer is zeroed
    assert!(arena.buffer.iter().all(|&b| b == 0));
}

#[tokio::test]
async fn test_zk_roundtrip() {
    let plaintext = "Contact john@example.com";
    
    // Client-side: Encrypt
    let dek = generate_dek();
    let encrypted_text = encrypt(plaintext, &dek);
    let (wrapped_dek, kem_ct) = kem_encapsulate(server_pk, &dek);
    
    // Server-side: Process
    let result = zk_processor.process_secure(
        &encrypted_text,
        &wrapped_dek,
        &client_kem_pk,
    ).await.unwrap();
    
    // Client-side: Decrypt result
    let dek_out = kem_decapsulate(client_sk, &result.ml_kem_ciphertext, &result.wrapped_dek);
    let redacted = decrypt(&result.encrypted_redacted_text, &dek_out);
    
    // Verify
    assert!(redacted.contains("<<PII_EMAIL"));
    assert!(!redacted.contains("john@example.com"));
}
```

### 5.2 Performance Testing

```rust
// benches/detection_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn detection_benchmark(c: &mut Criterion) {
    let processor = create_test_processor();
    let text = "Contact john@test.com, IC: 850101-14-5123";
    
    c.bench_function("pattern_detection", |b| {
        b.iter(|| {
            processor.detect_patterns(black_box(text))
        });
    });
    
    c.bench_function("full_pipeline", |b| {
        b.iter(|| async {
            processor.process(black_box(text)).await
        });
    });
}
```

---

## 6. Deployment

### 6.1 Phase 1: MVP Deployment

```yaml
# docker-compose.yml
version: '3.8'

services:
  pii-redacta:
    image: pii-redacta:latest
    environment:
      - DATABASE_URL=postgres://pii_redacta:${DB_PASSWORD}@postgres:5432/pii_redacta
      - JWT_SECRET=${JWT_SECRET}
      - RUST_LOG=info
      - SECURITY_MODE=standard
    ports:
      - "8080:8080"
    depends_on:
      - postgres
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '2.0'

  postgres:
    image: postgres:18-alpine
    environment:
      - POSTGRES_USER=pii_redacta
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### 6.2 Phase 2: ZK Mode Deployment

```yaml
# docker-compose.yml with ZK mode
services:
  pii-redacta:
    image: pii-redacta:latest
    environment:
      - SECURITY_MODE=hybrid
      - ZK_MODE_ENABLED=true
      - ML_KEM_PRIVATE_KEY_PATH=/secrets/ml-kem.key
    volumes:
      - ./secrets:/secrets:ro
    # ... rest of config
```

---

## 7. Implementation Roadmap

### Phase 1: MVP (Weeks 1-8) - Standard Mode

| Week | Deliverable |
|------|-------------|
| 1 | Project setup, CI/CD, secure memory arena |
| 2 | Pattern detection (Rust NIF) |
| 3 | GLiNER integration (ONNX) |
| 4 | Tokenization engine |
| 5 | REST API (detect, redact, restore) |
| 6 | File processing (PDF, DOCX) |
| 7 | Background workers, database |
| 8 | Testing, documentation, MVP release |

### Phase 2: Enterprise (Weeks 9-12) - Zero-Knowledge

| Week | Deliverable |
|------|-------------|
| 9 | ML-KEM implementation, crypto layer |
| 10 | ZK mode API endpoints |
| 11 | gRPC API, monitoring |
| 12 | Testing, documentation, enterprise release |

### Phase 3: Scale (Weeks 13-16)

| Week | Deliverable |
|------|-------------|
| 13 | Performance optimization |
| 14 | Additional file formats |
| 15 | Advanced analytics |
| 16 | Production hardening |

---

## 8. Migration from SecureSharing

For SecureSharing integration:

1. **Phase 1:** Deploy PII Redacta in Standard Mode
   - Use for non-sensitive documents
   - Build integration patterns

2. **Phase 2:** Enable ZK Mode
   - Classified documents use ZK endpoints
   - Maintain existing security guarantees

3. **Gradual Transition:**
   - Standard mode for general processing
   - ZK mode for sensitive/classified content

---

**Document Version:** 2.0  
**Last Updated:** 2026-02-27  
**Status:** MVP-focused with hybrid security model
