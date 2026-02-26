# PII Redacta - Procurement Guide for Critical Sectors

## Overview

This guide helps procurement teams in government, banking, insurance, and security sectors navigate the acquisition of PII Redacta. It addresses sector-specific requirements, compliance frameworks, and contracting vehicles.

---

## 1. Government Procurement

### 1.1 Federal (United States)

#### Required Contracting Vehicles

| Vehicle | Agency | PII Redacta Availability | Typical Timeline |
|---------|--------|-------------------------|------------------|
| **GSA Schedule 70** | General Services Administration | ✅ Apply in progress | 6-12 months to list |
| **CIO-SP3** | NIH | ✅ Subcontractor route | 3-6 months |
| **SEWP VI** | NASA | ✅ Through prime | 2-4 months |
| **CHELSEA** | Army | ✅ Custom contract | 12-18 months |
| **Defense ITAR** | DoD | ✅ Direct negotiation | 6-12 months |

#### FedRAMP Authorization

**Status Required:** FedRAMP Moderate (minimum) or High

**Timeline:**
```
Month 1-3:   Documentation, gap analysis
Month 4-6:   Remediation, control implementation
Month 7-9:   3PAO assessment (FedRAMP High)
Month 10-12: JAB review or Agency ATO
Month 13-18: Authorization to Operate (ATO)
```

**Cost:**
- FedRAMP Moderate: $250,000-400,000
- FedRAMP High: $500,000-800,000
- Annual maintenance: $100,000-150,000

**PII Redacta Advantage:** Built for compliance, reduces authorization time by 6 months vs. generic platforms.

#### Budget Cycle Alignment

| Budget Phase | Federal FY | Action Required |
|--------------|-----------|-----------------|
| Planning | Q1 (Oct-Dec) | Submit requirements |
| Formulation | Q2 (Jan-Mar) | Budget justification |
| Congressional | Q3 (Apr-Sep) | Await appropriation |
| Execution | Q4 (Oct+) | Contract award |

**Recommendation:** Begin procurement discussions in Q1 for Q4 award.

### 1.2 State & Local Government

#### Cooperative Purchasing Agreements

| Agreement | States Covered | Discount | Eligibility |
|-----------|---------------|----------|-------------|
| **NASPO ValuePoint** | 30+ states | 5-10% | All state agencies |
| **NYS OGS** | New York + others | 8% | State/local NY |
| **DIR Texas** | Texas + 10 states | 6% | Texas government |
| **GSA State & Local** | All 50 states | 5% | All S&L |

#### Procurement Templates

**RFP Requirements Checklist:**
- [ ] SOC 2 Type II (within 12 months)
- [ ] ISO 27001 certification
- [ ] Data residency (US-based)
- [ ] Encryption standards (FIPS 140-2)
- [ ] Audit logging (1+ year retention)
- [ ] API availability (99.99% SLA)
- [ ] Disaster recovery (4-hour RTO)
- [ ] Background checks for personnel
- [ ] Insurance ($5M+ liability)

### 1.3 International Government

| Country | Framework | Compliance Requirements |
|---------|-----------|------------------------|
| **Australia** | IRAP | Protected/Secret classification |
| **UK** | G-Cloud 13 | OFFICIAL-SENSITIVE |
| **Canada** | SSC | PBMM (Protected B)
| **Germany** | BSI | C5 criteria, GDPR |
| **Singapore** | IM8 | Government standards |
| **Malaysia** | MAMPU | PDPA, government security |
| **EU** | European Commission | GDPR, NIS2 |

---

## 2. Banking & Financial Services

### 2.1 Regulatory Requirements

| Regulation | Requirement | PII Redacta Compliance |
|------------|-------------|----------------------|
| **PCI DSS** | Cardholder data protection | Level 1 service provider |
| **GLBA** | Financial privacy | Encryption, access controls |
| **SOX** | Audit trail | Immutable logs, 7-year retention |
| **GDPR** | EU data subjects | Data residency, DPA |
| **CCPA/CPRA** | California privacy | Consumer rights automation |
| **Basel III** | Operational risk | BC/DR, SLA commitments |

