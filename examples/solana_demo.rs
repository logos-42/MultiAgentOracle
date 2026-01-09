//! Solana集成演示程序
//!
//! 展示如何使用Solana进行智能体身份注册

use multi_agent_oracle::solana::{demo_identity_registration, SolanaConfig, IdentityRegistryClient, SolanaClient};
use solana_sdk::signature::Keypair;
use std::error::Error;
use std::iter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", "=".repeat(60));
    println!("Solana集成演示程序");
    println!("{}", "=".repeat(60));
    
    // 演示1: 运行完整的演示
    println!("\n📋 演示1: 完整的Solana身份注册演示");
    println!("{}", "-".repeat(40));
    
    match demo_identity_registration().await {
        Ok(_) => println!("✅ 演示成功完成!"),
        Err(e) => println!("⚠️  演示遇到错误: {}", e),
    }
    
    // 演示2: 手动创建客户端和注册身份
    println!("\n📋 演示2: 手动客户端配置");
    println!("{}", "-".repeat(40));
    
    // 创建配置
    let config = SolanaConfig {
        rpc_url: "https://api.devnet.solana.com".to_string(),
        ws_url: "wss://api.devnet.solana.com".to_string(),
        program_id: "DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b".to_string(),
        commitment: "confirmed".to_string(),
    };
    
    // 创建Solana客户端
    let solana_client = SolanaClient::new(config.clone());
    
    // 创建测试钱包
    let test_keypair = Keypair::new();
    solana_client.set_wallet(test_keypair).await;
    
    println!("✅ Solana客户端创建成功");
    println!("   钱包地址: {}", solana_client.get_wallet_address().await?);
    
    // 检查连接
    match solana_client.check_connection().await {
        Ok(true) => println!("✅ 成功连接到Solana网络"),
        Ok(false) => println!("❌ 无法连接到Solana网络"),
        Err(e) => println!("⚠️  连接检查错误: {}", e),
    }
    
    // 演示3: 创建身份注册客户端
    println!("\n📋 演示3: 身份注册客户端");
    println!("{}", "-".repeat(40));
    
    match IdentityRegistryClient::from_config(&config) {
        Ok(registry_client) => {
            println!("✅ 身份注册客户端创建成功");
            println!("   程序ID: {}", registry_client.program_id());
            
            // 检查身份是否已注册
            let test_did = "did:example:test123";
            match registry_client.is_identity_registered(test_did).await {
                Ok(true) => println!("   DID '{}' 已注册", test_did),
                Ok(false) => println!("   DID '{}' 未注册", test_did),
                Err(e) => println!("⚠️  检查注册状态错误: {}", e),
            }
        }
        Err(e) => println!("❌ 创建身份注册客户端失败: {}", e),
    }
    
    // 演示4: 模拟身份注册流程
    println!("\n📋 演示4: 模拟身份注册流程");
    println!("{}", "-".repeat(40));
    
    let mock_did = "did:agent:test-001";
    let mock_public_key = [1u8; 32]; // 模拟公钥
    let mock_metadata_uri = "https://ipfs.io/ipfs/QmTestMetadata".to_string();
    
    println!("模拟注册身份:");
    println!("  DID: {}", mock_did);
    println!("  公钥: {:?}...", &mock_public_key[..8]);
    println!("  元数据URI: {}", mock_metadata_uri);
    
    // 注意: 实际注册需要已部署的程序和足够的SOL余额
    println!("\n⚠️  注意: 实际注册需要:");
    println!("  1. 已部署的Solana程序");
    println!("  2. 足够的SOL余额支付交易费用");
    println!("  3. 正确的程序ID配置");
    
    println!("\n🎉 Solana集成演示完成!");
    println!("\n下一步:");
    println!("  1. 部署Solana程序: cd solana-oracle && anchor deploy");
    println!("  2. 更新程序ID配置");
    println!("  3. 获取测试SOL: solana airdrop 1");
    println!("  4. 运行实际注册测试");
    
    Ok(())
}
