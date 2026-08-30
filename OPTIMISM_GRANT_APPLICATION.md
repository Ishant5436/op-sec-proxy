# Optimism Builder Grant Application: OP Security Proxy

**Grant Program:** Optimism Foundation Builder Grants (Season 7)  
**Track:** Developer Tooling & Superchain Public Goods  
**Requested Amount:** 10,000 OP (~$15,000 USD equivalent)  
**Project Repository:** [https://github.com/Ishant5436/op-sec-proxy](https://github.com/Ishant5436/op-sec-proxy)  
**Governance Discussion:** [Discourse Thread #10810](https://gov.optimism.io/t/op-security-proxy-a-local-rpc-middleware-to-prevent-paying-for-failed-l2-executions/10810)  
**Applicant:** Ishant Panchal (`Ishant5436` / `ishant.p@somaiya.edu`)  

---

## 1. Executive Summary

**OP Security Proxy** is a high-throughput, non-custodial JSON-RPC middleware built in Rust that sits between end-user wallets (or automated AI agents) and OP Stack RPC nodes.

### The Problem
On EVM networks, users pay transaction fees even when transactions revert on-chain due to localized slippage, insufficient allowance, stale oracle data, or sandwich attacks. For retail users, paying gas for a failed execution creates confusion and dissatisfaction; for wallet providers, it creates customer support friction.

### The Solution
`op-sec-proxy` intercepts outbound `eth_sendRawTransaction` payloads and simulates them locally using `revm` (Rust Ethereum Virtual Machine) against real-time state before broadcasting. If a transaction reverts:
1. The broadcast is preemptively aborted (0 gas burned on-chain).
2. The proxy returns a standard JSON-RPC error containing decoded Solidity custom errors.
3. The response includes an `estimated_gas_saved` metric so wallets can confirm protected funds to the user.

---

## 2. Current Implementation & Technical Evidence

* **Core Engine:** Built in Rust using `tokio` (async runtime), `hyper` (HTTP server), `alloy` (Ethereum types), and `revm` (in-memory execution).
* **Test Coverage:** 26 automated unit and integration tests passing (`cargo test`).
* **Routing Overhead:** Measured at **-71.76 ms** relative to direct upstream RPC on standard non-mutating calls (`eth_blockNumber`, `eth_call`) due to connection pooling (`hyper` connection pool).
* **Open Source:** Permissive MIT License.

---

## 3. Scope of Work & Milestone Roadmap

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Milestone 1 (Month 1): In-Memory LRU State Cache & Revert Decoder ($5,000 OP)        │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 2 (Month 2): Wallet Integration SDK & Gas Savings Engine ($5,000 OP)         │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 3 (Month 3): Superchain Expansion & Production Docker Packaging ($5,000 OP)  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Milestone 1: In-Memory LRU State Cache & Revert Decoder
* **Deliverable:** Replace full-network RPC state fetching with a multi-level in-memory LRU trie cache for warm contract bytecode, balances, and storage slots.
* **KPI / Target:** Reduce simulation latency from ~2,900 ms down to **< 65 ms** for frequent DeFi interactions (Velodrome, Uniswap pools).
* **Deliverable:** Integrated 4-byte selector decoding for standard errors (`Error(string)`, `Panic(uint256)`) and custom Solidity revert signatures.
* **Funding:** 3,333 OP (~$5,000 USD).

### Milestone 2: Wallet Integration SDK & Standard JSON-RPC Error Schema
* **Deliverable:** Lightweight TypeScript/JavaScript SDK (`@op-sec/client`) that standardizes error interception for wallet clients (GemWallet, MetaMask, Coinbase Wallet).
* **KPI / Target:** Provide complete drop-in error handling documentation and a live demo web client demonstrating transaction interception and gas saved rendering.
* **Funding:** 3,333 OP (~$5,000 USD).

### Milestone 3: Superchain Multi-Chain Support & Enterprise Packaging
* **Deliverable:** Multi-chain routing supporting OP Mainnet, Base, Mode, Zora, and Fraxtal with chain-aware state caching.
* **Deliverable:** One-click Docker container and lightweight systemd/Homebrew daemon packaging for developers and node operators.
* **KPI / Target:** <0.1% CPU utilization during idle polling, zero memory leaks under 48-hour soak testing.
* **Funding:** 3,334 OP (~$5,000 USD).

---

## 4. Total Budget Breakdown

| Budget Category | Description | Amount |
|---|---|:---:|
| **Engineering & Rust Development** | Core `revm` state cache, EVM simulator, and connection pool | 5,000 OP |
| **Wallet Tooling & SDK** | TypeScript integration library, documentation, demo client | 3,000 OP |
| **Superchain Multi-Chain & QA** | Base/Mode/Zora testing, Docker packaging, benchmark suite | 2,000 OP |
| **Total Requested** | **Optimism Foundation Builder Grant** | **10,000 OP (~$15,000)** |

---

## 5. Ecosystem Impact & Alignment

* **User Protection:** Prevents thousands of failed transaction gas burns for Superchain users.
* **Sequencer Efficiency:** Decreases mempool congestion by filtering out unviable transactions before they reach the block builder.
* **Ecosystem Feedback:** Actively discussed on the [Optimism Governance Forum (#10810)](https://gov.optimism.io/t/op-security-proxy-a-local-rpc-middleware-to-prevent-paying-for-failed-l2-executions/10810) with community delegates and wallet developers.

---

## 6. Verification & Reproducibility

```bash
# 1. Clone & Test Rust Suite (26/26 Tests Passing)
git clone https://github.com/Ishant5436/op-sec-proxy.git
cd op-sec-proxy
cargo test

# 2. Run Local Proxy Server
cargo run -- --rpc-url https://mainnet.optimism.io --port 8545
```
