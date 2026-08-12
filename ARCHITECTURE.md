# Architecture — B2B Conditional Settlement Platform

**Status:** Draft · agreed before implementation  
**Scope:** Repository layout, services, and evolution from MVP (Phase 1) to production  
**Goal:** See [README.md](./README.md)

---

## 1. Principles

| Principle | Meaning |
|-----------|---------|
| **Monorepo** | All modules built from the repository root until team/compliance justify repo splits |
| **20% on-chain / 80% off-chain** | Solidity: rules + settlement. Rust: API, indexer, reconciliation, policy, compliance, adapters |
| **Platform, not one app** | Construction / trade import = **policy configs** on the same engine |
| **Chain = settlement truth** | PostgreSQL mirrors chain via indexer; reconciliation detects drift |
| **Phased delivery** | README roadmap Phases 1–4; do not skip phases without explicit decision |

### Fixed decisions (see also [README.md — Resolved decisions](./README.md#resolved-decisions))

- **Network (portfolio):** Sepolia **L1** testnet (not L2; Base/Arbitrum evaluated post-portfolio)  
- **Escrow pattern:** factory-per-deal  
- **Session keys (Phase 2+):** provider SDK (e.g. ZeroDev / Safe), not custom validator from scratch  
- **RWA (Phase 4):** real testnet token class when implemented, not simulated-only  
- **Settlement tokens:** USDC primary on Sepolia; EURC supported via same ERC-20 interface for trade_import scenarios  

---

## 2. Repository layout (root)

```
./                            # repository root
├── AGENTS.md                 # Agent behavior (not product/architecture)
├── ARCHITECTURE.md           # This file
├── README.md                 # Product brief / goal
├── theory/
│   ├── Theory.md
│   └── FurtherReading.md
│
├── contracts/                # Foundry — on-chain
├── crates/                   # Rust workspace — off-chain
├── configs/                  # Policy YAML, chain config, env templates
├── migrations/               # SQL migrations (shared by all services)
├── deploy/                   # Deployed addresses, ABIs, deployment records
├── docker/                   # Local dev compose
├── scripts/                  # Demo CLI, seed, e2e smoke tests
└── docs/                     # ADRs, diagrams, runbooks
```

### When to split repos (production, later)

| Repo | Reason |
|------|--------|
| `contracts` (audited, tagged releases) | Audit trail, immutable bytecode references |
| `platform-services` | Independent deploy cadence for Rust binaries |
| `infra` | Terraform / K8s / secrets |

Until then: **single monorepo**.

---

## 3. On-chain layer (`contracts/`)

```
contracts/
├── foundry.toml
├── remappings.txt
├── src/
│   ├── EscrowFactory.sol
│   ├── EscrowDeal.sol
│   ├── interfaces/
│   │   ├── IEscrowDeal.sol
│   │   └── ISettlementToken.sol
│   └── libs/
│       └── MilestoneLib.sol
├── test/
├── script/
│   └── Deploy.s.sol
└── deployments/
    └── sepolia.json
```

### Responsibilities

- **EscrowFactory** — deploy one `EscrowDeal` per business deal  
- **EscrowDeal** — hold stablecoin, milestone release, retention, dispute lock, refund  
- **Settlement token** — Sepolia USDC (primary), EURC-compatible ERC-20, or `MockERC20` for isolated CI  

### Deal state machine (on-chain)

```
Draft → Funded → InProgress → PendingRelease → Released
                              ↘ Disputed → (refund | partialRelease | release)
                              ↘ Refunded
```

### Events (indexer contract)

Minimum set for Phase 1:

- `DealCreated(dealId, payer, payee, token, …)`  
- `Funded(dealId, amount)`  
- `MilestoneReleased(dealId, milestoneIndex, amount, retentionHeld)`  
- `Disputed(dealId)`  
- `Refunded(dealId, amount)`  

### On-chain evolution by phase

| Phase | Additions |
|-------|-----------|
| **1 — MVP** | Factory, Deal, EOA/simple `AccessControl` approver |
| **2** | ERC-4337 org accounts, session key validator (SDK), paymaster |
| **3** | `AllowlistRegistry` or permissioned token hooks (optional ERC-3643-style) |
| **4** | `RwaVault` interface — idle escrow → RWA token deposit/withdraw |
| **Prod** | Audited contracts, multisig owner, pause/dispute roles, factory versioning (v2, not silent upgrade of live deals) |

