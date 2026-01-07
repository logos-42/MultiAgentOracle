//! Types for Solana integration

use serde::{Deserialize, Serialize};

/// Solana configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    /// RPC endpoint URL
    pub rpc_url: String,
    /// WebSocket endpoint URL
    pub ws_url: String,
    /// Program ID for identity registry
    pub program_id: String,
    /// Commitment level
    pub commitment: String,
}

impl Default for SolanaConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ws_url: "wss://api.devnet.solana.com".to_string(),
            program_id: "DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b".to_string(),
            commitment: "confirmed".to_string(),
        }
    }
}

/// Agent identity data for Solana
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaAgentIdentity {
    /// Decentralized Identifier
    pub did: String,
    /// Agent's public key (32 bytes)
    pub public_key: [u8; 32],
    /// Metadata URI
    pub metadata_uri: String,
    /// Wallet address (owner)
    pub owner: String,
    /// Registration timestamp
    pub registered_at: i64,
    /// Whether identity is active
    pub is_active: bool,
    /// Whether identity is verified
    pub is_verified: bool,
    /// Reputation score
    pub reputation_score: u64,
}

/// Verification request data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    /// Request ID
    pub request_id: u64,
    /// Agent DID
    pub did: String,
    /// Proof data
    pub proof_data: Vec<u8>,
    /// Request timestamp
    pub requested_at: i64,
    /// Verification status
    pub status: VerificationStatus,
}

/// Verification status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationStatus {
    Pending,
    Approved,
    Rejected,
    Revoked,
}

/// Error types for Solana operations
#[derive(Debug, thiserror::Error)]
pub enum SolanaError {
    #[error("RPC connection error: {0}")]
    RpcError(String),
    
    #[error("Transaction failed: {0}")]
    TransactionError(String),
    
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    #[error("Program error: {0}")]
    ProgramError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Wallet error: {0}")]
    WalletError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Result type for Solana operations
pub type SolanaResult<T> = Result<T, SolanaError>;

/// Helper functions
#[cfg(feature = "solana")]
pub fn bytes_to_base58(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

#[cfg(feature = "solana")]
pub fn base58_to_bytes(s: &str) -> SolanaResult<Vec<u8>> {
    bs58::decode(s)
        .into_vec()
        .map_err(|e| SolanaError::SerializationError(format!("Invalid base58: {}", e)))
}

/// Validate Solana address format
pub fn validate_solana_address(address: &str) -> bool {
    if address.len() != 44 {
        return false;
    }
    
    // Simple base58 validation
    #[cfg(feature = "solana")]
    {
        bs58::decode(address).into_vec().is_ok()
    }
    
    #[cfg(not(feature = "solana"))]
    {
        // Simple pattern check when bs58 is not available
        address.chars().all(|c| c.is_ascii_alphanumeric())
    }
}

/// Generate mock address for testing
pub fn generate_mock_address() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    
    #[cfg(feature = "solana")]
    {
        bs58::encode(bytes).into_string()
    }
    
    #[cfg(not(feature = "solana"))]
    {
        // Simple mock when bs58 is not available
        format!("mock_{:x}", rand::random::<u64>())
    }
}