// Solana预言机系统CLI客户端
// 用于与部署的智能合约交互

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer, read_keypair_file},
    transaction::Transaction,
};
use std::str::FromStr;
use std::env;

const PROGRAM_ID: &str = "GoQFXtbPyBaghGLF138djbmBTKKZXwTPfesh4J7SSPot";
const RPC_URL: &str = "http://localhost:8899";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Solana预言机系统CLI客户端");
    println!("=============================\n");
    
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    let command = &args[1];
    
    // 连接到本地测试网
    let client = RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    
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
    
    // 程序ID
    let program_id = Pubkey::from_str(PROGRAM_ID)?;
    println!("📋 程序ID: {}\n", program_id);
    
    match command.as_str() {
        "info" => {
            println!("📊 系统信息:");
            println!("   程序ID: {}", PROGRAM_ID);
            println!("   RPC URL: {}", RPC_URL);
            
            // 获取程序信息
            match client.get_account(&program_id) {
                Ok(account) => {
                    println!("   ✅ 程序已部署");
                    println!("   所有者: {}", account.owner);
                    println!("   数据长度: {} bytes", account.data.len());
                    println!("   Lamports: {}", account.lamports);
                }
                Err(e) => {
                    println!("   ❌ 无法获取程序信息: {}", e);
                }
            }
        }
        
        "create-agent" => {
            if args.len() < 3 {
                println!("❌ 用法: cli create-agent <DID> [metadata_uri]");
                return Ok(());
            }
            
            let did = &args[2];
            let metadata_uri = if args.len() > 3 { &args[3] } else { "" };
            
            println!("🤖 创建智能体:");
            println!("   DID: {}", did);
            println!("   元数据URI: {}", metadata_uri);
            
            // 这里需要实现实际的智能体创建逻辑
            // 需要调用智能合约的register_agent指令
            println!("   ⚠️  功能开发中...");
        }
        
        "list-agents" => {
            println!("📋 列出所有智能体:");
            println!("   ⚠️  功能开发中...");
            // 这里需要实现从智能合约读取所有智能体的逻辑
        }
        
        "test" => {
            println!("🧪 运行测试:");
            run_tests(&client, &program_id).await?;
        }
        
        "help" | "--help" | "-h" => {
            print_usage();
        }
        
        _ => {
            println!("❌ 未知命令: {}", command);
            print_usage();
        }
    }
    
    Ok(())
}

async fn run_tests(client: &RpcClient, program_id: &Pubkey) -> Result<(), Box<dyn std::error::Error>> {
    println!("1. 测试连接...");
    match client.get_version() {
        Ok(version) => println!("   ✅ 连接成功: {:?}", version),
        Err(e) => {
            println!("   ❌ 连接失败: {}", e);
            return Ok(());
        }
    }
    
    println!("2. 检查程序状态...");
    match client.get_account(program_id) {
        Ok(account) => {
            println!("   ✅ 程序存在");
            println!("     所有者: {}", account.owner);
            println!("     数据长度: {} bytes", account.data.len());
            println!("     Lamports: {}", account.lamports);
        }
        Err(e) => {
            println!("   ❌ 程序不存在: {}", e);
            return Ok(());
        }
    }
    
    println!("3. 检查网络状态...");
    match client.get_slot() {
        Ok(slot) => println!("   ✅ 当前slot: {}", slot),
        Err(e) => println!("   ❌ 获取slot失败: {}", e),
    }
    
    println!("\n✅ 所有测试完成!");
    Ok(())
}

fn print_usage() {
    println!("用法: cli <命令> [参数]");
    println!();
    println!("命令:");
    println!("  info               显示系统信息");
    println!("  create-agent <DID> [metadata_uri]  创建新智能体");
    println!("  list-agents        列出所有智能体");
    println!("  test               运行系统测试");
    println!("  help               显示帮助信息");
    println!();
    println!("示例:");
    println!("  cli info");
    println!("  cli create-agent did:example:agent1 https://example.com/metadata.json");
    println!("  cli test");
}
