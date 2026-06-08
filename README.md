# TerraLedger

> **Verified carbon credits. Permanent retirement. Full provenance.**
> A decentralized carbon credit marketplace on Stellar where carbon projects mint tokenized RWAs, corporations buy and retire them on-chain, and every credit carries an immutable audit trail from issuance to retirement.

[![Stellar](https://img.shields.io/badge/Stellar-Soroban-7C3AED?style=for-the-badge&logo=stellar&logoColor=white)](https://stellar.org)
[![Rust](https://img.shields.io/badge/Rust-Smart_Contracts-orange?style=for-the-badge&logo=rust&logoColor=white)](https://rust-lang.org)
[![Next.js](https://img.shields.io/badge/Next.js-14-black?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org)
[![USDC](https://img.shields.io/badge/Stablecoin-USDC-2775CA?style=for-the-badge)](https://circle.com)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](./LICENSE)
[![Status](https://img.shields.io/badge/Status-In_Development-yellow?style=for-the-badge)](https://github.com)
[![Tests](https://img.shields.io/badge/Tests-30_passing-brightgreen?style=for-the-badge)](./contracts)
[![Testnet](https://img.shields.io/badge/Deployed-Stellar_Testnet-blueviolet?style=for-the-badge)](https://stellar.expert/explorer/testnet)

---

## Table of Contents

- [The Problem](#-the-problem)
- [The Solution](#-the-solution)
- [How It Works](#️-how-it-works)
- [Architecture](#-architecture)
- [Smart Contracts](#-smart-contracts)
- [Tech Stack](#️-tech-stack)
- [Getting Started](#-getting-started)
- [Contract Deployment](#-contract-deployment)
- [Running Tests](#-running-tests)
- [User Roles](#-user-roles)
- [Credit Lifecycle](#-credit-lifecycle)
- [Roadmap](#-roadmap)
- [Contributing](#-contributing)
- [Security](#-security)
- [License](#-license)

---

## 🎯 New Contributors Start Here

**Want to contribute?** Get up and running in under 30 minutes:

- 📖 **[New Contributor Guide](./docs/NEW_CONTRIBUTOR_GUIDE.md)** — Complete project overview
- 🚀 **[Quick Start Guide](./docs/QUICK_START.md)** — Setup in 15–25 minutes
- ✅ **[Setup Checklist](./docs/SETUP_CHECKLIST.md)** — Verify your environment
- 🔧 **[Troubleshooting](./docs/TROUBLESHOOTING.md)** — 25 common issues solved
- 📋 **[Quick Reference](./docs/QUICK_REFERENCE.md)** — One-page command reference
- 🔑 **[Configuration Guide](./docs/configuration.md)** — Every environment variable explained
- ♻️ **[Credit Lifecycle](./docs/carbon-credit-lifecycle.md)** — Full on-chain lifecycle reference

```bash
# Verify your setup in one command
./scripts/verify-setup.sh
```

---

## The Problem

The voluntary carbon credit market moves over **$2 billion annually** — yet it is riddled with:

- **Fraud** — projects claiming credits for sequestration that never happened
- **Double-counting** — the same tonne of CO₂ sold to multiple buyers
- **Opacity** — corporations have no way to verify what they actually bought
- **Greenwashing** — retired credits with no on-chain proof of retirement
- **Inaccessibility** — small projects cannot afford traditional registry fees

The result: companies pay real money for credits that may not represent real impact, with no way to prove otherwise to regulators or the public.

---

## The Solution

**CarbonLedger** puts the entire carbon credit lifecycle on Stellar:

- Every credit is minted with a **unique serial number** — double counting is mathematically impossible
- Every retirement is **permanently irreversible on-chain** — greenwashing is eliminated
- Every credit carries **full provenance** — from project registration to satellite monitoring to issuance to retirement
- Every retirement generates a **verifiable certificate** with a permanent public URL
- The entire audit trail is **publicly accessible without a wallet**

---

## ⚙️ How It Works

```
PROJECT DEVELOPER          CARBONLEDGER               CORPORATION
       │                        │                           │
       │── Register project ───►│                           │
       │◄── Pending status ─────│                           │
       │                        │◄── Oracle monitoring ─────│
       │◄── Project verified ───│    (satellite data)       │
       │── Request minting ────►│                           │
       │◄── Credits minted ─────│                           │
       │   (serial numbers      │                           │
       │    assigned)           │                           │
       │                        │◄── Browse marketplace ────│
       │                        │◄── Purchase credits ──────│
       │◄── USDC payment ───────│                           │
       │                        │◄── Retire credits ────────│
       │                        │──► Certificate issued ────►│
       │                        │    (permanent on-chain)   │
```

---

## Architecture

```
┌───────────────────────────────────────────────────────────┐
│                  NEXT.JS 14 FRONTEND                      │
│   Landing │ Marketplace │ Audit │ Dashboard │ Retire      │
└─────────────────────────┬─────────────────────────────────┘
                          │  @stellar/stellar-sdk
                          │  @stellar/freighter-api
┌─────────────────────────▼─────────────────────────────────┐
│                SOROBAN CONTRACTS (Rust)                   │
│  carbon_registry │ carbon_credit                          │
│  carbon_marketplace │ carbon_oracle                       │
└─────────────────────────┬─────────────────────────────────┘
                          │  stellar-sdk (Python)
┌─────────────────────────▼─────────────────────────────────┐
│          ORACLE / VERIFICATION BRIDGE (Python)            │
│  verification_listener │ price_oracle │ satellite_monitor │
└─────────────────────────┬─────────────────────────────────┘
                          │
┌─────────────────────────▼─────────────────────────────────┐
│        OFF-CHAIN LAYER (PostgreSQL + IPFS)                │
│  Projects │ Batches │ Listings │ Retirements │ Certs      │
└───────────────────────────────────────────────────────────┘
```

---

## Smart Contracts

CarbonLedger deploys 4 Soroban contracts written in Rust:

### `carbon_registry`
Manages carbon project registration, verification, and lifecycle.

| Function | Description |
|---|---|
| `register_project()` | Submit new project for verification |
| `verify_project()` | Accredited verifier approves project |
| `reject_project()` | Permanently reject fraudulent project |
| `suspend_project()` | Halt issuance from project under investigation |
| `increment_issued()` | Track total credits minted per project |
| `increment_retired()` | Track total credits retired per project |
| `is_verified()` | Check project verification status |

### `carbon_credit`
Mints, transfers, and permanently retires tokenized carbon credits.

| Function | Description |
|---|---|
| `mint_credits()` | Mint credits with unique serial numbers |
| `retire_credits()` | Permanently and irreversibly retire credits |
| `transfer_credits()` | Transfer credits between accounts |
| `credit_to_address()` | Assign purchased credits to buyer |
| `get_credit_batch()` | Query batch details |
| `get_retirement_certificate()` | Retrieve permanent retirement certificate |

### `carbon_marketplace`
Handles listings, purchases, and bulk corporate buying.

| Function | Description |
|---|---|
| `list_credits()` | List credits for sale with price per tonne |
| `delist_credits()` | Remove active listing (seller only) |
| `purchase_credits()` | Buy credits — USDC to seller, credits to buyer |
| `bulk_purchase()` | Buy from multiple projects atomically |
| `get_active_listings()` | Browse available credits |
| `get_listings_by_vintage()` | Filter by vintage year |

### `carbon_oracle`
Receives satellite monitoring data and carbon price feeds.

| Function | Description |
|---|---|
| `submit_monitoring_data()` | Push satellite monitoring data on-chain |
| `update_credit_price()` | Push benchmark price per methodology/vintage |
| `flag_project()` | Flag project for investigation |
| `is_monitoring_current()` | False if no data in last 365 days |
| `get_benchmark_price()` | Get current price per methodology |

### Error Enum

```rust
pub enum CarbonError {
    ProjectNotFound=1,    ProjectNotVerified=2,   ProjectSuspended=3,
    InsufficientCredits=4, AlreadyRetired=5,      SerialNumberConflict=6,
    UnauthorizedVerifier=7, UnauthorizedOracle=8, InvalidVintageYear=9,
    ListingNotFound=10,   InsufficientLiquidity=11, PriceNotSet=12,
    MonitoringDataStale=13, DoubleCountingDetected=14, RetirementIrreversible=15,
    ZeroAmountNotAllowed=16, ProjectAlreadyExists=17, InvalidSerialRange=18,
}
```

---

## 🛠️ Tech Stack

| Layer | Technology |
|---|---|
| Smart Contracts | Rust + Soroban SDK 21.0.0 |
| Blockchain | Stellar Mainnet / Testnet |
| Frontend | Next.js 14 (App Router) + TypeScript |
| Wallet | Freighter (@stellar/freighter-api) |
| Stellar SDK | @stellar/stellar-sdk |
| Payments | USDC on Stellar |
| Oracle Bridge | Python + stellar-sdk |
| Satellite Data | Google Earth Engine |
| Price Feeds | Xpansiv CBL |
| Database | PostgreSQL + Prisma ORM |
| File Storage | IPFS via Pinata |
| Auth | JWT + Stellar keypair |
| Backend API | NestJS |
| Infra | Docker Compose + GitHub Actions |

---

## 🚀 Getting Started

**Estimated time: 25–40 minutes**

### Prerequisites

| Tool | Version |
|---|---|
| Node.js | 18+ |
| Rust | 1.74+ |
| Python | 3.10+ |
| PostgreSQL | 14+ |
| Redis | 6+ |

### Quick Setup

```bash
# 1. Clone and configure
git clone https://github.com/dev-fatima-24/carbonledger.git
cd carbonledger
cp .env.example .env

# 2. Rust toolchain
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --version 21.0.0

# 3. Generate funded testnet account
stellar keys generate deployer --network testnet --fund

# 4. Install dependencies
cd backend && npm install && npx prisma generate && npx prisma migrate dev && cd ..
cd frontend && npm install && cd ..
cd oracle && pip3 install -r requirements.txt && cd ..

# 5. Build and deploy contracts
cd contracts && cargo build --target wasm32-unknown-unknown --release && cd ..
./scripts/deploy-contracts.sh

# 6. Start everything
cd backend && npm run start:dev &    # :3001
cd frontend && npm run dev &         # :3000

# 7. Run tests
./scripts/test-all.sh
```

### Docker Alternative

```bash
cp .env.example .env
docker-compose up --build
```

---

## Contract Deployment

```bash
./scripts/deploy-contracts.sh
# Deploys all 4 contracts and writes IDs to .env automatically
```

Manual deployment:

```bash
cd contracts
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/carbon_registry.wasm \
  --source deployer --network testnet
# Repeat for carbon_credit, carbon_marketplace, carbon_oracle
```

---

## Running Tests

```bash
# All tests
./scripts/test-all.sh

# Contracts only (30 tests)
cd contracts && cargo test --workspace

# Backend only
cd backend && npm test

# Frontend only
cd frontend && npm test
```

| Contract | Tests |
|---|---|
| carbon_registry | 7 |
| carbon_credit | 10 |
| carbon_marketplace | 7 |
| carbon_oracle | 6 |

---

## User Roles

**Project Developer** — Register projects, track issued vs retired, receive USDC payments.

**Corporation** — Browse credits, purchase, retire on-chain, download ESG certificates.

**Verifier** — Approve projects for credit issuance, submit on-chain attestations.

**Public / Auditor** — Browse full audit trail without a wallet. Verify any certificate by serial number or cert ID.

---

## Credit Lifecycle

```
Registered → Verified → Oracle Monitoring → Minted (serials assigned)
→ Listed → Purchased → Retired (irreversible) → Certificate Issued → ESG Report
```

See **[docs/carbon-credit-lifecycle.md](./docs/carbon-credit-lifecycle.md)** for full reference with sequence diagrams.

---

## Roadmap

### Phase 1 — Contracts ✅
- [x] 4 Soroban contracts in Rust
- [x] 30 unit tests passing
- [x] Stellar testnet deployment
- [x] GitHub Actions CI

### Phase 2 — Oracle Layer
- [ ] Verification listener service
- [ ] Xpansiv CBL price feed integration
- [ ] Google Earth Engine satellite webhook

### Phase 3 — Frontend
- [ ] Freighter wallet integration
- [ ] Public audit explorer
- [ ] Corporate bulk purchase flow
- [ ] Retirement certificate PDF generator

### Phase 4 — Mainnet
- [ ] Smart contract security audit
- [ ] Verra VCS + Gold Standard validation
- [ ] Mainnet deployment

---

## Contributing

```bash
# Fork, branch, build, test, commit, push
git checkout -b feat/your-feature
./scripts/test-all.sh
git commit -m "feat: your feature"
git push origin feat/your-feature
```

See **[CONTRIBUTING.md](./CONTRIBUTING.md)** and **[docs/NEW_CONTRIBUTOR_GUIDE.md](./docs/NEW_CONTRIBUTOR_GUIDE.md)**.

Good first issues tagged: `good first issue`, `documentation`, `tests`, `frontend`

---

## 🔒 Security

Do not open public issues for vulnerabilities. Email **security@carbonledger.io** with details.
See **[SECURITY.md](./SECURITY.md)** for our full policy and threat model.

---

## License

MIT — see [LICENSE](./LICENSE)

---

## Acknowledgements

- [Stellar Development Foundation](https://stellar.org) — Soroban infrastructure
- [Verra VCS](https://verra.org) — carbon methodology standards
- [Gold Standard](https://goldstandard.org) — verification framework
- [Xpansiv CBL](https://xpansiv.com) — carbon price data
- [Google Earth Engine](https://earthengine.google.com) — satellite monitoring

---

**Built on Stellar. Built for the planet.**

⭐ Star this repo if CarbonLedger matters to you

[Website](#) · [Audit Explorer](#) · [Twitter](#) · [Discord](#)
