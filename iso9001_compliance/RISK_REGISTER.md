# Software Quality Risk Register (FMEA Matrix): OP Security Proxy
## Conforming to ISO/DIS 9001:2026 Clause 6 (Risk-Based Thinking)

This document tracks identified operational risks, security failure modes, and automated mitigations for `op-sec-proxy`.

---

## 1. Risk Evaluation Scale
- **Severity (S):** 1 (Negligible) to 5 (Catastrophic node crash / false transaction execution)
- **Likelihood (L):** 1 (Extremely Rare) to 5 (Frequent without controls)
- **Risk Priority Number (RPN):** $S \times L$ (Scale 1 to 25). RPN $\ge 12$ mandates automated gating.

---

## 2. Failure Modes and Effects Analysis (FMEA)

| Risk ID | Potential Failure Mode | Impact / Effect | Severity (S) | Likelihood (L) | Initial RPN | Automated Mitigation & Quality Control | Residual RPN |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **RSK-OP-01** | Unbounded memory growth in fork RPC state cache | Out-of-memory kill, server crash during high traffic | 5 | 4 | **20** | Capacity-bounded LRU cache with strict least-recently-used eviction; verified in `lru.rs` (`test_lru_insertion_and_eviction`) | **2** (S=2, L=1) |
| **RSK-OP-02** | Panic or unhandled error on malformed JSON-RPC body | Proxy crash and denial of service | 4 | 3 | **12** | Structured JSON error handling returning HTTP 400 without panicking; verified in `test_server_returns_400_for_invalid_json` | **2** (S=2, L=1) |
| **RSK-OP-03** | Missing method field or non-string method in RPC payload | Undefined behavior or silent dropped request | 4 | 3 | **12** | Explicit input validation guards; verified in `interceptor_handles_missing_method_field` and `interceptor_handles_non_string_method` | **2** (S=2, L=1) |
| **RSK-OP-04** | Payload size exhaustion attack (giant calldata) | Thread starvation / network buffer bloat | 4 | 3 | **12** | Hard HTTP payload size limit; verified in `server_rejects_payload_exceeding_body_size_limit` | **2** (S=2, L=1) |
| **RSK-OP-05** | Silent failure to simulate transaction revert | Malicious or failing transaction submitted on-chain | 5 | 3 | **15** | Strict Revm simulation status check returning high confidence simulation halt; verified in `test_simulate_tx_revert_due_to_insufficient_funds` | **2** (S=2, L=1) |
| **RSK-OP-06** | Truncated revert calldata panicking decoder | Unhandled panic during error message formatting | 4 | 3 | **12** | Length boundary checks in `revert_decoder.rs`; verified in `test_truncated_calldata_returns_none` | **2** (S=2, L=1) |

---

## 3. Governance
Audited on each commit via `make audit-iso9001` and continuous integration.
