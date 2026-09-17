# Software Quality Management System (QMS) Manual: OP Security Proxy
## Conforming to ISO/DIS 9001:2026 (Draft International Standard)

---

### 1. Scope & Application
This Quality Manual establishes the Software Quality Management System (QMS) policies, procedures, and deterministic controls implemented across `op-sec-proxy`. It formalizes quality engineering for Optimism Superchain transaction simulation, JSON-RPC proxying, and revert decoding under **ISO/DIS 9001:2026**.

---

### 2. Clause 4: Context of the Organization & Digital Infrastructure
- **4.1 Technological Context:** Operates in Rust (`revm`, `tokio`, `actix-web`/`hyper`) delivering memory-safe, sub-millisecond EVM state forking and transaction pre-simulation for OP Mainnet, Base, and Superchain rollups.
- **4.2 Stakeholder Expectations:** Optimism Foundation grant reviewers, node operators, and wallet users require zero server panics, memory bounds on state caching, accurate gas limit heuristics, and human-readable revert decoding.
- **4.3 Scope of the QMS:** Covers all Rust core components (`simulator.rs`, `revert_decoder.rs`, `fork_db.rs`, `lru.rs`, `server.rs`, `decoder.rs`), TypeScript SDK, and Web telemetry cockpit.
- **4.4 Automated Quality Pipeline:** Managed through [`Makefile`](file:///Users/ishantpanchal/op-sec-proxy/Makefile) with Cargo unit, doc, and integration tests.

---

### 3. Clause 5: Leadership & Quality Culture
- **5.1 Leadership & Commitment:** Adheres to the **Zero Completion Claims Without Verification** directive. Releases require 100% green Cargo tests and zero Clippy warnings.
- **5.2 Quality Policy:** Committed to Rust memory safety, zero naked `unwrap()` panics on runtime paths, bounded LRU caches, and standard ERC-6093 revert introspection.
- **5.3 Roles & Responsibilities:** Automated compiler type checking, borrow checkers, and integration suites act as deterministic quality gates.

---

### 4. Clause 6: Planning & Risk-Based Thinking
- **6.1 Actions to Address Risks & Opportunities:** The QMS maintains an active [`RISK_REGISTER.md`](file:///Users/ishantpanchal/op-sec-proxy/iso9001_compliance/RISK_REGISTER.md) evaluating failure modes like unbounded fork cache growth, unhandled JSON-RPC bodies, and false positive simulation halts.
- **6.2 Quality Objectives:**
  - *Reliability:* 100% test pass rate across unit, binary, and integration suites (59/59 passing).
  - *Memory Bounds:* Hard LRU capacity eviction preventing out-of-memory crashes during high RPC load.
  - *Error Decoding:* 100% accurate classification of custom ERC-6093 reverts, `Panic(uint256)`, and standard `Error(string)`.

---

### 5. Clause 7: Support & Tool Qualification
- **7.1 Resources & Qualified Compilers:**
  - Rust Compiler: `rustc` 1.80+ (stable).
  - Package Manager: `cargo`.
  - Node.js Runtime: Node 20+ for TypeScript SDK verification.
- **7.2 Competence & Documentation:** Documented in [`README.md`](file:///Users/ishantpanchal/op-sec-proxy/README.md) and governance proposals.
- **7.5 Documented Information:** Cargo test binaries and JSON-RPC logs serve as tamper-evident quality records.

---

### 6. Clause 8: Operational Planning and Control (Software V&V)
- **8.1 Verification and Validation Protocol:**
  - *Verification (Unit Tests):* Verification of LRU eviction, canonical selector hashes, and panic code decoding.
  - *Validation (Integration Tests):* Full HTTP JSON-RPC verification of `eth_sendRawTransaction` interception, passthrough forwarding, and payload size bounds.
- **8.7 Control of Non-conforming Outputs:** Invalid JSON-RPC calls or payloads exceeding body size limits return structured HTTP 400 Bad Request responses.

---

### 7. Clause 9: Performance Evaluation
- **9.1 Monitoring & Measurement:** Continuous benchmarking of EVM state simulation latency (<5ms target) and gas estimations.
- **9.2 Internal Audit:** Automated QMS audit executed via [`scripts/audit_iso9001_compliance.py`](file:///Users/ishantpanchal/op-sec-proxy/scripts/audit_iso9001_compliance.py).

---

### 8. Clause 10: Continual Improvement
- **10.1 Non-conformity and Corrective Action:** Unrecognized revert selectors trigger additions to `revert_decoder.rs` selector registries.
- **10.2 Continual Improvement Cycle:** Continuous integration with new Superchain L2 network IDs and EVM hardfork upgrades.
