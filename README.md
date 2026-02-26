# PII Redacta

## Executive Summary

**PII Redacta** is a high-performance, standalone PII (Personally Identifiable Information) detection and redaction service built with Rust. It is designed as an extraction and modernization of the PII service from SecureSharing, optimized for speed, memory efficiency, and zero-knowledge architecture.

**Scope:** Text-based content and document processing (PDF, Word, Excel, CSV, etc.)

**Security Model:** Hybrid approach
- **Standard Mode (MVP):** Simple API, server-side processing
- **Zero-Knowledge Mode (Enterprise):** Encrypted processing, client holds keys

---

## Key Highlights

| Metric | Existing (Elixir) | PII Redacta (Rust) | Improvement |
|--------|-------------------|-------------------|-------------|
| Pattern Detection Latency (P99) | ~5ms | <1ms | **5x faster** |
| Memory Footprint (Base) | ~100MB | ~30MB | **70% reduction** |
| Memory Footprint (With Models) | ~500MB | ~200MB | **60% reduction** |
| Throughput | 2,000 req/s | 10,000+ req/s | **5x throughput** |
| Binary Size | ~80MB (BEAM) | ~15MB | **5x smaller** |
| Cold Start | 2-3s | <100ms | **20x faster** |
| Document Formats | 4 formats | 10+ formats | **3x more** |

---

## Architecture Decision Summary

### Technology Stack: Rust

After evaluating Go, Zig, Bun/TypeScript, and maintaining Elixir, **Rust** was selected for the following reasons:

| Criteria | Rust | Go | Elixir | Recommendation |
|----------|------|-----|--------|----------------|
| Memory Safety | ✓ Compile-time | GC | GC | **Rust** |
| Performance | Native | Native | VM | Tie |
| Memory Control | Full | Limited | Limited | **Rust** |
| Secure Zeroization | ✓ Guaranteed | ✗ | ✗ | **Rust** |
| PQ Crypto Ecosystem | ✓ Mature | Partial | Custom | **Rust** |
| PostgreSQL 18 Support | ✓ sqlx | ✓ pgx | ✓ Ecto | Tie |
| Concurrency Model | Async/await | Goroutines | Actor model | Tie |
| Deployment Complexity | Low | Low | Medium | Tie |

### Key Rust Crates

| Function | Crate | Version |
|----------|-------|---------|
| gRPC Server | `tonic` | 0.11 |
| REST Server | `axum` | 0.7 |
| Database | `sqlx` | 0.7 |
| Async Runtime | `tokio` | 1.x |
| AES-256-GCM | `aes-gcm` | 0.10 |
| ML-KEM-768 | `ml-kem` | 0.1 |
| Pattern Matching | `aho-corasick` | 1.1 |
| Secure Memory | `memsec` + `zeroize` | Latest |

---

## Zero-Knowledge Architecture

PII Redacta maintains strict zero-knowledge principles:

```
┌─────────┐         ┌──────────────┐         ┌──────────┐
│ Client  │         │ PII Redacta  │         │ Database │
│ (Owner  │         │ (Never sees  │         │ (Stores  │
│  of     │◄───────►│  plaintext)  │◄───────►│ encrypted│
│  keys)  │         │              │         │  data)   │
└────┬────┘         └──────────────┘         └──────────┘
     │                      │
     │ 1. Client encrypts   │
     │    token map with    │
     │    DEK               │
     │                      │
     │ 2. Client wraps DEK  │
     │    with service's    │
     │    ML-KEM public key │
     │                      │
     │ 3. Service can only  │
     │    decrypt DEK, not  │
     │    the token map     │
     │                      │
     │ 4. Service uses DEK  │
     │    to encrypt new    │
     │    token maps        │
```

**Security Guarantees:**
- ✓ Server never stores plaintext PII
- ✓ Server cannot decrypt token maps without client's DEK
- ✓ All detection happens in secure memory (mlock)
- ✓ Automatic zeroization after use
- ✓ Post-quantum cryptography (ML-KEM-768, FIPS 203)

---

## Project Timeline

```
Week:  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16
       ├─ Foundation ─┤├── Core Engine ──┤
                                 ├─ API Layer ─┤ MVP
                                                ├─ Advanced ─┤
                                                               ├─ Production ─┤
                                                                               ├─ Post-Launch ┤
       ████ Security
            ████ Database
                ████ Patterns
                    ████ Tokenization
                        ████ Pipeline
                            ████ gRPC
                                ████ REST + MVP
                                    ████ ML Integration
                                        ████ File Processing
                                            ████ Monitoring
                                                ████ Security Hardening
                                                    ████ Performance
                                                        ████ Production
                                                            ████ Stabilization
                                                                ████ Enhancements
```

**Milestone: MVP (Week 8)**  
**Milestone: Production (Week 14)**

---

## Documents Reference

All documentation is in the [`docs/`](./docs) folder:

