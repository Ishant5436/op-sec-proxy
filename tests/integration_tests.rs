use serde_json::{json, Value};
use op_sec_proxy::interceptor::check_payload;

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
    let result = check_payload(&payload);
    assert!(result.is_err(), "eth_sendRawTransaction must be blocked");

    let err = result.unwrap_err();
    assert_eq!(err["error"]["code"], -32000);
    assert_eq!(
        err["error"]["message"],
        "AESI: Transaction Blocked by Security Heuristics"
    );
    assert_eq!(err["id"], 1, "Error response must preserve the original id");
    assert_eq!(err["jsonrpc"], "2.0");
}

#[test]
fn interceptor_preserves_string_id() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": ["0xabc"],
        "id": "my-custom-id"
    });
    let err = check_payload(&payload).unwrap_err();
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
    let err = check_payload(&payload).unwrap_err();
    assert!(err["id"].is_null());
}

#[test]
fn interceptor_handles_missing_id() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": ["0xabc"]
    });
    let err = check_payload(&payload).unwrap_err();
    assert!(err["id"].is_null(), "Missing id should default to null");
}

#[test]
fn interceptor_allows_eth_block_number() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });
    assert!(check_payload(&payload).is_ok());
}

#[test]
fn interceptor_allows_eth_chain_id() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_chainId",
        "params": [],
        "id": 2
    });
    assert!(check_payload(&payload).is_ok());
}

#[test]
fn interceptor_allows_eth_get_balance() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": ["0x1234", "latest"],
        "id": 3
    });
    assert!(check_payload(&payload).is_ok());
}

#[test]
fn interceptor_allows_eth_call() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{"to": "0xabc"}, "latest"],
        "id": 4
    });
    assert!(check_payload(&payload).is_ok());
}

#[test]
fn interceptor_allows_eth_estimate_gas() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_estimateGas",
        "params": [{"to": "0xabc"}],
        "id": 5
    });
    assert!(check_payload(&payload).is_ok());
}

#[test]
fn interceptor_handles_missing_method_field() {
    let payload = json!({
        "jsonrpc": "2.0",
        "params": [],
        "id": 1
    });
    assert!(check_payload(&payload).is_ok(), "Missing method should pass through");
}

#[test]
fn interceptor_handles_non_string_method() {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": 12345,
        "params": [],
        "id": 1
    });
    assert!(check_payload(&payload).is_ok(), "Non-string method should pass through");
}

#[test]
fn interceptor_handles_empty_object() {
    let payload = json!({});
    assert!(check_payload(&payload).is_ok());
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
        check_payload(&payload).is_ok(),
        "Case-different method must NOT be blocked (JSON-RPC is case-sensitive)"
    );
}

#[test]
fn interceptor_rejects_only_exact_method_name() {
    // Should NOT block similar method names
    let similar_methods = vec![
        "eth_sendRawTransaction_v2",
        "eth_sendTransaction",
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
            check_payload(&payload).is_ok(),
            "Method '{}' should NOT be blocked",
            method
        );
    }
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

    assert_eq!(resp.status(), 200, "Intercepted response should still be 200");

    let body: Value = resp.json().await.expect("Response should be valid JSON");
    assert_eq!(body["error"]["code"], -32000);
    assert_eq!(
        body["error"]["message"],
        "AESI: Transaction Blocked by Security Heuristics"
    );
    assert_eq!(body["id"], 42);
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
    assert_eq!(body["result"], "0xa", "OP Mainnet chain ID should be 0xa (10)");
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
    drop(listener); // Release the port so the server can bind it

    tokio::spawn(async move {
        // Use a non-routable address so forwarding errors out quickly
        op_sec_proxy::server::run_server(port, "http://192.0.2.1:1".to_string())
            .await
            .ok();
    });

    // Give the server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    port
}

/// Spawns a test server with the real Optimism public RPC upstream.
async fn spawn_test_server_with_upstream() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind test port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    tokio::spawn(async move {
        op_sec_proxy::server::run_server(port, "https://mainnet.optimism.io".to_string())
            .await
            .ok();
    });

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    port
}
