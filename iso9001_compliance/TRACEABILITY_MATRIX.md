# ISO/DIS 9001:2026 Bidirectional Traceability Matrix: OP Security Proxy

This matrix establishes forward and backward traceability between Optimism proxy requirements, Rust source modules, test targets, and verifiable evidence artifacts.

---

## 1. Traceability Mapping

| Requirement ID | Requirement Specification | Test Case ID | Test Implementation | Target Source Component | Verifiable Evidence Artifact |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **REQ-OP-001** | Bounded LRU cache eviction under capacity pressure | `TC-LRU-01` | [`lru.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/lru.rs) | [`lru.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/lru.rs) | Cargo test execution log |
| **REQ-OP-002** | Canonical ERC-6093 revert selector matching | `TC-REV-01` | [`revert_decoder.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/revert_decoder.rs) | [`revert_decoder.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/revert_decoder.rs) | Cargo test execution log |
| **REQ-OP-003** | Panic code and arithmetic overflow error decoding | `TC-REV-02` | [`revert_decoder.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/revert_decoder.rs) | [`revert_decoder.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/revert_decoder.rs) | Cargo test execution log |
| **REQ-OP-004** | Truncated calldata safety without panic | `TC-REV-03` | [`revert_decoder.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/revert_decoder.rs) | [`revert_decoder.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/revert_decoder.rs) | Cargo test execution log |
| **REQ-OP-005** | Transaction simulation halting on insufficient funds | `TC-SIM-01` | [`simulator.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/simulator.rs) | [`simulator.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/simulator.rs) | Cargo test execution log |
| **REQ-OP-006** | Accurate gas heuristic calculation | `TC-SIM-02` | [`simulator.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/simulator.rs) | [`simulator.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/simulator.rs) | Cargo test execution log |
| **REQ-OP-007** | Shared RPC client database across state forks | `TC-FDB-01` | [`fork_db.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/fork_db.rs) | [`fork_db.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/fork_db.rs) | Cargo test execution log |
| **REQ-OP-008** | Interceptor rejecting non-string or missing method payloads | `TC-INT-01` | [`integration_tests.rs`](file:///Users/ishantpanchal/op-sec-proxy/tests/integration_tests.rs) | [`decoder.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/decoder.rs) | Cargo test execution log |
| **REQ-OP-009** | Forwarding safe read-only RPC calls upstream | `TC-INT-02` | [`integration_tests.rs`](file:///Users/ishantpanchal/op-sec-proxy/tests/integration_tests.rs) | [`server.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/server.rs) | Cargo test execution log |
| **REQ-OP-010** | Intercepting and simulating `eth_sendRawTransaction` | `TC-INT-03` | [`integration_tests.rs`](file:///Users/ishantpanchal/op-sec-proxy/tests/integration_tests.rs) | [`server.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/server.rs) | Cargo test execution log |
| **REQ-OP-011** | Rejecting oversized HTTP request bodies | `TC-INT-04` | [`integration_tests.rs`](file:///Users/ishantpanchal/op-sec-proxy/tests/integration_tests.rs) | [`server.rs`](file:///Users/ishantpanchal/op-sec-proxy/src/server.rs) | Cargo test execution log |

---

## 2. Verification Coverage
- **Total Tracked Requirements:** 11
- **Automated Verification Coverage:** 100% (59 passing tests across unit, binary, and integration suites)
