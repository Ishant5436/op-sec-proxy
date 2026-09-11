use reqwest::Client;
use revm::{
    DatabaseRef,
    bytecode::Bytecode,
    database::DBErrorMarker,
    primitives::{Address, B256, U256},
    state::AccountInfo,
};
use serde_json::{Value, json};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    handle: Option<tokio::runtime::Handle>,
    account_cache: Arc<Mutex<LruCache<Address, AccountInfo>>>,
    storage_cache: Arc<Mutex<LruCache<(Address, U256), U256>>>,
}

impl RpcDb {
    pub fn new(rpc_url: String) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(20)
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");
        Self::new_with_client_and_caches(
            client,
            rpc_url,
            Arc::new(Mutex::new(LruCache::new(MAX_ACCOUNT_CACHE_CAPACITY))),
            Arc::new(Mutex::new(LruCache::new(MAX_STORAGE_CACHE_CAPACITY))),
        )
    }

    #[allow(dead_code)]
    pub fn new_with_caches(
        rpc_url: String,
        account_cache: Arc<Mutex<LruCache<Address, AccountInfo>>>,
        storage_cache: Arc<Mutex<LruCache<(Address, U256), U256>>>,
    ) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(20)
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");
        Self::new_with_client_and_caches(client, rpc_url, account_cache, storage_cache)
    }

    pub fn new_with_client_and_caches(
        client: Client,
        rpc_url: String,
        account_cache: Arc<Mutex<LruCache<Address, AccountInfo>>>,
        storage_cache: Arc<Mutex<LruCache<(Address, U256), U256>>>,
    ) -> Self {
        let handle = tokio::runtime::Handle::try_current().ok();
        Self {
            client,
            rpc_url,
            handle,
            account_cache,
            storage_cache,
        }
    }

    fn rpc_call(&self, method: &str, params: Value) -> Result<Value, RpcDbError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let send_fut = self.client.post(&self.rpc_url).json(&payload).send();

        let resp = match &self.handle {
            Some(h) => h.block_on(send_fut),
            None => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| RpcDbError(format!("Failed to spawn local runtime: {}", e)))?;
                rt.block_on(send_fut)
            }
        }
        .map_err(|e| RpcDbError(format!("RPC '{}' request failed: {}", method, e)))?;

        let json_fut = resp.json();
        let json_resp: Value = match &self.handle {
            Some(h) => h.block_on(json_fut),
            None => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| RpcDbError(format!("Failed to spawn local runtime: {}", e)))?;
                rt.block_on(json_fut)
            }
        }
        .map_err(|e| RpcDbError(format!("RPC '{}' response parse failed: {}", method, e)))?;

        if let Some(err) = json_resp.get("error") {
            return Err(RpcDbError(format!(
                "RPC '{}' returned error: {}",
                method, err
            )));
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
        let nonce = if nonce_str.is_empty() {
            0
        } else {
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
        let val = U256::from_str(val_str).map_err(|e| {
            RpcDbError(format!(
                "Failed to parse storage value '{}': {}",
                val_str, e
            ))
        })?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_db_client_sharing() {
        let client = Client::builder().build().unwrap();
        let db1 = RpcDb::new_with_client_and_caches(
            client.clone(),
            "http://127.0.0.1:8545".to_string(),
            Arc::new(Mutex::new(LruCache::new(10))),
            Arc::new(Mutex::new(LruCache::new(10))),
        );
        let db2 = RpcDb::new_with_client_and_caches(
            client,
            "http://127.0.0.1:8545".to_string(),
            Arc::new(Mutex::new(LruCache::new(10))),
            Arc::new(Mutex::new(LruCache::new(10))),
        );
        assert_eq!(db1.rpc_url, db2.rpc_url);
        assert!(db1.handle.is_some());
    }
}
