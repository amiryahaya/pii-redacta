# PII Redacta - Architecture Gap Analysis

## Overview

This document identifies gaps between the current PII Service codebase (SecureSharing) and the proposed PII Redacta architecture. It highlights missing components, outdated specifications, and areas requiring additional attention.

**Analysis Date:** 2026-02-27  
**Source Code Version:** PII Service (Elixir/Phoenix + Rust NIFs)

---

## 1. Major Gaps Identified

### 1.1 GLINER NER Service (MISSING in Architecture.md)

**Status:** ⚠️ **CRITICAL GAP**

The existing codebase includes a **GLiNER (Generalist Model for Named Entity Recognition)** Python service that is not documented in ARCHITECTURE.md.

**What is GLiNER:**
- Zero-shot NER model for custom entity extraction
- Runs as a separate Python microservice
- Uses `urchade/gliner_medium-v2.1` model
- Accessible via HTTP API on port 5000/5001

**Architecture:**
```
┌─────────────────┐      HTTP      ┌──────────────────┐
│  PII Service    │ ◄─────────────►│  GLiNER Service  │
│  (Elixir)       │   /extract     │  (Python/Flask)  │
└─────────────────┘                └──────────────────┘
                                          │
                                          ▼
                                    ┌──────────────┐
                                    │ GLiNER Model │
                                    │  (PyTorch)   │
                                    └──────────────┘
```

**Malaysian PII Labels Supported:**
```python
MALAYSIAN_PII_LABELS = [
    "person",           # Names
    "nric",             # Malaysian IC numbers
    "address",          # Full addresses
    "organization",     # Companies, agencies
    "phone number",     # Phone numbers
    "email",            # Email addresses
    "date",             # Dates
    "postcode",         # Postal codes
    "bank account",     # Bank account numbers
    "passport",         # Passport numbers
]
```

**Gap Impact:** HIGH  
**Recommendation:** Add GLiNER as Tier 2.5 in detection pipeline (between patterns and Presidio)

---

### 1.2 LLM Gateway Architecture (INCOMPLETE)

**Current Implementation (More Advanced than Documented):**

| Feature | ARCHITECTURE.md | Actual Code | Gap |
|---------|-----------------|-------------|-----|
| Providers | OpenAI, Anthropic, Google | ✅ Same | None |
| Failover | Mentioned | ✅ Implemented with `chat_completion_with_failover` | None |
| System Prompt | Generic | ✅ Token-aware custom prompt | Document |
| Provider Health | Not mentioned | ✅ `check_provider_health/1` | Add |
| Rate Limiting | Not mentioned | ✅ Retry-after handling | Add |
| Mock Providers | Not mentioned | ✅ 4 mock providers for testing | Add |

**System Prompt (Actual):**
```
You are a helpful AI assistant analyzing documents and data. Important guidelines:

1. Some values in the content are replaced with tokens like <<PII_EMAIL_abc123>>...
2. When referring to these values in your response, use the exact token...
3. Treat all tokenized values as placeholders...
4. Focus on the patterns, structure, and insights in the data...
5. If asked about a specific person's data, reference them by their token...
```

**Gap Impact:** MEDIUM  
**Recommendation:** Update Architecture.md with complete LLM Gateway details

---

### 1.3 Ask AI Service (MISSING)

**Status:** ⚠️ **CRITICAL GAP**

The `PiiService.LLM.AskAI` module orchestrates the complete AI conversation flow:

**Flow:**
```
User Message → Tokenize PII → Build Context → Call LLM → Store Response
                    ↓                              ↓
            Token Manager                    Tokenized Storage
                    ↓                              ↓
            Encrypt Token Map ──────────►   Return to Client
```

**Key Features Not Documented:**
- Async processing support (`process_ask_async`)
- Session key (DEK) generation per conversation
- Token map versioning
- KEM key registration for post-quantum DEK wrapping
- Fallback encryption when KEM unavailable

**Gap Impact:** HIGH  
**Recommendation:** Add dedicated "AI Conversation Service" section

---

### 1.4 Query Processor (MISSING)

**Status:** ⚠️ **MISSING COMPONENT**

`PiiService.QueryProcessor` - Dedicated module for processing user queries:

```elixir
# Processes user queries by detecting and tokenizing PII
{:ok, result} = QueryProcessor.process_query(
  "What is John Doe's diagnosis?", 
  manager
)

# Returns:
# - tokenized_query: "What is <<PII_NAME_abc12345>>'s diagnosis?"
# - original_query: "What is John Doe's diagnosis?"
# - pii_detected: true
# - detected_entities: [...]
```

