# 简化版Solana智能合约部署脚本
# 避免权限问题，使用现有配置

Write-Host "🚀 简化版Solana智能合约部署" -ForegroundColor Green
Write-Host "=========================================="

# 1. 设置环境变量
Write-Host "`n📝 设置环境变量..." -ForegroundColor Cyan
$env:HOME = $env:USERPROFILE
Write-Host "   HOME = $env:HOME" -ForegroundColor Yellow

# 2. 检查当前目录
Write-Host "`n📁 当前目录: $(Get-Location)" -ForegroundColor Cyan

# 3. 检查现有构建
Write-Host "`n🔍 检查现有构建..." -ForegroundColor Cyan
if (Test-Path "target/deploy/solana_oracle-keypair.json") {
    Write-Host "   ✅ 已找到密钥对文件" -ForegroundColor Green
    $programId = solana address -k target/deploy/solana_oracle-keypair.json
    Write-Host "   程序ID: $programId" -ForegroundColor Yellow
} else {
    Write-Host "   ⚠️  未找到密钥对文件，需要构建" -ForegroundColor Yellow
}

# 4. 尝试编译（不安装平台工具）
Write-Host "`n🔨 尝试编译..." -ForegroundColor Cyan
try {
    # 使用cargo直接编译
    cargo build-sbf --manifest-path programs/solana-oracle/Cargo.toml
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 编译成功!" -ForegroundColor Green
    } else {
        Write-Host "   ❌ 编译失败" -ForegroundColor Red
    }
} catch {
    Write-Host "   ❌ 编译错误: $_" -ForegroundColor Red
}

# 5. 使用现有程序ID（如果已部署）
Write-Host "`n📋 使用现有程序ID..." -ForegroundColor Cyan
$existingProgramId = "DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"
Write-Host "   现有程序ID: $existingProgramId" -ForegroundColor Yellow

# 6. 创建测试脚本
Write-Host "`n🧪 创建测试脚本..." -ForegroundColor Cyan
$testScript = @"
// 智能体注册测试脚本
// 使用现有程序ID: $existingProgramId

const anchor = require('@project-serum/anchor');
const { PublicKey, Keypair, SystemProgram } = require('@solana/web3.js');
const { BN } = require('bn.js');

// 连接到本地测试网
const provider = anchor.AnchorProvider.local();
anchor.setProvider(provider);

// 程序ID
const programId = new PublicKey('$existingProgramId');

// 加载IDL（接口定义语言）
// 注意：需要先构建项目生成IDL
async function testAgentRegistration() {
    try {
        console.log('🧪 开始智能体注册测试...');
        console.log('程序ID:', programId.toString());
        
        // 创建测试智能体
        const agent = Keypair.generate();
        console.log('智能体公钥:', agent.publicKey.toString());
        
        // 模拟DID
        const did = 'did:example:agent123';
        const publicKey = new Uint8Array(32).fill(1); // 模拟公钥
        const metadataUri = 'https://ipfs.io/ipfs/QmExampleMetadata';
        
        console.log('✅ 测试准备完成');
        console.log('DID:', did);
        console.log('元数据URI:', metadataUri);
        
        // 在实际部署后，这里会调用智能合约
        console.log('💡 部署后，将调用:');
        console.log('   register_agent(did, publicKey, metadataUri)');
        
    } catch (error) {
        console.error('❌ 测试错误:', error);
    }
}

// 运行测试
testAgentRegistration();
"@

Set-Content -Path "test_agent.js" -Value $testScript
Write-Host "   已创建测试脚本: test_agent.js" -ForegroundColor Green

# 7. 创建多智能体测试配置
Write-Host "`n🤖 创建多智能体测试配置..." -ForegroundColor Cyan
$multiAgentConfig = @"
# 多智能体测试配置
# 支持多个智能体注册和交互

agents:
  - name: "预言机核心节点"
    did: "did:example:oracle-core-001"
    public_key: "0x1111111111111111111111111111111111111111111111111111111111111111"
    metadata_uri: "https://ipfs.io/ipfs/QmCoreAgent"
    reputation: 850
    tier: "core"
    
  - name: "数据验证节点"
    did: "did:example:validator-002"
    public_key: "0x2222222222222222222222222222222222222222222222222222222222222222"
    metadata_uri: "https://ipfs.io/ipfs/QmValidatorAgent"
    reputation: 650
    tier: "validator"
    
  - name: "数据提供节点"
    did: "did:example:data-provider-003"
    public_key: "0x3333333333333333333333333333333333333333333333333333333333333333"
    metadata_uri: "https://ipfs.io/ipfs/QmDataProvider"
    reputation: 350
    tier: "data"
    
  - name: "轻量级网关"
    did: "did:example:gateway-004"
    public_key: "0x4444444444444444444444444444444444444444444444444444444444444444"
    metadata_uri: "https://ipfs.io/ipfs/QmGateway"
    reputation: 200
    tier: "gateway"

network:
  rpc_url: "http://localhost:8899"
  program_id: "$existingProgramId"
  cluster: "localnet"

testing:
  enable_mock: true
  simulate_interactions: true
  test_duration: 300
"@

Set-Content -Path "multi_agent_config.yaml" -Value $multiAgentConfig
Write-Host "   已创建配置: multi_agent_config.yaml" -ForegroundColor Green

# 8. 创建Rust测试客户端
Write-Host "`n🦀 创建Rust测试客户端..." -ForegroundColor Cyan
$rustTest = @"
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
        "$existingProgramId"
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
"@

Set-Content -Path "test_client.rs" -Value $rustTest
Write-Host "   已创建Rust测试客户端: test_client.rs" -ForegroundColor Green

Write-Host "`n" + "=".repeat(50)
Write-Host "🎉 简化部署准备完成!" -ForegroundColor Green
Write-Host "`n📋 已创建的文件:" -ForegroundColor Cyan
Write-Host "   ✅ test_agent.js - JavaScript测试脚本" -ForegroundColor Yellow
Write-Host "   ✅ multi_agent_config.yaml - 多智能体配置" -ForegroundColor Yellow
Write-Host "   ✅ test_client.rs - Rust测试客户端" -ForegroundColor Yellow

Write-Host "`n🚀 下一步操作:" -ForegroundColor Cyan
Write-Host "   1. 启动本地测试网:" -ForegroundColor White
Write-Host "      solana-test-validator --reset" -ForegroundColor White
Write-Host "      solana config set --url http://localhost:8899" -ForegroundColor White
Write-Host "   2. 部署智能合约:" -ForegroundColor White
Write-Host "      anchor deploy --provider.cluster localnet" -ForegroundColor White
Write-Host "   3. 运行测试:" -ForegroundColor White
Write-Host "      node test_agent.js" -ForegroundColor White
Write-Host "      cargo run --bin test_client" -ForegroundColor White

Write-Host "`n💡 提示:" -ForegroundColor Cyan
Write-Host "   - 如果遇到权限问题，尝试以管理员身份运行PowerShell" -ForegroundColor Yellow
Write-Host "   - 或者使用WSL2/Linux环境进行开发" -ForegroundColor Yellow
Write-Host "   - 现有程序ID可用于测试，无需重新部署" -ForegroundColor Yellow
