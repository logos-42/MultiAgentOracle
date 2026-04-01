//! SDK 配置构建器
//!
//! 提供 Builder 模式的配置 API。

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// SDK 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    /// Solana RPC 端点
    pub solana_rpc_url: Option<String>,
    /// Agent 数量
    pub agent_count: usize,
    /// 共识阈值 (0.0-1.0)
    pub consensus_threshold: f64,
    /// 查询超时
    pub query_timeout: Duration,
    /// 是否启用因果指纹验证
    pub enable_causal_fingerprint: bool,
    /// 是否启用谱分析
    pub enable_spectral_analysis: bool,
    /// 最小 Agent 数量
    pub min_agents: usize,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            solana_rpc_url: None,
            agent_count: 5,
            consensus_threshold: 0.67,
            query_timeout: Duration::from_secs(30),
            enable_causal_fingerprint: true,
            enable_spectral_analysis: true,
            min_agents: 3,
            max_retries: 3,
        }
    }
}

impl SdkConfig {
    /// 创建配置构建器
    pub fn builder() -> SdkConfigBuilder {
        SdkConfigBuilder::default()
    }
}

/// SDK 配置构建器
#[derive(Debug, Clone)]
pub struct SdkConfigBuilder {
    config: SdkConfig,
}

impl Default for SdkConfigBuilder {
    fn default() -> Self {
        Self {
            config: SdkConfig::default(),
        }
    }
}

impl SdkConfigBuilder {
    /// 设置 Solana RPC 端点
    pub fn with_solana_rpc(mut self, url: impl Into<String>) -> Self {
        self.config.solana_rpc_url = Some(url.into());
        self
    }

    /// 设置 Agent 数量
    pub fn with_agent_count(mut self, count: usize) -> Self {
        self.config.agent_count = count;
        self
    }

    /// 设置共识阈值
    pub fn with_consensus_threshold(mut self, threshold: f64) -> Self {
        self.config.consensus_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// 设置查询超时
    pub fn with_query_timeout(mut self, timeout: Duration) -> Self {
        self.config.query_timeout = timeout;
        self
    }

    /// 启用/禁用因果指纹验证
    pub fn with_causal_fingerprint(mut self, enabled: bool) -> Self {
        self.config.enable_causal_fingerprint = enabled;
        self
    }

    /// 启用/禁用谱分析
    pub fn with_spectral_analysis(mut self, enabled: bool) -> Self {
        self.config.enable_spectral_analysis = enabled;
        self
    }

    /// 设置最小 Agent 数量
    pub fn with_min_agents(mut self, count: usize) -> Self {
        self.config.min_agents = count;
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// 构建配置
    pub fn build(self) -> SdkConfig {
        self.config
    }
}

/// 预言机构建器
#[derive(Debug)]
pub struct OracleBuilder {
    config: SdkConfig,
}

impl OracleBuilder {
    /// 创建新的构建器
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    /// 获取配置
    pub fn config(&self) -> &SdkConfig {
        &self.config
    }
}
