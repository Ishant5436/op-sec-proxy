# Arbitrum Foundation Grant Application: EVM Security Proxy

**Grant Program:** Arbitrum Foundation Grants Program (Phase 3)  
**Track:** Developer Tooling, Security & RPC Infrastructure  
**Requested Amount:** $15,000 USD (in ARB or USDC equivalent)  
**Project Repository:** [https://github.com/Ishant5436/op-sec-proxy](https://github.com/Ishant5436/op-sec-proxy)  
**Applicant:** Ishant Panchal (`Ishant5436` / `ishant.p@somaiya.edu`)  

---

## 1. Executive Summary

**EVM Security Proxy (`op-sec-proxy`)** is a high-throughput, non-custodial JSON-RPC middleware built in pure Rust that sits locally between client applications (wallets, algorithmic trading bots, automated agents) and Arbitrum RPC nodes (Arbitrum One, Arbitrum Nova, and Orbit chains).

### The Problem on Arbitrum
Arbitrum users and programmatic bots frequently burn transaction fees on failed transactions resulting from:
1. Decentralized Exchange (DEX) slippage on high-volatility pairs (Camelot, Uniswap v3, GMX).
2. Stale or racing oracle updates on perpetual and lending protocols.
3. Unverified contract approval deficits and allowance mismatches.

While Arbitrum execution fees are low, cumulative failed transaction costs—combined with L1 calldata posting fees—accumulate significant capital drain for high-frequency trading bots, automated arbitrageurs, and retail users.

### The Solution
The proxy intercepts outbound `eth_sendRawTransaction` payloads and simulates execution locally in-memory using `revm` (Rust Ethereum Virtual Machine) against real-time Arbitrum state before network broadcast:
- **Preemptive Abort:** If simulation reverts, the transaction broadcast is halted immediately, preserving 100% of the gas and priority fee.
- **Decoded Reverts:** Translates raw revert hex bytes into standard Solidity error definitions (`Error(string)`, `Panic(uint256)`, and custom contract error selectors).
- **Quantified Protection:** Calculates and returns the exact `estimated_gas_saved` in the JSON-RPC response so client interfaces can display verified protected capital.

---

## 2. Technical Architecture & Delivered Benchmarks

The core engine is implemented and open-sourced under the MIT license:

* **Engine:** Pure asynchronous Rust with `tokio`, `hyper`, `alloy`, and `revm`.
* **Zero Allocation on Hot Paths:** Pre-allocated state caches and zero-copy JSON parsing adhering to Holzmann's Power of 10 Safety Invariants.
* **Empirical Benchmarks:**
  - Warm Keep-Alive Interception Overhead: **+11.80 ms**.
  - Cold Request Latency Reduction: **-97.09 ms** (achieved via Hyper connection pooling against upstream RPC endpoints).
  - Uncached Remote Simulation: ~2.7 seconds (Milestone 1 targets **< 65 ms** via local trie caching).
* **Test Suite:** 41/41 automated tests passing across the Rust core, RPC interceptor, and LRU cache (`cargo test`).

---

## 3. Scope of Work & Milestone Roadmap

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Milestone 1 (Month 1): In-Memory LRU State Cache & Arbitrum Nitro Precompiles ($5,000) │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 2 (Month 2): Arbitrum Orbit & Stylus Execution Support ($5,000)              │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 3 (Month 3): Client SDK & Production Docker Distribution ($5,000)            │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Milestone 1: In-Memory LRU State Cache & Nitro Precompile Compatibility
* **Deliverable:** Multi-tier in-memory LRU cache storing contract bytecode, nonces, and storage slots to eliminate redundant upstream RPC calls during simulation.
* **Target KPI:** Reduce local pre-execution simulation latency from ~2,700 ms down to **< 65 ms** on hot Arbitrum DeFi contracts (Camelot router, GMX vault).
* **Deliverable:** Full compatibility with Arbitrum Nitro precompiles (NodeInterface, ArbSys, ArbGasInfo, ArbAddressTable).
* **Funding:** $5,000 USD (in ARB/USDC).

### Milestone 2: Arbitrum Orbit & Custom Gas Token Support
* **Deliverable:** Configurable chain profile definitions enabling turnkey sidecar deployment for Arbitrum Orbit chains with custom gas tokens and alternate data availability layers.
* **Deliverable:** Structured revert decoding for Stylus (WASM) contract panics alongside standard EVM Solidity custom errors.
* **Target KPI:** Provide end-to-end integration examples for Orbit rollups deploying as sovereign L3s.
* **Funding:** $5,000 USD (in ARB/USDC).

### Milestone 3: Client Integration SDK & Production Packaging
* **Deliverable:** Lightweight TypeScript/JavaScript client package (`@arb-sec/client`) providing seamless drop-in middleware support for ethers.js, viem, and web3.js.
* **Deliverable:** Multi-architecture Docker images (`linux/amd64`, `linux/arm64`) with automated health checks, Prometheus metrics endpoints (`simulations_total`, `reverts_blocked_total`, `gas_saved_wei_total`), and Helm charts for Kubernetes deployments.
* **Funding:** $5,000 USD (in ARB/USDC).

---

## 4. Budget & Funding Allocation

| Milestone | Deliverables | Target Timeline | Funding |
| :--- | :--- | :--- | :--- |
| **Milestone 1** | LRU state cache, Nitro precompiles, <65ms simulation | Month 1 | $5,000 |
| **Milestone 2** | Orbit chain profiles, Stylus error decoder, custom gas | Month 2 | $5,000 |
| **Milestone 3** | TypeScript SDK, Prometheus metrics, Docker/Helm images | Month 3 | $5,000 |
| **Total** | **Arbitrum Foundation Developer Tooling Grant** | **3 Months** | **$15,000** |

---

## 5. Ecosystem Impact for Arbitrum

1. **User Experience Protection:** Eliminates the frustration of paying fees for reverted transactions in retail wallets.
2. **Bot & Agent Capital Efficiency:** Allows automated trading bots and AI agents to submit transactions aggressively without capital bleed from unexpected slippage or mempool front-running.
3. **Sequencer Hygiene:** Intercepting bad transactions locally removes invalid state mutations from the Arbitrum mempool and sequencer feed, reducing network waste.

---

## 6. Verification & Reproducibility

```bash
# Clone and verify test suite (41/41 tests passing)
git clone https://github.com/Ishant5436/op-sec-proxy.git
cd op-sec-proxy
cargo test

# Run benchmarks against Arbitrum One
cargo build --release
PROXY_PORT=8080 OP_RPC_URL=https://arb1.arbitrum.io/rpc ./target/release/op-sec-proxy &
python3 scripts/benchmark.py
```
