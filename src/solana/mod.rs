//! Solana blockchain integration for multi-agent oracle system
//! 
//! Lightweight HTTP RPC client for Solana integration

#[cfg(feature = "solana")]
pub mod rpc_client;
#[cfg(feature = "solana")]
pub mod identity_registry;
#[cfg(feature = "solana")]
pub mod types;

#[cfg(feature = "solana")]
pub use rpc_client::SolanaRpcClient;
#[cfg(feature = "solana")]
pub use identity_registry::IdentityRegistryClient;
#[cfg(feature = "solana")]
pub use types::*;

#[cfg(feature = "solana")]
pub use identity_registry::demo_identity_registration;