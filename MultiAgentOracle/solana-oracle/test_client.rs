// Rust测试客户端
// 用于测试智能体注册功能

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动Rust测试客户端");
    
    // 连接到本地测试网
    let rpc_url = "http://localhost:8899".to_string();
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    
    // 检查连接
    match client.get_version() {
        Ok(version) => println!("✅ 连接到Solana节点: {:?}", version),
        Err(e) => {
            println!("❌ 连接失败: {}", e);
            println!("💡 请确保本地测试网正在运行:");
            println!("   solana-test-validator --reset");
            return Ok(());
        }
    }
    
    // 创建测试智能体
    let agent = Keypair::new();
    println!("🤖 创建测试智能体:");
    println!("   公钥: {}", agent.pubkey());
    
    // 程序ID
    let program_id = solana_sdk::pubkey::Pubkey::from_str(
        "DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"
    )?;
    println!("📋 程序ID: {}", program_id);
    
    // 模拟智能体数据
    let did = "did:example:test-agent".to_string();
    let metadata_uri = "https://ipfs.io/ipfs/QmTestMetadata".to_string();
    
    println!("📊 智能体信息:");
    println!("   DID: {}", did);
    println!("   元数据URI: {}", metadata_uri);
    
    println!("\n✅ 测试客户端准备完成!");
    println!("💡 下一步:");
    println!("   1. 部署智能合约到本地测试网");
    println!("   2. 运行JavaScript测试: node test_agent.js");
    println!("   3. 或运行Rust集成测试");
    
    Ok(())
}
