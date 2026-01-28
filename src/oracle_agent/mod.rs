//! 预言机智能体模块
//!
//! 每个智能体都是一个独立的预言机节点，使用DIAP身份进行认证。

mod agent;
mod data_collection;
mod data_types;
mod config;
mod llm_client;

// 重新导出
pub use agent::OracleAgent;
pub use data_collection::{DataCollectionResult, DataCollector};
pub use data_types::{OracleDataType, OracleData, DataValue};
pub use config::{OracleAgentConfig, DataSource};
pub use llm_client::{LlmClient, LlmClientConfig, LlmProvider, LlmResponse, Usage};

// 内部模块
pub(crate) mod http_client;
pub(crate) mod validation;
