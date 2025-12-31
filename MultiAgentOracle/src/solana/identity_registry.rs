//! Identity registry client for Solana program
//! 
//! Simplified client for demonstration purposes

use crate::solana::{
    rpc_client::SolanaRpcClient,
    types::{SolanaAgentIdentity, SolanaConfig, SolanaError, SolanaResult, VerificationRequest, VerificationStatus},
};
use chrono::Utc;

/// Identity registry client
pub struct IdentityRegistryClient {
    /// Solana RPC client
    rpc_client: SolanaRpcClient,
    /// Program ID
    program_id: String,
}

impl IdentityRegistryClient {
    /// Create a new identity registry client
    pub fn new(rpc_client: SolanaRpcClient, program_id: &str) -> Self {
        Self {
            rpc_client,
            program_id: program_id.to_string(),
        }
    }
    
    /// Create from configuration
    pub fn from_config(config: &SolanaConfig) -> SolanaResult<Self> {
        let rpc_client = SolanaRpcClient::new(config.clone());
        Ok(Self::new(rpc_client, &config.program_id))
    }
    
    /// Get program ID
    pub fn program_id(&self) -> &str {
        &self.program_id
    }
    
    /// Register a new agent identity (simulated)
    pub async fn register_agent(
        &self,
        did: String,
        public_key: [u8; 32],
        metadata_uri: String,
    ) -> SolanaResult<String> {
        // Validate inputs
        if did.is_empty() || did.len() > 128 {
            return Err(SolanaError::ConfigError("Invalid DID length".to_string()));
        }
        
        if metadata_uri.len() > 256 {
            return Err(SolanaError::ConfigError("Metadata URI too long".to_string()));
        }
        
        // Simulate transaction
        let mock_signature = format!("mock_signature_{}", Utc::now().timestamp());
        
        println!("✅ Simulated identity registration:");
        println!("   DID: {}", did);
        println!("   Public Key: {:?}...", &public_key[..8]);
        println!("   Metadata URI: {}", metadata_uri);
        println!("   Program ID: {}", self.program_id);
        println!("   Mock Transaction: {}", mock_signature);
        
        Ok(mock_signature)
    }
    
    /// Get agent identity (simulated)
    pub async fn get_agent_identity(&self, did: &str) -> SolanaResult<Option<SolanaAgentIdentity>> {
        // Simulate fetching identity
        Ok(Some(SolanaAgentIdentity {
            did: did.to_string(),
            public_key: [0; 32],
            metadata_uri: "https://example.com/metadata.json".to_string(),
            owner: "mock_owner_address".to_string(),
            registered_at: Utc::now().timestamp(),
            is_active: true,
            is_verified: false,
            reputation_score: 100,
        }))
    }
    
    /// Update agent identity (simulated)
    pub async fn update_identity(
        &self,
        did: String,
        new_public_key: Option<[u8; 32]>,
        new_metadata_uri: Option<String>,
    ) -> SolanaResult<String> {
        println!("✅ Simulated identity update:");
        println!("   DID: {}", did);
        
        if let Some(pk) = new_public_key {
            println!("   New Public Key: {:?}...", &pk[..8]);
        }
        
        if let Some(uri) = &new_metadata_uri {
            println!("   New Metadata URI: {}", uri);
        }
        
        let mock_signature = format!("mock_update_{}", Utc::now().timestamp());
        Ok(mock_signature)
    }
    
    /// Request verification (simulated)
    pub async fn request_verification(
        &self,
        did: String,
        proof_data: Vec<u8>,
    ) -> SolanaResult<String> {
        println!("✅ Simulated verification request:");
        println!("   DID: {}", did);
        println!("   Proof Data Size: {} bytes", proof_data.len());
        
        let mock_signature = format!("mock_verification_{}", Utc::now().timestamp());
        Ok(mock_signature)
    }
    
    /// Check if identity is registered (simulated)
    pub async fn is_identity_registered(&self, did: &str) -> SolanaResult<bool> {
        // For demo, assume all DIDs starting with "did:" are registered
        Ok(did.starts_with("did:"))
    }
    
