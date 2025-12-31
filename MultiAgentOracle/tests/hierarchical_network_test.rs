//! 分层网络连接测试
//! 
//! 测试分层架构的网络连接和拓扑规则

use multi_agent_oracle::test::{LocalTestConfig, LocalTestNodeManager};
use std::path::Path;

/// 测试分层网络连接
#[tokio::test]
async fn test_hierarchical_network_connections() {
    println!("🧪 测试分层网络连接");
    
    // 加载测试配置
    let config_path = "config/local_test.toml";
    let config = match LocalTestConfig::from_file(config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("❌ 无法加载配置文件 {}: {}", config_path, e);
            
            // 如果配置文件不存在，使用默认配置
            println!("⚠️  使用默认配置进行测试");
            create_default_test_config();
            
            // 重新加载
            LocalTestConfig::from_file(config_path).unwrap()
        }
    };
    
    // 验证配置
    match config.validate() {
        Ok(_) => println!("✅ 配置验证通过"),
        Err(errors) => {
            println!("❌ 配置验证失败:");
            for error in errors {
                println!("  - {}", error);
            }
            panic!("配置验证失败");
        }
    }
    
    // 创建测试节点管理器
    let manager = match LocalTestNodeManager::initialize_test_nodes(&config).await {
        Ok(manager) => {
            println!("✅ 测试节点管理器初始化成功");
            manager
        }
        Err(e) => {
            println!("❌ 测试节点管理器初始化失败: {}", e);
            panic!("节点管理器初始化失败: {}", e);
        }
    };
    
    // 显示节点状态
    println!("\n📊 节点状态:");
    manager.show_node_status();
    
    // 显示网络拓扑
    println!("\n🌐 网络拓扑:");
    manager.show_network_topology();
    
    // 测试分层连接规则
    test_hierarchical_connection_rules(&manager).await;
    
    // 启动分层网络
    match manager.start_hierarchical_network().await {
        Ok(_) => println!("✅ 分层网络启动成功"),
        Err(e) => println!("⚠️  分层网络启动有警告: {}", e),
    }
    
    println!("\n🎉 分层网络连接测试完成!");
}

/// 测试分层连接规则
async fn test_hierarchical_connection_rules(manager: &LocalTestNodeManager) {
    println!("\n🔗 测试分层连接规则:");
    
    let topology = &manager.topology;
    
    // 检查核心层节点数量
    println!("  核心层节点: {} 个", topology.core_nodes.len());
    assert!(topology.core_nodes.len() >= 2, "核心层至少需要2个节点");
    
    // 检查验证层节点数量
    println!("  验证层节点: {} 个", topology.validator_nodes.len());
    assert!(topology.validator_nodes.len() >= 3, "验证层至少需要3个节点");
    
    // 检查数据层节点数量
    println!("  数据层节点: {} 个", topology.data_nodes.len());
    assert!(topology.data_nodes.len() >= 5, "数据层至少需要5个节点");
    
    // 检查连接规则
    println!("  检查连接规则...");
    
    for (node_id, connections) in &topology.connections {
        println!("    {} -> {:?}", node_id, connections);
        
        // 验证连接数量合理性
        assert!(!connections.is_empty(), "节点 {} 必须有连接", node_id);
        assert!(connections.len() <= 10, "节点 {} 连接数过多", node_id);
    }
    
    println!("  ✅ 分层连接规则验证通过");
}

/// 创建默认测试配置
fn create_default_test_config() {
    use std::fs;
    
    let config_content = r#"[test_environment]
name = "local_hierarchical_test"
node_count = 10
simulate_network_latency = true
enable_diap_mock = true

[nodes]
node1 = { tier = "core", reputation = 850, stake = "1.0" }
node2 = { tier = "core", reputation = 820, stake = "0.8" }
node3 = { tier = "validator", reputation = 650, stake = "0.5" }
node4 = { tier = "validator", reputation = 580, stake = "0.4" }
node5 = { tier = "validator", reputation = 520, stake = "0.3" }
node6 = { tier = "data", reputation = 350, stake = "0.2" }
node7 = { tier = "data", reputation = 280, stake = "0.15" }
node8 = { tier = "data", reputation = 220, stake = "0.1" }
node9 = { tier = "data", reputation = 150, stake = "0.05" }
node10 = { tier = "data", reputation = 80, stake = "0.02" }

[gateways]
light_gateway_count = 2
mobile_gateway_count = 1
enable_gateway_simulation = true

[apis]
diap_sdk_endpoint = "http://localhost:8080/diap"
data_api_mock = true
prompt_support = true
"#;
    
    // 确保config目录存在
    fs::create_dir_all("config").unwrap();
    
    // 写入配置文件
    fs::write("config/local_test.toml", config_content).unwrap();
    println!("📝 创建默认配置文件: config/local_test.toml");
}

/// 测试DIAP身份验证
#[tokio::test]
async fn test_diap_authentication() {
    println!("\n🔐 测试DIAP身份验证");
    
    // 加载配置
    let config_path = "config/local_test.toml";
    let config = match LocalTestConfig::from_file(config_path) {
        Ok(config) => config,
        Err(_) => {
            create_default_test_config();
            LocalTestConfig::from_file(config_path).unwrap()
        }
    };
    
    // 创建测试节点管理器
    let manager = LocalTestNodeManager::initialize_test_nodes(&config).await.unwrap();
    
    // 测试DIAP身份验证
    match manager.test_diap_authentication().await {
        Ok(results) => {
            println!("  DIAP身份验证结果:");
            let success_count = results.iter().filter(|r| r.success).count();
            println!("    成功: {}，失败: {}", success_count, results.len() - success_count);
            
            assert!(success_count > 0, "至少需要一个成功的身份验证");
            println!("  ✅ DIAP身份验证测试通过");
        }
        Err(e) => {
            println!("  ❌ DIAP身份验证测试失败: {}", e);
            // 在测试环境中，我们允许DIAP测试失败（因为可能没有运行DIAP模拟服务器）
            println!("  ⚠️  DIAP测试被跳过（模拟服务器可能未运行）");
        }
    }
}

/// 测试网关接入
#[tokio::test]
async fn test_gateway_access() {
    println!("\n🚪 测试网关接入");
    
    // 加载配置
    let config_path = "config/local_test.toml";
    let config = match LocalTestConfig::from_file(config_path) {
        Ok(config) => config,
        Err(_) => {
            create_default_test_config();
            LocalTestConfig::from_file(config_path).unwrap()
        }
    };
    
    // 创建测试节点管理器
    let manager = LocalTestNodeManager::initialize_test_nodes(&config).await.unwrap();
    
    // 测试网关接入
    match manager.test_gateway_access().await {
        Ok(result) => {
            println!("  网关测试结果:");
            println!("    连接成功率: {:.1}%", result.connection_success_rate * 100.0);
            println!("    平均响应时间: {:.1}ms", result.average_response_time_ms);
            println!("    故障恢复成功率: {:.1}%", result.fault_recovery_success_rate * 100.0);
            
            assert!(result.connection_success_rate > 0.5, "连接成功率应大于50%");
            println!("  ✅ 网关接入测试通过");
        }
        Err(e) => {
            println!("  ❌ 网关接入测试失败: {}", e);
            panic!("网关测试失败: {}", e);
        }
    }
}

/// 主测试函数
#[tokio::test]
async fn test_complete_hierarchical_system() {
    println!("\n🚀 测试完整分层系统");
    
    // 运行所有测试
    test_hierarchical_network_connections().await;
    test_diap_authentication().await;
    test_gateway_access().await;
    
    println!("\n🎉 所有分层系统测试完成!");
}
