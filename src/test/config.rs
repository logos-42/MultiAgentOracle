//! 本地测试配置模块
//! 
//! 定义本地分层架构测试的配置结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 测试环境配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironmentConfig {
    pub name: String,
    pub node_count: u32,
    pub simulate_network_latency: bool,
    pub enable_diap_mock: bool,
    pub log_level: String,
    pub data_dir: String,
}

/// 网络模拟配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSimulationConfig {
    pub latency_min_ms: u32,
    pub latency_max_ms: u32,
    pub packet_loss_rate: f64,
    pub bandwidth_limit_mbps: u32,
}

/// 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub tier: String,
    pub reputation: f64,
    pub stake: f64,
    pub address: String,
    pub data_types: Vec<String>,
}

/// 网关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub address: String,
    pub max_connections: u32,
    pub supported_tiers: Vec<String>,
    pub optimized_for_mobile: Option<bool>,
}

/// 网关集合配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewaysConfig {
    pub light_gateway_count: u32,
    pub mobile_gateway_count: u32,
    pub enterprise_gateway_count: u32,
    pub regional_gateway_count: u32,
    pub enable_gateway_simulation: bool,
    
    #[serde(default)]
    pub light_gateway1: Option<GatewayConfig>,
    
    #[serde(default)]
    pub light_gateway2: Option<GatewayConfig>,
    
    #[serde(default)]
    pub mobile_gateway1: Option<GatewayConfig>,
}

/// API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub diap_sdk_endpoint: String,
    pub data_api_mock: bool,
    pub prompt_support: bool,
    
    #[serde(default)]
    pub mock_data: Option<MockDataConfig>,
}

/// 模拟数据配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockDataConfig {
    pub crypto_price_range: [f64; 2],
    pub stock_price_range: [f64; 2],
    pub weather_temp_range: [f64; 2],
}

/// 层级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyConfig {
    pub levels: u32,
    pub promotion_thresholds: Vec<f64>,
    pub demotion_thresholds: Vec<f64>,
    
    #[serde(default)]
    pub tiers: HashMap<String, TierConfig>,
}

/// 层级详细配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    pub min_reputation: f64,
    pub max_reputation: f64,
    pub voting_weight_multiplier: f64,
    pub max_connections: u32,
    pub required_stake: f64,
}

/// 共识配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub min_quorum_ratio: f64,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub confirmation_rounds: u32,
    
    #[serde(default)]
    pub weight_calculation: Option<WeightCalculationConfig>,
}

/// 权重计算配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightCalculationConfig {
    pub base_weight: f64,
    pub reputation_weight_factor: f64,
    pub stake_weight_factor: f64,
    pub tier_weight_multipliers: HashMap<String, f64>,
}

/// 测试场景配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenariosConfig {
    pub enable_network_test: bool,
    pub enable_consensus_test: bool,
    pub enable_diap_test: bool,
    pub enable_gateway_test: bool,
    pub enable_prompt_test: bool,
    
    #[serde(default)]
    pub network_test: Option<NetworkTestConfig>,
    
    #[serde(default)]
    pub consensus_test: Option<ConsensusTestConfig>,
    
    #[serde(default)]
    pub diap_test: Option<DiapTestConfig>,
}

/// 网络测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTestConfig {
    pub test_duration_secs: u64,
    pub connection_attempts: u32,
    pub expected_success_rate: f64,
}

/// 共识测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusTestConfig {
    pub test_iterations: u32,
    pub data_types: Vec<String>,
    pub expected_consensus_rate: f64,
}

/// DIAP测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiapTestConfig {
    pub identity_registration_attempts: u32,
    pub verification_attempts: u32,
    pub expected_success_rate: f64,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub console_level: String,
    pub file_level: String,
    pub log_file: String,
    pub max_file_size_mb: u32,
    pub max_backup_files: u32,
}

/// 完整的本地测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTestConfig {
    pub test_environment: TestEnvironmentConfig,
    
    #[serde(default)]
    pub network_simulation: Option<NetworkSimulationConfig>,
    
    pub nodes: HashMap<String, NodeConfig>,
    
    #[serde(default)]
    pub gateways: Option<GatewaysConfig>,
    
    #[serde(default)]
    pub apis: Option<ApiConfig>,
    
    #[serde(default)]
    pub hierarchy: Option<HierarchyConfig>,
    
    #[serde(default)]
    pub consensus: Option<ConsensusConfig>,
    
    #[serde(default)]
    pub test_scenarios: Option<TestScenariosConfig>,
    
    #[serde(default)]
    pub logging: Option<LoggingConfig>,
}

impl LocalTestConfig {
    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// 获取节点ID列表
    pub fn get_node_ids(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }
    
    /// 获取指定层级的节点
    pub fn get_nodes_by_tier(&self, tier: &str) -> Vec<(String, NodeConfig)> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.tier == tier)
            .map(|(id, node)| (id.clone(), node.clone()))
            .collect()
    }
    
    /// 获取核心层节点
    pub fn get_core_nodes(&self) -> Vec<(String, NodeConfig)> {
        self.get_nodes_by_tier("core")
    }
    
    /// 获取验证层节点
    pub fn get_validator_nodes(&self) -> Vec<(String, NodeConfig)> {
        self.get_nodes_by_tier("validator")
    }
    
    /// 获取数据层节点
    pub fn get_data_nodes(&self) -> Vec<(String, NodeConfig)> {
        self.get_nodes_by_tier("data")
    }
    
    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // 验证节点数量
        if self.nodes.len() != self.test_environment.node_count as usize {
            errors.push(format!(
                "配置的节点数量({})与声明的节点数量({})不匹配",
                self.nodes.len(),
                self.test_environment.node_count
            ));
        }
        
        // 验证层级配置
        if let Some(hierarchy) = &self.hierarchy {
            if hierarchy.levels != 3 {
                errors.push("层级数量必须为3".to_string());
            }
            
            if hierarchy.promotion_thresholds.len() != 2 {
                errors.push("升级阈值数量必须为2".to_string());
            }
            
            if hierarchy.demotion_thresholds.len() != 2 {
                errors.push("降级阈值数量必须为2".to_string());
            }
        }
        
        // 验证节点信誉分在合理范围内
        for (node_id, node) in &self.nodes {
            if node.reputation < 0.0 || node.reputation > 1000.0 {
                errors.push(format!(
                    "节点{}的信誉分{}超出范围[0, 1000]",
                    node_id, node.reputation
                ));
            }
            
            if node.stake < 0.0 {
                errors.push(format!("节点{}的质押金额不能为负数", node_id));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
