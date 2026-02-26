# PII Redacta - Architecture Specification

## Executive Summary

**PII Redacta** is a standalone, high-performance PII (Personally Identifiable Information) detection and redaction service built with Rust. Designed for document and text processing with a focus on simplicity, speed, and enterprise security.

**Scope:** Text-based content and document processing (PDF, Word, Excel, CSV, etc.)

**MVP Focus:** Standard mode (server-side processing) with optional Zero-Knowledge mode for enterprise customers.

### Key Differentiators

| Feature | PII Redacta | Traditional Solutions |
|---------|-------------|----------------------|
| Latency (patterns) | <1ms | 10-50ms |
| Memory footprint | ~30MB base | 200MB-1GB |
| Integration time | Hours | Days/weeks |
| Document formats | 10+ formats native | Limited |
| Zero-knowledge option | Available (enterprise) | Rare |
| Post-quantum crypto | ML-KEM-768 (FIPS 203) | Rare |

---

## 1. System Architecture

### 1.1 High-Level Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PII Redacta Service                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐     ┌─────────────────────────────────────────────────┐   │
│  │   API Layer  │     │         Detection Pipeline (4-Tier)             │   │
│  │              │     │                                                 │   │
│  │  ┌────────┐  │     │  ┌─────────────┐    ┌───────────────────────┐  │   │
│  │  │  gRPC  │  │────▶│  │ Tier 1:     │───▶│ Tier 2: GLiNER NER    │  │   │
│  │  │(Tonic) │  │     │  │ Patterns    │    │ (~150ms, optional)    │  │   │
│  │  └────────┘  │     │  │ (<1ms)      │    └───────────────────────┘  │   │
│  │              │     │  └─────────────┘              │                 │   │
│  │  ┌────────┐  │     │         │                       │               │   │
│  │  │  REST  │  │     │         ▼                       ▼               │   │
│  │  │ (Axum) │  │     │  ┌─────────────────────────────────────────┐   │   │
│  │  └────────┘  │     │  │ Merge Results                           │   │   │
│  │              │     │  └──────────────────┬──────────────────────┘   │   │
│  └──────────────┘     │                     │                          │   │
│          │            │         ┌───────────┴───────────┐              │   │
│          ▼            │         ▼                       ▼              │   │
│  ┌──────────────┐     │  ┌──────────────┐      ┌──────────────┐       │   │
│  │   Auth/JWT   │     │  │ Tier 3:      │      │ Tier 4:      │       │   │
│  │ (jsonwebtoken│     │  │ Document     │      │ Context      │       │   │
│  │  + rustls)   │     │  │ Classify     │      │ Validate     │       │   │
│  └──────────────┘     │  │ (~50ms, opt) │      │ (~300ms, opt)│       │   │
│                       │  └──────────────┘      └──────────────┘       │   │
│                       └────────────────────────────────────────────────┘   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │  Tokenization│  │    File      │  │  Background  │  │   Security   │   │
│  │    Engine    │  │  Processing  │  │    Workers   │  │    Layer     │   │
│  │              │  │              │  │              │  │              │   │
│  │ • Determinin-│  │ • 10+ formats│  │ • Oban queue │  │ • Secure mem │   │
│  │   stic hash │  │ • Extract    │  │ • Async      │  │ • Zeroization│   │
│  │ • Token map │  │ • Redact     │  │ • Scalable   │  │ • ZK option  │   │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                      │
│  │   Database   │  │     Event    │  │   Storage    │                      │
│  │ (PostgreSQL  │  │    System    │  │    (S3)      │                      │
│  │      18)     │  │(Idempotency) │  │              │                      │
│  └──────────────┘  └──────────────┘  └──────────────┘                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Security Modes

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TWO SECURITY MODES                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────────────────┐  ┌──────────────────────────────────┐ │
│  │     STANDARD MODE (Default)      │  │    ZERO-KNOWLEDGE MODE (Opt-in) │ │
│  │         ─ MVP FOCUS ─            │  │        ─ ENTERPRISE ─           │ │
│  │                                  │  │                                 │ │
│  │  POST /api/v1/detect             │  │  POST /api/v1/detect/secure     │ │
│  │                                  │  │                                 │ │
│  │  Request:                        │  │  Request:                       │ │
│  │  {                               │  │  {                              │ │
│  │    "text": "john@example.com"    │  │    "encrypted_text": "...",     │ │
│  │  }                               │  │    "encrypted_dek": "..."       │ │
│  │                                  │  │  }                              │ │
│  │                                  │  │                                 │ │
│  │  Response:                       │  │  Response:                      │ │
│  │  {                               │  │  {                              │ │
│  │    "redacted_text": "<<EMAIL>>", │  │    "encrypted_result": "..."    │ │
│  │    "token_map": {...}            │  │  }                              │ │
│  │  }                               │  │                                 │ │
│  │                                  │  │                                 │ │
│  │  ✓ Simple integration            │  │  ✓ Server never sees PII        │ │
│  │  ✓ Hours to integrate            │  │  ✓ Cryptographic guarantees     │ │
│  │  ✓ No client crypto              │  │  ✓ Compliance-ready             │ │
│  │                                  │  │                                 │ │
│  │  Use: General processing         │  │  Use: Classified, medical,      │ │
│  │       Document scanning          │  │       legal, SecureSharing      │ │
│  │       Log scrubbing              │  │                                 │ │
│  │                                  │  │  Status: Post-MVP               │ │
│  │  Status: MVP                     │  │                                 │ │
│  └──────────────────────────────────┘  └──────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Standard Mode (MVP)