    /// Get all registered identities (simulated)
    pub async fn get_all_identities(&self) -> SolanaResult<Vec<SolanaAgentIdentity>> {
        // Return mock identities for demo
        Ok(vec![
            SolanaAgentIdentity {
                did: "did:agent:test-001".to_string(),
                public_key: [1; 32],
                metadata_uri: "https://ipfs.io/ipfs/QmTest1".to_string(),
                owner: "8c2oMVcEe7KY3PdGwo8UYa65FbKcxdpsZc7zZnBNZzQS".to_string(),
                registered_at: Utc::now().timestamp() - 86400,
                is_active: true,
                is_verified: true,
                reputation_score: 850,
            },
            SolanaAgentIdentity {
                did: "did:agent:test-002".to_string(),
                public_key: [2; 32],
                metadata_uri: "https://ipfs.io/ipfs/QmTest2".to_string(),
                owner: "MMyshFPdwqdPGbSDaFtFMBmcrzpVnRAoMjaohJSurbm".to_string(),
                registered_at: Utc::now().timestamp() - 43200,
                is_active: true,
                is_verified: false,
                reputation_score: 650,
            },
        ])
    }
}

/// Demo function to test identity registration
pub async fn demo_identity_registration() -> SolanaResult<()> {
    println!("🚀 Starting Solana identity registration demo...");
    println!("{}", "-".repeat(50));
    
    // Create client with default devnet config
    let config = SolanaConfig::default();
    let client = IdentityRegistryClient::from_config(&config)?;
    
    println!("✅ Client created");
    println!("   Program ID: {}", client.program_id());
    println!("   RPC URL: {}", config.rpc_url);
    
    // Check connection
    let rpc_client = SolanaRpcClient::new(config);
    match rpc_client.check_connection().await {
        Ok(true) => println!("✅ Connected to Solana network"),
        Ok(false) => println!("⚠️  Not connected to Solana network"),
        Err(e) => println!("⚠️  Connection check error: {}", e),
    }
    
    // Get network info
    match rpc_client.get_network_info().await {
        Ok(info) => println!("📡 Network: {}", info),
        Err(e) => println!("⚠️  Failed to get network info: {}", e),
    }
    
    // Demo 1: Register a new identity
    println!("\n📋 Demo 1: Register New Identity");
    println!("{}", "-".repeat(30));
    
    let test_did = "did:agent:demo-001";
    let test_public_key = [42u8; 32];
    let test_metadata_uri = "https://ipfs.io/ipfs/QmDemoMetadata".to_string();
    
    match client.register_agent(
        test_did.to_string(),
        test_public_key,
        test_metadata_uri,
    ).await {
        Ok(signature) => println!("✅ Registration simulated: {}", signature),
        Err(e) => println!("❌ Registration failed: {}", e),
    }
    
    // Demo 2: Check if identity is registered
    println!("\n📋 Demo 2: Check Identity Registration");
    println!("{}", "-".repeat(30));
    
    match client.is_identity_registered(test_did).await {
        Ok(true) => println!("✅ Identity '{}' is registered", test_did),
        Ok(false) => println!("❌ Identity '{}' is not registered", test_did),
        Err(e) => println!("⚠️  Check failed: {}", e),
    }
    
    // Demo 3: Get identity details
    println!("\n📋 Demo 3: Get Identity Details");
    println!("{}", "-".repeat(30));
    
    match client.get_agent_identity(test_did).await {
        Ok(Some(identity)) => {
            println!("✅ Identity found:");
            println!("   DID: {}", identity.did);
            println!("   Owner: {}", identity.owner);
            println!("   Active: {}", identity.is_active);
            println!("   Verified: {}", identity.is_verified);
            println!("   Reputation: {}", identity.reputation_score);
        }
        Ok(None) => println!("❌ Identity not found"),
        Err(e) => println!("⚠️  Failed to get identity: {}", e),
    }
    
    // Demo 4: List all identities
    println!("\n📋 Demo 4: List All Identities");
    println!("{}", "-".repeat(30));
    
    match client.get_all_identities().await {
        Ok(identities) => {
            println!("✅ Found {} identities:", identities.len());
            for (i, identity) in identities.iter().enumerate() {
                println!("   {}. {} (Owner: {}...)", 
                    i + 1, 
                    identity.did,
                    &identity.owner[..8]
                );
            }
        }
        Err(e) => println!("⚠️  Failed to list identities: {}", e),
    }
    
    println!("\n🎉 Demo completed successfully!");
    println!("\n📝 Next steps for real implementation:");
    println!("   1. Deploy Solana program: cd solana-oracle && anchor deploy");
    println!("   2. Update program ID in configuration");
    println!("   3. Implement real transaction signing");
    println!("   4. Add proper error handling and retry logic");
    
    Ok(())
}