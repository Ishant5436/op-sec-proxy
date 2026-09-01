use revm::{
    DatabaseRef,
    state::AccountInfo,
    bytecode::Bytecode,
    primitives::{Address, B256, U256},
    database::DBErrorMarker,
};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::str::FromStr;
use std::time::Duration;
use std::sync::{Arc, Mutex};

/// Custom error type for RPC database operations.
/// Replaces `Infallible` to allow graceful error propagation
/// instead of panicking on network failures.
#[derive(Debug, Clone)]
pub struct RpcDbError(pub String);

impl std::fmt::Display for RpcDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RpcDbError: {}", self.0)
    }
}

impl std::error::Error for RpcDbError {}

impl DBErrorMarker for RpcDbError {}

use crate::lru::LruCache;

pub const MAX_ACCOUNT_CACHE_CAPACITY: usize = 4096;
pub const MAX_STORAGE_CACHE_CAPACITY: usize = 16384;

#[derive(Clone)]
pub struct RpcDb {
    client: Client,
    rpc_url: String,
    account_cache: Arc<Mutex<LruCache<Address, AccountInfo>>>,
    storage_cache: Arc<Mutex<LruCache<(Address, U256), U256>>>,
}

impl RpcDb {
    pub fn new(rpc_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to build blocking HTTP client"),
            rpc_url,
            account_cache: Arc::new(Mutex::new(LruCache::new(MAX_ACCOUNT_CACHE_CAPACITY))),
            storage_cache: Arc::new(Mutex::new(LruCache::new(MAX_STORAGE_CACHE_CAPACITY))),
        }
    }
    
    fn rpc_call(&self, method: &str, params: Value) -> Result<Value, RpcDbError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        
        let resp = self.client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .map_err(|e| RpcDbError(format!("RPC '{}' request failed: {}", method, e)))?;

        let json_resp: Value = resp.json()
            .map_err(|e| RpcDbError(format!("RPC '{}' response parse failed: {}", method, e)))?;
        
        if let Some(err) = json_resp.get("error") {
            return Err(RpcDbError(format!("RPC '{}' returned error: {}", method, err)));
        }
        
        Ok(json_resp["result"].clone())
    }
}

impl DatabaseRef for RpcDb {
    type Error = RpcDbError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // 1. Check in-memory state LRU cache with mutex poison recovery
        {
            let mut guard = self.account_cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(cached) = guard.get(&address) {
                return Ok(Some(cached));
            }
        }

        let addr_str = address.to_string();
        
        // Fetch balance
        let bal_res = self.rpc_call("eth_getBalance", json!([addr_str, "latest"]))?;
        let bal_str = bal_res.as_str().unwrap_or("0x0");
        let balance = U256::from_str(bal_str)
            .map_err(|e| RpcDbError(format!("Failed to parse balance '{}': {}", bal_str, e)))?;
        
        // Fetch nonce
        let nonce_res = self.rpc_call("eth_getTransactionCount", json!([addr_str, "latest"]))?;
        let nonce_str = nonce_res.as_str().unwrap_or("0x0").trim_start_matches("0x");
        let nonce = if nonce_str.is_empty() { 0 } else { 
            u64::from_str_radix(nonce_str, 16)
                .map_err(|e| RpcDbError(format!("Failed to parse nonce '{}': {}", nonce_str, e)))?
        };
        
        // Fetch code
        let code_res = self.rpc_call("eth_getCode", json!([addr_str, "latest"]))?;
        let code_hex = code_res.as_str().unwrap_or("0x").trim_start_matches("0x");
        let code_bytes = alloy::hex::decode(code_hex)
            .map_err(|e| RpcDbError(format!("Failed to decode bytecode hex: {}", e)))?;
        let bytecode = Bytecode::new_raw(alloy::primitives::Bytes::from(code_bytes));
        
        let account_info = AccountInfo {
            balance,
            nonce,
            code_hash: bytecode.hash_slow(),
            code: Some(bytecode),
            ..Default::default()
        };

        // Cache the fetched account state in O(1) LRU
        {
            let mut guard = self.account_cache.lock().unwrap_or_else(|e| e.into_inner());
            guard.insert(address, account_info.clone());
        }

        Ok(Some(account_info))
    }

    fn code_by_hash_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        // Return empty bytecode — CacheDB should handle lookups by hash
        // if the code was previously fetched via basic_ref.
        Ok(Bytecode::default())
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        // 1. Check in-memory storage LRU cache with mutex poison recovery
        {
            let mut guard = self.storage_cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(cached_val) = guard.get(&(address, index)) {
                return Ok(cached_val);
            }
        }

        let addr_str = address.to_string();
        let idx_str = format!("0x{:x}", index);
        
        let res = self.rpc_call("eth_getStorageAt", json!([addr_str, idx_str, "latest"]))?;
        let val_str = res.as_str().unwrap_or("0x0");
        let val = U256::from_str(val_str)
            .map_err(|e| RpcDbError(format!("Failed to parse storage value '{}': {}", val_str, e)))?;

        // Cache the fetched storage slot in O(1) LRU
        {
            let mut guard = self.storage_cache.lock().unwrap_or_else(|e| e.into_inner());
            guard.insert((address, index), val);
        }

        Ok(val)
    }

    fn block_hash_ref(&self, _number: u64) -> Result<B256, Self::Error> {
        // Block hash logic is omitted for MVP simplicity
        Ok(B256::default())
    }
}
