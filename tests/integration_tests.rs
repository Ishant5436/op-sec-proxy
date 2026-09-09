use op_sec_proxy::interceptor::check_payload;
use serde_json::{Value, json};

// Dummy upstream URL used for tests that don't need real network access.
// The simulation will fail during decode (invalid tx hex) before any RPC call is made.
const TEST_UPSTREAM: &str = "http://192.0.2.1:1";

// ═══════════════════════════════════════════════════════════════════
//  INTERCEPTOR UNIT TESTS — Exhaustive edge-case coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn interceptor_blocks_eth_send_raw_transaction() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": ["0xdeadbeef"],
        "id": 1
    });
    let result = check_payload(&payload, TEST_UPSTREAM);
    assert!(result.is_err(), "eth_sendRawTransaction must be blocked");

    let err = result.unwrap_err();
    assert_eq!(err["error"]["code"], -32000);
    // Error message now uses structured format instead of AESI prefix
    let msg = err["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("reverted") || msg.contains("error"),
        "Error message should describe the revert, got: {}",
        msg
    );
    assert_eq!(err["id"], 1, "Error response must preserve the original id");
    assert_eq!(err["jsonrpc"], "2.0");
    // Verify structured data object is present with simulation_latency_ms
    assert!(
        err["error"]["data"]["simulation_latency_ms"].is_number(),
        "Structured data must include simulation_latency_ms"
    );
}

#[test]
fn interceptor_preserves_string_id() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": ["0xabc"],
        "id": "my-custom-id"
    });
    let err = check_payload(&payload, TEST_UPSTREAM).unwrap_err();
    assert_eq!(err["id"], "my-custom-id");
}

#[test]
fn interceptor_handles_null_id() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": ["0xabc"],
        "id": null
    });
    let err = check_payload(&payload, TEST_UPSTREAM).unwrap_err();
    assert!(err["id"].is_null());
}

#[test]
fn interceptor_handles_missing_id() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": ["0xabc"]
    });
    let err = check_payload(&payload, TEST_UPSTREAM).unwrap_err();
    assert!(err["id"].is_null(), "Missing id should default to null");
}

#[test]
fn interceptor_blocks_empty_params() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": [],
        "id": 1
    });
    let result = check_payload(&payload, TEST_UPSTREAM);
    assert!(
        result.is_err(),
        "Empty params should be blocked at decode stage"
    );
}

#[test]
fn interceptor_allows_eth_block_number() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });
    assert!(check_payload(&payload, TEST_UPSTREAM).is_ok());
}

#[test]
fn interceptor_allows_eth_chain_id() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_chainId",
        "params": [],
        "id": 2
    });
    assert!(check_payload(&payload, TEST_UPSTREAM).is_ok());
}

#[test]
fn interceptor_allows_eth_get_balance() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": ["0x1234", "latest"],
        "id": 3
    });
    assert!(check_payload(&payload, TEST_UPSTREAM).is_ok());
}

#[test]
fn interceptor_allows_eth_call() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{"to": "0xabc"}, "latest"],
        "id": 4
    });
    assert!(check_payload(&payload, TEST_UPSTREAM).is_ok());
}

#[test]
fn interceptor_allows_eth_estimate_gas() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_estimateGas",
        "params": [{"to": "0xabc"}],
        "id": 5
    });
    assert!(check_payload(&payload, TEST_UPSTREAM).is_ok());
}

#[test]
fn interceptor_handles_missing_method_field() {
    let payload = json!({
        "jsonrpc": "2.0",
        "params": [],
        "id": 1
    });
    assert!(
        check_payload(&payload, TEST_UPSTREAM).is_ok(),
        "Missing method should pass through"
    );
}

#[test]
fn interceptor_handles_non_string_method() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": 12345,
        "params": [],
        "id": 1
    });
    assert!(
        check_payload(&payload, TEST_UPSTREAM).is_ok(),
        "Non-string method should pass through"
    );
}

