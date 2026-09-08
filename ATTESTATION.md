# OP Security Proxy: Onchain Attestation & Governance Provenance

This document records the immutable onchain registration and governance records for **OP Security Proxy** prior to the OP Atlas frontend sunset on September 18, 2026.

---

## 1. Onchain Identity & Ethereum Attestation Service (EAS) Record

* **Project Identifier (OP Atlas UID):** `0x6ab1a8cbf07a602a09c6e754b34361f336ba8e62c1a4792659d7b5ac4a27663b`
* **EAS Attestation UID:** `0x5c6d0c752bb6dcf67de95bbd973d11a0248a2336b7b61980393951ff6d67ba56`
* **EAS Scan Explorer:** [https://optimism.easscan.org/attestation/view/0x5c6d0c752bb6dcf67de95bbd973d11a0248a2336b7b61980393951ff6d67ba56](https://optimism.easscan.org/attestation/view/0x5c6d0c752bb6dcf67de95bbd973d11a0248a2336b7b61980393951ff6d67ba56)
* **Atlas Project URL:** [https://atlas.optimism.io/project/0x6ab1a8cbf07a602a09c6e754b34361f336ba8e62c1a4792659d7b5ac4a27663b](https://atlas.optimism.io/project/0x6ab1a8cbf07a602a09c6e754b34361f336ba8e62c1a4792659d7b5ac4a27663b)
* **Attestation Timestamp:** `2026-09-08 19:31:00 UTC`
* **Network:** OP Mainnet (Chain ID 10)
* **Project Admin:** `optimist-2520469a`
* **License Classification:** Open Source (MIT)

---

## 2. GitHub Repository Provenance

* **Repository:** [https://github.com/Ishant5436/op-sec-proxy](https://github.com/Ishant5436/op-sec-proxy)
* **Attestation Manifest:** [`funding.json`](./funding.json)
* **Verification Commit:** `3b39f8c`
* **Test Suite:** 45 automated tests passing (`cargo test` + Node SDK via `make test`)

---

## 3. Governance Proposal

* **Forum:** Optimism Collective Governance Forum
* **Category:** `Grants 🔴 / Governance Fund Missions`
* **Proposal Thread:** [Thread #10845](https://gov.optimism.io/t/builder-grant-proposal-op-security-proxy-local-pre-execution-revert-interception-for-the-superchain/10845)
* **Target Grant:** 10,000 OP (~$15,000 USD)
* **Ecosystem Feedback:** GemWallet validation confirmed; fail-open simulation implemented in commit `8376212`.
* **Cross-Link Update:** Published in Thread #10845 Post #5.