---

## 4. Off-chain layer (`crates/` — Rust workspace)

Root `Cargo.toml` defines workspace members.

```
crates/
├── domain/                   # Shared types, enums, errors — no I/O
├── db/                       # sqlx repositories, queries
├── chain/                    # alloy: RPC, contract bindings, event decode
├── platform-api/             # binary: HTTP API (axum)
├── indexer/                  # binary: chain listener → PostgreSQL
├── reconciliation/           # binary: drift detection worker
├── policy/                   # YAML → validated DealPolicy (Phase 3 lib; stub in Phase 1)
├── compliance/               # KYB, screening, Travel Rule store (Phase 3)
└── adapters/
    ├── erp/
    ├── fiat/
    ├── swift/
    └── webhook/
```

### Services (binaries)

| Binary | Phase | Role |
|--------|-------|------|
| `platform-api` | 1 | REST: orgs, deals, approve/release **intent**, audit log |
| `indexer` | 1 | Subscribe/poll chain events → `chain_events` + update deal state |
| `reconciliation` | 1 | Compare chain escrow balance vs internal ledger vs ERP flags |
| `policy` | 3 | Library; validate deal creation against YAML policy |
| `compliance` | 3 | Library; pre-screen before release; Travel Rule metadata; mock allowlist in Phase 1 |
| `adapters/*` | 3 | Pluggable integrations; mock in portfolio |

**Phase 1 approver:** treasury **EOA** or simple `AccessControl` role — not session keys (Phase 2). Portfolio success criteria for session keys apply from Phase 2 demo onward.

**Phase 1 recommendation:** two binaries (`platform-api` + `indexer`) from day one. Optional: run indexer as task inside API for local-only hack, then split before demo.

---

## 5. Data layer

### Migrations (`migrations/`)

```
migrations/
├── 001_organizations.sql
├── 002_wallets.sql
├── 003_deals.sql
├── 004_milestones.sql
├── 005_ledger_entries.sql
├── 006_chain_events.sql
├── 007_reconciliation_runs.sql
├── 008_audit_log.sql
└── 009_compliance_metadata.sql   # Phase 3; table can exist early
```

### Core entities

| Table | Purpose |
|-------|---------|
| `organizations` | Payer / payee orgs, KYB status |
| `wallets` | On-chain address, org_id, role (treasury, payee, approver) |
| `deals` | Off-chain UUID, on-chain `deal_address`, policy_ref, status, external_refs (invoice, PO) |
| `milestones` | Index, amount, status, retention rules |
| `ledger_entries` | Internal mirror of settlement movements |
| `chain_events` | Raw indexed events (tx_hash, block, log_index, payload) |
| `reconciliation_runs` | Drift reports and alerts |
| `audit_log` | API actions + linked tx hashes |
| `compliance_metadata` | Travel Rule / screening refs by deal_id (off-chain PII) |

**ORM:** sqlx with compile-time checked queries.

### Configs (`configs/`)

```
configs/
├── policies/
│   ├── construction.yaml
│   └── trade_import.yaml
├── chains/
│   └── sepolia.toml
└── env.example
```

---

## 6. Request and settlement flow

```
[Client / CLI / future UI]
         │
         ▼
   platform-api ──► policy (Phase 3) ──► compliance gate (Phase 3)
         │
         ▼
    PostgreSQL ◄──── indexer ◄──── Sepolia RPC
         │
         ▼
  reconciliation ──► adapters/erp (mark paid)
                    └── webhook
```

### Release path

1. Client: `POST /deals/{id}/milestones/{n}/approve`  
2. Compliance (Phase 3): block until screening / Travel Rule OK  
3. API returns **calldata** or UserOp payload; approver/treasury **signs** (Phase 1: EOA; Phase 2: smart account + session key)  
4. Transaction mined → indexer writes event  
5. Reconciliation updates ledger; ERP adapter marks invoice paid  

**Rule:** settlement finality on-chain; PostgreSQL is operational mirror + audit, not source of truth for balances.

---

## 7. Integration adapters (pluggable)

| Adapter | Purpose | Portfolio | Production |
|---------|---------|-----------|------------|
| **erp** | Bind `external_invoice_id`; mark PAID on release | Mock HTTP | SAP / Oracle connector |
| **fiat** | On-ramp / off-ramp | Mock | Circle Mint / bank partner |
| **swift** | Wire fallback when policy flag set | Mock | Bank API |
| **webhook** | Notify corp systems on state change | HTTP | Signed webhooks + retry |

