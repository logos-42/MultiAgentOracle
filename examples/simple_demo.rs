//! 简单演示
//!
//! 演示多智能体预言机系统的核心功能，不依赖P2P网络。

use multi_agent_oracle::{
    OracleAgent, OracleAgentConfig, OracleDataType, DataSource,
    ReputationManager, ReputationConfig,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use log::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    info!("🚀 多智能体预言机系统简单演示");
    info!("==========================================");
    
    // 演示1: 创建预言机智能体
    info!("🧪 演示1: 创建预言机智能体");
    demo_oracle_agent().await?;
    
    // 演示2: 测试信誉系统
    info!("🧪 演示2: 测试信誉系统");
    demo_reputation_system().await?;
    
    // 演示3: 测试数据采集
    info!("🧪 演示3: 测试数据采集");
    demo_data_collection().await?;
    
    info!("==========================================");
    info!("🎉 所有演示完成!");
    info!("📊 演示总结:");
    info!("   - 预言机智能体创建: ✅");
    info!("   - 信誉系统操作: ✅");
    info!("   - 数据采集功能: ✅");
    
    Ok(())
}

/// 演示预言机智能体
async fn demo_oracle_agent() -> Result<(), Box<dyn std::error::Error>> {
    info!("  创建BTC价格预言机智能体...");
    
    let config = OracleAgentConfig {
        name: "BTC价格预言机".to_string(),
        data_sources: vec![
            DataSource::new("CoinGecko", "https://api.coingecko.com/api/v3/simple/price", 0.8),
            DataSource::new("Binance", "https://api.binance.com/api/v3/ticker/price", 0.9),
        ],
        min_confidence: 0.7,
        max_timeout_secs: 30,
        initial_reputation: 100.0,
        initial_stake: 1000,
        supported_data_types: vec![
            OracleDataType::CryptoPrice { symbol: "BTC".to_string() },
            OracleDataType::CryptoPrice { symbol: "ETH".to_string() },
            OracleDataType::CryptoPrice { symbol: "SOL".to_string() },
        ],
        cache_ttl_secs: 300,
        auto_cache_cleanup: true,
        cache_cleanup_interval_secs: 60,
    };
    
    let mut agent = OracleAgent::new(config)?;
    agent.set_diap_identity(
        "did:diap:test_btc_oracle".to_string(),
        vec![1, 2, 3, 4, 5],
    );
    
    info!("  ✅ 预言机智能体创建成功");
    info!("     名称: {}", agent.get_info().name);
    info!("     DID: {}", agent.get_did().unwrap_or("未知".to_string()));
    info!("     支持的数据类型: {} 种", agent.get_supported_data_types().len());
    
    // 测试智能体功能
    info!("  测试智能体功能...");
    
    let btc_data_type = OracleDataType::CryptoPrice { symbol: "BTC".to_string() };
    let eth_data_type = OracleDataType::CryptoPrice { symbol: "ETH".to_string() };
    
    info!("     检查BTC支持: {}", agent.supports_data_type(&btc_data_type));
    info!("     检查ETH支持: {}", agent.supports_data_type(&eth_data_type));
    info!("     当前信誉分: {:.2}", agent.get_reputation_score());
    info!("     当前质押金额: {}", agent.get_staked_amount());
    
    // 增加质押
    agent.stake(500);
    info!("     增加质押后: {}", agent.get_staked_amount());
    
    Ok(())
}

