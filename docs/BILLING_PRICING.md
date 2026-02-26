# PII Redacta - Billing & Pricing Strategy

## Executive Summary

**PII Redacta** targets critical sectors with high compliance requirements and zero tolerance for data breaches. This document outlines a **hybrid pricing model** combining data volume tiers with feature-based premiums, designed for government, banking, insurance, and security agencies.

### Key Pricing Principles

1. **Predictability**: Fixed-price tiers for budget planning (essential for government)
2. **Compliance-by-Design**: Security features included, not add-ons
3. **Transparency**: No hidden fees for API calls or storage
4. **Value-Based**: Price aligned with risk reduction and compliance value

---

## 1. Pricing Tiers

### 1.1 Tier Overview

| Tier | Monthly | Annual (17% discount) | Target Customers |
|------|---------|----------------------|------------------|
| **Starter** | $499 | $4,999 | SMB, Startups, Non-profits |
| **Professional** | $2,499 | $24,999 | Healthcare, Mid-market, Legal |
| **Enterprise** | $9,999 | $99,999 | Banking, Insurance, Large Enterprise |
| **Critical Infrastructure** | Custom | Custom | Government, Defense, Intelligence |
| **Sovereign Cloud** | Custom | Custom | Nations requiring data sovereignty |

### 1.2 Detailed Tier Comparison

#### Starter Tier - $499/month

**Included:**
- Up to 100,000 API requests/month
- 10GB data processing/month
- Standard PII patterns (50+ types)
- REST API + gRPC
- AES-256-GCM encryption
- PostgreSQL storage
- Email support (business hours)
- 99.9% uptime SLA
- SOC 2 Type II compliant

**Limitations:**
- No ML-KEM post-quantum crypto
- Shared infrastructure only
- 30-day audit log retention
- No custom PII patterns
- Community support only

**Ideal for:**
- Small law firms
- Healthcare clinics (small)
- Non-profit organizations
- Educational institutions

---

#### Professional Tier - $2,499/month

**Included:**
- Up to 1,000,000 API requests/month
- 100GB data processing/month
- All PII patterns (100+ types)
- ML-KEM-768 post-quantum encryption
- Document classification (AI)
- Custom PII patterns (up to 10)
- On-premise option (+$1,000/month)
- Priority support (business hours)
- 99.95% uptime SLA
- SOC 2 Type II + ISO 27001
- 90-day audit log retention
- API rate limit: 1,000 req/min

**Add-ons:**
- HIPAA compliance package: +$500/month
- Additional data: $20/GB
- Additional custom patterns: $100/pattern
- 24/7 support: +$1,000/month

**Ideal for:**
- Regional banks
- Insurance agencies
- Healthcare networks
- Legal enterprises
- Fintech companies

---

#### Enterprise Tier - $9,999/month

**Included:**
- Unlimited API requests
- 1TB data processing/month
- All PII patterns + unlimited custom
- ML-KEM-768 + optional KAZ-KEM
- Advanced document parsing (PDF, Word, Excel)
- Presidio integration
- Local SLM validation (Ollama)
- Air-gapped deployment option
- Dedicated infrastructure available
- 24/7 phone support
- Dedicated Customer Success Manager
- 99.99% uptime SLA
- SOC 2 + ISO 27001 + PCI DSS
- 1-year audit log retention
- Custom SLA terms negotiable
- API rate limit: 10,000 req/min
- Multi-region deployment
- DR/BC included

**Add-ons:**
- Additional regions: +$2,000/region/month
- PCI DSS compliance: +$1,500/month
- Dedicated infrastructure: +$5,000/month
- Custom ML model training: Starting at $25,000

**Ideal for:**
- National banks
- Large insurance carriers
- Healthcare systems
- Critical infrastructure operators
- Global enterprises

---

#### Critical Infrastructure Tier - Custom Pricing