Implement as **traits** in `crates/adapters/`; register implementations via config.

---

## 8. Phase map: MVP → production

| Area | Phase 1 MVP | Phase 2 | Phase 3 | Phase 4 | Production |
|------|-------------|---------|---------|---------|------------|
| **Contracts** | Factory + Deal, USDC/mock | AA, paymaster, session keys | Allowlist registry | RWA vault | Audited, multisig, monitoring |
| **Wallets** | Treasury EOA approver | Org smart accounts | Invited payee wallets | — | Custody partner / HSM |
| **API** | Deal CRUD, approve intent | UserOp relay endpoint | Policy + adapter hooks; **treasury dashboard** endpoints (API-first) | Yield allocation API | HA, rate limits, mTLS |
| **Workers** | indexer + reconciliation | Optional bundler relay | ERP/fiat mock → real; **reporting export** | RWA rebalance job | K8s, queue, DLQ |
| **Compliance** | Mock allowlist gate in API | — | KYB, Travel Rule, sanctions; on-chain allowlist registry | — | Licensed CASP, IVMS101 |
| **Deploy** | docker-compose | Sepolia | Demo scripts | RWA testnet | Staging + prod K8s |
| **Observability** | Structured logs | — | Basic metrics | — | Prometheus, alerts on drift |

---

## 9. Production topology (target)

Not required for portfolio; design boundaries so this is an **incremental** step.

```
                 ┌──────────────┐
                 │  API (×N)    │
                 └──────┬───────┘
                        │
      ┌─────────────────┼─────────────────┐
      ▼                 ▼                 ▼
 PostgreSQL          Redis            Secrets / Vault
 (+ read replica)
      ▲
      │
┌─────┴─────┐      ┌─────────────┐
│  Indexer  │─────►│ Queue (opt.)│──► reconciliation, erp, webhooks
└─────┬─────┘      └─────────────┘
      │
 Sepolia L1 RPC (portfolio; L2 later)
      │
 Smart contracts
```

**Portfolio minimum:** `docker-compose` with PostgreSQL + `platform-api` + `indexer`.

---

## 10. Deploy and demo (`deploy/`, `scripts/`)

```
deploy/
├── sepolia/
│   ├── addresses.json
│   └── abis/
scripts/
├── demo_construction.sh
├── demo_trade_import.sh
└── seed_dev.sql
```

Success criterion (README): end-to-end demo reproducible via CLI/API in **< 5 minutes**.

---

## 11. Implementation order

1. `contracts/` — EscrowFactory, EscrowDeal, Foundry tests, deploy script  
2. `migrations/` + `crates/domain` + `crates/db`  
3. `crates/chain` — ABI bindings, event decoding  
4. `crates/indexer` — index `Funded`, `MilestoneReleased`, etc.  
5. `crates/platform-api` — create deal, register chain address, approve (return calldata)  
6. `crates/reconciliation` — escrow balance vs DB  
7. `docker/docker-compose.yml` + `scripts/demo_*.sh`  
8. Phase 2 — accounts, paymaster, session keys (SDK)  
9. Phase 3 — `policy/`, `compliance/`, `adapters/`  
10. Phase 4 — RWA module  

---

## 12. Out of scope for MVP

- One microservice per adapter  
- Kubernetes / multi-region  
- Live CASP, SWIFT, or ERP production connectors  
- RWA yield  
- Custom ERC-4337 validator from scratch  
- Full UI (API-first + CLI demo is sufficient for portfolio)  

---

## 13. Open architecture items

Track decisions here as they are made:

| Item | Status | Notes |
|------|--------|-------|
| Sepolia USDC vs MockERC20 for Phase 1 | TBD | Mock simplifies CI; USDC is more realistic for demo |
| EURC on Sepolia for trade_import | TBD | Use same ERC-20 interface; deploy mock EURC if unavailable |
| Indexer: polling vs WebSocket | TBD | Start polling + N confirmations; WebSocket later |
| Deal ID mapping | TBD | Off-chain UUID ↔ on-chain `bytes32 dealId` |
| API auth | TBD | API keys for portfolio; OAuth/mTLS for prod |
| Dispute resolver on-chain role | TBD | Arbiter multisig vs API-only off-chain resolution |

---

*Construction is a use case. The product is the platform.*