### 2.1 Overview

**Standard mode** is the default operating mode for PII Redacta. The server processes plaintext and returns redacted results. This provides the fastest integration path for most use cases.

**Security Model:**
- Server processes plaintext in secure memory (mlock)
- Immediate zeroization after processing
- No PII in logs or persistent storage
- Audit trails contain only metadata

### 2.2 Data Flow

```
┌─────────┐                    ┌───────────────────┐                    ┌──────────┐
│ Client  │                    │  PII Redacta      │                    │ Database │
│         │                    │  (Standard Mode)  │                    │          │
└────┬────┘                    └─────────┬─────────┘                    └────┬─────┘
     │                                   │                                   │
     │ 1. POST /api/v1/detect            │                                   │
     │    {                              │                                   │
     │      "text": "john@example.com"   │                                   │
     │    }                              │                                   │
     │ ─────────────────────────────────▶│                                   │
     │                                   │                                   │
     │                                   │ 2. Load into secure memory        │
     │                                   │    (mlock + encrypted swap)       │
     │                                   │                                   │
     │                                   │ 3. Detect PII                     │
     │                                   │    - Pattern matching (<1ms)      │
     │                                   │    - GLiNER NER (~150ms)          │
     │                                   │                                   │
     │                                   │ 4. Tokenize                       │
     │                                   │    "<<PII_EMAIL_abc123>>"         │
     │                                   │                                   │
     │                                   │ 5. Zeroize memory                 │
     │                                   │                                   │
     │ 6. Return redacted result         │                                   │
     │ ◀─────────────────────────────────│                                   │
     │    {                              │                                   │
     │      "redacted_text": "...",      │                                   │
     │      "token_map": {...},          │                                   │
     │      "entities": [...]            │                                   │
     │    }                              │                                   │
     │                                   │                                   │
     │                                   │ 7. Store audit log (metadata only)│
     │                                   │ ─────────────────────────────────▶│
```

### 2.3 API Specification

#### Detect PII in Text

```http
POST /api/v1/detect
Content-Type: application/json
Authorization: Bearer {jwt_token}

{
  "text": "Contact John at john.doe@example.com or call +60-12-345-6789",
  "options": {
    "redact": true,
    "return_entities": true,
    "confidence_threshold": 0.7
  }
}
```

**Response:**
```json
{
  "redacted_text": "Contact <<PII_NAME_abc123>> at <<PII_EMAIL_def456>> or call <<PII_PHONE_ghi789>>",
  "entities": [
    {
      "type": "NAME",
      "value": "John",
      "start": 8,
      "end": 12,
      "confidence": 0.92,
      "source": "gliner"
    },
    {
      "type": "EMAIL",
      "value": "john.doe@example.com",
      "start": 16,
      "end": 36,
      "confidence": 0.98,
      "source": "patterns"
    },
    {
      "type": "PHONE",
      "value": "+60-12-345-6789",
      "start": 46,
      "end": 61,
      "confidence": 0.95,
      "source": "patterns"
    }
  ],
  "entity_count": 3,
  "entity_types": ["NAME", "EMAIL", "PHONE"],
  "processing_time_ms": 45,
  "token_map": {
    "<<PII_NAME_abc123>>": "John",
    "<<PII_EMAIL_def456>>": "john.doe@example.com",
    "<<PII_PHONE_ghi789>>": "+60-12-345-6789"
  }
}
```

