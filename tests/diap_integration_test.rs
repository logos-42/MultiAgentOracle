//! DIAP集成测试
//!
//! 测试DIAP SDK与多智能体预言机系统的集成功能。

use multi_agent_oracle::diap::{
    DiapConfig, DiapIdentityManager, DiapNetworkAdapter, 
    AgentIdentity, IdentityStatus, DiapError
};
use multi_agent_oracle::oracle_agent::{OracleAgent, OracleAgentConfig};
use multi_agent_oracle::consensus::{Vote, algorithms::DiapEnhancedBFT};
use std::sync::Arc;
use tempfile::TempDir;

/// 测试DIAP身份管理器
#[tokio::test]
async fn test_diap_identity_manager() -> Result<(), Box<dyn std::error::Error>> {
    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let mut config = DiapConfig::default();
    
    // 修改存储路径到临时目录
    config.storage.identity_store_path = temp_dir.path().join("identities");
    config.storage.proof_store_path = temp_dir.path().join("proofs");
    config.storage.network_store_path = temp_dir.path().join("network");
    
    // 创建身份管理器
    let manager = DiapIdentityManager::new(config).await?;
    
    // 测试身份注册
    let identity = manager.register_identity("test-agent", Some("Test agent for integration testing")).await?;
    
    assert_eq!(identity.name, "test-agent");
    assert_eq!(identity.status, IdentityStatus::Registered);
    assert!(identity.public_key.len() > 0);
    
    // 测试身份验证
    let auth_result = manager.verify_identity(&identity.id, identity.proof_hash.as_deref()).await?;
    assert!(auth_result.authenticated);
    
    // 测试获取当前身份
    manager.set_current_identity(&identity.id).await?;
    let current_identity = manager.get_current_identity().await;
    assert!(current_identity.is_some());
    assert_eq!(current_identity.unwrap().id, identity.id);
    
    // 测试获取所有身份
    let all_identities = manager.get_all_identities().await;
    assert_eq!(all_identities.len(), 1);
    
    println!("✅ DIAP身份管理器测试通过");
    Ok(())
}

/// 测试OracleAgent与DIAP集成
#[tokio::test]
async fn test_oracle_agent_diap_integration() -> Result<(), Box<dyn std::error::Error>> {
    // 创建OracleAgent配置
    let agent_config = OracleAgentConfig {
        name: "test-oracle".to_string(),
        supported_data_types: vec![],
        data_sources: vec![],
        reputation_score: 100.0,
        staked_amount: 1000,
        ..Default::default()
    };
    
    // 创建OracleAgent
    let mut agent = OracleAgent::new(agent_config)?;
    
    // 初始化DIAP身份系统
    agent.init_diap_identity(None).await?;
    
    // 测试获取DIAP身份状态
    let status = agent.get_diap_identity_status().await;
    assert!(status.contains("已注册") || status.contains("已初始化"));
    
    // 测试获取当前DIAP身份
    let current_identity = agent.get_current_diap_identity().await;
    if let Some(identity) = current_identity {
        assert_eq!(identity.name, "oracle-agent-test-oracle");
    }
    
    // 测试数据签名
    let test_data = b"test data for signing";
    let signature = agent.sign_data_with_diap(test_data).await?;
    assert!(!signature.is_empty());
    
    println!("✅ OracleAgent DIAP集成测试通过");
    Ok(())
}

/// 测试DIAP增强的共识算法
#[tokio::test]
async fn test_diap_enhanced_consensus() -> Result<(), Box<dyn std::error::Error>> {
    // 创建临时目录和身份管理器
    let temp_dir = TempDir::new()?;
    let mut config = DiapConfig::default();
    config.storage.identity_store_path = temp_dir.path().join("identities");
    
    let identity_manager = Arc::new(DiapIdentityManager::new(config).await?);
    
    // 注册测试身份
    let identity1 = identity_manager.register_identity("consensus-agent-1", None).await?;
    let identity2 = identity_manager.register_identity("consensus-agent-2", None).await?;
    
    // 创建DIAP增强的BFT算法
    let diap_bft = DiapEnhancedBFT::new(1, 3, Some(identity_manager.clone()), false)?;
    
    // 创建测试投票
    let votes = vec![
        Vote::new_with_diap_identity(
            "agent-1".to_string(),
            identity1.id.clone(),
            identity1.proof_hash.clone(),
            100.0,
            0.9,
            vec!["source1".to_string()],
        ),
        Vote::new_with_diap_identity(
            "agent-2".to_string(),
            identity2.id.clone(),
            identity2.proof_hash.clone(),
            105.0,
            0.8,
            vec!["source2".to_string()],
        ),
        Vote::new(
            "agent-3".to_string(),
            110.0,
            0.7,
            vec!["source3".to_string()],
        ),
    ];
    
    // 测试共识检查
    let consensus_result = diap_bft.check_consensus_with_diap(&votes).await?;
    assert!(consensus_result.is_some());
    
    // 测试法定人数检查
    let has_quorum = diap_bft.check_quorum_with_diap(&votes).await?;
    assert!(has_quorum);
    
    // 测试统计信息
    let stats = diap_bft.get_diap_statistics(&votes).await;
    assert_eq!(stats.total_votes, 3);
    assert_eq!(stats.diap_votes, 2);
    assert_eq!(stats.non_diap_votes, 1);
    
    println!("✅ DIAP增强共识算法测试通过");
    Ok(())
}

/// 测试DIAP网络适配器
#[tokio::test]
async fn test_diap_network_adapter() -> Result<(), Box<dyn std::error::Error>> {
    // 创建临时目录和身份管理器
    let temp_dir = TempDir::new()?;
    let mut config = DiapConfig::default();
    config.storage.identity_store_path = temp_dir.path().join("identities");
    config.network.enable_p2p = false; // 在测试中禁用P2P网络
    
    let identity_manager = Arc::new(DiapIdentityManager::new(config.clone()).await?);
    
    // 创建网络适配器
    let adapter = DiapNetworkAdapter::new(config, identity_manager).await?;
    
    // 测试网络状态
    let status = adapter.check_network_status().await;
    assert!(!status.is_running); // 网络未启动
    
    // 启动网络（在测试模式下应该快速完成）
    adapter.start().await?;
    
    // 停止网络
    adapter.stop().await?;
    
    println!("✅ DIAP网络适配器测试通过");
    Ok(())
}

/// 测试完整的DIAP集成流程
#[tokio::test]
async fn test_complete_diap_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始完整DIAP集成流程测试");
    
    // 1. 测试身份管理器
    test_diap_identity_manager().await?;
    
    // 2. 测试OracleAgent集成
    test_oracle_agent_diap_integration().await?;
    
    // 3. 测试共识算法
    test_diap_enhanced_consensus().await?;
    
    // 4. 测试网络适配器
    test_diap_network_adapter().await?;
    
    println!("🎉 所有DIAP集成测试通过！");
    Ok(())
}

/// 主测试函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("=".repeat(60));
    println!("DIAP集成测试套件");
    println!("=".repeat(60));
    
    match test_complete_diap_integration().await {
        Ok(_) => {
            println!("=".repeat(60));
            println!("✅ 所有测试通过！");
            println!("=".repeat(60));
            Ok(())
        }
        Err(e) => {
            println!("=".repeat(60));
            println!("❌ 测试失败: {}", e);
            println!("=".repeat(60));
            Err(e)
        }
    }
}
