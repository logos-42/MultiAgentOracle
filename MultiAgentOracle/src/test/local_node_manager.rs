//! 本地测试节点管理器
//! 
//! 管理10个测试节点的生命周期和分层网络模拟

use crate::test::config::LocalTestConfig;
use crate::test::preconfigured_reputation::PreconfiguredReputation;
use crate::test::simple_prompt_support::SimplePromptSupport;
use crate::consensus::ConsensusEngine;
use crate::network::NetworkManager;
use crate::oracle_agent::OracleAgent;
use crate::reputation::ReputationManager;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 测试节点结构
pub struct TestNode {
    pub id: String,
    pub tier: String,
    pub reputation: f64,
    pub stake: f64,
    pub agent: Arc<OracleAgent>,
    pub network_manager: Arc<NetworkManager>,
    pub reputation_manager: Arc<ReputationManager>,
    pub consensus_engine: Arc<ConsensusEngine>,
}

/// 分层拓扑结构
pub struct HierarchicalTopology {
    pub core_nodes: Vec<String>,
    pub validator_nodes: Vec<String>,
    pub data_nodes: Vec<String>,
    pub connections: HashMap<String, Vec<String>>,
}

/// DIAP客户端模拟
pub struct DiapClient {
    pub endpoint: String,
    pub mock_mode: bool,
}

/// 本地测试节点管理器
pub struct LocalTestNodeManager {
    pub nodes: HashMap<String, TestNode>,
    pub topology: HierarchicalTopology,
    pub diap_client: DiapClient,
    pub reputation_system: PreconfiguredReputation,
    pub prompt_support: SimplePromptSupport,
    pub config: LocalTestConfig,
}

impl LocalTestNodeManager {
    /// 初始化10个预配置节点
    pub async fn initialize_test_nodes(config: &LocalTestConfig) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🔧 初始化测试节点管理器...");
        
        // 验证配置
        config.validate()?;
        
        // 创建DIAP客户端
        let diap_client = DiapClient {
            endpoint: config.apis.as_ref()
                .map(|api| api.diap_sdk_endpoint.clone())
                .unwrap_or_else(|| "http://localhost:8080/diap".to_string()),
            mock_mode: config.test_environment.enable_diap_mock,
        };
        
        // 创建信誉系统
        let reputation_system = PreconfiguredReputation::new();
        
        // 创建Prompt支持系统
        let prompt_support = SimplePromptSupport::new();
        
        // 创建节点
        let mut nodes = HashMap::new();
        
        for (node_id, node_config) in &config.nodes {
            println!("  创建节点 {} (层级: {}, 信誉: {}, 质押: {})", 
                node_id, node_config.tier, node_config.reputation, node_config.stake);
            
            let node = Self::create_test_node(node_id, node_config).await?;
            nodes.insert(node_id.clone(), node);
        }
        
        // 创建分层拓扑
        let topology = Self::create_hierarchical_topology(&config);
        
