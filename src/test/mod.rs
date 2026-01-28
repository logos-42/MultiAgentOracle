//! 测试模块
//!
//! 本地分层架构测试的实现

use std::collections::HashMap;

pub mod config;
pub mod local_node_manager;
pub mod preconfigured_reputation;
pub mod simple_prompt_support;
pub mod visualization;

// 重新导出
pub use config::LocalTestConfig;
pub use local_node_manager::LocalTestNodeManager;
pub use preconfigured_reputation::PreconfiguredReputation;
pub use simple_prompt_support::SimplePromptSupport;
pub use visualization::visualize_test_results;

/// 测试结果类型
#[derive(Debug, Clone)]
pub struct TestResults {
    pub network_test: NetworkTestResult,
    pub consensus_test: ConsensusTestResult,
    pub diap_test: DiapTestResult,
    pub gateway_test: GatewayTestResult,
    pub prompt_test: PromptTestResult,
}

/// 网络测试结果
#[derive(Debug, Clone)]
pub struct NetworkTestResult {
    /// 连接成功率 (0.0-1.0)
    pub connection_success_rate: f64,
    /// 平均延迟（毫秒）
    pub average_latency_ms: f64,
    /// 各层级的连接统计信息
    pub tier_connection_stats: HashMap<String, TierConnectionStats>,
    /// 错误信息列表
    pub errors: Vec<String>,
}

/// 层级连接统计
#[derive(Debug, Clone)]
pub struct TierConnectionStats {
    /// 层级名称
    pub tier: String,
    /// 成功连接数
    pub successful_connections: u32,
    /// 失败连接数
    pub failed_connections: u32,
    /// 平均连接时间（毫秒）
    pub average_connection_time_ms: f64,
}

/// 共识测试结果
#[derive(Debug, Clone)]
pub struct ConsensusTestResult {
    /// 共识成功率 (0.0-1.0)
    pub consensus_success_rate: f64,
    /// 平均共识时间（毫秒）
    pub average_consensus_time_ms: f64,
    /// 各层级的共识统计信息
    pub tier_consensus_stats: HashMap<String, TierConsensusStats>,
    /// 权重影响分析
    pub weight_influence_analysis: WeightInfluenceAnalysis,
}

/// 层级共识统计
#[derive(Debug, Clone)]
pub struct TierConsensusStats {
    /// 层级名称
    pub tier: String,
    /// 参与率 (0.0-1.0)
    pub participation_rate: f64,
    /// 平均投票权重
    pub average_voting_weight: f64,
    /// 共识准确度 (0.0-1.0)
    pub consensus_accuracy: f64,
}

/// 权重影响分析
#[derive(Debug, Clone)]
pub struct WeightInfluenceAnalysis {
    /// 信誉权重相关性
    pub reputation_weight_correlation: f64,
    /// 质押权重相关性
    pub stake_weight_correlation: f64,
    /// 层级权重相关性
    pub tier_weight_correlation: f64,
}

/// DIAP测试结果
#[derive(Debug, Clone)]
pub struct DiapTestResult {
    /// 身份注册成功率 (0.0-1.0)
    pub identity_registration_success_rate: f64,
    /// 验证成功率 (0.0-1.0)
    pub verification_success_rate: f64,
    /// 平均注册时间（毫秒）
    pub average_registration_time_ms: f64,
    /// 平均验证时间（毫秒）
    pub average_verification_time_ms: f64,
    /// 各层级的认证统计信息
    pub tier_authentication_stats: HashMap<String, TierAuthStats>,
}

/// 层级认证统计
#[derive(Debug, Clone)]
pub struct TierAuthStats {
    /// 层级名称
    pub tier: String,
    /// 认证成功率 (0.0-1.0)
    pub auth_success_rate: f64,
    /// 平均认证时间（毫秒）
    pub average_auth_time_ms: f64,
    /// 跨层级认证成功率 (0.0-1.0)
    pub cross_tier_auth_success_rate: f64,
}

/// 网关测试结果
#[derive(Debug, Clone)]
pub struct GatewayTestResult {
    /// 网关负载分布统计
    pub gateway_load_distribution: HashMap<String, GatewayLoadStats>,
    /// 连接成功率 (0.0-1.0)
    pub connection_success_rate: f64,
    /// 平均响应时间（毫秒）
    pub average_response_time_ms: f64,
    /// 故障恢复成功率 (0.0-1.0)
    pub fault_recovery_success_rate: f64,
}

/// 网关负载统计
#[derive(Debug, Clone)]
pub struct GatewayLoadStats {
    /// 网关ID
    pub gateway_id: String,
    /// 网关类型
    pub gateway_type: String,
    /// 活跃连接数
    pub active_connections: u32,
    /// 总请求数
    pub total_requests: u64,
    /// 平均负载百分比
    pub average_load_percentage: f64,
    /// 错误率
    pub error_rate: f64,
}

/// Prompt测试结果
#[derive(Debug, Clone)]
pub struct PromptTestResult {
    /// Prompt成功率 (0.0-1.0)
    pub prompt_success_rate: f64,
    /// 平均响应时间（毫秒）
    pub average_response_time_ms: f64,
    /// 命令覆盖统计
    pub command_coverage: HashMap<String, CommandStats>,
    /// 各层级的响应统计
    pub tier_response_stats: HashMap<String, TierResponseStats>,
}

/// 命令统计
#[derive(Debug, Clone)]
pub struct CommandStats {
    /// 命令名称
    pub command: String,
    /// 成功次数
    pub success_count: u32,
    /// 失败次数
    pub failure_count: u32,
    /// 平均响应时间（毫秒）
    pub average_response_time_ms: f64,
}

/// 层级响应统计
#[derive(Debug, Clone)]
pub struct TierResponseStats {
    /// 层级名称
    pub tier: String,
    /// 响应成功率 (0.0-1.0)
    pub response_success_rate: f64,
    /// 平均响应质量 (0.0-1.0)
    pub average_response_quality: f64,
}
