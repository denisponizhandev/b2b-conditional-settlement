# B2B Conditional Settlement Platform

**Product Brief** · v0.1 · Draft

---

## Vision

An **institutional-grade payment and settlement platform** for corporations and financial institutions.

The platform provides **modular infrastructure** for conditional B2B payments: funds are locked under agreed rules and released only when business conditions are met. **Construction milestones** are one configured use case; the same engine supports trade finance, agency deliverables, retention holdbacks, and other milestone-based flows.

Target operator: **corporate treasury**, **digital assets desk**, or **bank/fintech** offering white-label settlement to enterprise clients.

Counterparties (subcontractors, suppliers) participate as **invited payees** — they do not buy or operate the platform.

---

## Problem

Large B2B payers face recurring failures in traditional payment rails:


| Pain                            | Current reality                                          | Platform response                                             |
| ------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------- |
| **Post-payment trust gap**      | Pay after work → supplier/sub carries credit risk        | Pre-funded **escrow** with rule-based release                 |
| **Slow conditional settlement** | Invoices approved in ERP; money moves days later         | **24/7 tokenized settlement** on release                      |
| **Cross-border friction**       | Bank holds, SWIFT delays, cut-off times                  | Stablecoin rail + optional **fiat fallback**                  |
| **Idle committed capital**      | Escrow balances earn nothing                             | **Treasury yield module** (RWA / MMF token) on idle portion   |
| **Weak controls**               | Single approver, opaque audit trail                      | **Org wallets**, session keys, multi-approval, full audit log |
| **Reconciliation gap**          | Treasury paid; finance cannot match invoice ↔ settlement | **Indexer + ERP adapter** closes invoices on-chain event      |


This is the same product logic used in **trade finance (Letter of Credit)**, **construction retention**, and **corporate treasury** — delivered as a unified, configurable platform with on-chain settlement.

---

## Product Positioning

```
┌─────────────────────────────────────────────────────────────────┐
│         B2B CONDITIONAL SETTLEMENT PLATFORM                      │
│         (enterprise / bank infrastructure)                       │
├─────────────────────────────────────────────────────────────────┤
│  MODULES                    │  USE CASES (policy configs)        │
│  • Escrow engine            │  • Construction milestones       │
│  • Stablecoin settlement    │  • Import/export PO delivery     │
│  • Treasury yield (RWA)     │  • Agency / SOW deliverables     │
│  • Org smart wallets (AA)   │  • Retention & warranty holdback │
│  • Session keys & policies  │                                  │
│  • Compliance & KYB         │                                  │
│  • Reconciliation & audit   │                                  │
│  • Fiat adapters            │                                  │
│  • ERP / invoice connectors │                                  │
└─────────────────────────────────────────────────────────────────┘
```

**What this is:** internal or white-label infrastructure a corporation *could* build — comparable in shape to digital assets / tokenized payment platforms at global banks.

---

## Personas


| Persona               | Role                                | Goals                                                      |
| --------------------- | ----------------------------------- | ---------------------------------------------------------- |
| **Platform operator** | Corp treasury / bank digital assets | Configure modules, limits, compliance; platform-wide audit |
| **Payer org**         | Developer, importer, EPC treasury   | Fund deals, control release policy, optimize idle cash     |
| **Payee**             | Subcontractor, supplier             | Receive timely settlement; optional fiat off-ramp          |
| **Approver**          | PM, inspector, trade ops            | Approve milestone/delivery without treasury key access     |
| **Finance / CFO**     | Controller, reconciliation          | Invoice closed ↔ settlement ref; regulatory export         |


---

## Core Modules

### 1. Deal & Escrow Engine (on-chain)

- Create **deals** with one or more **milestones** (or single delivery condition)
- **Fund** escrow with stablecoin (USDC / EURC class)
- States: `Draft → Funded → InProgress → PendingRelease → Released | Refunded | Disputed`
- **Pull-style release**: payee receives on approved milestone (not push batch to many)
- Optional **retention** (% held until warranty period)
- Custom errors and rich **events** for downstream systems

### 2. Stablecoin Settlement

- Non-volatile settlement token (ERC-20: USDC / EURC class)
- Support for **exact-amount** funding and partial milestone releases
- Portfolio deploys on **Sepolia L1**; L2 (Base / Arbitrum) is a later optimization for lower fees

### 3. Treasury Yield on Idle Escrow (RWA module)

When a deal is pre-funded but milestones release over months, **unallocated balance** sits idle.

- **Liquid buffer**: portion kept as stablecoin for imminent releases
- **Idle portion**: optionally allocated to **tokenized money-market / T-bill product** (RWA)
- Yield accrues to **payer org** (treasury), not payee
- Before each release: RWA → stablecoin → payee

*Analogue: corporate treasury parking cash in MMF overnight — applied to escrow balances.*

### 4. Wallet & Identity (Account Abstraction)

- **Organization wallets** — hierarchical (group treasury → project / SPV wallet)
- **Payee wallets** — invited, KYC-gated
- **Paymaster** — gas abstraction; business users never hold ETH for fees
- **Session keys** — scoped approver keys (e.g. approve milestone on deal X only, 30-day expiry)
- Optional **multi-sig / multi-approval** for high-value releases

### 5. Compliance & Counterparty Registry

- KYB onboarding for payer and payee organizations
- On-chain **allowlist** (or permissioned token hooks) — only verified addresses participate *(full on-chain registry: Phase 3; Phase 1: mock allowlist gate in API)*
- Sanctions / screening hook (adapter to vendor in production; mock in portfolio)
- Travel Rule metadata stored off-chain, referenced by deal ID

