# Multi-stage production build for OP Security Proxy
FROM rust:1.80-bullseye as builder

WORKDIR /usr/src/op-sec-proxy
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Distroless / minimal runtime stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 appuser

WORKDIR /app
COPY --from=builder /usr/src/op-sec-proxy/target/release/op-sec-proxy /usr/local/bin/op-sec-proxy

USER appuser

ENV OP_RPC_URL="https://mainnet.optimism.io"
ENV PROXY_PORT=3000
ENV CHAIN_ID=10

EXPOSE 3000

HEALTHCHECK --interval=10s --timeout=3s --retries=3 \
  CMD curl -f -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://localhost:3000 || exit 1

ENTRYPOINT ["/usr/local/bin/op-sec-proxy"]
