use serde_json::{json, Value};

pub fn check_payload(payload: &Value) -> Result<(), Value> {
    if payload.get("method").and_then(|v| v.as_str()) == Some("eth_sendRawTransaction") {
        let id = payload.get("id").cloned().unwrap_or(json!(null));
        let error_response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32000,
                "message": "AESI: Transaction Blocked by Security Heuristics"
            }
        });
        return Err(error_response);
    }
    Ok(())
}
