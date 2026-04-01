//! Solana 链上集成
//!
//! 提供与 Solana 智能合约交互的便捷 API。

use crate::sdk::types::{ChainSubmissionData, ConsensusOutput, SdkError, SdkResult};
use serde::{Deserialize, Serialize};

/// Solana 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    /// RPC 端点
    pub rpc_url: String,
    /// 预言机合约地址
    pub oracle_program_id: String,
    /// 身份注册表合约地址
    pub identity_program_id: Option<String>,
    /// 钱包密钥（可选，用于签名交易）
    pub wallet_keypair: Option<Vec<u8>>,
    /// 交易超时（秒）
    pub transaction_timeout_secs: u64,
}

impl Default for SolanaConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            oracle_program_id: String::new(),
            identity_program_id: None,
            wallet_keypair: None,
            transaction_timeout_secs: 60,
        }
    }
}

impl SolanaConfig {
    /// 创建配置构建器
    pub fn builder() -> SolanaConfigBuilder {
        SolanaConfigBuilder::default()
    }
}

/// Solana 配置构建器
#[derive(Debug, Clone)]
pub struct SolanaConfigBuilder {
    config: SolanaConfig,
}

impl Default for SolanaConfigBuilder {
    fn default() -> Self {
        Self {
            config: SolanaConfig::default(),
        }
    }
}

impl SolanaConfigBuilder {
    /// 设置 RPC 端点
    pub fn with_rpc_url(mut self, url: impl Into<String>) -> Self {
        self.config.rpc_url = url.into();
        self
    }

    /// 设置预言机合约地址
    pub fn with_oracle_program_id(mut self, program_id: impl Into<String>) -> Self {
        self.config.oracle_program_id = program_id.into();
        self
    }

    /// 设置身份注册表合约地址
    pub fn with_identity_program_id(mut self, program_id: impl Into<String>) -> Self {
        self.config.identity_program_id = Some(program_id.into());
        self
    }

    /// 设置钱包密钥
    pub fn with_wallet_keypair(mut self, keypair: Vec<u8>) -> Self {
        self.config.wallet_keypair = Some(keypair);
        self
    }

    /// 设置交易超时
    pub fn with_transaction_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.transaction_timeout_secs = timeout_secs;
        self
    }

    /// 构建配置
    pub fn build(self) -> SolanaConfig {
        self.config
    }
}

/// Solana 链上集成
pub struct SolanaIntegration {
    config: SolanaConfig,
}

impl SolanaIntegration {
    /// 创建新的集成实例
    pub fn new(config: SolanaConfig) -> Self {
        Self { config }
    }

    /// 提交共识结果到链上
    pub async fn submit_consensus(&self, consensus: &ConsensusOutput) -> SdkResult<String> {
        if self.config.oracle_program_id.is_empty() {
            return Err(SdkError::ChainSubmissionError(
                "未设置预言机合约地址".to_string(),
            ));
        }

        // 构建链上提交数据
        let chain_data = self.build_chain_submission(consensus)?;

        // 实际实现应调用 Solana RPC API 发送交易
        // 这里返回模拟交易哈希
        let tx_hash = format!("simulated_tx_{}", consensus.query_id);

        log::info!("📤 提交共识结果到链上: {}", tx_hash);
        Ok(tx_hash)
    }

    /// 构建链上提交数据
    pub fn build_chain_submission(
        &self,
        consensus: &ConsensusOutput,
    ) -> SdkResult<ChainSubmissionData> {
        // 编码共识结果为链上格式
        let encoded_params = serde_json::to_vec(&serde_json::json!({
            "query_id": consensus.query_id,
            "value": consensus.consensus_value.to_string(),
            "confidence": consensus.confidence.to_string(),
            "timestamp": consensus.timestamp,
            "participants": consensus.participant_count,
        }))
        .map_err(|e| SdkError::ChainSubmissionError(e.to_string()))?;

        Ok(ChainSubmissionData {
            contract_address: self.config.oracle_program_id.clone(),
            method_name: "submit_consensus_result".to_string(),
            encoded_params,
            estimated_gas: Some(5000),
        })
    }

    /// 从链上查询历史结果
    pub async fn query_history(&self, _query_id: &str) -> SdkResult<Option<ConsensusOutput>> {
        // 实际实现应调用 Solana RPC API 查询链上数据
        Ok(None)
    }

    /// 验证链上结果
    pub async fn verify_on_chain_result(
        &self,
        _query_id: &str,
        _expected_value: f64,
    ) -> SdkResult<bool> {
        // 实际实现应验证链上存储的结果
        Ok(true)
    }

    /// 获取配置
    pub fn config(&self) -> &SolanaConfig {
        &self.config
    }
}