/// 演示信誉系统
async fn demo_reputation_system() -> Result<(), Box<dyn std::error::Error>> {
    info!("  初始化信誉管理器...");
    
    let config = ReputationConfig::default();
    let reputation_manager = Arc::new(ReputationManager::new(config));
    
    // 注册测试智能体
    let test_agents = vec![
        "did:diap:agent_alpha".to_string(),
        "did:diap:agent_beta".to_string(),
        "did:diap:agent_gamma".to_string(),
    ];

    for did in &test_agents {
        reputation_manager.register_agent(did.clone()).await?;
        info!("  ✅ 注册智能体: {}", did);
    }
    
    // 模拟逻辑一致性更新
    info!("  模拟逻辑一致性更新...");

    let updates = vec![
        ("did:diap:agent_alpha", 0.95, false, 0), // 高一致性，不是离群点
        ("did:diap:agent_beta", 0.75, true, 1),  // 低一致性，是离群点
        ("did:diap:agent_gamma", 0.92, false, 2), // 高一致性，不是离群点
    ];

    for (did, cosine_similarity, is_outlier, cluster_position) in updates {
        match reputation_manager.update_for_logical_consistency(
            did,
            *cosine_similarity,
            *is_outlier,
            *cluster_position,
        ).await {
            Ok(delta) => {
                info!("     📊 {}: Δ = {:.2}", did, delta);
            }
            Err(e) => {
                warn!("     ⚠️ {} 信誉更新失败: {}", did, e);
            }
        }
    }
    
    // 查看信誉排名
    info!("  查看信誉排名...");
    
    let rankings = reputation_manager.get_rankings(5).await;
    info!("  ✅ 信誉排名获取成功: {} 个智能体", rankings.len());
    
    println!("\n信誉排名:");
    println!("{:<5} {:<30} {:<10}", "排名", "智能体DID", "因果信用分");
    println!("{}", "-".repeat(50));

    for (i, ranking) in rankings.iter().enumerate() {
        println!("{:<5} {:<30} {:<10.2}",
            i + 1,
            ranking.agent_did,
            ranking.causal_credit
        );
    }
    
    // 查看特定智能体信誉
    info!("  查看特定智能体信誉...");
    
    if let Some(score) = reputation_manager.get_score("did:diap:agent_alpha").await {
        println!("\n智能体详情:");
        println!("  DID: {}", score.agent_did);
        println!("  因果信用分: {:.2}", score.causal_credit);
        println!("  成功率: {:.2}%", score.success_rate() * 100.0);
        println!("  总任务数: {}", score.total_tasks);
        println!("  成功任务数: {}", score.successful_tasks);
    }
    
    Ok(())
}

/// 演示数据采集
async fn demo_data_collection() -> Result<(), Box<dyn std::error::Error>> {
    info!("  测试数据采集功能...");
    
    // 创建测试智能体
    let config = OracleAgentConfig::default_with_name("data_collector");
    let mut agent = OracleAgent::new(config)?;
    
    // 测试不同的数据类型
    let test_cases = vec![
        ("BTC价格", OracleDataType::CryptoPrice { symbol: "BTC".to_string() }),
        ("ETH价格", OracleDataType::CryptoPrice { symbol: "ETH".to_string() }),
        ("SOL价格", OracleDataType::CryptoPrice { symbol: "SOL".to_string() }),
    ];
    
    for (name, data_type) in test_cases {
        info!("  采集{}...", name);
        
        match agent.collect_data(&data_type).await {
            Ok(result) => {
                if result.success {
                    info!("  ✅ {}采集成功", name);
                    if let Some(data) = result.data {
                        println!("     数据类型: {:?}", data.data_type);
                        if let Some(value) = data.get_number() {
                            println!("     数值: {:.2}", value);
                        } else if let Some(text) = data.get_string() {
                            println!("     文本: {}", text);
                        }
                        println!("     置信度: {:.2}", data.confidence);
                        println!("     数据源: {:?}", data.sources_used);
                        println!("     时间戳: {}", data.timestamp);
                        println!("     采集耗时: {}ms", result.collection_time_ms);
                    }
                } else {
                    warn!("  ⚠️ {}采集失败: {:?}", name, result.error);
                }
            }
            Err(e) => {
                warn!("  ❌ {}采集错误: {}", name, e);
            }
        }
        
        // 短暂延迟，避免请求过快
        sleep(Duration::from_millis(500)).await;
    }
    
    // 测试缓存功能
    info!("  测试缓存功能...");
    
    agent.cleanup_cache();
    info!("  ✅ 缓存清理完成");
    
    // 获取智能体信息
    let info = agent.get_info();
    println!("\n智能体信息:");
    println!("  名称: {}", info.name);
    println!("  DID: {}", info.did);
    println!("  当前信誉分: {:.2}", agent.get_reputation_score());
    println!("  支持的数据类型: {} 种", info.supported_data_types.len());
    println!("  数据源数量: {}", info.data_source_count);
    println!("  缓存大小: {} 个条目", info.cache_size);
    
    Ok(())
}
