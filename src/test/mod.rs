//! 测试模块
//! 
//! 本地分层架构测试的实现

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
    pub connection_success_rate: f64,
    pub average_latency_ms: f64,
    pub tier_connection_stats: HashMap<String, TierConnectionStats>,
    pub errors: Vec<String>,
}

/// 层级连接统计
#[derive(Debug, Clone)]
pub struct TierConnectionStats {
    pub tier: String,
    pub successful_connections: u32,
    pub failed_connections: u32,
    pub average_connection_time_ms: f64,
}

/// 共识测试结果
#[derive(Debug, Clone)]
pub struct ConsensusTestResult {
    pub consensus_success_rate: f64,
    pub average_consensus_time_ms: f64,
    pub tier_consensus_stats: HashMap<String, TierConsensusStats>,
    pub weight_influence_analysis: WeightInfluenceAnalysis,
}

/// 层级共识统计
#[derive(Debug, Clone)]
pub struct TierConsensusStats {
    pub tier: String,
    pub participation_rate: f64,
    pub average_voting_weight: f64,
    pub consensus_accuracy: f64,
}

/// 权重影响分析
#[derive(Debug, Clone)]
pub struct WeightInfluenceAnalysis {
    pub reputation_weight_correlation: f64,
    pub stake_weight_correlation: f64,
    pub tier_weight_correlation: f64,
}

/// DIAP测试结果
#[derive(Debug, Clone)]
pub struct DiapTestResult {
    pub identity_registration_success_rate: f64,
    pub verification_success_rate: f64,
    pub average_registration_time_ms: f64,
    pub average_verification_time_ms: f64,
    pub tier_authentication_stats: HashMap<String, TierAuthStats>,
}

/// 层级认证统计
#[derive(Debug, Clone)]
pub struct TierAuthStats {
    pub tier: String,
    pub auth_success_rate: f64,
    pub average_auth_time_ms: f64,
    pub cross_tier_auth_success_rate: f64,
}

/// 网关测试结果
#[derive(Debug, Clone)]
pub struct GatewayTestResult {
    pub gateway_load_distribution: HashMap<String, GatewayLoadStats>,
    pub connection_success_rate: f64,
    pub average_response_time_ms: f64,
    pub fault_recovery_success_rate: f64,
}

/// 网关负载统计
#[derive(Debug, Clone)]
pub struct GatewayLoadStats {
    pub gateway_id: String,
    pub gateway_type: String,
    pub active_connections: u32,
    pub total_requests: u64,
    pub average_load_percentage: f64,
    pub error_rate: f64,
}

/// Prompt测试结果
#[derive(Debug, Clone)]
pub struct PromptTestResult {
    pub prompt_success_rate: f64,
    pub average_response_time_ms: f64,
    pub command_coverage: HashMap<String, CommandStats>,
    pub tier_response_stats: HashMap<String, TierResponseStats>,
}

/// 命令统计
#[derive(Debug, Clone)]
pub struct CommandStats {
    pub command: String,
    pub success_count: u32,
    pub failure_count: u32,
    pub average_response_time_ms: f64,
}

/// 层级响应统计
#[derive(Debug, Clone)]
pub struct TierResponseStats {
    pub tier: String,
    pub response_success_rate: f64,
    pub average_response_quality: f64,
}

use std::collections::HashMap;