### 6. Reconciliation & Audit (off-chain, Rust)

- **Indexer**: chain events → PostgreSQL
- **Reconciliation worker**: chain state must match internal ledger; alert on drift
- **Audit log**: API actions + on-chain tx hashes
- **Reporting export** for finance and compliance

### 7. Integration Adapters (pluggable)


| Adapter                   | Purpose                                          | Portfolio v1     |
| ------------------------- | ------------------------------------------------ | ---------------- |
| **ERP / Invoice**         | Bind `external_invoice_id`; mark PAID on release | Mock API         |
| **Fiat on-ramp**          | Fiat deposit → mint stablecoin to org wallet     | Mock             |
| **Fiat off-ramp**         | Payee: stablecoin → IBAN withdrawal              | Mock             |
| **SWIFT / wire fallback** | Primary on-chain; fallback marks wire settlement | Mock policy flag |
| **Webhooks**              | Notify corp systems on state changes             | HTTP             |


---

## Use Case: Construction Milestones (reference config)

**Buyer:** real estate developer (payer org on platform)  
**Payee:** EPC subcontractor (invited payee wallet)

**Policy example:**

```yaml
vertical: construction
milestones: 5
retention_pct: 10
arbiter_required: true
yield_on_idle: enabled
liquid_buffer_pct: 20
fiat_offramp_for_payee: enabled
erp_invoice_binding: required
```

**Flow** *(full product; Phase 1 MVP uses EOA approver instead of session keys — see Roadmap)*:

1. Developer treasury creates deal linked to contract / invoice refs
2. Funds escrow (full project or per phase) from project wallet
3. Idle balance above buffer → RWA yield module *(Phase 4, optional)*
4. Sub completes milestone → approver signs *(Phase 2: session key; Phase 1: treasury EOA approver)*
5. Platform releases stablecoin → sub wallet
6. Sub optionally off-ramps to bank account *(Phase 3 adapter)*
7. ERP invoice milestone marked **PAID** with settlement reference *(Phase 3 adapter)*

**Other configs:** same platform, `vertical: trade_import` — single milestone, release on delivery attestation, stronger cross-border metadata.

---

## Technology Stack


| Layer                   | Technology                                         | Responsibility                                        |
| ----------------------- | -------------------------------------------------- | ----------------------------------------------------- |
| **Settlement rules**    | Solidity (EVM)                                     | Escrow, milestones, token flows, events               |
| **Stablecoin / RWA**    | ERC-20 (+ RWA token interface v2)                  | Value transfer, yield deposit/withdraw                |
| **Wallets & execution** | ERC-4337 (smart accounts, paymaster, session keys) | Enterprise UX, scoped approvals                       |
| **Platform services**   | Rust (axum, tokio, alloy)                          | API, indexer, reconciliation, policy engine, adapters |
| **Data**                | PostgreSQL                                         | Deals, ledger, audit, org hierarchy                   |
| **Deployment**          | Sepolia testnet (portfolio) → L2 / mainnet later     | Portfolio: Sepolia L1 only                            |


**Design principle:** ~20% of value on-chain (rules + settlement), ~80% off-chain (orchestration, compliance, integration) — matching how institutional platforms are built in practice.

---

## Roadmap

### Phase 1 — Core settlement

- Escrow contract + milestone logic + Foundry tests  
- Rust indexer + REST API + reconciliation  
- End-to-end: fund → approve → release on testnet

### Phase 2 — Enterprise wallet layer

- Org + payee smart wallets (AA)  
- Paymaster (gasless)  
- Session key policy for approvers

### Phase 3 — Platform modules

- Policy engine (construction + trade configs)  
- Mock ERP, fiat, compliance adapters  
- Treasury dashboard (API-first; minimal UI)

### Phase 4 — Treasury yield (optional v2)

- RWA idle balance module with **real testnet token** integration (e.g. BUIDL-class)
- Liquidity buffer rules

---

## Success Criteria (portfolio)

- [ ] One construction deal and one trade deal run on **same platform** with policy swap only  
- [ ] Chain events fully drive internal ledger state  
- [ ] Approver operates via **session key** without treasury master key *(Phase 2; Phase 1 may use scoped EOA approver)*  
- [ ] README + ARCHITECTURE.md explain module boundaries  
- [ ] Demo script reproducible via CLI/API in < 5 minutes  

---

## Competitive / Analogue Landscape


| Analogue                                           | Relationship                                                          |
| -------------------------------------------------- | --------------------------------------------------------------------- |
| Bank **digital assets platform** (Citi, JPM class) | Same platform category; this project is a demonstrable vertical slice |
| **Letter of Credit / trade finance**               | Same conditional payment logic                                        |
| **Corporate treasury MMF**                         | Same idle-cash yield logic as RWA module                              |
| **Stripe Bridge / Circle Mint**                    | Settlement rail; this platform adds escrow + policy + ERP seam        |
| Construction retention (off-chain)                 | Same business rules; on-chain settlement upgrade                      |


---

## Resolved decisions

| Decision | Choice |
| -------- | ------ |
| Portfolio network | **Sepolia L1** testnet (L2 evaluation post-portfolio) |
| Escrow pattern | **Factory-per-deal** |
| Session keys | **Provider SDK** (ZeroDev / Safe), not custom validator from scratch |
| RWA (Phase 4) | **Real testnet token** (BUIDL-class), not simulated-only |

---

## Author & Status

**Status:** Product definition / pre-implementation  
**Author:** Denis Ponizhan  
**Purpose:** Portfolio demonstration of institutional conditional settlement platform architecture — blockchain engineer / digital assets hiring profile.

---

*Construction is a use case. The product is the platform.*