//! DIAP集成演示程序
//!
//! 展示DIAP SDK如何与多智能体预言机系统集成。

use multi_agent_oracle::diap::{DiapConfig, DiapIdentityManager, DiapNetworkAdapter};
use multi_agent_oracle::oracle_agent::{OracleAgent, OracleAgentConfig};
use multi_agent_oracle::consensus::{Vote, algorithms::DiapEnhancedBFT};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("=".repeat(60));
    println!("DIAP集成演示程序");
    println!("=".repeat(60));
    
    // 演示1: DIAP身份管理器
    println!("\n📋 演示1: DIAP身份管理器");
    println!("{}", "-".repeat(40));
    
    let mut diap_config = DiapConfig::default();
    diap_config.identity.name = "demo-oracle-agent".to_string();
    diap_config.identity.description = Some("演示用的预言机智能体".to_string());
    
    let identity_manager = DiapIdentityManager::new(diap_config.clone()).await?;
    let identity_manager_arc = Arc::new(identity_manager);
    
    // 注册身份
    let identity = identity_manager_arc.register_identity("demo-agent", Some("演示智能体")).await?;
    println!("✅ 身份注册成功:");
    println!("   ID: {}", identity.id);
    println!("   名称: {}", identity.name);
    println!("   公钥: {}...", &identity.public_key[..20]);
    println!("   状态: {:?}", identity.status);
    
    // 演示2: OracleAgent与DIAP集成
    println!("\n📋 演示2: OracleAgent与DIAP集成");
    println!("{}", "-".repeat(40));
    
    let agent_config = OracleAgentConfig {
        name: "demo-oracle".to_string(),
        supported_data_types: vec![],
        data_sources: vec![],
        reputation_score: 100.0,
        staked_amount: 1000,
        ..Default::default()
    };
    
    let mut agent = OracleAgent::new(agent_config)?;
    agent.init_diap_identity(Some(diap_config)).await?;
    
    let identity_status = agent.get_diap_identity_status().await;
    println!("✅ OracleAgent DIAP状态: {}", identity_status);
    
    // 演示3: DIAP增强的共识算法
    println!("\n📋 演示3: DIAP增强的共识算法");
    println!("{}", "-".repeat(40));
    
    let diap_bft = DiapEnhancedBFT::new(
        1, // 容错节点数
        5, // 总节点数
        Some(identity_manager_arc.clone()),
        false, // 不要求DIAP身份
    )?;
    
    // 创建模拟投票
    let votes = vec![
        Vote::new_with_diap_identity(
            "agent-1".to_string(),
            identity.id.clone(),
            identity.proof_hash.clone(),
            100.5,
            0.95,
            vec!["coingecko".to_string(), "binance".to_string()],
        ),
        Vote::new_with_diap_identity(
            "agent-2".to_string(),
            "another-identity".to_string(), // 不同的身份
            None,
            101.2,
            0.88,
            vec!["kraken".to_string()],
        ),
        Vote::new(
            "agent-3".to_string(), // 传统身份
            99.8,
            0.92,
            vec!["coinbase".to_string()],
        ),
    ];
    
    // 检查共识
    match diap_bft.check_consensus_with_diap(&votes).await? {
        Some(value) => println!("✅ 达成共识: {:.2}", value),
        None => println!("⚠️ 未达成共识"),
    }
    
    // 获取统计信息
    let stats = diap_bft.get_diap_statistics(&votes).await;
    println!("📊 DIAP共识统计:");
    println!("   {}", stats.summary());
    
    // 演示4: DIAP网络适配器
    println!("\n📋 演示4: DIAP网络适配器");
    println!("{}", "-".repeat(40));
    
    let mut network_config = DiapConfig::default();
    network_config.network.enable_p2p = true;
    network_config.network.p2p_type = multi_agent_oracle::diap::config::P2pType::Hybrid;
    
    let network_adapter = DiapNetworkAdapter::new(network_config, identity_manager_arc.clone()).await?;
    
    println!("✅ DIAP网络适配器创建成功");
    println!("   网络类型: Hybrid (libp2p + Iroh)");
    println!("   引导节点: {} 个", network_config.network.bootstrap_nodes.len());
    
    // 演示5: 完整的DIAP工作流程
    println!("\n📋 演示5: 完整的DIAP工作流程");
    println!("{}", "-".repeat(40));
    
    println!("1. 📝 身份注册: 智能体注册DIAP身份");
    println!("2. 🔐 身份验证: 使用零知识证明验证身份");
    println!("3. 📊 数据收集: 智能体收集预言机数据");
    println!("4. 🗳️ 共识投票: 使用DIAP身份参与共识");
    println!("5. 🌐 网络通信: 通过DIAP网络交换数据");
    println!("6. ✅ 结果验证: 验证共识结果和身份");
    
    println!("\n🎉 DIAP集成演示完成！");
    println!("=".repeat(60));
    println!("\n关键特性总结:");
    println!("• 去中心化身份: 每个智能体都有唯一的DIAP身份");
    println!("• 零知识证明: 保护隐私的同时验证身份");
    println!("• 增强安全性: DIAP身份提供更强的抗Sybil攻击能力");
    println!("• 权重增强: DIAP认证的投票有更高权重");
    println!("• 网络集成: 支持libp2p和Iroh网络");
    
    Ok(())
}
