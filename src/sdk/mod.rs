//! SDK 便捷层 - 面向智能合约开发者
//!
//! 提供高级 API，简化多智能体预言机系统的集成。
//!
//! # 快速开始
//!
//! ```rust,ignore
//! use multi_agent_oracle::sdk::{Oracle, SdkConfig};
//!
//! // 创建预言机实例
//! let config = SdkConfig::builder()
//!     .with_solana_rpc("https://api.devnet.solana.com")
//!     .build();
//!
//! let oracle = Oracle::new(config)?;
//!
//! // 发起查询
//! let result = oracle.query("BTC price").await?;
//! ```

pub mod builder;
pub mod oracle;
pub mod solana;
pub mod types;

// 重新导出
pub use builder::{OracleBuilder, SdkConfig, SdkConfigBuilder};
pub use oracle::Oracle;
pub use solana::{SolanaConfig, SolanaConfigBuilder, SolanaIntegration};
pub use types::{
    AgentSubmission, ChainSubmissionData, ConsensusOutput, OracleQuery, OracleResponse,
    OracleResult, SdkError, SdkResult,
};