#[test]
fn interceptor_handles_empty_object() {
    let payload = json!({});
    assert!(check_payload(&payload, TEST_UPSTREAM).is_ok());
}

#[test]
fn interceptor_is_case_sensitive() {
    // Method names in JSON-RPC are case-sensitive
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "ETH_SENDRAWTRANSACTION",
        "params": ["0xabc"],
        "id": 1
    });
    assert!(
        check_payload(&payload, TEST_UPSTREAM).is_ok(),
        "Case-different method must NOT be blocked (JSON-RPC is case-sensitive)"
    );
}

#[test]
fn interceptor_rejects_only_exact_method_name() {
    // Should NOT block similar-but-distinct method names
    let similar_methods = vec![
        "eth_sendRawTransaction_v2",
        "eth_sendRawTransactio",
        "debug_sendRawTransaction",
    ];
    for method in similar_methods {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": [],
            "id": 1
        });
        assert!(
            check_payload(&payload, TEST_UPSTREAM).is_ok(),
            "Method '{}' should NOT be blocked",
            method
        );
    }

    // eth_sendTransaction IS now intercepted (same as eth_sendRawTransaction)
    let send_tx_payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendTransaction",
        "params": ["0xdeadbeef"],
        "id": 1
    });
    assert!(
        check_payload(&send_tx_payload, TEST_UPSTREAM).is_err(),
        "eth_sendTransaction must now be intercepted"
    );
}

#[test]
fn interceptor_reports_confidence_on_decode_error() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": ["0xdeadbeef"],
        "id": 101
    });
    let res = op_sec_proxy::interceptor::check_payload_opt(&payload, TEST_UPSTREAM, false);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err["error"]["code"], -32000);
    assert_eq!(err["error"]["data"]["confidence"], "uncertain");
}

#[test]
fn interceptor_distinguishes_confidence_modes() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": ["0xdeadbeef"],
        "id": 102
    });
    let res = op_sec_proxy::interceptor::check_payload(&payload, TEST_UPSTREAM);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err["error"]["data"]["confidence"].is_string());
}

// ═══════════════════════════════════════════════════════════════════
//  INTEGRATION TESTS — Full server round-trip
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn server_rejects_get_requests() {
    let port = spawn_test_server().await;

    let resp = reqwest::Client::new()
        .get(format!("http://127.0.0.1:{}", port))
        .send()
        .await
        .expect("GET request failed");

    assert_eq!(resp.status(), 404, "GET should return 404 Not Found");
}

#[tokio::test]
async fn server_returns_400_for_invalid_json() {
    let port = spawn_test_server().await;

    let resp = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{}", port))
        .body("this is not json")
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(resp.status(), 400, "Non-JSON body should return 400");
}

#[tokio::test]
async fn server_intercepts_send_raw_transaction() {
    let port = spawn_test_server().await;

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": ["0xdeadbeef"],
        "id": 42
    });

    let resp = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{}", port))
        .json(&payload)
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(
        resp.status(),
        200,
        "Intercepted response should still be 200"
    );

    let body: Value = resp.json().await.expect("Response should be valid JSON");
    assert_eq!(body["error"]["code"], -32000);
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("reverted") || msg.contains("error"),
        "Error message should describe the revert, got: {}",
        msg
    );
    assert_eq!(body["id"], 42);
    // Verify structured data object is present
    assert!(
        body["error"]["data"]["simulation_latency_ms"].is_number(),
        "Structured data must include simulation_latency_ms"
    );
}

#[tokio::test]
async fn server_forwards_safe_rpc_calls_upstream() {
    let port = spawn_test_server_with_upstream().await;

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_chainId",
        "params": [],
        "id": 1
    });

    let resp = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{}", port))
        .json(&payload)
        .send()
        .await
        .expect("POST request failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.expect("Response should be valid JSON");
    // OP Mainnet chain ID is 0xa (10)
    assert_eq!(
        body["result"], "0xa",
        "OP Mainnet chain ID should be 0xa (10)"
    );
}