### 2.2 Procurement Considerations

#### Vendor Risk Management (VRM)

**Due Diligence Checklist:**

**Financial Stability:**
- [ ] 3 years audited financials
- [ ] D&B rating (minimum 80)
- [ ] Cyber insurance ($10M+)
- [ ] Business continuity plan

**Operational Resilience:**
- [ ] Geographic redundancy
- [ ] 24/7 NOC
- [ ] Incident response plan
- [ ] Penetration testing (quarterly)

**Compliance:**
- [ ] SOC 2 Type II
- [ ] PCI DSS AOC
- [ ] ISO 27001
- [ ] External audit reports

#### Contract Terms - Banking Sector

| Term | Standard | Negotiable Range |
|------|----------|------------------|
| **Contract Length** | 3 years | 1-5 years |
| **Termination** | 90-day notice | 30-180 days |
| **Auto-renewal** | Yes | No (many banks prefer) |
| **Price Protection** | 3% annual uplift | 0-5% |
| **SLA Credits** | 100% monthly fee cap | 50-150% |
| **Liability Cap** | 12 months fees | 6-24 months |
| **Indemnification** | Mutual | Customer-favorable |

### 2.3 Implementation Timeline

**Typical Bank Implementation:**

```
Week 1-2:   Contract execution, security review
Week 3-4:   Architecture design, integration planning
Week 5-8:   Development, sandbox testing
Week 9-10:  UAT, security validation
Week 11-12: Production deployment, monitoring
Week 13-16: Optimization, training, handover
```

**Critical Path Items:**
- Security architecture approval (2-3 weeks)
- Network connectivity (firewall rules)
- Data classification assessment
- Compliance sign-off

---

## 3. Insurance Industry

### 3.1 Industry-Specific Requirements

| Requirement | Description | PII Redacta Feature |
|-------------|-------------|---------------------|
| **NAIC Model Laws** | Data security standards | SOC 2, encryption |
| **State Regulators** | NY DFS, CA DOI compliance | State-specific configs |
| **Policy Data** | PHI in insurance context | HIPAA BAA available |
| **Claims Processing** | High volume, low latency | 99.99% SLA, <10ms detection |
| **Agent Networks** | Distributed users | Multi-tenant, RBAC |
| **Reinsurance** | Cross-border data | Multi-region, GDPR |

### 3.2 Pricing Models for Insurance

#### Per-Policy Pricing

**Best for:** Life insurance, annuities, long-term policies

```
Monthly Cost = Base + (Policies Processed × Rate)

Example - Life Insurance Carrier:
Base (Enterprise):                     $9,999
Policies processed: 2,000,000
Rate: $0.005 per policy =             $10,000
─────────────────────────────────────────────
Monthly Total:                         $19,999
Annual:                                $239,988

Volume Discount:
>5M policies: -20%
>10M policies: -30%
```

#### Per-Claim Pricing

**Best for:** P&C insurance, high-frequency claims

```
Monthly Cost = Base + (Claims Processed × Rate)

Example - Auto Insurance:
Base (Professional):                   $2,499
Claims processed: 500,000
Rate: $0.01 per claim =               $5,000
─────────────────────────────────────────────
Monthly Total:                         $7,499
Annual:                                $89,988
```

#### Annual Premium Value (APV) Model

**Best for:** Large multi-line carriers

```
Monthly Cost = 0.001% of APV

Example:
Annual Premium Volume: $5 billion
Monthly Cost: $5,000,000 × 0.001% = $50,000
Annual: $600,000

Includes: Unlimited processing, Enterprise tier
```

### 3.3 Integration Requirements