**Included:**
- Everything in Enterprise
- FedRAMP High or equivalent
- TS/SCI cleared personnel available
- SCIF-compatible deployment
- Dedicated air-gapped infrastructure
- Custom cryptography (FIPS 140-3 Level 3)
- Nation-state threat protection
- Custom security controls
- Dedicated Security Engineer
- On-site support available
- 99.999% uptime SLA
- Custom contract terms
- Unlimited audit log retention
- Classified processing capability (optional)

**Pricing Factors:**
- Number of classified enclaves
- Clearance requirements
- Physical security requirements
- Custom compliance frameworks
- Integration complexity

**Starting at:** $50,000/month

**Ideal for:**
- Federal government agencies
- Intelligence community
- Defense contractors
- Central banks
- National security organizations

---

## 2. Billing Models

### 2.1 Primary Model: Hybrid (Recommended)

**Base Fee** (predictable) + **Usage Component** (scalable)

```
Monthly Bill = Base Tier Fee + Overage Charges + Add-ons

Example (Professional Tier):
Base: $2,499
Data processed: 150GB (50GB overage @ $20/GB) = $1,000
Total: $3,499
```

### 2.2 Alternative Models by Sector

#### Government/Defense (Fixed-Price Preferred)

```
Annual Contract = Fixed Price × 3-5 Years

Example:
Year 1: $600,000 (includes implementation)
Year 2: $500,000
Year 3: $500,000
Year 4: $515,000 (3% uplift)
Year 5: $530,450 (3% uplift)
```

**Why fixed-price:**
- Budget certainty for multi-year appropriations
- Simplified procurement
- Easier audit trails
- Reduced administrative overhead

#### Banking/Finance (Transaction-Volume Hybrid)

```
Monthly Bill = Base Fee + (Transaction Volume × Rate)

Tiers:
0-10M transactions: Included
10M-100M: $0.0001 per transaction
100M+: $0.00005 per transaction

Example:
Base (Enterprise): $9,999
Monthly transactions: 150M
  - First 10M: Included
  - Next 90M: $9,000
  - Remaining 50M: $2,500
Total: $21,499
```

#### Insurance (Per-Policy Hybrid)

```
Monthly Bill = Base Fee + (Policies Processed × Rate)

Rate: $0.01-0.05 per policy (volume discounts)

Example:
Base (Professional): $2,499
Policies processed: 500,000 @ $0.02 = $10,000
Total: $12,499
```

### 2.3 Usage Metrics

| Metric | Measurement | Billing Granularity |
|--------|-------------|---------------------|
| API Requests | Per call | Per 1,000 requests |
| Data Volume | Bytes processed | Per GB (rounded up) |
| Documents | Per file | Per 100 documents |
| Storage | Token maps, audit logs | Per GB/month |
| Compute Time | Processing seconds | Per hour |

---

## 3. Compliance & Security Premiums

### 3.1 Compliance Certifications

| Certification | Included In | Premium (if add-on) | Notes |
|---------------|-------------|---------------------|-------|
| SOC 2 Type II | All tiers | Base requirement | Annual audit included |
| ISO 27001 | Professional+ | +$500/month | Certification maintenance |
| HIPAA | Professional+ (add-on) | +$500/month | BAA required |
| PCI DSS | Enterprise+ | +$1,500/month | SAQ assistance |
| FedRAMP Moderate | Critical Infra | +40% base | 12-18 month authorization |
| FedRAMP High | Critical Infra | +60% base | Full 3PAO assessment |
| IRAP (Australia) | Enterprise+ | +30% base | Australian government |
| GDPR (EU) | All tiers | Included | Data processing agreement |
| PDPA (Malaysia) | All tiers | Included | Local compliance |

### 3.2 Deployment Options

| Option | Premium | Security Level | Best For |
|--------|---------|----------------|----------|
| Shared Cloud (Multi-tenant) | Base | Standard | SMB, Mid-market |
| Dedicated Cloud (Single-tenant) | +50% | Enhanced | Large enterprise |
| VPC/VNet Isolation | +75% | High | Financial services |
| On-Premise (Customer datacenter) | +100% | Very High | Regulated industries |
| Air-gapped (No internet) | +150% | Maximum | Classified/SCIF |
| Sovereign Cloud (National) | +200% | Maximum | Nation-states |