#### Restore Redacted Text

```http
POST /api/v1/restore
Content-Type: application/json
Authorization: Bearer {jwt_token}

{
  "redacted_text": "Contact <<PII_NAME_abc123>> at <<PII_EMAIL_def456>>",
  "token_map": {
    "<<PII_NAME_abc123>>": "John",
    "<<PII_EMAIL_def456>>": "john.doe@example.com"
  }
}
```

**Response:**
```json
{
  "restored_text": "Contact John at john.doe@example.com"
}
```

---

## 3. Zero-Knowledge Mode (Enterprise)

### 3.1 Overview

**Zero-knowledge mode** is an optional enterprise feature for customers requiring cryptographic guarantees that the server never sees plaintext PII.

**Status:** Post-MVP feature

### 3.2 When to Use Zero-Knowledge Mode

| Use Case | Recommended Mode |
|----------|-----------------|
| Classified documents | Zero-Knowledge |
| Medical records (HIPAA) | Zero-Knowledge |
| Legal documents | Zero-Knowledge |
| General enterprise docs | Standard |
| Log scrubbing | Standard |
| Document scanning | Standard |

### 3.3 API Specification

```http
POST /api/v1/detect/secure
Content-Type: application/json
Authorization: Bearer {jwt_token}

{
  "encrypted_text": "base64_encoded_encrypted_text",
  "encrypted_dek": "base64_encoded_data_encryption_key",
  "ml_kem_public_key": "base64_encoded_ml_kem_public_key",
  "options": {
    "redact": true
  }
}
```

**Response:**
```json
{
  "encrypted_redacted_text": "base64_encoded_encrypted_redacted",
  "encrypted_token_map": "base64_encoded_encrypted_token_map",
  "ml_kem_ciphertext": "base64_encoded_kem_ciphertext",
  "wrapped_dek": "base64_encoded_wrapped_dek",
  "entity_count": 3,
  "entity_types": ["NAME", "EMAIL", "PHONE"]
}
```

---

## 4. Detection Pipeline

### 4.1 Pipeline Tiers

| Tier | Component | Latency | Status |
|------|-----------|---------|--------|
| 1 | Pattern Matching (Rust NIF) | <1ms | **MVP** |
| 2 | GLiNER NER | ~150ms | **MVP** |
| 3 | Document Classification | ~50ms | Optional |
| 4 | Context Validation | ~300ms | Optional |

### 4.2 Pattern Matching (Tier 1)

**Implementation:** Rust NIF using Aho-Corasick algorithm

**Supported Patterns:**
- EMAIL (RFC 5322)
- PHONE (Malaysian + international)
- NRIC (Malaysian IC)
- PASSPORT
- CREDIT_CARD (Luhn validated)
- BANK_ACCOUNT
- IP_ADDRESS (v4/v6)
- DATE_OF_BIRTH

### 4.3 GLiNER NER (Tier 2)

**Model:** `urchade/gliner_medium-v2.1`

**Supported Labels:**
- `person` - Full names
- `organization` - Companies, agencies
- `address` - Full addresses
- `phone number` - Phone numbers
- `email` - Email addresses
- `nric` - Malaysian IC numbers
- `postcode` - Postal codes
- `bank account` - Bank account numbers
- `passport` - Passport numbers
- `date` - Dates

**Deployment Options:**
1. **Embedded (MVP):** ONNX Runtime in Rust
2. **Sidecar:** Python service via HTTP
3. **Remote:** External GLiNER service

---

## 5. File Processing

### 5.1 Supported Formats