**Core Systems:**
- Policy Administration Systems (PAS)
- Claims Management Systems
- Customer Communication Management (CCM)
- Document Management Systems (DMS)

**API Integration Pattern:**
```
Policy/Claim Data → PII Redacta API → Redacted Output → Core System
                         ↓
                   Token Map (encrypted)
                         ↓
                   Audit Log (compliance)
```

---

## 4. Security & Intelligence Agencies

### 4.1 Classification Levels

| Level | Processing | PII Redacta Support | Pricing Premium |
|-------|-----------|---------------------|-----------------|
| **Unclassified** | Standard cloud | ✅ Yes | Base |
| **CUI/FOUO** | GCC/GovCloud | ✅ Yes | +20% |
| **Confidential** | Air-gapped | ✅ Yes | +50% |
| **Secret** | SCIF required | ✅ With setup | +100% |
| **TS/SCI** | Compartmented | ✅ Custom | +150% |

### 4.2 Security Requirements

#### Personnel Requirements

| Role | Clearance | Background Check | Cost Addition |
|------|-----------|------------------|---------------|
| Customer Success | None | Standard | Base |
| Technical Support | None | Enhanced | +$5K/month |
| Security Engineer | Secret | SSBI | +$15K/month |
| Deployment Team | TS/SCI | Full Scope | +$30K/month |

#### Facility Requirements

| Requirement | Description | Cost |
|-------------|-------------|------|
| **SCIF Construction** | Sensitive Compartmented Info Facility | $500K-2M |
| **TEMPEST** | Electromagnetic shielding | $100K-500K |
| **Physical Security** | Guards, access control | $20K-50K/year |
| **Secure Transport** | Classified courier service | Per incident |

### 4.3 Custom Development

**Intelligence Community Features:**

| Feature | Description | Development Cost |
|---------|-------------|------------------|
| **Custom PII Patterns** | Language-specific, codeword detection | $50K-100K |
| **Cross-domain Gateway** | Transfer between classification levels | $200K-500K |
| **Audit Integration** | SIEM/Splunk classified connectors | $75K-150K |
| **Multi-tenant Isolation** | Strict enclave separation | $100K-200K |
| **FIPS 140-3 Level 3** | Hardware security modules | $300K-600K |

### 4.4 Contract Vehicles (Intelligence Community)

| Vehicle | Agency | Timeline | Ceiling |
|---------|--------|----------|---------|
| **C2E** | CIA | 12-18 months | $500M |
| **Solutions Marketplace** | NSA | 6-12 months | $100M |
| **I2S** | NRO | 12-24 months | $200M |
| **CIO-SP3** | NIH/IC | 3-6 months | $40B |
| **Seaport-NxG** | Navy | 6-12 months | $5B |

---

## 5. Compliance Framework Mapping

### 5.1 Certification Alignment

| Framework | PII Redacta Certification | Customer Responsibility |
|-----------|--------------------------|------------------------|
| **SOC 2 Type II** | ✅ Vendor provides | Review report |
| **ISO 27001** | ✅ Vendor provides | Integration scope |
| **PCI DSS** | ✅ Level 1 Service Provider | SAQ or ROC |
| **HIPAA** | ✅ BAA available | Covered entity duties |
| **FedRAMP** | ✅ Moderate/High | Agency ATO |
| **StateRAMP** | ✅ In progress | State ATO |
| **CMMC 2.0** | ✅ Level 2 capable | Organization certification |
| **NIST 800-53** | ✅ Controls mapped | System implementation |
| **IRAP** | ✅ In progress | Agency assessment |
| **GDPR** | ✅ DPA available | Controller obligations |

### 5.2 Shared Responsibility Model

