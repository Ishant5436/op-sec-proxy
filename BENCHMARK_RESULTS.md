# OP Security Proxy Benchmark & Latency Telemetry

**Date:** 2026-09-04 10:16:37 UTC  
**Commit:** `31fa5fa`  
**Hardware:** Apple MacBook Pro (Apple Silicon ARM64)  
**Upstream RPC:** `https://mainnet.optimism.io`  
**Proxy RPC:** `http://127.0.0.1:8080`  

---

## 1. Executive Summary & Honest Methodology

These benchmarks evaluate two distinct operational regimes:
1. **Fair Warm Keep-Alive:** Both the direct upstream client and the proxy client reuse persistent HTTP sessions (`requests.Session` with TCP keep-alive). This measures the true proxy processing overhead (JSON parsing, deserialization, routing).
2. **Cold Connection Baseline:** Fresh connections are established per request, illustrating the amortization benefit of the proxy's internal Hyper connection pool against remote public endpoint TLS handshakes.

---

## 2. Benchmark Telemetry Results

### Passthrough Latency (`eth_blockNumber`) — Fair Warm Baseline (50 Interleaved Iterations)
| Arm | Mean Latency | Median | Min | Max | StdDev | Samples |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **Upstream Direct** (Reused Session) | 418.17 ms | 406.85 ms | 392.13 ms | 641.26 ms | 40.87 ms | 50 |
| **Via Proxy** (Reused Session) | 429.98 ms | 414.67 ms | 392.91 ms | 564.05 ms | 39.44 ms | 50 |
| **Net Proxy Processing Overhead** | **+11.80 ms** | **+7.82 ms** | - | - | - | - |

> **Methodology Caveat:** In a fair warm-connection comparison, the proxy introduces an expected minimal processing overhead (+11.80 ms) due to local JSON deserialization and forwarding. It is **not** faster than an already-warm direct connection to the same physical node.

### Cold Connection Handshake Amortization (20 Iterations)
| Arm | Mean Latency | Median | Min | Max | StdDev |
|---|:---:|:---:|:---:|:---:|:---:|
| **Upstream Direct** (Fresh TLS 1.3 Handshake) | 518.13 ms | 457.26 ms | 434.82 ms | 869.55 ms | 140.43 ms |
| **Via Proxy** (Local Loopback + Reused Pool) | 421.04 ms | 417.03 ms | 401.77 ms | 448.15 ms | 14.23 ms |
| **Cold Connection Delta** | **-97.09 ms** | - | - | - | - |

> **Context:** When naive clients open and close connections per request without session reuse, the proxy's internal connection pool spares them the repetitive ~97 ms TLS handshake penalty to the public remote endpoint.

### EVM Zero-Trust Simulation & Interception (`eth_sendRawTransaction`) (20 Iterations)
| Metric | Value | Notes |
|---|:---:|---|
| **Mean Interception Latency** | **2699.11 ms** | End-to-end RLP decode, revm state execution, error formatting |
| **Median Latency** | **2633.26 ms** | Standard execution path |
| **Min / Max Latency** | **2512.64 ms / 3137.96 ms** | Jitter profile |
| **Reverted Transactions Blocked** | **20/20 (100%)** | Error code `-32000` returned with decoded reason |
| **Gas Burned On-Chain** | **0 wei (100% saved)** | Broadcast preemptively halted before sequencer submission |

---

## 3. Reproduction Command

```bash
# Terminal 1: Launch release server on port 8080
PROXY_PORT=8080 OP_RPC_URL=https://mainnet.optimism.io cargo run --release

# Terminal 2: Run benchmark suite
python3 scripts/benchmark.py
```