| Format | MIME Type | Priority |
|--------|-----------|----------|
| Plain Text | `text/plain` | MVP |
| CSV | `text/csv` | MVP |
| Markdown | `text/markdown` | MVP |
| PDF | `application/pdf` | MVP |
| Word (DOCX) | `application/vnd.openxmlformats-officedocument.wordprocessingml.document` | MVP |
| Excel (XLSX) | `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` | Post-MVP |
| HTML | `text/html` | Post-MVP |
| JSON | `application/json` | Post-MVP |
| XML | `application/xml` | Post-MVP |
| RTF | `application/rtf` | Post-MVP |

### 5.2 File Processing API

#### Submit File for Redaction

```http
POST /api/v1/files/submit
Content-Type: multipart/form-data
Authorization: Bearer {jwt_token}

file: [binary file data]
options: {"redact": true, "output_format": "same"}
```

**Response:**
```json
{
  "job_id": "job-uuid",
  "status": "pending",
  "estimated_completion": "2025-01-31T12:00:00Z"
}
```

#### Check Job Status

```http
GET /api/v1/jobs/{job_id}
Authorization: Bearer {jwt_token}
```

**Response:**
```json
{
  "job_id": "job-uuid",
  "status": "completed",
  "redacted_file_url": "https://...",
  "entity_count": 15,
  "entity_types": ["EMAIL", "PHONE", "NAME"],
  "completed_at": "2025-01-31T12:00:00Z"
}
```

---

## 6. Tokenization Engine

### 6.1 Token Format

```
<<PII_{TYPE}_{HASH}>>

Example: <<PII_EMAIL_a1b2c3d4e5f6g7h8>>

Components:
- TYPE: PII category (EMAIL, PHONE, NRIC, NAME, etc.)
- HASH: Deterministic hash of (tenant_id + value)
```

### 6.2 Deterministic Tokenization

- Same PII value → Same token within a tenant
- Enables consistent redaction across documents
- Reduces token map size

### 6.3 Token Map Structure

```json
{
  "<<PII_EMAIL_a1b2c3d4>>": "john@example.com",
  "<<PII_PHONE_e5f6g7h8>>": "+60-12-345-6789",
  "<<PII_NAME_i9j0k1l2>>": "John Doe"
}
```

---

## 7. Security Implementation

### 7.1 Secure Memory Management

```rust
pub struct SecureArena {
    buffer: Vec<u8>,
    locked: bool,
}

impl SecureArena {
    pub fn new(size: usize) -> Result<Self, Error> {
        let mut buffer = vec![0u8; size];
        // Prevent swapping to disk
        let locked = unsafe { 
            libc::mlock(buffer.as_mut_ptr(), buffer.len()) == 0 
        };
        Ok(Self { buffer, locked })
    }
    
    pub fn process<F, R>(&mut self, data: &[u8], f: F) -> R 
    where F: FnOnce(&[u8]) -> R {
        self.buffer.copy_from_slice(data);
        let result = f(&self.buffer);
        self.zeroize();
        result
    }
}

impl Drop for SecureArena {
    fn drop(&mut self) {
        self.buffer.zeroize();
        if self.locked {
            unsafe { libc::munlock(self.buffer.as_ptr(), self.buffer.len()) };
        }
    }
}
```

### 7.2 Audit Logging (No PII)

```rust
pub struct AuditEvent {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub action: String,        // "detect", "redact", "restore"
    pub entity_count: u32,     // Count only, no values
    pub entity_types: Vec<String>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    // NEVER: entity values, token mappings
}
```

### 7.3 Zero-Knowledge Preparation

Even in standard mode, architecture supports future ZK mode:

```rust
pub enum ProcessingMode {
    Standard,        // Current MVP
    ZeroKnowledge,   // Enterprise option
}

pub struct DetectionRequest {
    pub mode: ProcessingMode,
    pub text: Option<String>,           // Standard mode
    pub encrypted_text: Option<Vec<u8>>, // ZK mode
    pub encrypted_dek: Option<Vec<u8>>,  // ZK mode
}
```

---

## 8. Database Schema

### 8.1 Tables