**Gap Impact:** MEDIUM  
**Recommendation:** Add to Detection Pipeline section

---

### 1.5 Response Packager (MISSING)

**Status:** ⚠️ **CRITICAL GAP**

`PiiService.ResponsePackager` - Packages LLM responses with encrypted token maps:

**Features:**
- Packages tokenized responses for client consumption
- Encrypts token maps with session keys (DEK)
- Wraps DEK using hybrid KEM (ML-KEM-768 + optional KAZ-KEM)
- Client-side restoration of original text
- Database storage preparation (tokenized only)

**Security Model:**
```
Server: Tokenized Response + Encrypted Token Map + KEM-wrapped DEK
                         ↓
Client: Unwrap DEK using ML-KEM secret key
                         ↓
Client: Decrypt token map using DEK
                         ↓
Client: Restore original PII values
```

**Gap Impact:** HIGH  
**Recommendation:** This is a core component - must be documented

---

### 1.6 File Processing Pipeline (INCOMPLETE)

**Current Architecture (More Complex):**

| Component | Status | Gap |
|-----------|--------|-----|
| File Upload | Documented | ✅ Complete |
| Text Extraction | Partial | ⚠️ Missing supported formats |
| Content Extraction | Missing | ⚠️ `PiiService.Files.Extractor` not documented |
| Storage | Documented | ✅ Complete |

**Supported Formats (from code):**
- Plain text, CSV, Markdown, HTML, XML
- PDF (requires `pdftotext` from poppler-utils)
- DOCX (Office Open XML)
- XLSX (Office Open XML)
- RTF
- JSON

**Gap Impact:** MEDIUM  
**Recommendation:** Add detailed file extraction architecture

---

### 1.7 Event System (MISSING)

**Status:** ⚠️ **MISSING**

`PiiService.Events.ProcessedEvent` - Idempotency tracking:

```elixir
schema "processed_events" do
  field :event_id, :binary_id  # External event ID
  field :event_type, :string   # file.deleted, file.updated, etc.
  field :metadata, :map
end
```

**Event Types:**
- `file.deleted`
- `file.updated`
- `user.deleted`
- `tenant.deleted`

**Purpose:** Ensure events from main SecureSharing system are processed exactly once.

**Gap Impact:** MEDIUM  
**Recommendation:** Add Event Processing section

---

## 2. Detection Pipeline Gaps

### 2.1 Actual Pipeline (More Complex)

**Documented:**
```
Tier 1: Pattern Detection (Rust)
Tier 2: Presidio NER
Tier 3: Document Classification
Tier 4: Context Validation
```

**Actual (from code):**
```
Tier 1: Pattern Detection (Rust NIF)        ~5ms
Tier 2: GLiNER NER (Python service)         ~150ms  ← MISSING
Tier 3: Presidio NER (optional)             ~30ms
Tier 4: Document Classification (Phi-3)     ~50ms
Tier 5: Context Validation (Mistral 7B)     ~300ms
Tier 6: SLM Validation (optional)           ~200ms
```

### 2.2 Context Validator (Not Detailed)

`PiiService.Detection.ContextValidator`:
- Validates entities based on surrounding context
- Uses regex patterns for format validation
- Adjusts confidence scores

**Gap Impact:** MEDIUM

### 2.3 Rules Engine (Not Detailed)

`PiiService.Detection.RulesEngine`:
- Domain-specific rules (medical, financial, legal)
- Confidence adjustments by domain
- Whitelist filtering

**Gap Impact:** MEDIUM

---

## 3. Database Schema Gaps

### 3.1 Missing Tables

| Table | Purpose | Status |
|-------|---------|--------|
| `processed_events` | Idempotency tracking | ❌ Missing |
| `audit_logs` | Security auditing | ✅ Documented |

### 3.2 Missing Fields

**conversations table:**
- `llm_provider` - Selected LLM provider
- `llm_model` - Selected model
- `ml_kem_public_key` - Post-quantum public key
- `kaz_kem_public_key` - Optional KAZ-KEM public key

**messages table:**
- `content_tokenized` - Tokenized message content
- `token_map_version` - Version for synchronization
- `pii_in_message` - Metadata about PII detected
- `llm_metadata` - Model, tokens used, latency

**Gap Impact:** HIGH  
**Recommendation:** Update schema documentation

---

## 4. Security Implementation Gaps

### 4.1 KEM Wrapper (Not Fully Documented)

