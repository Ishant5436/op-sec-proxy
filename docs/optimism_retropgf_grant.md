Project Name: OP Security Proxy
Repository: https://github.com/Ishant5436/op-sec-proxy
Categories: Developer Tooling & Infrastructure, Security & User Protection

Description:
I wrote OP Security Proxy in Rust to act as a fast JSON-RPC middleware layer. It basically sits right between a standard wallet or client and the public Optimism RPC nodes.

Whenever a transaction goes out via eth_sendRawTransaction, the proxy catches it. It uses the revm engine to fork the network state locally and run a quick simulation. If that transaction is going to revert, halt, or do something weird like burn the whole gas limit without touching the state, the proxy just drops it before it ever hits the public mempool.

The Problem it Solves:
When you use Ethereum-equivalent chains, you still have to pay the L2 execution fee up to the point of failure if a transaction reverts on-chain. We see this all the time with MEV bots, slippage issues, or unexpected state changes. This proxy is a public good that prevents regular users from paying for failed executions, which saves them money and keeps junk traffic off the sequencer.

Performance Data:
I ran some tests against the mainnet.optimism.io endpoint to see the impact. For standard read requests like eth_call, there is basically no overhead. Actually, because I am using tokio and hyper for connection pooling, the jitter was slightly better than hitting the upstream directly (Upstream: ~516ms, Proxy: ~444ms).

When doing the actual transaction simulation, it takes about 2.9 seconds. That overhead comes from having to fetch nonces, balances, and raw bytecodes over the network on the fly so we can reconstruct the state from scratch.

Architecture:
- The stack is mostly Rust, relying on tokio for the async side of things and hyper to handle the HTTP server.
- I pull in alloy to handle the Ethereum primitives and RPC parsing.
- I use revm to run the local EVM simulations.
- Wrote a bunch of strict TDD tests to make sure it doesn't panic if an upstream node times out.

Future Work:
1. I want to add LRU caching for the state trie nodes to get that simulation time under 100ms.
2. Add local heuristics to catch sandwich attacks.
3. Get it working smoothly with Base and other Superchain networks.
