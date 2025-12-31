//! Lightweight Solana RPC client using HTTP requests

use crate::solana::types::{SolanaConfig, SolanaError, SolanaResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Solana RPC request
#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

/// Solana RPC response
#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    jsonrpc: String,
    id: u64,
    result: Option<T>,
    error: Option<RpcError>,
}

/// RPC error
#[derive(Debug, Deserialize)]
struct RpcError {
    code: i32,
    message: String,
}

/// Balance response
#[derive(Debug, Deserialize)]
struct BalanceResponse {
    value: u64,
}

/// Version response
#[derive(Debug, Deserialize)]
struct VersionResponse {
    #[serde(rename = "solana-core")]
    solana_core: String,
}

/// Lightweight Solana RPC client
#[derive(Clone)]
pub struct SolanaRpcClient {
    /// HTTP client
    http_client: Arc<Client>,
    /// Configuration
    config: SolanaConfig,
    /// Request counter
    request_id: Arc<tokio::sync::Mutex<u64>>,
}

impl SolanaRpcClient {
    /// Create a new Solana RPC client
    pub fn new(config: SolanaConfig) -> Self {
        Self {
            http_client: Arc::new(Client::new()),
            config,
            request_id: Arc::new(tokio::sync::Mutex::new(1)),
        }
    }
    
    /// Create a client with default devnet configuration
    pub fn new_devnet() -> Self {
        Self::new(SolanaConfig::default())
    }
    
    /// Get configuration
    pub fn config(&self) -> &SolanaConfig {
        &self.config
    }
    
    /// Make RPC request
    async fn rpc_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> SolanaResult<T> {
        let mut request_id = self.request_id.lock().await;
        let id = *request_id;
        *request_id += 1;
        
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };
        
        let response = self.http_client
            .post(&self.config.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SolanaError::RpcError(format!("HTTP request failed: {}", e)))?;
        
        let rpc_response: RpcResponse<T> = response
            .json()
            .await
            .map_err(|e| SolanaError::RpcError(format!("Failed to parse response: {}", e)))?;
        
        if let Some(error) = rpc_response.error {
            return Err(SolanaError::RpcError(format!("RPC error {}: {}", error.code, error.message)));
        }
        
        rpc_response.result
            .ok_or_else(|| SolanaError::RpcError("No result in response".to_string()))
    }
    
    /// Get balance for an address
    pub async fn get_balance(&self, address: &str) -> SolanaResult<u64> {
        let params = json!([address, {"commitment": "confirmed"}]);
        
        let response: BalanceResponse = self.rpc_request("getBalance", params).await?;
        Ok(response.value)
    }
    
    /// Get current slot
    pub async fn get_slot(&self) -> SolanaResult<u64> {
        let params = json!([{"commitment": "confirmed"}]);
        
        self.rpc_request("getSlot", params).await
    }
    
    /// Get version
    pub async fn get_version(&self) -> SolanaResult<String> {
        let response: VersionResponse = self.rpc_request("getVersion", json!([])).await?;
        Ok(response.solana_core)
    }
    
    /// Get recent blockhash
    pub async fn get_recent_blockhash(&self) -> SolanaResult<(String, u64)> {
        #[derive(Debug, Deserialize)]
        struct BlockhashResponse {
            value: BlockhashValue,
        }
        
        #[derive(Debug, Deserialize)]
        struct BlockhashValue {
            blockhash: String,
            #[serde(rename = "lastValidBlockHeight")]
            last_valid_block_height: u64,
        }
        
        let params = json!([{"commitment": "confirmed"}]);
        
        let response: BlockhashResponse = self.rpc_request("getLatestBlockhash", params).await?;
        Ok((response.value.blockhash, response.value.last_valid_block_height))
    }
    
    /// Request airdrop (devnet only)
    pub async fn request_airdrop(&self, address: &str, amount: u64) -> SolanaResult<String> {
        let params = json!([address, amount, {"commitment": "confirmed"}]);
        
        #[derive(Debug, Deserialize)]
        struct AirdropResponse {
            signature: String,
        }
        
        let response: AirdropResponse = self.rpc_request("requestAirdrop", params).await?;
        Ok(response.signature)
    }
    
    /// Get transaction status
    pub async fn get_transaction_status(&self, signature: &str) -> SolanaResult<bool> {
        let params = json!([signature, {"commitment": "confirmed"}]);
        
        #[derive(Debug, Deserialize)]
        struct TransactionResponse {
            slot: u64,
            // other fields omitted
        }
        
        match self.rpc_request::<TransactionResponse>("getTransaction", params).await {
            Ok(_) => Ok(true),
            Err(SolanaError::RpcError(e)) if e.contains("not found") => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    /// Check if connected to network
    pub async fn check_connection(&self) -> SolanaResult<bool> {
        match self.get_version().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Get network info
    pub async fn get_network_info(&self) -> SolanaResult<String> {
        let version = self.get_version().await?;
        let slot = self.get_slot().await?;
        
        Ok(format!("Solana {} at slot {}", version, slot))
    }
}

/// Helper function to validate Solana address
pub fn validate_address(address: &str) -> bool {
    if address.len() != 44 {
        return false;
    }
    
    // Simple base58 validation
    bs58::decode(address).into_vec().is_ok()
}

/// Helper function to generate a mock address for testing
pub fn generate_mock_address() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    bs58::encode(bytes).into_string()
}