```sql
-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tenants
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    name VARCHAR(255) NOT NULL,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Processing jobs (for async file processing)
CREATE TABLE processing_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    
    job_type VARCHAR(50) NOT NULL CHECK (job_type IN ('text', 'document', 'batch')),
    status VARCHAR(20) DEFAULT 'pending' 
        CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    
    source_type VARCHAR(50),
    source_reference TEXT,
    
    entity_types TEXT[],
    entity_count INTEGER DEFAULT 0,
    
    result_storage_key TEXT,
    error_message TEXT,
    
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Redacted files
CREATE TABLE redacted_files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    job_id UUID REFERENCES processing_jobs(id),
    
    original_filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(100),
    size_bytes BIGINT,
    
    storage_bucket VARCHAR(255),
    storage_key VARCHAR(1024),
    
    entity_count INTEGER DEFAULT 0,
    entity_types TEXT[],
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Event idempotency tracking
CREATE TABLE processed_events (
    event_id UUID PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL 
        CHECK (event_type IN ('file.deleted', 'file.updated', 'user.deleted', 'tenant.deleted')),
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit logs (metadata only, no PII)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    metadata JSONB,  -- {entity_count, entity_types, duration_ms}
    ip_address INET,
    created_at TIMESTAMPTZ DEFAULT NOW()
) PARTITION BY RANGE (created_at);

-- Indexes
CREATE INDEX idx_jobs_tenant ON processing_jobs(tenant_id, status);
CREATE INDEX idx_files_tenant ON redacted_files(tenant_id, user_id);
CREATE INDEX idx_audit_tenant ON audit_logs(tenant_id, created_at DESC);
```

---

## 9. API Endpoints

### 9.1 REST API

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/health` | Health check | None |
| POST | `/api/v1/detect` | Detect PII in text | JWT |
| POST | `/api/v1/redact` | Redact text | JWT |
| POST | `/api/v1/restore` | Restore redacted text | JWT |
| POST | `/api/v1/files/submit` | Submit file for redaction | JWT |
| GET | `/api/v1/jobs/{id}` | Get job status | JWT |
| GET | `/api/v1/files/{id}/download` | Download redacted file | JWT |
| POST | `/api/v1/detect/secure` | Zero-knowledge detect | JWT |

### 9.2 gRPC Services

```protobuf
syntax = "proto3";
package pii_redacta.v1;

service DetectionService {
  rpc Detect(DetectRequest) returns (DetectResponse);
  rpc Redact(RedactRequest) returns (RedactResponse);
  rpc Restore(RestoreRequest) returns (RestoreResponse);
}

service FileProcessingService {
  rpc SubmitFile(SubmitFileRequest) returns (Job);
  rpc GetJobStatus(GetJobRequest) returns (Job);
  rpc DownloadRedactedFile(DownloadRequest) returns (stream Chunk);
}
```

---

## 10. Deployment

### 10.1 Docker Compose (MVP)

```yaml
version: '3.8'

services:
  pii-redacta:
    image: pii-redacta:latest
    environment:
      - DATABASE_URL=postgres://pii_redacta:${DB_PASSWORD}@postgres:5432/pii_redacta
      - JWT_SECRET=${JWT_SECRET}
      - RUST_LOG=info
    ports:
      - "8080:8080"    # REST
      - "50051:50051"  # gRPC
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

### 10.2 Resource Requirements

| Component | Memory | CPU | Notes |
|-----------|--------|-----|-------|
| Base Service | 50MB | 0.1 cores | Pattern detection only |
| With GLiNER (ONNX) | 250MB | 0.5 cores | Recommended |
| PostgreSQL | 256MB | 0.5 cores | Shared instance ok |

---

## 11. Configuration

### 11.1 MVP Configuration

```toml
# config.toml

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
mode = "standard"  # "standard" or "zero-knowledge"
jwt_secret_path = "/secrets/jwt.key"

[audit]
enabled = true
retention_days = 90
```

---

## 12. Implementation Roadmap

### Phase 1: MVP (Weeks 1-8)
- Standard mode only
- Pattern detection (Rust NIF)
- GLiNER integration (ONNX)
- REST API
- Text processing
- Basic file processing (PDF, DOCX)

### Phase 2: Enterprise (Weeks 9-12)
- Zero-knowledge mode
- gRPC API
- Advanced file formats
- Background workers
- Monitoring/Metrics

### Phase 3: Scale (Weeks 13-16)
- Performance optimization
- Additional ML models
- Advanced analytics
- Enterprise features

---

**Document Version:** 3.0  
**Last Updated:** 2026-02-27  
**Status:** MVP-focused with hybrid security model