`PiiService.Security.KemWrapper`:
- Hybrid KEM key wrapping
- ML-KEM-768 + optional KAZ-KEM
- Fallback encryption modes
- Configuration: `require_kem_keys?()`

**Gap Impact:** MEDIUM

### 4.2 Signature Service (MISSING)

`PiiService.Security.SignatureService`:
- Purpose not fully clear from code
- Likely for signing redacted files

**Gap Impact:** LOW (investigate further)

---

## 5. API Endpoint Gaps

### 5.1 Missing Endpoints (from router)

| Endpoint | Controller | Status |
|----------|------------|--------|
| `POST /api/v1/conversations/:id/ask` | `ConversationController` | ❌ Missing |
| `GET /api/v1/ai/providers` | `ProviderController` | ❌ Missing |
| `POST /api/v1/ai/providers/:provider/configure` | `ProviderController` | ❌ Missing |
| `GET /api/v1/policies/meta/categories` | `PolicyController` | ❌ Missing |
| `POST /redactors/:id/accept-policy` | `RedactorController` | ❌ Missing |

### 5.2 Redactor Service (MISSING)

`PiiService.Redactors` context:
- Redaction policies
- Redaction workflows
- Redactor management
- Policy acceptance tracking

**Gap Impact:** HIGH  
**Recommendation:** This appears to be a major subsystem - investigate further

---

## 6. Worker/Background Job Gaps

### 6.1 File Processing Worker

`PiiService.Workers.FileProcessingWorker`:
- Background job for file processing
- Uses Oban job queue
- Implements `Oban.Worker`

**Gap Impact:** MEDIUM  
**Recommendation:** Add background processing section

---

## 7. Monitoring Gaps

### 7.1 Usage Tracker

`PiiService.Monitoring.UsageTracker`:
- Tracks API usage metrics
- Likely for billing/metering

### 7.2 Metrics

`PiiService.Monitoring.Metrics`:
- Prometheus metrics collection
- Custom telemetry events

**Gap Impact:** LOW  
**Recommendation:** Add monitoring section

---

## 8. Recommended Architecture Updates

### 8.1 High Priority Updates

1. **Add GLiNER Service Section**
   - Architecture diagram
   - Deployment considerations
   - Model specifications
   - API contract

2. **Expand LLM Gateway**
   - Failover mechanism
   - Provider health checks
   - Rate limiting
   - Mock providers for testing

3. **Document Ask AI Service**
   - Conversation flow
   - Token map versioning
   - KEM integration
   - Async processing

4. **Add Response Packager**
   - Client-server encryption flow
   - KEM wrapping process
   - Restoration mechanism

5. **Update Database Schema**
   - All missing fields
   - Missing tables
   - Index updates

### 8.2 Medium Priority Updates

1. File extraction pipeline details
2. Event system architecture
3. Query processor
4. Background workers
5. Redactor service (investigate scope)

### 8.3 Low Priority Updates

1. Monitoring/Metrics
2. Signature service
3. Usage tracking

---

## 9. Technical Debt Concerns

### 9.1 Python Service Integration

**Issue:** GLiNER service is a Python microservice requiring:
- Separate deployment
- Model management (2GB+ GPU memory)
- Network communication overhead
- Health monitoring

**Recommendation for PII Redacta:**
Consider ONNX export or Rust-based NER model to eliminate Python dependency.

### 9.2 Ollama Integration Complexity

**Issue:** Multiple SLM tiers (Phi-3, Mistral 7B) add operational complexity.

**Current:**
- Document Classification: Phi-3 mini (~2GB)
- Context Validation: Mistral 7B (~4GB)
- SLM Cache for performance

**Recommendation for PII Redacta:**
Make SLM validation truly optional with graceful degradation.

### 9.3 KEM Key Management

**Issue:** KEM public key registration per conversation adds complexity.

**Current Flow:**
1. Client generates ML-KEM keypair
2. Client registers public key with conversation
3. Server uses key for DEK wrapping

**Recommendation for PII Redacta:**
Simplify to tenant-level keys or document the current approach clearly.

---

## 10. Specification Alignment Checklist