| Document | Purpose | Status |
|----------|---------|--------|
| [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) | Complete system architecture, database schema, API specs | ✅ Complete v3.0 |
| [docs/ARCHITECTURE_DECISION_ZK.md](./docs/ARCHITECTURE_DECISION_ZK.md) | Hybrid security model decision rationale | ✅ Complete |
| [docs/ARCHITECTURE_GAPS.md](./docs/ARCHITECTURE_GAPS.md) | Gap analysis vs. existing SecureSharing codebase | ✅ Complete |
| [docs/ROADMAP.md](./docs/ROADMAP.md) | 16-week development roadmap with milestones | ✅ Complete |
| [docs/IMPLEMENTATION_PLAN.md](./docs/IMPLEMENTATION_PLAN.md) | Implementation guide (Standard Mode MVP + ZK Phase 2) | ✅ Complete v2.0 |
| [docs/TECH_STACK_RESEARCH.md](./docs/TECH_STACK_RESEARCH.md) | Technology evaluation and comparison | ✅ Complete |
| [docs/BILLING_PRICING.md](./docs/BILLING_PRICING.md) | Comprehensive pricing strategy for critical sectors | ✅ Complete |
| [docs/PROCUREMENT_GUIDE.md](./docs/PROCUREMENT_GUIDE.md) | Procurement guide for government, banking, insurance | ✅ Complete |
| [docs/PRICING_SUMMARY.md](./docs/PRICING_SUMMARY.md) | Quick reference pricing for all tiers | ✅ Complete |

---

## Quick Start (Future)

```bash
# Clone repository
git clone https://github.com/your-org/pii-redacta.git
cd pii-redacta

# Start dependencies
docker-compose up -d postgres

# Run migrations
sqlx migrate run

# Start development server
cargo run --bin pii-redacta-server

# Test detection
curl -X POST http://localhost:8080/api/v1/detect \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{"text": "Contact john@example.com"}'
```

---

## API Example

### Detect PII

```bash
curl -X POST https://api.pii-redacta.com/api/v1/detect \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact John at john.doe@example.com or +60-12-345-6789",
    "options": {
      "patterns_enabled": true,
      "presidio_enabled": true,
      "slm_validation": false
    }
  }'
```

**Response:**
```json
{
  "entities": [
    {
      "type": "EMAIL",
      "value": "john.doe@example.com",
      "start": 20,
      "end": 40,
      "confidence": 0.98,
      "source": "patterns"
    },
    {
      "type": "PHONE",
      "value": "+60-12-345-6789",
      "start": 44,
      "end": 59,
      "confidence": 0.95,
      "source": "patterns"
    }
  ],
  "entity_count": 2,
  "entity_types": ["EMAIL", "PHONE"]
}
```

### Tokenize with Zero-Knowledge

```bash
curl -X POST https://api.pii-redacta.com/api/v1/tokenize \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Email: john@example.com",
    "conversation_id": "conv-123",
    "encrypted_dek": "base64wrappedDEK..."
  }'
```

**Response:**
```json
{
  "tokenized_text": "Email: <<PII_EMAIL_a1b2c3d4>>",
  "encrypted_token_map": "base64encryptedMap...",
  "entity_count": 1,
  "entity_types": ["EMAIL"]
}
```

---

## Supported PII Types

| Type | Pattern Detection | ML NER | Domain-Specific |
|------|------------------|--------|-----------------|
| EMAIL | ✅ | ✅ | - |
| PHONE (Malaysian) | ✅ | ✅ | - |
| NRIC (Malaysian IC) | ✅ | - | - |
| PASSPORT | ✅ | - | - |
| CREDIT_CARD | ✅ | - | Finance |
| BANK_ACCOUNT | ✅ | - | Finance |
| IP_ADDRESS | ✅ | - | - |
| DATE_OF_BIRTH | ✅ | ✅ | - |
| ADDRESS | Partial | ✅ | - |
| NAME | - | ✅ | - |
| MEDICAL_RECORD | - | ✅ | Healthcare |
| CUSTOM | ✅ | - | Any |

---

## System Requirements

### Minimum (Development)
- CPU: 2 cores
- RAM: 4GB
- Disk: 20GB
- PostgreSQL: 18+
- OS: Linux/macOS

### Recommended (Production)
- CPU: 4+ cores
- RAM: 8GB+
- Disk: 100GB SSD
- PostgreSQL: 18+ (dedicated instance)
- Network: 1Gbps
- OS: Linux (Ubuntu 22.04 LTS recommended)

### Optional (With SLM)
- GPU: NVIDIA with 8GB+ VRAM
- RAM: 16GB+
- Ollama with Phi-3 and Mistral 7B

---

## Cost Comparison

### Infrastructure Costs (Monthly, Estimated)

| Component | Elixir Service | PII Redacta | Savings |
|-----------|---------------|-------------|---------|
| Compute (4 vCPU, 8GB) | $100 | $60 | 40% |
| Memory optimization | - | -20% | - |
| Throughput efficiency | Baseline | +400% | - |
| **Total** | **$100** | **$48** | **52%** |

*Based on AWS EC2 t3.large equivalent pricing*

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ML-KEM library issues | Low | High | Test with multiple PQ crypto libraries |
| Performance below target | Low | Medium | Early benchmarking, optimization sprints |
| Migration complexity | Medium | Medium | Phased rollout, compatibility layer |
| Team Rust expertise | Medium | Medium | Training, code reviews, pair programming |
| Security vulnerabilities | Low | Critical | Security audit, penetration testing |

---

## Next Steps

1. **Review Architecture** - Stakeholder review of [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)
2. **Approve Roadmap** - Confirm 16-week timeline and milestones
3. **Assemble Team** - Assign engineers (Rust, Backend, DevOps, QA)
4. **Setup Repository** - Initialize with proposed structure
5. **Begin Phase 1** - Foundation (Week 1-3)

---

## Contact

For questions or clarifications about this project:

- **Technical Lead:** [Your Name]
- **Security Review:** Security Team
- **Project Manager:** [PM Name]

---

## License

Proprietary - All rights reserved.

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-27  
**Status:** Ready for Review
