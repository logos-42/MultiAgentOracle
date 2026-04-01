//! SDK 专用类型定义
//!
//! 提供面向智能合约开发者的高级数据类型。

use serde::{Deserialize, Serialize};
use std::fmt;

/// SDK 结果类型别名
pub type SdkResult<T> = Result<T, SdkError>;

/// SDK 错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SdkError {
    /// 配置错误
    ConfigError(String),
    /// 查询错误
    QueryError(String),
    /// 共识错误
    ConsensusError(String),
    /// 链上提交错误
    ChainSubmissionError(String),
    /// 超时错误
    TimeoutError(String),
    /// 内部错误
    InternalError(String),
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdkError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            SdkError::QueryError(msg) => write!(f, "查询错误: {}", msg),
            SdkError::ConsensusError(msg) => write!(f, "共识错误: {}", msg),
            SdkError::ChainSubmissionError(msg) => write!(f, "链上提交错误: {}", msg),
            SdkError::TimeoutError(msg) => write!(f, "超时错误: {}", msg),
            SdkError::InternalError(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl std::error::Error for SdkError {}

impl From<anyhow::Error> for SdkError {
    fn from(err: anyhow::Error) -> Self {
        SdkError::InternalError(err.to_string())
    }
}

/// 预言机查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleQuery {
    /// 查询 ID（唯一标识）
    pub query_id: String,
    /// 查询内容
    pub query: String,
    /// 数据类型（如 BTC 价格、天气数据等）
    pub data_type: String,
    /// 额外参数
    pub params: Option<serde_json::Value>,
    /// 超时时间（秒）
    pub timeout_secs: Option<u64>,
    /// 所需 Agent 数量
    pub required_agents: Option<usize>,
}

impl OracleQuery {
    /// 创建新的查询
    pub fn new(query_id: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            query_id: query_id.into(),
            query: query.into(),
            data_type: String::new(),
            params: None,
            timeout_secs: None,
            required_agents: None,
        }
    }

    /// 设置数据类型
    pub fn with_data_type(mut self, data_type: impl Into<String>) -> Self {
        self.data_type = data_type.into();
        self
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    /// 设置所需 Agent 数量
    pub fn with_required_agents(mut self, count: usize) -> Self {
        self.required_agents = Some(count);
        self
    }

    /// 设置额外参数
    pub fn with_params(mut self, params: serde_json::Value) -> Self {
        self.params = Some(params);
        self
    }
}

/// Agent 提交数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSubmission {
    /// Agent ID
    pub agent_id: String,
    /// 预测值
    pub value: f64,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 因果指纹
    pub causal_fingerprint: Vec<f64>,
    /// 时间戳
    pub timestamp: u64,
    /// 签名
    pub signature: Option<String>,
}

/// 预言机响应（单个 Agent 的响应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    /// 查询 ID
    pub query_id: String,
    /// Agent 提交
    pub submission: AgentSubmission,
    /// 元数据
    pub metadata: Option<serde_json::Value>,
}

/// 共识输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusOutput {
    /// 查询 ID
    pub query_id: String,
    /// 共识值
    pub consensus_value: f64,
    /// 共识置信度
    pub confidence: f64,
    /// 参与 Agent 数量
    pub participant_count: usize,
    /// 谱分析特征
    pub spectral_features: Option<Vec<f64>>,
    /// 异常检测结果
    pub anomaly_detected: bool,
    /// 时间戳
    pub timestamp: u64,
    /// 链上提交数据
    pub chain_data: Option<ChainSubmissionData>,
}

/// 链上提交数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSubmissionData {
    /// 合约地址
    pub contract_address: String,
    /// 方法名
    pub method_name: String,
    /// 编码后的参数
    pub encoded_params: Vec<u8>,
    /// 预估 Gas 费用
    pub estimated_gas: Option<u64>,
}

/// 完整查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResult {
    /// 查询 ID
    pub query_id: String,
    /// 所有 Agent 响应
    pub responses: Vec<OracleResponse>,
    /// 共识输出
    pub consensus: Option<ConsensusOutput>,
    /// 查询时间戳
    pub timestamp: u64,
}

impl OracleResult {
    /// 获取共识结果
    pub fn consensus_output(&self) -> SdkResult<&ConsensusOutput> {
        self.consensus
            .as_ref()
            .ok_or_else(|| SdkError::ConsensusError("共识结果不存在".to_string()))
    }

    /// 获取共识值
    pub fn consensus_value(&self) -> SdkResult<f64> {
        self.consensus_output().map(|c| c.consensus_value)
    }

    /// 检查是否有异常
    pub fn has_anomaly(&self) -> bool {
        self.consensus
            .as_ref()
            .map(|c| c.anomaly_detected)
            .unwrap_or(false)
    }
}