#[tokio::test]
async fn server_forwards_block_number_and_returns_hex() {
    let port = spawn_test_server_with_upstream().await;

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 99
    });

    let resp = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{}", port))
        .json(&payload)
        .send()
        .await
        .expect("POST request failed");

    let body: Value = resp.json().await.expect("Response should be valid JSON");
    let result = body["result"].as_str().expect("result should be a string");
    assert!(
        result.starts_with("0x"),
        "Block number should be a hex string, got: {}",
        result
    );
    assert_eq!(body["id"], 99, "Response id must match request id");
}

#[tokio::test]
async fn server_rejects_payload_exceeding_body_size_limit() {
    let port = spawn_test_server().await;

    // Construct a payload strictly exceeding MAX_REQUEST_BODY_SIZE (2MB + 100KB)
    let padding = "a".repeat(2 * 1024 * 1024 + 100 * 1024);
    let oversized_payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [padding],
        "id": 100
    });

    let resp = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{}", port))
        .json(&oversized_payload)
        .send()
        .await
        .expect("POST request should receive response");

    assert_eq!(
        resp.status(),
        413,
        "Server must return 413 Payload Too Large"
    );
    let body: Value = resp
        .json()
        .await
        .expect("Response should be valid JSON-RPC error");
    assert_eq!(body["error"]["code"], -32600);
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Request body too large"),
        "Error message should explain body size limit"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  TEST HELPERS
// ═══════════════════════════════════════════════════════════════════

/// Spawns a test server with a dummy upstream (will fail on forwarding,
/// but interceptor tests don't need a real upstream).
async fn spawn_test_server() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind test port");
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        // Use a non-routable address so forwarding errors out quickly
        op_sec_proxy::server::run_server_listener(listener, "http://192.0.2.1:1".to_string(), true)
            .await
            .ok();
    });

    port
}

/// Spawns a test server with a local deterministic JSON-RPC upstream that responds with OP Mainnet values.
/// Eliminates external network latency, rate limits, and flakiness from public RPC providers.
async fn spawn_test_server_with_upstream() -> u16 {
    let mock_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind mock upstream port");
    let mock_port = mock_listener.local_addr().unwrap().port();
    let upstream_url = format!("http://127.0.0.1:{}", mock_port);

    tokio::spawn(async move {
        loop {
            let (stream, _) = match mock_listener.accept().await {
                Ok(s) => s,
                Err(_) => break,
            };
            let io = hyper_util::rt::TokioIo::new(stream);
            tokio::spawn(async move {
                use http_body_util::BodyExt;
                let service = hyper::service::service_fn(
                    |req: hyper::Request<hyper::body::Incoming>| async move {
                        let body_bytes = req
                            .into_body()
                            .collect()
                            .await
                            .map(|c| c.to_bytes())
                            .unwrap_or_default();
                        let val: serde_json::Value =
                            serde_json::from_slice(&body_bytes).unwrap_or_default();
                        let method = val["method"].as_str().unwrap_or("");
                        let id = val.get("id").cloned().unwrap_or(serde_json::Value::Null);

                        let resp_body = match method {
                            "eth_chainId" => serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": "0xa"
                            }),
                            "eth_blockNumber" => serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": "0x123456"
                            }),
                            _ => serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": "0x0"
                            }),
                        };

                        let resp = hyper::Response::builder()
                            .header("content-type", "application/json")
                            .body(http_body_util::Full::new(hyper::body::Bytes::from(
                                resp_body.to_string(),
                            )))
                            .unwrap();
                        Ok::<_, hyper::Error>(resp)
                    },
                );

                hyper::server::conn::http1::Builder::new()
                    .serve_connection(io, service)
                    .await
                    .ok();
            });
        }
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind test port");
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        op_sec_proxy::server::run_server_listener(listener, upstream_url, true)
            .await
            .ok();
    });

    port
}
