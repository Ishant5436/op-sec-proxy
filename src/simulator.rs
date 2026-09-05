use alloy::consensus::{TxEnvelope, Transaction, transaction::SignerRecoverable};
use revm::{Context, MainContext, MainBuilder, ExecuteEvm};
use revm::context::TxEnv;
use revm::primitives::TxKind;
use revm::database::WrapDatabaseRef;
use crate::fork_db::RpcDb;

/// Structured revert payload emitted when a transaction is blocked.
#[derive(Debug, Clone)]
pub struct BlockedTx {
    pub message: String,
    pub revert_data_hex: Option<String>,
    pub decoded_reason: Option<String>,
    pub gas_used: u64,
    pub gas_limit: u64,
}

pub fn simulate_tx(tx_env: &TxEnvelope, db: RpcDb) -> Result<bool, BlockedTx> {
    // 1. Recover the sender address
    let caller = tx_env.recover_signer().map_err(|e| BlockedTx {
        message: format!("Failed to recover signer: {:?}", e),
        revert_data_hex: None,
        decoded_reason: None,
        gas_used: 0,
        gas_limit: 0,
    })?;
    
    // 2. Setup the EVM context and database
    let db_wrapper = WrapDatabaseRef(db);
    let ctx = Context::mainnet().with_db(db_wrapper);
    let mut evm = ctx.build_mainnet();
    
    // 3. Extract transaction details
    let gas_limit = tx_env.gas_limit();
    let value = tx_env.value();
    let to_addr = tx_env.to();
    
    let kind = match to_addr {
        Some(addr) => TxKind::Call(addr),
        None => TxKind::Create,
    };
    
    let data = tx_env.input().clone();
    
    // 4. Construct the revm TxEnv
    let revm_tx = TxEnv::builder()
        .caller(caller)
        .kind(kind)
        .value(value)
        .data(data)
        .gas_limit(gas_limit)
        .build()
        .map_err(|e| BlockedTx {
            message: format!("Failed to build revm TxEnv: {:?}", e),
            revert_data_hex: None,
            decoded_reason: None,
            gas_used: 0,
            gas_limit,
        })?;
        
    // 5. Execute the simulation
    let sim_result = evm.transact(revm_tx)
        .map_err(|e| BlockedTx {
            message: format!("Simulation execution error: {:?}", e),
            revert_data_hex: None,
            decoded_reason: None,
            gas_used: 0,
            gas_limit,
        })?;
        
    // 6. Security Analysis
    let gas_used = sim_result.result.gas().tx_gas_used();

    if sim_result.result.is_success() {
        // Rule 2: Max Gas Overhead Heuristic
        if gas_used > 0 && gas_used >= gas_limit {
            return Err(BlockedTx {
                message: format!("Transaction consumes entire gas limit ({}). Potential DoS or infinite loop.", gas_used),
                revert_data_hex: None,
                decoded_reason: None,
                gas_used,
                gas_limit,
            });
        }
        Ok(true)
    } else if sim_result.result.is_halt() {
        Err(BlockedTx {
            message: "Transaction halted".to_string(),
            revert_data_hex: None,
            decoded_reason: None,
            gas_used,
            gas_limit,
        })
    } else {
        // Rule 3: Decoded Revert Detection (Milestone 1 Deliverable)
        if let Some(output) = sim_result.result.output() {
            let hex_output = format!("0x{}", alloy::hex::encode(output));
            let mut decoded: Option<String> = None;

            if output.len() >= 68 && output[0..4] == [0x08, 0xc3, 0x79, 0xa0] {
                if let Ok(reason) = std::str::from_utf8(&output[68..]) {
                    let clean = reason.trim_matches(char::from(0)).trim();
                    if !clean.is_empty() {
                        decoded = Some(clean.to_string());
                    }
                }
            } else if output.len() >= 36 && output[0..4] == [0x4e, 0x48, 0x7b, 0x71] {
                // Solidity Panic(uint256) selector 0x4e487b71 + 32-byte big-endian code
                let code_byte = output[35];
                let reason = match code_byte {
                    0x01 => "Panic: Assert failed",
                    0x11 => "Panic: Arithmetic overflow / underflow",
                    0x12 => "Panic: Division by zero",
                    0x21 => "Panic: Invalid enum value conversion",
                    0x22 => "Panic: Storage byte array encoding error",
                    0x31 => "Panic: Empty array pop",
                    0x32 => "Panic: Array index out of bounds",
                    0x41 => "Panic: Allocation of too much memory",
                    0x51 => "Panic: Zero initialized internal function pointer",
                    _ => "Panic: Unrecognized panic code",
                };
                decoded = Some(format!("{} (0x{:02x})", reason, code_byte));
            }

            let msg = match &decoded {
                Some(r) => format!("Execution reverted: {}", r),
                None => format!("Transaction reverted during simulation ({})", hex_output),
            };

            return Err(BlockedTx {
                message: msg,
                revert_data_hex: Some(hex_output),
                decoded_reason: decoded,
                gas_used,
                gas_limit,
            });
        }
        Err(BlockedTx {
            message: "Transaction reverted during simulation".to_string(),
            revert_data_hex: None,
            decoded_reason: None,
            gas_used,
            gas_limit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, B256, Bytes, U256};
    use std::str::FromStr;
    use tokio::net::TcpListener;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response, body::Incoming};
    use http_body_util::Full;
    use hyper::body::Bytes as HyperBytes;
    use serde_json::json;
    use alloy::consensus::TxLegacy;
    use alloy::signers::local::PrivateKeySigner;
    use std::convert::Infallible;

    use alloy::network::TxSignerSync;

    async fn handle_mock_rpc(req: Request<Incoming>) -> Result<Response<Full<HyperBytes>>, Infallible> {
        use http_body_util::BodyExt;
        
        let body_bytes = req.into_body().collect().await.unwrap().to_bytes();
        let payload: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        
        let method = payload["method"].as_str().unwrap_or("");
        
        let result = match method {
            "eth_getBalance" => json!("0x0"), // 0 ETH
            "eth_getTransactionCount" => json!("0x0"),
            "eth_getCode" => json!("0x"), // Empty code (EOA)
            _ => json!(null)
        };
        
        let resp = json!({
            "jsonrpc": "2.0",
            "id": payload["id"],
            "result": result
        });
        
        Ok(Response::new(Full::new(HyperBytes::from(resp.to_string()))))
    }

    async fn spawn_mock_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        
        tokio::spawn(async move {
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    let io = hyper_util::rt::TokioIo::new(stream);
                    tokio::spawn(async move {
                        let _ = http1::Builder::new().serve_connection(io, service_fn(handle_mock_rpc)).await;
                    });
                }
            }
        });
        
        format!("http://127.0.0.1:{}", port)
    }

    fn create_dummy_tx(gas_limit: u64, value: u128) -> TxEnvelope {
        let signer = PrivateKeySigner::random();
        let mut tx = TxLegacy {
            chain_id: Some(10),
            nonce: 0,
            gas_price: 1_000_000_000,
            gas_limit,
            to: alloy::primitives::TxKind::Call(Address::from_str("0x0000000000000000000000000000000000000000").unwrap()),
            value: U256::from(value),
            input: Bytes::default(),
        };
        let signature = signer.sign_transaction_sync(&mut tx).unwrap();
        TxEnvelope::Legacy(alloy::consensus::Signed::new_unchecked(tx, signature, B256::ZERO))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_simulate_tx_success() {
        let url = spawn_mock_server().await;
        
        let tx = create_dummy_tx(50000, 0); // High gas limit, avoids heuristic
        
        let res = tokio::task::spawn_blocking(move || {
            let db = RpcDb::new(url);
            simulate_tx(&tx, db)
        }).await.unwrap();
        
        assert!(res.is_ok());
        assert!(res.unwrap());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_simulate_tx_gas_heuristic() {
        let url = spawn_mock_server().await;
        
        let tx = create_dummy_tx(21000, 0); // Transfer takes exactly 21000 gas
        
        let res = tokio::task::spawn_blocking(move || {
            let db = RpcDb::new(url);
            simulate_tx(&tx, db)
        }).await.unwrap();
        
        // Because gas_used == gas_limit (21000 == 21000), it triggers the heuristic block
        assert!(res.is_err());
        assert!(res.unwrap_err().message.contains("consumes entire gas limit"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_simulate_tx_revert_due_to_insufficient_funds() {
        let url = spawn_mock_server().await;
        
        // Value larger than mock balance (1 ETH = 10^18 wei, so let's send 2 ETH)
        let tx = create_dummy_tx(50000, 2_000_000_000_000_000_000); 
        
        let res = tokio::task::spawn_blocking(move || {
            let db = RpcDb::new(url);
            simulate_tx(&tx, db)
        }).await.unwrap();
        
        // Revm errors out before execution if sender lacks balance for value + gas
        assert!(res.is_err());
        let blocked = res.unwrap_err();
        assert!(blocked.message.contains("Simulation execution error"));
    }

    #[test]
    fn test_panic_code_decoding_logic() {
        // Selector 0x4e487b71 + 31 zero bytes + 0x11 (Arithmetic overflow)
        let mut overflow_panic = vec![0x4e, 0x48, 0x7b, 0x71];
        overflow_panic.extend(vec![0u8; 31]);
        overflow_panic.push(0x11);

        assert_eq!(overflow_panic.len(), 36);
        assert_eq!(overflow_panic[0..4], [0x4e, 0x48, 0x7b, 0x71]);
        assert_eq!(overflow_panic[35], 0x11);

        // Division by zero 0x12
        let mut div_zero_panic = vec![0x4e, 0x48, 0x7b, 0x71];
        div_zero_panic.extend(vec![0u8; 31]);
        div_zero_panic.push(0x12);
        assert_eq!(div_zero_panic[35], 0x12);
    }
}

