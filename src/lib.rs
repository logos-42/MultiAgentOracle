//! 多智能体预言机系统库
//!
//! 去中心化多智能体预言机网络，每个智能体都是独立的预言机节点。

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

// 导出常用类型
pub use anyhow::Result;
pub use serde::{Deserialize, Serialize};

// 模块声明
pub mod oracle_agent;
pub mod reputation;
pub mod consensus;
pub mod blockchain;
pub mod network;
pub mod types;
// // pub mod diap;
pub mod solana;
pub mod test;
pub mod zkp;
pub mod causal_graph;

// 重新导出主要类型
pub use oracle_agent::{OracleAgent, OracleAgentConfig, OracleDataType, OracleData, DataSource};
pub use reputation::{ReputationManager, ReputationScore, ReputationConfig, ReputationMetrics, ReputationUpdate};
pub use consensus::{ConsensusEngine, ConsensusResult, SpectralFeatures, extract_spectral_features};
pub use blockchain::{BlockchainAdapter, ChainType};
pub use network::{NetworkManager, NetworkConfig, PeerInfo};
// // pub use diap::{DiapConfig, DiapIdentityManager, AgentIdentity, DiapNetworkAdapter, DiapError, AuthResult, IdentityStatus};
pub use types::{NodeId, NodeInfo, Timestamp, current_timestamp, SystemError, NetworkMessage};
pub use zkp::{ZkpGenerator, ZkpConfig, ZkProof, PublicInputs, PrivateInputs, CircuitInputs};
pub use causal_graph::{CausalGraph, CausalGraphBuilder, GraphBuilderConfig, CausalEffect, Intervention, DoOperatorResult};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = "多智能体预言机系统";
