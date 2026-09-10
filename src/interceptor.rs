use crate::decoder::decode_tx;
use crate::fork_db::RpcDb;
use crate::simulator::{SimulationConfidence, simulate_tx};
use serde_json::{Value, json};
use std::time::Instant;

/// Intercepts eth_sendRawTransaction and eth_sendTransaction,
/// simulates via revm, and returns a structured JSON-RPC -32000 error
/// with revert_data, decoded_reason, estimated_gas_saved, confidence, and
/// simulation_latency_ms when the transaction would revert on-chain.
#[allow(dead_code)]
pub fn check_payload(payload: &Value, upstream_url: &str) -> Result<(), Value> {
    check_payload_opt(payload, upstream_url, true)
}

pub fn check_payload_opt(
    payload: &Value,
    upstream_url: &str,
    fail_open: bool,
) -> Result<(), Value> {
    let db = RpcDb::new(upstream_url.to_string());
    check_payload_with_db(payload, db, fail_open)
}

pub fn check_payload_with_db(
    payload: &Value,
    db: RpcDb,
    fail_open: bool,
) -> Result<(), Value> {
    let method = payload.get("method").and_then(|v| v.as_str()).unwrap_or("");

    if method == "eth_sendRawTransaction" || method == "eth_sendTransaction" {
        let id = payload.get("id").cloned().unwrap_or(json!(null));
        let sim_start = Instant::now();

        // 1. Decode transaction
        let tx_env = match decode_tx(payload) {
            Ok(tx) => tx,
            Err(e) => {
                let elapsed_ms = sim_start.elapsed().as_millis() as u64;
                return Err(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": format!("Execution reverted: decode error: {}", e),
                        "data": {
                            "decoded_reason": format!("decode error: {}", e),
                            "simulation_latency_ms": elapsed_ms,
                            "confidence": "uncertain"
                        }
                    }
                }));
            }
        };

        // 2. Simulate Transaction with persistent RpcDb
        match simulate_tx(&tx_env, db) {
            Ok(true) => {
                // Simulation succeeded and passed heuristics
                return Ok(());
            }
            Ok(false) => {
                let elapsed_ms = sim_start.elapsed().as_millis() as u64;
                return Err(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": "Execution reverted: simulation flagged transaction",
                        "data": {
                            "decoded_reason": "simulation flagged transaction",
                            "simulation_latency_ms": elapsed_ms,
                            "confidence": "high"
                        }
                    }
                }));
            }
            Err(blocked) => {
                // Ambiguous simulation handling:
                // If the simulation is uncertain (due to upstream state fetch timeout, RPC error, or stale state)
                // and fail_open is true (default for user wallets), do not block the user. Allow pass-through
                // to upstream sequencer.
                if blocked.confidence == SimulationConfidence::Uncertain && fail_open {
                    eprintln!(
                        "[WARN] Ambiguous simulation ({}); failing open to sequencer with confidence=uncertain",
                        blocked.message
                    );
                    return Ok(());
                }

                let elapsed_ms = sim_start.elapsed().as_millis() as u64;
                let estimated_gas_saved = if blocked.gas_used > 0 {
                    blocked.gas_used
                } else {
                    blocked.gas_limit
                };

                let mut data_obj = json!({
                    "estimated_gas_saved": estimated_gas_saved,
                    "simulation_latency_ms": elapsed_ms,
                    "confidence": blocked.confidence
                });

                if let Some(ref rd) = blocked.revert_data_hex {
                    data_obj["revert_data"] = json!(rd);
                }
                if let Some(ref dr) = blocked.decoded_reason {
                    data_obj["decoded_reason"] = json!(dr);
                }

                return Err(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": blocked.message,
                        "data": data_obj
                    }
                }));
            }
        }
    }
    Ok(())
}