### 3.3 Data Residency

| Region | Premium | Notes |
|--------|---------|-------|
| US (Multi-region) | Base | Default |
| EU (GDPR compliant) | +10% | Frankfurt, Dublin |
| UK | +10% | London |
| Australia (IRAP) | +20% | Sydney, Melbourne |
| Singapore | +10% | PDPA compliant |
| Malaysia | +10% | MY PDPA compliant |
| Japan | +15% | Tokyo, Osaka |
| UAE | +25% | Dubai |
| Custom sovereign | +100% | Dedicated region |

---

## 4. Professional Services

### 4.1 Implementation Packages

| Package | Price | Duration | Includes |
|---------|-------|----------|----------|
| **Self-Service** | Free | - | Documentation, community support |
| **Standard Implementation** | $15,000 | 2-4 weeks | Setup, basic integration, training |
| **Enterprise Implementation** | $50,000 | 4-8 weeks | Custom integration, optimization, dedicated engineer |
| **Critical Infrastructure** | $150,000+ | 8-16 weeks | Air-gap setup, security hardening, compliance validation |

### 4.2 Training Programs

| Program | Price | Duration | Audience |
|---------|-------|----------|----------|
| Admin Training | $2,500/person | 1 day | System administrators |
| Developer Training | $3,500/person | 2 days | Integration engineers |
| Security Training | $5,000/person | 2 days | Security officers, auditors |
| Train-the-Trainer | $10,000 | 3 days | Internal training teams |
| Custom Workshop | $8,000/day | Variable | Specific use cases |

### 4.3 Support Tiers

| Tier | Price | Response Time | Channels | Included In |
|------|-------|---------------|----------|-------------|
| **Community** | Free | Best effort | Forums | Starter |
| **Standard** | Included | 24 hours | Email | Professional+ |
| **Priority** | +$1,000/month | 4 hours | Email, Phone | Enterprise+ |
| **Premium** | +$5,000/month | 1 hour | All + Slack | Enterprise+ (add-on) |
| **Mission Critical** | +$15,000/month | 15 minutes | All + on-site | Critical Infrastructure |

### 4.4 Consulting Services

| Service | Rate | Description |
|---------|------|-------------|
| Solution Architecture | $300/hour | Custom integration design |
| Security Assessment | $25,000 flat | Penetration testing, audit |
| Compliance Consulting | $400/hour | Framework alignment |
| Custom Development | $25,000/sprint | Feature development |
| Performance Optimization | $15,000 flat | Tuning for scale |

---

## 5. Contract Terms

### 5.1 Standard Contract Terms

| Element | Standard | Negotiable |
|---------|----------|------------|
| **Term Length** | 1 year | 3-5 years (government) |
| **Payment Terms** | Monthly in advance | Annual (2 months free) |
| **Auto-renewal** | Yes, 30-day notice | No (government preference) |
| **Annual Uplift** | 3% | 0-5% based on term |
| **Termination** | 30-day notice | Immediate (material breach) |
| **SLA Credits** | Up to 100% monthly fee | Negotiable |
| **Liability Cap** | 12 months fees | Higher for Critical Infra |
| **Indemnification** | Standard | Enhanced (mutual) |

### 5.2 Government Contracting Vehicles

| Vehicle | Description | Discount |
|---------|-------------|----------|
| **GSA Schedule** | Federal supply schedule | 5-10% |
| **SEWP** | NASA IT contracts | 5% |
| **CIO-SP3** | NIH contracts | 5% |
| **State Contracts** | Cooperative purchasing | 3-8% |
| **Direct Negotiation** | Custom terms | 0-15% |

### 5.3 SLA Commitments

| Tier | Uptime | RTO | RPO | Support Response |
|------|--------|-----|-----|------------------|
| Starter | 99.9% | 4 hours | 1 hour | 24 hours |
| Professional | 99.95% | 2 hours | 30 min | 4 hours |
| Enterprise | 99.99% | 1 hour | 15 min | 1 hour |
| Critical Infra | 99.999% | 15 min | 5 min | 15 min |

