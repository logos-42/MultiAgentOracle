//! Solana blockchain integration for multi-agent oracle system
//! 
//! Lightweight HTTP RPC client for Solana integration

#[cfg(feature = "solana")]
pub mod rpc_client;
#[cfg(feature = "solana")]
pub mod identity_registry;
#[cfg(feature = "solana")]
pub mod types;
pub mod consensus_deployer;
// pub mod real_consensus_deployer;
pub mod simple_solana_deployer;
pub mod true_solana_deployer;
pub mod real_devnet_deployer;

#[cfg(feature = "solana")]
pub use rpc_client::SolanaRpcClient;
#[cfg(feature = "solana")]
pub use identity_registry::IdentityRegistryClient;
#[cfg(feature = "solana")]
pub use types::*;
pub use consensus_deployer::*;
// pub use real_consensus_deployer::*;
pub use simple_solana_deployer::*;
pub use true_solana_deployer::*;
pub use real_devnet_deployer::*;

#[cfg(feature = "solana")]
pub use identity_registry::demo_identity_registration;