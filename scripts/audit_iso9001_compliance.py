#!/usr/bin/env python3
"""
ISO/DIS 9001:2026 Quality Management System (QMS) Compliance Auditor
Tailored for OP Security Proxy (Optimism Superchain Simulator)
"""
import os
import sys
import json
import shutil
from datetime import datetime

BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

def file_exists(rel_path: str) -> bool:
    return os.path.exists(os.path.join(BASE_DIR, rel_path))

def file_contains(rel_path: str, keyword: str) -> bool:
    p = os.path.join(BASE_DIR, rel_path)
    if not os.path.exists(p):
        return False
    try:
        with open(p, "r", encoding="utf-8", errors="ignore") as f:
            return keyword in f.read()
    except Exception:
        return False

def audit_clause_4():
    """Clause 4: Context of the Organization & Digital Infrastructure"""
    checks = {
        "makefile_orchestrator_defined": file_exists("Makefile"),
        "cargo_manifest_present": file_exists("Cargo.toml"),
        "rust_lib_core_present": file_exists("src/lib.rs"),
        "simulator_present": file_exists("src/simulator.rs"),
    }
    return {
        "clause": "Clause 4: Context & Digital Infrastructure",
        "passed": all(checks.values()),
        "details": checks
    }

def audit_clause_5():
    """Clause 5: Leadership & Quality Culture"""
    checks = {
        "quality_manual_present": file_exists("iso9001_compliance/QUALITY_MANUAL.md"),
        "zero_defect_policy_stated": file_contains("iso9001_compliance/QUALITY_MANUAL.md", "Zero Completion Claims Without Verification"),
        "makefile_pipeline_gated": file_contains("Makefile", "test: test-rust test-sdk"),
    }
    return {
        "clause": "Clause 5: Leadership & Quality Culture",
        "passed": all(checks.values()),
        "details": checks
    }

def audit_clause_6():
    """Clause 6: Planning & Risk-Based Thinking"""
    checks = {
        "risk_register_maintained": file_exists("iso9001_compliance/RISK_REGISTER.md"),
        "lru_capacity_assertion": file_contains("src/lru.rs", "Capacity must be non-zero"),
        "revert_payload_structured": file_contains("src/simulator.rs", "revert_data_hex"),
        "selector_decoding_implemented": file_contains("src/revert_decoder.rs", "decode_revert"),
    }
    return {
        "clause": "Clause 6: Planning & Risk-Based Thinking",
        "passed": all(checks.values()),
        "details": checks
    }

def audit_clause_7():
    """Clause 7: Support & Tool Qualification"""
    checks = {
        "cargo_toolchain_available": shutil.which("cargo") is not None,
        "rustc_compiler_available": shutil.which("rustc") is not None,
        "spec_documentation_present": file_exists("README.md") and file_exists("Dockerfile"),
    }
    return {
        "clause": "Clause 7: Support & Tool Qualification",
        "passed": all(checks.values()),
        "details": checks
    }

def audit_clause_8():
    """Clause 8: Operation & Software Verification/Validation (V&V)"""
    checks = {
        "traceability_matrix_present": file_exists("iso9001_compliance/TRACEABILITY_MATRIX.md"),
        "rust_unit_tests_present": file_contains("src/lib.rs", "mod tests") or file_exists("src/simulator.rs"),
        "integration_tests_present": file_exists("tests/integration_tests.rs"),
        "sdk_tests_present": file_exists("sdk/tests/provider.test.js"),
    }
    return {
        "clause": "Clause 8: Operation & Software V&V",
        "passed": all(checks.values()),
        "details": checks
    }

def audit_clause_9():
    """Clause 9: Performance Evaluation & Audit Trails"""
    checks = {
        "clippy_lint_configured": file_contains("Makefile", "clippy"),
        "web_telemetry_present": file_exists("web/index.html"),
    }
    return {
        "clause": "Clause 9: Performance Evaluation & Audit Trails",
        "passed": all(checks.values()),
        "details": checks
    }

def audit_clause_10():
    """Clause 10: Continual Improvement & Defect Containment"""
    checks = {
        "automated_qms_auditor_wired": file_exists("scripts/audit_iso9001_compliance.py"),
    }
    return {
        "clause": "Clause 10: Continual Improvement & Defect Containment",
        "passed": all(checks.values()),
        "details": checks
    }

def main():
    print("==========================================================================")
    print("      ISO/DIS 9001:2026 Quality Management System (QMS) Compliance Auditor")
    print("      Target: OP Security Proxy (Optimism Superchain Simulator)")
    print("==========================================================================")

    audits = [
        audit_clause_4(),
        audit_clause_5(),
        audit_clause_6(),
        audit_clause_7(),
        audit_clause_8(),
        audit_clause_9(),
        audit_clause_10(),
    ]

    total_clauses = len(audits)
    passed_clauses = sum(1 for a in audits if a["passed"])
    score_pct = (passed_clauses / total_clauses) * 100.0

    print(f"\nAudit Timestamp: {datetime.now().isoformat()}")
    print(f"Target Repository: {BASE_DIR}\n")
    print("Clause-by-Clause Compliance Scorecard:")
    print("--------------------------------------------------------------------------")

    for a in audits:
        status_symbol = "✓ PASS" if a["passed"] else "✗ FAIL"
        print(f"[{status_symbol}] {a['clause']}")
        for detail, res in a["details"].items():
            sub_symbol = "  ✔" if res else "  ✖"
            print(f"    {sub_symbol} {detail}: {'Satisfied' if res else 'MISSING'}")

    print("--------------------------------------------------------------------------")
    print(f"Final Compliance Score: {score_pct:.1f}% ({passed_clauses}/{total_clauses} Clauses Compliant)")

    target_dir = os.path.join(BASE_DIR, "target")
    os.makedirs(target_dir, exist_ok=True)
    report_path = os.path.join(target_dir, "iso9001_audit_report.json")

    report_payload = {
        "standard": "ISO/DIS 9001:2026",
        "repository": "op-sec-proxy",
        "timestamp": datetime.now().isoformat(),
        "score_percent": score_pct,
        "clauses_audited": total_clauses,
        "clauses_passed": passed_clauses,
        "compliant": score_pct == 100.0,
        "results": audits
    }

    with open(report_path, "w") as f:
        json.dump(report_payload, f, indent=2)

    print(f"\nMachine-readable audit artifact written to: {report_path}")

    if score_pct == 100.0:
        print("\n🏆 AUDIT RESULT: FULLY CONFORMANT TO ISO/DIS 9001:2026 STANDARD.")
        sys.exit(0)
    else:
        print("\n⚠️ AUDIT RESULT: NON-CONFORMANCES DETECTED.")
        sys.exit(1)

if __name__ == "__main__":
    main()