```
┌─────────────────────────────────────────────────────────────┐
│                    CUSTOMER RESPONSIBILITY                   │
├─────────────────────────────────────────────────────────────┤
│ • Access management (IAM)                                   │
│ • Data classification                                       │
│ • End-user training                                         │
│ • Integration security                                      │
│ • Incident response (customer-side)                         │
│ • Backup/DR strategy (data)                                 │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    SHARED RESPONSIBILITY                     │
├─────────────────────────────────────────────────────────────┤
│ • Encryption key management (customer holds keys)           │
│ • Network security (VPC/VNet configuration)                 │
│ • Logging configuration                                     │
│ • Patch management coordination                             │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                     PII REDACTA RESPONSIBILITY               │
├─────────────────────────────────────────────────────────────┤
│ • Infrastructure security                                   │
│ • Application security                                      │
│ • Data encryption (at rest, in transit)                     │
│ • Vulnerability management                                  │
│ • Physical security                                         │
│ • Personnel security                                        │
│ • Compliance certifications                                 │
└─────────────────────────────────────────────────────────────┘
```

---

## 6. Evaluation Scorecard

### 6.1 Technical Evaluation (40%)

| Criteria | Weight | Scoring |
|----------|--------|---------|
| Detection Accuracy | 10% | 95%+ = 10 pts, 90-95% = 7 pts, <90% = 4 pts |
| Processing Speed | 10% | <10ms = 10 pts, 10-50ms = 7 pts, >50ms = 4 pts |
| Supported Formats | 5% | 10+ formats = 10 pts, 5-9 = 7 pts, <5 = 4 pts |
| API Quality | 5% | REST+gRPC+SDKs = 10 pts, REST only = 7 pts |
| Integration Ease | 5% | Pre-built connectors = 10 pts, API only = 6 pts |
| Scalability | 5% | Horizontal scaling = 10 pts, Vertical only = 6 pts |

### 6.2 Security Evaluation (30%)

| Criteria | Weight | Scoring |
|----------|--------|---------|
| Certifications | 10% | SOC2+ISO27001+FedRAMP = 10 pts, SOC2 only = 6 pts |
| Encryption Standards | 5% | AES-256-GCM+PQ = 10 pts, AES only = 7 pts |
| Audit Logging | 5% | Immutable, detailed = 10 pts, basic = 6 pts |
| Incident History | 5% | No breaches = 10 pts, minor = 7 pts, major = 0 pts |
| Penetration Testing | 5% | Quarterly = 10 pts, annual = 7 pts, none = 0 pts |

### 6.3 Commercial Evaluation (20%)

| Criteria | Weight | Scoring |
|----------|--------|---------|
| Total Cost of Ownership | 10% | Lowest = 10 pts, within 20% = 7 pts, >20% = 4 pts |
| Contract Flexibility | 5% | Favorable terms = 10 pts, standard = 7 pts, rigid = 4 pts |
| Price Predictability | 5% | Fixed/transparent = 10 pts, variable = 6 pts |

### 6.4 Vendor Evaluation (10%)

| Criteria | Weight | Scoring |
|----------|--------|---------|
| Financial Stability | 3% | Profitable/growing = 10 pts, stable = 7 pts, risky = 3 pts |
| Customer References | 4% | Similar sector = 10 pts, other sectors = 6 pts, none = 2 pts |
| Support Quality | 3% | 24/7 dedicated = 10 pts, business hours = 6 pts |

---

## 7. Negotiation Strategies

### 7.1 Government Negotiation

**Best Alternative to Negotiated Agreement (BATNA):**
- Self-development: 18-24 months, $3-5M
- Competitor: Compare with BigID, OneTrust pricing
- Status quo: Risk of audit findings, fines

**Negotiation Levers:**
1. **Volume commitment:** Multi-year, multi-agency deals
2. **Reference ability:** Allow case studies, testimonials
3. **Pilot success:** Paid PoC leading to production
4. **GSA Schedule:** Pre-negotiated pricing

**Typical Government Discounts:**
- Federal: 5-10%
- State: 3-8%
- Multi-agency: Additional 3-5%

