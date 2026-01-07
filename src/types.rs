//! 通用类型定义模块
//! 
//! 定义系统中使用的通用数据类型和结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 节点ID类型
pub type NodeId = String;

/// 交易哈希类型
pub type TransactionHash = String;

/// 数据哈希类型
pub type DataHash = String;

/// 时间戳类型（毫秒）
pub type Timestamp = u64;

/// 获取当前时间戳
pub fn current_timestamp() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// 节点ID
    pub id: NodeId,
    /// 节点地址
    pub address: String,
    /// 节点层级
    pub tier: String,
    /// 信誉分数
    pub reputation: f64,
    /// 质押金额
    pub stake: f64,
    /// 最后活跃时间
    pub last_active: Timestamp,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 网络消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// 心跳消息
    Heartbeat {
        node_id: NodeId,
        timestamp: Timestamp,
    },
    /// 数据提交消息
    DataSubmission {
        node_id: NodeId,
        data_type: String,
        data: serde_json::Value,
        signature: String,
    },
    /// 共识投票消息
    ConsensusVote {
        node_id: NodeId,
        proposal_id: String,
        vote: bool,
        weight: f64,
    },
    /// 层级变更消息
    TierChange {
        node_id: NodeId,
        old_tier: String,
        new_tier: String,
        reason: String,
    },
    /// 错误消息
    Error {
        code: u32,
        message: String,
        details: Option<serde_json::Value>,
    },
}

/// 共识结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// 提案ID
    pub proposal_id: String,
    /// 是否通过
    pub approved: bool,
    /// 同意票数
    pub yes_votes: u32,
    /// 反对票数
    pub no_votes: u32,
    /// 总票数
    pub total_votes: u32,
    /// 通过阈值
    pub threshold: f64,
    /// 实际通过率
    pub approval_rate: f64,
    /// 投票节点列表
    pub voters: Vec<NodeId>,
    /// 时间戳
    pub timestamp: Timestamp,
}

/// 数据验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 数据哈希
    pub data_hash: DataHash,
    /// 是否有效
    pub valid: bool,
    /// 验证者节点ID
    pub validator_id: NodeId,
    /// 验证时间
    pub validation_time: Timestamp,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 验证签名
    pub signature: String,
}

/// 网关信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayInfo {
    /// 网关ID
    pub id: String,
    /// 网关类型
    pub gateway_type: GatewayType,
    /// 网关地址
    pub address: String,
    /// 最大连接数
    pub max_connections: u32,
    /// 当前连接数
    pub current_connections: u32,
    /// 支持的层级
    pub supported_tiers: Vec<String>,
    /// 是否启用
    pub enabled: bool,
}

/// 网关类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatewayType {
    /// 轻节点网关
    Light,
    /// 移动网关
    Mobile,
    /// 企业网关
    Enterprise,
    /// 区域网关
    Regional,
}

/// 系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// 总节点数
    pub total_nodes: u32,
    /// 在线节点数
    pub online_nodes: u32,
    /// 各层级节点数
    pub tier_distribution: HashMap<String, u32>,
    /// 平均信誉分
    pub average_reputation: f64,
    /// 总质押金额
    pub total_stake: f64,
    /// 最近共识成功率
    pub recent_consensus_success_rate: f64,
    /// 系统启动时间
    pub startup_time: Timestamp,
    /// 当前时间
    pub current_time: Timestamp,
}

/// 错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemError {
    /// 错误代码
    pub code: u32,
    /// 错误消息
    pub message: String,
    /// 错误详情
    pub details: Option<serde_json::Value>,
    /// 发生时间
    pub timestamp: Timestamp,
    /// 相关节点ID
    pub node_id: Option<NodeId>,
}

/// 配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOptions {
    /// 网络配置
    pub network: NetworkConfig,
    /// 共识配置
    pub consensus: ConsensusConfig,
    /// 信誉配置
    pub reputation: ReputationConfig,
    /// 网关配置
    pub gateway: GatewayConfig,
    /// 日志配置
    pub logging: LoggingConfig,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 最大连接数
    pub max_connections: u32,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 连接超时（秒）
    pub connection_timeout: u64,
    /// 是否启用加密
    pub enable_encryption: bool,
}

/// 共识配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// 共识阈值
    pub consensus_threshold: f64,
    /// 投票超时（秒）
    pub voting_timeout: u64,
    /// 最小投票节点数
    pub min_voters: u32,
    /// 是否启用权重投票
    pub enable_weighted_voting: bool,
}

/// 信誉配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    /// 初始信誉分
    pub initial_reputation: f64,
    /// 最大信誉分
    pub max_reputation: f64,
    /// 信誉衰减率
    pub decay_rate: f64,
    /// 奖励乘数
    pub reward_multiplier: f64,
    /// 惩罚乘数
    pub penalty_multiplier: f64,
}

/// 网关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// 启用网关
    pub enabled: bool,
    /// 网关类型
    pub gateway_types: Vec<GatewayType>,
    /// 最大网关数
    pub max_gateways: u32,
    /// 负载均衡策略
    pub load_balancing_strategy: LoadBalancingStrategy,
}

/// 负载均衡策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// 轮询
    RoundRobin,
    /// 最少连接
    LeastConnections,
    /// 基于地理位置
    Geographic,
    /// 基于层级
    TierBased,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 日志文件路径
    pub file_path: Option<String>,
    /// 最大文件大小（MB）
    pub max_file_size_mb: u32,
    /// 是否启用控制台输出
    pub enable_console: bool,
}
