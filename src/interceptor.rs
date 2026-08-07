use serde_json::{json, Value};
use crate::decoder::decode_tx;
use crate::fork_db::RpcDb;
use crate::simulator::simulate_tx;

pub fn check_payload(payload: &Value, upstream_url: &str) -> Result<(), Value> {
    if payload.get("method").and_then(|v| v.as_str()) == Some("eth_sendRawTransaction") {
        let id = payload.get("id").cloned().unwrap_or(json!(null));
        
        let block_req = |msg: &str| -> Value {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32000,
                    "message": format!("AESI: Transaction Blocked by Security Heuristics ({})", msg)
                }
            })
        };

        // 1. Decode transaction
        let tx_env = match decode_tx(payload) {
            Ok(tx) => tx,
            Err(e) => return Err(block_req(&format!("decode error: {}", e))),
        };

        // 2. Initialize Fork Database
        let db = RpcDb::new(upstream_url.to_string());

        // 3. Simulate Transaction
        match simulate_tx(&tx_env, db) {
            Ok(true) => {
                // Simulation succeeded and passed heuristics
                return Ok(());
            }
            Ok(false) => {
                return Err(block_req("simulation flagged transaction"));
            }
            Err(e) => {
                return Err(block_req(&e));
            }
        }
    }
    Ok(())
}
