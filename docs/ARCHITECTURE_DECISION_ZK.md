# Architecture Decision: Zero-Knowledge vs. Standalone Usability

## The Core Conflict

| Aspect | Zero-Knowledge (Current) | Standalone Usability |
|--------|-------------------------|---------------------|
| Client Requirement | Must provide DEK, manage KEM keys | Just send text/document |
| Integration Effort | High (crypto library, key management) | Low (simple API call) |
| Security | Maximum (server never sees PII) | Depends on trust model |
| Adoption Friction | High | Low |
| Target Users | SecureSharing, high-security orgs | General enterprise, SMB |

Your Question: "Does requiring DEK for document processing make sense?"

Answer: For a standalone product targeting broad adoption - NO. It creates too much friction.

---

## Recommended Approach: Hybrid Security Model

### Option A: Tiered Security (Recommended)

**Standard Mode (Default):**
- Client sends plaintext text/document
- Server processes in secure memory
- Returns redacted text + token map
- Trust: Server is secure, audited
- Use Case: General enterprise, quick integration

**Zero-Knowledge Mode (Opt-in):**
- Client sends encrypted document + DEK (wrapped with KEM)
- Server decrypts, processes, re-encrypts
- Returns encrypted package
- Trust: Cryptographic guarantees
- Use Case: SecureSharing, classified documents, compliance

### Option B: Trust-Based Configuration

```toml
# config.toml
[security]
mode = "standard"  # Options: "standard", "zero-knowledge", "hybrid"
```

---

## Detailed Analysis

### 1. Standard Mode (Server-Side Processing)

**Data Flow:**
```
Client -> Server: Plaintext text/document
Server: Detect PII -> Tokenize -> Redact
Client <- Redacted text + token map
```

**Pros:**
- Simple API - just send text
- No client-side crypto required
- Faster integration (hours vs. days)
- Works with existing workflows
- Easier debugging

**Cons:**
- Server sees plaintext (briefly)
- Requires trust in server security
- Not suitable for classified data

**Security Mitigations:**
- Process in secure memory arena (mlock)
- Immediate zeroization after processing
- No logging of PII values
- Audit trails only (no PII storage)

### 2. Zero-Knowledge Mode (Client-Side Control)

**Data Flow:**
```
Client -> Server: Encrypted document + DEK (wrapped)
Server: Decrypt -> Process -> Encrypt
Client <- Encrypted redacted text + encrypted token map
Client: Decrypts using KEM secret key
```

**Pros:**
- Server never sees plaintext PII
- Cryptographic guarantees
- Suitable for classified data
- Compliance-friendly (GDPR, HIPAA)

**Cons:**
- Complex client integration
- Client must manage KEM keys
- Higher latency (crypto overhead)

---

## Decision Matrix by Use Case

| Use Case | Recommended Mode | Reasoning |
|----------|-----------------|-----------|
| Document scanning for PII | Standard | One-time scan, no persistent storage |
| Legal document redaction | Zero-Knowledge | High sensitivity, compliance |
| Call center transcription | Standard | High volume, real-time |
| Medical records processing | Zero-Knowledge | HIPAA requirements |
| Log scrubbing | Standard | Non-sensitive, high throughput |
| Classified documents | Zero-Knowledge | National security requirements |
| Email filtering | Standard | Volume too high for ZK overhead |
| SecureSharing integration | Zero-Knowledge | Existing architecture |

---

## Implementation Strategy

### Phase 1: Standard Mode (MVP)
Build the standalone product with standard mode first:

```rust
POST /api/v1/detect
{
  "text": "Contact john@example.com",
  "options": { "redact": true }
}

// Response
{
  "redacted_text": "Contact <<PII_EMAIL_abc123>>",
  "token_map": {
    "<<PII_EMAIL_abc123>>": "john@example.com"
  }
}
```

### Phase 2: Zero-Knowledge Mode (Enterprise)
Add ZK mode for high-security customers:

```rust
POST /api/v1/detect/secure
{
  "encrypted_text": "base64...",
  "encrypted_dek": "base64...",
  "ml_kem_public_key": "base64..."
}
```

---

## Pricing Implications

| Mode | Pricing | Target Market |
|------|---------|---------------|
| Standard | All tiers (base) | SMB, Enterprise, quick integration |
| Zero-Knowledge | Professional+ (add-on 20-30%) | Regulated industries, SecureSharing |

---

## Final Recommendation

### Recommended: Hybrid Model

**Rationale:**
1. Market Fit: Standard mode enables broad adoption
2. Security: ZK mode maintains SecureSharing compatibility
3. Flexibility: Customers choose based on their threat model
4. Competitive: Matches AWS Comprehend, Google DLP

**Implementation Priority:**
1. Build Standard mode first (MVP, weeks 1-8)
2. Add ZK mode second (Enterprise feature, weeks 9-12)

**API Design:**
```
POST /api/v1/detect          # Standard (default)
POST /api/v1/detect/secure   # Zero-knowledge
```

---

**Decision Status:** PENDING APPROVAL

**Next Step:** Confirm hybrid approach or choose single mode
