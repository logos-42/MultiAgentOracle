//! SDK 快速开始示例
//!
//! 展示如何使用 SDK 的高级 API 进行预言机查询和链上提交。

use multi_agent_oracle::sdk::{Oracle, OracleQuery, SdkConfig, SolanaConfig, SolanaIntegration};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    env_logger::init();

    println!("🚀 SDK 快速开始示例\n");

    // 1. 创建 SDK 配置
    let config = SdkConfig::builder()
        .with_solana_rpc("https://api.devnet.solana.com")
        .with_agent_count(5)
        .with_consensus_threshold(0.67)
        .with_query_timeout(Duration::from_secs(30))
        .with_causal_fingerprint(true)
        .with_spectral_analysis(true)
        .with_min_agents(3)
        .with_max_retries(3)
        .build();

    println!("✅ SDK 配置创建完成");

    // 2. 创建预言机实例
    let oracle = Oracle::new(config)?;

    // 3. 初始化默认 Agents
    oracle.init_default_agents().await?;
    println!("✅ 初始化 {} 个 Agents", oracle.agent_count().await);

    // 4. 初始化共识引擎
    oracle.init_consensus().await?;
    println!("✅ 共识引擎初始化完成\n");

    // 5. 发起查询
    let query = OracleQuery::new("query_001", "BTC price prediction")
        .with_data_type("crypto_price")
        .with_timeout(30)
        .with_required_agents(3);

    println!("🔍 发起查询: {}", query.query);
    let result = oracle.query(query).await?;

    // 6. 获取共识结果
    let consensus = result.consensus_output()?;
    println!("\n📊 共识结果:");
    println!("   查询 ID: {}", consensus.query_id);
    println!("   共识值: {:.4}", consensus.consensus_value);
    println!("   置信度: {:.2}%", consensus.confidence * 100.0);
    println!("   参与 Agent 数量: {}", consensus.participant_count);
    println!("   异常检测: {}", if consensus.anomaly_detected { "是" } else { "否" });

    // 7. Solana 链上集成示例
    println!("\n🔗 Solana 链上集成:");

    let solana_config = SolanaConfig::builder()
        .with_rpc_url("https://api.devnet.solana.com")
        .with_oracle_program_id("YourOracleProgramId111111111111111111111111")
        .with_transaction_timeout(60)
        .build();

    let solana = SolanaIntegration::new(solana_config);

    // 构建链上提交数据
    let chain_data = solana.build_chain_submission(consensus)?;
    println!("   合约地址: {}", chain_data.contract_address);
    println!("   方法名: {}", chain_data.method_name);
    println!("   预估 Gas: {:?}", chain_data.estimated_gas);

    // 提交到链上（模拟）
    // let tx_hash = solana.submit_consensus(consensus).await?;
    // println!("   交易哈希: {}", tx_hash);

    println!("\n✅ 示例完成!");

    Ok(())
}