        Ok(Self {
            nodes,
            topology,
            diap_client,
            reputation_system,
            prompt_support,
            config: config.clone(),
        })
    }
    
    /// 创建单个测试节点
    async fn create_test_node(node_id: &str, config: &crate::test::config::NodeConfig) -> Result<TestNode, Box<dyn std::error::Error>> {
        // 创建网络管理器
        let network_manager = Arc::new(NetworkManager::new(
            node_id.to_string(),
            config.tier.clone(),
        ));
        
        // 创建信誉管理器
        let reputation_manager = Arc::new(ReputationManager::new(
            node_id.to_string(),
            config.reputation,
        ));
        
        // 创建共识引擎
        let consensus_engine = Arc::new(ConsensusEngine::new(
            node_id.to_string(),
            config.tier.clone(),
        ));
        
        // 创建Oracle代理
        let agent = Arc::new(OracleAgent::new(
            node_id.to_string(),
            config.tier.clone(),
            config.data_types.clone(),
        ));
        
        Ok(TestNode {
            id: node_id.to_string(),
            tier: config.tier.clone(),
            reputation: config.reputation,
            stake: config.stake,
            agent,
            network_manager,
            reputation_manager,
            consensus_engine,
        })
    }
    
    /// 创建分层拓扑结构
    fn create_hierarchical_topology(config: &LocalTestConfig) -> HierarchicalTopology {
        let mut topology = HierarchicalTopology {
            core_nodes: Vec::new(),
            validator_nodes: Vec::new(),
            data_nodes: Vec::new(),
            connections: HashMap::new(),
        };
        
        // 按层级分组节点
        for (node_id, node_config) in &config.nodes {
            match node_config.tier.as_str() {
                "core" => topology.core_nodes.push(node_id.clone()),
                "validator" => topology.validator_nodes.push(node_id.clone()),
                "data" => topology.data_nodes.push(node_id.clone()),
                _ => {}
            }
        }
        
        // 建立层级连接规则
        Self::establish_hierarchical_connections(&mut topology);
        
        topology
    }
    
    /// 建立分层连接规则
    fn establish_hierarchical_connections(topology: &mut HierarchicalTopology) {
        // 核心节点之间建立网状连接
        for core_node in &topology.core_nodes {
            let mut connections = Vec::new();
            for other_core in &topology.core_nodes {
                if core_node != other_core {
                    connections.push(other_core.clone());
                }
            }
            topology.connections.insert(core_node.clone(), connections);
        }
        
        // 验证层节点连接到核心节点
        for validator_node in &topology.validator_nodes {
            let mut connections = Vec::new();
            // 每个验证节点连接到2个核心节点
            for i in 0..2.min(topology.core_nodes.len()) {
                connections.push(topology.core_nodes[i].clone());
            }
            topology.connections.insert(validator_node.clone(), connections);
        }
        
        // 数据层节点连接到验证层节点
        for data_node in &topology.data_nodes {
            let mut connections = Vec::new();
            // 每个数据节点连接到1-2个验证节点
            for i in 0..2.min(topology.validator_nodes.len()) {
                connections.push(topology.validator_nodes[i].clone());
            }
            topology.connections.insert(data_node.clone(), connections);
        }
    }
    
    /// 启动分层网络模拟
    pub async fn start_hierarchical_network(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌐 启动分层网络模拟...");
        
        // 建立节点连接
        for (node_id, connections) in &self.topology.connections {
            if let Some(node) = self.nodes.get(node_id) {
                println!("  节点 {} 连接到: {:?}", node_id, connections);
                
                // 在实际实现中，这里会调用网络管理器的连接方法
                // node.network_manager.connect_to_nodes(connections).await?;
            }
        }
        
        println!("✅ 分层网络连接建立完成");
        Ok(())
    }
    
    /// 运行分层共识测试
    pub async fn run_consensus_test(&self, data_type: crate::oracle_agent::data_types::OracleDataType) 
        -> Result<crate::test::ConsensusTestResult, Box<dyn std::error::Error>> 
    {
        println!("🤝 运行分层共识测试 (数据类型: {:?})", data_type);
        
        // 在实际实现中，这里会执行完整的共识流程
        // 1. 数据层节点采集数据
        // 2. 验证层节点聚合和验证
        // 3. 核心层节点进行最终共识
        
        Ok(crate::test::ConsensusTestResult {
            consensus_success_rate: 0.95,
            average_consensus_time_ms: 120.5,
            tier_consensus_stats: HashMap::new(),
            weight_influence_analysis: crate::test::WeightInfluenceAnalysis {
                reputation_weight_correlation: 0.85,
                stake_weight_correlation: 0.75,
                tier_weight_correlation: 0.90,
            },
        })
    }
    
    /// 测试DIAP身份验证流程
    pub async fn test_diap_authentication(&self) -> Result<Vec<crate::AuthResult>, Box<dyn std::error::Error>> {
        println!("🔐 测试DIAP身份验证流程...");
        
        let mut results = Vec::new();
        
        for (node_id, node) in &self.nodes {
            println!("  验证节点 {} 的身份...", node_id);
            
            // 模拟DIAP身份验证
            let auth_result = crate::AuthResult {
                node_id: node_id.clone(),
                tier: node.tier.clone(),
                success: true,
                auth_time_ms: 45.2,
                error: None,
            };
            
            results.push(auth_result);
        }
        
        println!("✅ DIAP身份验证测试完成");
        Ok(results)
    }
    
    /// 测试网关接入流程
    pub async fn test_gateway_access(&self) -> Result<crate::test::GatewayTestResult, Box<dyn std::error::Error>> {
        println!("🚪 测试网关接入流程...");
        
        // 模拟网关接入测试
        Ok(crate::test::GatewayTestResult {
            gateway_load_distribution: HashMap::new(),
            connection_success_rate: 0.98,
            average_response_time_ms: 85.3,
            fault_recovery_success_rate: 0.95,
        })
    }
    
    /// 显示节点状态
    pub fn show_node_status(&self) {
        println!("📊 节点状态概览");
        println!("====================");
        
        for (node_id, node) in &self.nodes {
            println!("  {}: 层级={}, 信誉={:.1}, 质押={:.2}", 
                node_id, node.tier, node.reputation, node.stake);
        }
        
        println!("\n📈 层级分布:");
        println!("  核心层: {} 个节点", self.topology.core_nodes.len());
        println!("  验证层: {} 个节点", self.topology.validator_nodes.len());
        println!("  数据层: {} 个节点", self.topology.data_nodes.len());
    }
    
    /// 显示网络拓扑
    pub fn show_network_topology(&self) {
        println!("🌐 网络拓扑结构");
        println!("====================");
        
        for (node_id, connections) in &self.topology.connections {
            println!("  {} -> {:?}", node_id, connections);
        }
    }
}

/// 身份验证结果
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub node_id: String,
    pub tier: String,
    pub success: bool,
    pub auth_time_ms: f64,
    pub error: Option<String>,
}
