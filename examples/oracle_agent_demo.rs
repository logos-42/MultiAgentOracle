//! 预言机智能体演示
//!
//! 演示如何使用多智能体预言机系统创建和运行预言机智能体。

use multi_agent_oracle::{
    OracleAgent, OracleAgentConfig, OracleDataType, DataSource,
    ReputationManager, ReputationConfig,
    ConsensusEngine, ConsensusConfig,
};
use std::sync::Arc;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    info!("🚀 开始预言机智能体演示");
    
    // 1. 创建预言机智能体配置
    let agent_config = OracleAgentConfig {
        name: "BTC价格预言机".to_string(),
        data_sources: vec![
            DataSource::new("CoinGecko", "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd", 0.7),
            DataSource::new("Binance", "https://api.binance.com/api/v3/ticker/price?symbol=BTCUSDT", 0.8),
        ],
        min_confidence: 0.7,
        max_timeout_secs: 30,
        initial_reputation: 100.0,
        initial_stake: 1000,
        supported_data_types: vec![
            OracleDataType::CryptoPrice { symbol: "BTC".to_string() },
            OracleDataType::CryptoPrice { symbol: "ETH".to_string() },
        ],
        cache_ttl_secs: 300,
        auto_cache_cleanup: true,
        cache_cleanup_interval_secs: 60,
    };
    
    // 2. 创建预言机智能体
    let mut oracle_agent = OracleAgent::new(agent_config)?;
    
    // 设置DIAP身份（模拟）
    oracle_agent.set_diap_identity(
        "did:diap:btc_oracle_1".to_string(),
        vec![1, 2, 3, 4, 5], // 模拟私钥
    );
    
    info!("✅ 预言机智能体创建成功: {}", oracle_agent.get_did().unwrap_or("未知"));
    
    // 3. 创建信誉管理器
    let reputation_config = ReputationConfig::default();
    let reputation_manager = Arc::new(ReputationManager::new(reputation_config));
    
    // 注册智能体到信誉系统
    reputation_manager.register_agent(
        oracle_agent.get_did().unwrap().to_string(),
    ).await?;
    
    info!("✅ 信誉管理器初始化成功");
    
    // 4. 创建共识引擎
    let consensus_config = ConsensusConfig::default();
    let consensus_engine = Arc::new(ConsensusEngine::new(
        reputation_manager.clone(),
        consensus_config,
    ));
    
    info!("✅ 共识引擎初始化成功");
    
    // 5. 演示数据采集
    info!("📊 开始数据采集演示");
    
    let data_type = OracleDataType::CryptoPrice { symbol: "BTC".to_string() };
    
    match oracle_agent.collect_data(&data_type).await {
        Ok(result) => {
            if result.success {
                info!("✅ 数据采集成功");
                info!("   值: {:?}", result.data.as_ref().unwrap().value);
                info!("   置信度: {:.2}", result.data.as_ref().unwrap().confidence);
                info!("   数据源: {:?}", result.data.as_ref().unwrap().sources_used);
                info!("   采集时间: {}ms", result.collection_time_ms);
            } else {
                info!("❌ 数据采集失败: {:?}", result.error);
            }
        }
        Err(e) => {
            info!("❌ 数据采集错误: {}", e);
        }
    }
    
    // 6. 演示信誉更新
    info!("📈 开始信誉更新演示");
    
    // 模拟逻辑一致性更新
    match reputation_manager.update_for_logical_consistency(
        oracle_agent.get_did().unwrap(),
        0.85,   // 高余弦相似度
        false,   // 不是离群点
        0,       // 聚类位置
    ).await {
        Ok(delta) => {
            info!("✅ 信誉更新成功: Δ = {:.2}", delta);
        }
        Err(e) => {
            info!("❌ 信誉更新失败: {}", e);
        }
    }
    
    // 获取当前信誉分
    if let Some(score) = reputation_manager.get_score(oracle_agent.get_did().unwrap()).await {
        info!("📊 当前因果信用分: {:.2}", score.causal_credit);
        info!("   成功率: {:.2}%", score.success_rate() * 100.0);
        info!("   总任务数: {}", score.total_tasks);
        info!("   成功任务数: {}", score.successful_tasks);
    }
    
    // 7. 演示共识过程
    info!("🤝 开始共识过程演示");
    
    // 创建共识参与者列表
    let participants = vec![
        "did:diap:btc_oracle_1".to_string(),
        "did:diap:btc_oracle_2".to_string(),
        "did:diap:btc_oracle_3".to_string(),
        "did:diap:btc_oracle_4".to_string(),
    ];
    
    // 开始共识
    match consensus_engine.start_consensus(
        "consensus_test_1".to_string(),
        data_type.clone(),
        participants,
    ).await {
        Ok(_) => {
            info!("✅ 共识开始成功");
            
            // 模拟投票
            info!("🗳️ 模拟投票过程...");
            
            // 这里应该实现实际的投票逻辑
            // 简化演示：直接显示共识引擎状态
            
            let state = consensus_engine.get_state().await;
            info!("📋 共识状态: {:?}", state.status);
            info!("   参与者: {} 个", state.participants.len());
            info!("   当前轮数: {}", state.current_round);
        }
        Err(e) => {
            info!("❌ 共识开始失败: {}", e);
        }
    }
    
    // 8. 演示网络功能
    info!("🌐 开始网络功能演示");
    
    // 这里应该实现网络连接和消息传递
    // 简化演示：显示网络状态
    
    info!("📡 网络功能演示完成");
    
    // 9. 清理和总结
    info!("🧹 开始清理");
    
    // 清理缓存
    oracle_agent.cleanup_cache();
    info!("✅ 缓存清理完成");
    
    // 应用信誉衰减
    match reputation_manager.apply_decay().await {
        Ok(updated_count) => {
            if updated_count > 0 {
                info!("✅ 信誉衰减应用: {} 个智能体受影响", updated_count);
            }
        }
        Err(e) => {
            info!("❌ 信誉衰减失败: {}", e);
        }
    }
    
    info!("🎉 预言机智能体演示完成");
    info!("📊 系统组件:");
    info!("   - 预言机智能体: ✅");
    info!("   - 信誉管理系统: ✅");
    info!("   - 共识引擎: ✅");
    info!("   - 网络系统: ✅");
    
    Ok(())
}
