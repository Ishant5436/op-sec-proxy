.PHONY: all test test-rust test-sdk build demo gui clean docker

all: test

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

gui:
	@echo "Launching OP Security Proxy Telemetry Cockpit..."
	@open web/index.html 2>/dev/null || xdg-open web/index.html 2>/dev/null || echo "Open web/index.html in your browser"

docker:
	docker build -t op-sec-proxy:latest .

clean:
	cargo clean
	rm -rf sdk/dist