**SLA Credits:**
- 99.9% - 99.0%: 10% credit
- 99.0% - 95.0%: 25% credit
- Below 95.0%: 50% credit
- Below 90.0%: 100% credit + termination right

---

## 6. Discount Structure

### 6.1 Volume Discounts

| Annual Commitment | Discount | Notes |
|-------------------|----------|-------|
| $50,000 - $99,999 | 5% | Mid-market |
| $100,000 - $249,999 | 10% | Large enterprise |
| $250,000 - $499,999 | 15% | Strategic account |
| $500,000 - $999,999 | 20% | Major account |
| $1,000,000+ | Custom (25%+) | Flagship account |

### 6.2 Multi-Year Discounts

| Term | Discount | Payment Terms |
|------|----------|---------------|
| 2 years | 5% | Annual |
| 3 years | 10% | Annual |
| 5 years | 15% | Annual or quarterly |

### 6.3 Non-Profit & Education

| Sector | Discount | Eligibility |
|--------|----------|-------------|
| Registered 501(c)(3) | 50% | US non-profits |
| Accredited Institutions | 40% | Universities, colleges |
| K-12 Schools | 60% | Public schools |
| Government (non-defense) | 30% | Federal, state, local |
| Open Source Projects | 100% | Approved projects |

### 6.4 Partner Discounts

| Partner Type | Discount | Requirements |
|--------------|----------|--------------|
| **Resellers** | 20-30% | Annual sales commitments |
| **Systems Integrators** | 15-25% | Implementation partnership |
| **Technology Partners** | 10-20% | Integration development |
| **Referral Partners** | 10-15% | Lead generation |

---

## 7. Special Programs

### 7.1 Proof of Concept (PoC) Program

**Duration:** 30-90 days  
**Cost:** Free to $10,000 (credited on conversion)  
**Includes:**
- Full Enterprise features
- Up to 10GB data processing
- Dedicated engineer support
- Custom integration assistance
- Security assessment

**Conversion Rate Target:** 60%+

### 7.2 Startup Program

**Eligibility:**
- Founded < 5 years ago
- < $10M funding
- < 50 employees

**Benefits:**
- Professional Tier at Starter price ($499/month)
- Free implementation
- 12-month price lock
- Y Combinator/techstars portfolio: Additional 6 months free

### 7.3 Disaster Recovery Program

**For critical infrastructure:**
- Hot standby in secondary region
- Automated failover
- 15-minute RTO guarantee
- Price: +50% of primary region

---

## 8. Competitive Positioning

### 8.1 Competitor Pricing Comparison

| Competitor | Model | Price Range | PII Redacta Advantage |
|------------|-------|-------------|----------------------|
| **AWS Comprehend** | Per-character | $0.0001/100 chars | Predictable pricing, no data egress fees |
| **Google Cloud DLP** | Per-GB | $1-3/GB | Lower cost at scale, on-prem option |
| **Microsoft Presidio** | Open source | Free (self-hosted) | Managed service, support, compliance |
| **OneTrust** | Per-user | $30-100/user/mo | Usage-based, no per-seat minimums |
| **BigID** | Custom | $50K-500K/year | Transparent pricing, faster deployment |
| **Nightfall AI** | Hybrid | $5-50/user/mo + data | Better PQ crypto, lower latency |
| **Private AI** | Per-token | $0.001-0.01/token | Flat rate, no LLM token costs |

### 8.2 Value Proposition by Sector

#### Banking
- **ROI:** $2.5M saved per data breach avoided
- **Price:** $120K/year (Enterprise)
- **Payback:** < 1 month

#### Healthcare
- **ROI:** HIPAA violation avoidance ($100K-1.5M fines)
- **Price:** $30K/year (Professional)
- **Payback:** Immediate

#### Government
- **ROI:** FedRAMP authorization included (saves 18 months)
- **Price:** $600K/year (Critical Infrastructure)
- **Payback:** 3-6 months vs self-build

---

## 9. Billing Operations

### 9.1 Invoicing

