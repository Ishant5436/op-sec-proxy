# Optimism Builder Grant Application: OP Security Proxy

**Grant Program:** Optimism Foundation Builder Grants  
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
* **Test Coverage:** 40 automated tests passing across the Rust core and TypeScript client SDK (`make test`).
* **Routing Overhead:** Measured at **+11.80 ms** processing overhead on warm keep-alive sessions, and **-97.09 ms** latency reduction on cold requests via Hyper connection pool reuse (see `BENCHMARK_RESULTS.md`).
* **Simulation Baseline:** Uncached remote RPC simulation currently averages ~2.7s; Milestone 1 targets dropping this to **< 65 ms** via local state trie caching.
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
* **KPI / Target:** Reduce simulation latency from ~2,700 ms down to **< 65 ms** for frequent DeFi interactions (Velodrome, Uniswap pools).
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

| Milestone | Deliverable Scope | Requested OP | USD Equivalent |
|---|---|:---:|:---:|
| **Milestone 1** | In-Memory LRU State Cache & 4-Byte Revert Decoder | 3,333 OP | ~$5,000 |
| **Milestone 2** | Wallet Integration SDK & Standardized Error Interception | 3,333 OP | ~$5,000 |
| **Milestone 3** | Superchain Multi-Chain Routing & Production Packaging | 3,334 OP | ~$5,000 |
| **Total Requested** | **Optimism Foundation Builder Grant** | **10,000 OP** | **~$15,000** |

---

## 5. Ecosystem Impact & Alignment

* **User Protection:** Prevents failed transaction gas burns for Superchain users and programmatic agents.
* **Sequencer Efficiency:** Decreases mempool congestion by filtering out reverting payloads before broadcast.
* **Community Introduction:** Initial project introduction published on the [Optimism Governance Forum (#10810)](https://gov.optimism.io/t/op-security-proxy-a-local-rpc-middleware-to-prevent-paying-for-failed-l2-executions/10810).

---

## 6. Verification & Reproducibility

```bash
# 1. Clone & Run Test Suite (40/40 Tests Passing)
git clone https://github.com/Ishant5436/op-sec-proxy.git
cd op-sec-proxy
make test

# 2. Run Benchmarks Against Live Optimism Mainnet
cargo build --release
PROXY_PORT=8080 OP_RPC_URL=https://mainnet.optimism.io ./target/release/op-sec-proxy &
python3 scripts/benchmark.py
```