### 7.2 Banking Negotiation

**Key Concerns:**
- Risk mitigation (liability, insurance)
- Regulatory compliance burden
- Integration complexity
- Operational resilience

**Negotiation Levers:**
1. **Competitive pressure:** AWS Comprehend, Google DLP pricing
2. **Proof of concept:** Successful pilot reduces risk
3. **Multi-product:** Bundle with professional services
4. **Reference programs:** Co-marketing opportunities

**Typical Banking Terms:**
- 3-year term with 2 one-year extensions
- Annual true-up for volume
- Mutual indemnification
- SOC 2/PCI audit rights

### 7.3 Insurance Negotiation

**Key Concerns:**
- Seasonal volume fluctuations
- M&A integration (acquiring companies)
- Legacy system integration
- Cost per transaction

**Negotiation Levers:**
1. **Volume commitments:** Annual minimums with tiered pricing
2. **Seasonality:** True-up at year-end vs. monthly
3. **Acquisition clauses:** Rapid onboarding for acquired books

**Typical Insurance Terms:**
- Annual reconciliation (not monthly overage)
- Acquisition: 90-day onboarding guarantee
- Hurricane/seasonal: Burst capacity included

---

## 8. Implementation Checklist

### 8.1 Pre-Contract

- [ ] Technical evaluation completed
- [ ] Security review approved
- [ ] Legal review of terms
- [ ] Budget approved
- [ ] Procurement method determined
- [ ] Stakeholder sign-off

### 8.2 Contract Execution

- [ ] Master Service Agreement (MSA) signed
- [ ] Statement of Work (SOW) defined
- [ ] Data Processing Agreement (DPA) executed
- [ ] Business Associate Agreement (BAA) if HIPAA
- [ ] Insurance certificates received
- [ ] Security documentation exchanged

### 8.3 Post-Contract

- [ ] Kickoff meeting scheduled
- [ ] Technical team assigned
- [ ] Security onboarding initiated
- [ ] Integration planning begun
- [ ] Training schedule established
- [ ] Go-live date confirmed

---

## 9. Risk Mitigation

### 9.1 Procurement Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Budget cuts | Medium | High | Multi-year funding commitment |
| Scope creep | High | Medium | Clear SOW, change control |
| Vendor failure | Low | Critical | Escrow, financial review |
| Integration delays | Medium | Medium | Phased approach, sandbox |
| Compliance gaps | Medium | High | Pre-contract audit |

### 9.2 Contract Protections

**Recommended Clauses:**

1. **Termination for Convenience**
   - Government standard: 30-day notice
   - Commercial: Negotiate based on term

2. **Service Level Agreements**
   - Specific, measurable metrics
   - Credit mechanism defined
   - Cure period specified

3. **Data Handling**
   - Deletion procedures
   - Return of data format
   - Verification requirements

4. **Intellectual Property**
   - Customer data ownership
   - Derivative works (if custom dev)
   - License grants

5. **Limitation of Liability**
   - Mutual cap (typically 12 months fees)
   - Exclusions (gross negligence, willful misconduct)
   - Insurance requirements

---

## 10. Contact Information

### PII Redacta Procurement Team

| Role | Contact | Purpose |
|------|---------|---------|
| **Federal Sales** | federal@piiredacta.com | Government contracts |
| **Commercial Sales** | sales@piiredacta.com | Banking, insurance |
| **Security Office** | security@piiredacta.com | Compliance, audits |
| **Legal** | legal@piiredacta.com | Contract terms |
| **Partnerships** | partners@piiredacta.com | Resellers, SIs |

### Regional Offices

| Region | Location | Coverage |
|--------|----------|----------|
| Americas | Washington, DC | US Federal, Canada, LATAM |
| EMEA | London, UK | UK, EU, Middle East, Africa |
| APAC | Singapore | Asia-Pacific, Australia |

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-27  
**Classification:** Public (Unclassified)