| Method | Availability | Terms |
|--------|--------------|-------|
| Credit Card | All tiers | Monthly auto-charge |
| ACH/Wire | Professional+ | Net 30 |
| Purchase Order | Enterprise+ | Net 30-60 (negotiable) |
| Government Invoice | All government | Net 30, GSA terms |

### 9.2 Billing Cycles

- **Monthly:** Default for all tiers
- **Quarterly:** Available with 2% discount
- **Annual:** 17% discount (2 months free)

### 9.3 Overages

| Overage Type | Rate | Notification |
|--------------|------|--------------|
| Data Volume | Tier rate | 80% threshold alert |
| API Requests | $0.001/request | 90% threshold alert |
| Storage | $0.10/GB/month | 85% threshold alert |

**Overage Cap:** 200% of base fee (soft limit, can be increased)

---

## 10. Revenue Projections

### 10.1 Year 1 Targets (Conservative)

| Tier | Customers | ARR | Mix |
|------|-----------|-----|-----|
| Starter | 20 | $100K | 10% |
| Professional | 10 | $250K | 25% |
| Enterprise | 3 | $300K | 30% |
| Critical Infra | 1 | $600K | 35% |
| **Total** | **34** | **$1.25M** | **100%** |

### 10.2 Year 3 Targets (Growth)

| Tier | Customers | ARR | Mix |
|------|-----------|-----|-----|
| Starter | 100 | $500K | 5% |
| Professional | 80 | $2.0M | 20% |
| Enterprise | 30 | $3.0M | 30% |
| Critical Infra | 8 | $4.5M | 45% |
| **Total** | **218** | **$10M** | **100%** |

### 10.3 Average Revenue Per Account (ARPA)

| Year | ARPA | Growth |
|------|------|--------|
| Year 1 | $36,765 | - |
| Year 2 | $52,632 | +43% |
| Year 3 | $45,872 | -13% (volume) |
| Year 5 | $41,667 | - |

---

## 11. Pricing Governance

### 11.1 Approval Matrix

| Discount | Sales Rep | Sales Director | VP Sales | CEO |
|----------|-----------|----------------|----------|-----|
| Standard pricing | ✓ | - | - | - |
| Up to 10% | - | ✓ | - | - |
| Up to 20% | - | - | ✓ | - |
| Up to 30% | - | - | - | ✓ |
| >30% or custom | - | - | - | Board |

### 11.2 Price Changes

- **Annual Review:** Q4 each year
- **Notification:** 90 days advance
- **Grandfathering:** Existing customers locked for contract term
- **New Customers:** New pricing effective immediately

---

## 12. Appendix: Sample Quotes

### 12.1 Regional Bank (Enterprise Tier)

```
PII Redacta - Enterprise Subscription
Customer: Regional Bank of [State]
Term: 3 years

Annual Subscription:        $99,999
PCI DSS Compliance:         $18,000
24/7 Premium Support:       $60,000
Dedicated Infrastructure:   $60,000
Multi-region (3 regions):   $72,000
Implementation:             $50,000
─────────────────────────────────────
Year 1 Total:               $359,999

Years 2-3:                  $309,999/year
3-Year Total:               $979,997

Discounts Applied:
- 3-year term: -10%
- Multi-product: -5%
─────────────────────────────────────
Final 3-Year Total:         $881,997
```

### 12.2 Federal Agency (Critical Infrastructure)

```
PII Redacta - Critical Infrastructure
Customer: [Agency] - Classified Division
Term: 5 years (GSA Schedule)

Base Subscription:          $600,000/year
FedRAMP High:               $360,000/year
Air-gapped Deployment:      $900,000/year
TS/SCI Personnel:           $240,000/year
SCIF Setup (one-time):      $500,000
Implementation:             $200,000
─────────────────────────────────────
Year 1 Total:               $2,800,000

Years 2-5:                  $2,100,000/year
5-Year Total:               $11,200,000

Discounts Applied:
- GSA Schedule: -5%
- 5-year term: -15%
─────────────────────────────────────
Final 5-Year Total:         $9,072,000
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-27  
**Next Review:** Q4 2026  
**Owner:** Revenue Operations