| Component | In ARCHITECTURE.md | Correct | Action |
|-----------|-------------------|---------|--------|
| Rust NIF Patterns | ✅ | ✅ | None |
| AES-256-GCM Encryption | ✅ | ✅ | None |
| ML-KEM-768 | ✅ | ✅ | None |
| Token Format | ✅ | ✅ | None |
| Zero-Knowledge Flow | ✅ | ✅ | None |
| PostgreSQL Schema | ✅ | ⚠️ Partial | Update |
| gRPC API | ✅ | ✅ | None |
| REST API | ✅ | ⚠️ Partial | Update |
| GLiNER Service | ❌ | N/A | **Add** |
| LLM Gateway | ✅ | ⚠️ Partial | Update |
| Ask AI Service | ❌ | N/A | **Add** |
| Response Packager | ❌ | N/A | **Add** |
| Query Processor | ❌ | N/A | **Add** |
| File Extraction | ✅ | ⚠️ Partial | Update |
| Event System | ❌ | N/A | **Add** |
| Background Workers | ❌ | N/A | **Add** |
| Redactor Service | ❌ | N/A | **Investigate** |

---

## 11. Revised Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PII Redacta Service (Revised)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        API Layer                                     │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │   │
│  │  │    gRPC      │  │    REST      │  │    Webhooks (Events)     │  │   │
│  │  │   (Tonic)    │  │   (Axum)     │  │                          │  │   │
│  │  └──────┬───────┘  └──────┬───────┘  └──────────────────────────┘  │   │
│  │         └─────────────────┬────────────────────────────────────────┘   │
│  └───────────────────────────┼────────────────────────────────────────────┘
│                              ▼
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    AI Conversation Service                           │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │   │
│  │  │  Query       │  │    Ask       │  │     Response             │  │   │
│  │  │  Processor   │──┤     AI       │──┤      Packager            │  │   │
│  │  │              │  │              │  │  (KEM-wrapped DEK)       │  │   │
│  │  └──────────────┘  └──────┬───────┘  └──────────────────────────┘  │   │
│  │                           │                                         │   │
│  │                  ┌────────▼────────┐                               │   │
│  │                  │   LLM Gateway   │                               │   │
│  │                  │  - OpenAI       │                               │   │
│  │                  │  - Anthropic    │                               │   │
│  │                  │  - Google       │                               │   │
│  │                  │  - Failover     │                               │   │
│  │                  └─────────────────┘                               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │
│                              ▼
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │              Detection Pipeline (6-Tier)                             │   │
│  │                                                                      │   │
│  │  Tier 1: Rust NIF Patterns (<1ms)                                    │   │
│  │  Tier 2: GLiNER NER (~150ms)  ← NEW                                  │   │
│  │  Tier 3: Presidio NER (~30ms, optional)                              │   │
│  │  Tier 4: Document Classification (~50ms)                             │   │
│  │  Tier 5: Context Validation (~300ms)                                 │   │
│  │  Tier 6: SLM Validation (~200ms, optional)                           │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │
│                              ▼
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Tokenization Engine                               │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │   │
│  │  │   Token      │  │  Token Map   │  │      Token Map           │  │   │
│  │  │  Generator   │  │   Manager    │  │      Encryptor           │  │   │
│  │  │              │  │  (Versions)  │  │   (AES-256-GCM)          │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │
│                              ▼
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    File Processing Service                           │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │   │
│  │  │   Content    │  │  Background  │  │       Storage            │  │   │
│  │  │  Extractor   │  │    Worker    │  │       (S3)               │  │   │
│  │  │ (10+ formats)│  │   (Oban)     │  │                          │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │
│                              ▼
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Security Layer                                    │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │   │
│  │  │    KEM       │  │   Token Map  │  │     Secure Memory        │  │   │
│  │  │   Wrapper    │  │  Encryption  │  │        Arena             │  │   │
│  │  │ (ML-KEM-768) │  │ (AES-256-GCM)│  │       (mlock)            │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │
│                              ▼
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Data Layer                                        │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │   │
│  │  │  PostgreSQL  │  │    Event     │  │       Audit              │  │   │
│  │  │     18       │  │   Tracking   │  │        Logs              │  │   │
│  │  │              │  │(Idempotency) │  │                          │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │              External Services                                       │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │   │
│  │  │   GLiNER     │  │   Presidio   │  │       Ollama             │  │   │
│  │  │  (Python)    │  │   (Optional) │  │    (SLM - Optional)      │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 12. Conclusion

**Critical Gaps to Address:**
1. GLiNER Service integration
2. Ask AI service architecture
3. Response Packager flow
4. Complete database schema
5. Redactor service scope

**Estimated Impact:**
- The documented architecture covers ~70% of actual implementation
- Missing components are primarily around AI/LLM integration and advanced PII detection
- No fundamental architectural conflicts found

**Recommendation:**
Update ARCHITECTURE.md with identified gaps before proceeding to implementation phase.

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-27  
**Next Review:** Before implementation kickoff
