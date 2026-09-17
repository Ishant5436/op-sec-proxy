.PHONY: all test test-rust test-sdk build demo gui clean docker lint

all: test

lint:
	cargo clippy --all-targets -- -D warnings
	cargo fmt --check

test: test-rust test-sdk

test-rust:
	cargo test --quiet

test-sdk:
	node --test sdk/tests/provider.test.js

build:
	cargo build --release
	cd sdk && npx tsc

demo:
	cargo test --quiet
	node --test sdk/tests/provider.test.js
	@echo "=== OP Security Proxy: 100% Verified Across Rust Core & TS SDK ==="

audit-iso9001:
	@echo "=== Verifying OP Security Proxy Against ISO/DIS 9001:2026 Standards ==="
	python3 scripts/audit_iso9001_compliance.py

gui:
	@echo "Launching OP Security Proxy Telemetry Cockpit..."
	@open web/index.html 2>/dev/null || xdg-open web/index.html 2>/dev/null || echo "Open web/index.html in your browser"

docker:
	docker build -t op-sec-proxy:latest .

clean:
	cargo clean
	rm -rf sdk/dist target/iso9001_audit_report.json

.PHONY: all test test-rust test-sdk build demo gui clean docker lint audit-iso9001
