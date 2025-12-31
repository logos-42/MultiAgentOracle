# Devnet实际部署脚本
# 部署智能体注册程序到Solana Devnet

Write-Host "🚀 Solana Devnet实际部署" -ForegroundColor Green
Write-Host "=========================================="

# 1. 设置环境
Write-Host "`n📝 设置环境..." -ForegroundColor Cyan
$env:HOME = $env:USERPROFILE
Write-Host "   HOME环境变量已设置" -ForegroundColor Yellow

# 2. 切换到Devnet
Write-Host "`n🌐 切换到Devnet网络..." -ForegroundColor Cyan
solana config set --url https://api.devnet.solana.com

Write-Host "   当前配置:" -ForegroundColor Yellow
solana config get

# 3. 检查Devnet连接
Write-Host "`n🔍 检查Devnet连接..." -ForegroundColor Cyan
try {
    $version = solana cluster-version
    Write-Host "   ✅ Devnet连接正常: $version" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Devnet连接失败: $_" -ForegroundColor Red
    Write-Host "   请检查网络连接" -ForegroundColor Yellow
    exit 1
}

# 4. 检查余额
Write-Host "`n💰 检查余额..." -ForegroundColor Cyan
$balance = solana balance
Write-Host "   当前余额: $balance" -ForegroundColor Yellow

if ($balance -eq "0 SOL") {
    Write-Host "   请求空投..." -ForegroundColor Yellow
    solana airdrop 1
    Start-Sleep -Seconds 5
    $balance = solana balance
    Write-Host "   新余额: $balance" -ForegroundColor Green
}

# 5. 构建项目
Write-Host "`n🔨 构建智能合约..." -ForegroundColor Cyan
Write-Host "   构建中，请稍候..." -ForegroundColor Yellow

try {
    anchor build
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 构建成功!" -ForegroundColor Green
    } else {
        Write-Host "   ❌ 构建失败" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "   ❌ 构建错误: $_" -ForegroundColor Red
    exit 1
}

# 6. 获取程序ID
Write-Host "`n📝 获取程序ID..." -ForegroundColor Cyan
$programId = solana address -k target/deploy/solana_oracle-keypair.json
Write-Host "   程序ID: $programId" -ForegroundColor Yellow

# 7. 更新源代码
Write-Host "`n🔄 更新源代码中的程序ID..." -ForegroundColor Cyan
$libRsPath = "programs/solana-oracle/src/lib.rs"
if (Test-Path $libRsPath) {
    $content = Get-Content $libRsPath -Raw
    $updatedContent = $content -replace 'declare_id\(".*"\)', "declare_id(`"$programId`")"
    Set-Content $libRsPath -Value $updatedContent
    Write-Host "   ✅ 已更新程序ID" -ForegroundColor Green
} else {
    Write-Host "   ❌ 找不到源文件: $libRsPath" -ForegroundColor Red
    exit 1
}

# 8. 重新构建
Write-Host "`n🔨 重新构建..." -ForegroundColor Cyan
anchor build
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ 重新构建成功!" -ForegroundColor Green
} else {
    Write-Host "   ❌ 重新构建失败" -ForegroundColor Red
    exit 1
}

# 9. 部署到Devnet
Write-Host "`n🚀 部署到Devnet..." -ForegroundColor Cyan
Write-Host "   部署中，这可能需要几分钟..." -ForegroundColor Yellow

try {
    $deployOutput = anchor deploy --provider.cluster devnet 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 部署成功!" -ForegroundColor Green
        Write-Host "   程序地址: $programId" -ForegroundColor Cyan
    } else {
        Write-Host "   ❌ 部署失败" -ForegroundColor Red
        Write-Host "   错误信息:" -ForegroundColor Red
        Write-Host $deployOutput -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "   ❌ 部署错误: $_" -ForegroundColor Red
    exit 1
}

# 10. 验证部署
Write-Host "`n🔍 验证部署..." -ForegroundColor Cyan
try {
    $programInfo = solana program show $programId
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 程序验证成功!" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️ 程序验证失败" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ⚠️ 验证错误: $_" -ForegroundColor Yellow
}

# 11. 创建部署记录
Write-Host "`n📋 创建部署记录..." -ForegroundColor Cyan
$deploymentRecord = @{
    ProgramId = $programId
    Network = "devnet"
    DeployedAt = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    RpcUrl = "https://api.devnet.solana.com"
    ExplorerUrl = "https://explorer.solana.com/address/$programId?cluster=devnet"
    Balance = $balance
} | ConvertTo-Json

Set-Content -Path "devnet_deployment.json" -Value $deploymentRecord
Write-Host "   部署记录已保存: devnet_deployment.json" -ForegroundColor Green

# 12. 多智能体测试准备
Write-Host "`n🤖 多智能体测试准备..." -ForegroundColor Cyan

$testConfig = @"
{
    "program_id": "$programId",
    "network": "devnet",
    "rpc_url": "https://api.devnet.solana.com",
    "agents": [
        {
            "name": "预言机核心节点",
            "did": "did:example:oracle-core-001",
            "public_key": "0x1111111111111111111111111111111111111111111111111111111111111111",
            "metadata_uri": "https://ipfs.io/ipfs/QmCoreAgent",
            "reputation": 850,
            "tier": "core"
        },
        {
            "name": "数据验证节点",
            "did": "did:example:validator-002",
            "public_key": "0x2222222222222222222222222222222222222222222222222222222222222222",
            "metadata_uri": "https://ipfs.io/ipfs/QmValidator",
            "reputation": 650,
            "tier": "validator"
        },
        {
            "name": "数据提供节点",
            "did": "did:example:data-provider-003",
            "public_key": "0x3333333333333333333333333333333333333333333333333333333333333333",
            "metadata_uri": "https://ipfs.io/ipfs/QmDataProvider",
            "reputation": 350,
            "tier": "data"
        },
        {
            "name": "轻量级网关",
            "did": "did:example:gateway-004",
            "public_key": "0x4444444444444444444444444444444444444444444444444444444444444444",
            "metadata_uri": "https://ipfs.io/ipfs/QmGateway",
            "reputation": 200,
            "tier": "gateway"
        }
    ]
}
"@

Set-Content -Path "devnet_test_config.json" -Value $testConfig
Write-Host "   测试配置已保存: devnet_test_config.json" -ForegroundColor Green

# 13. 创建测试脚本
Write-Host "`n🧪 创建测试脚本..." -ForegroundColor Cyan

$testScript = @"
// Devnet智能体注册测试
const programId = '$programId';

console.log('🚀 Devnet智能体注册测试');
console.log('程序ID:', programId);
console.log('网络: devnet');
console.log('RPC: https://api.devnet.solana.com');
console.log('智能体数量: 4');

console.log('\n📋 测试智能体:');
const agents = [
    '预言机核心节点 (did:example:oracle-core-001)',
    '数据验证节点 (did:example:validator-002)',
    '数据提供节点 (did:example:data-provider-003)',
    '轻量级网关 (did:example:gateway-004)'
];

agents.forEach((agent, index) => {
    console.log(\`  \${index + 1}. \${agent}\`);
});

console.log('\n✅ 测试环境准备完成!');
console.log('💡 下一步: 运行实际交易测试');
console.log('💡 查看部署: https://explorer.solana.com/address/' + programId + '?cluster=devnet');
"@

Set-Content -Path "devnet_test.js" -Value $testScript
Write-Host "   测试脚本已创建: devnet_test.js" -ForegroundColor Green

# 运行测试
Write-Host "`n🧪 运行测试..." -ForegroundColor Cyan
node devnet_test.js

# 14. 显示成功信息
Write-Host "`n" + "=".repeat(50)
Write-Host "🎉 Devnet部署成功完成!" -ForegroundColor Green

Write-Host "`n📋 部署信息:" -ForegroundColor Cyan
Write-Host "   程序ID: $programId" -ForegroundColor Yellow
Write-Host "   网络: devnet" -ForegroundColor Yellow
Write-Host "   RPC端点: https://api.devnet.solana.com" -ForegroundColor Yellow
Write-Host "   余额: $balance" -ForegroundColor Yellow
Write-Host "   智能体: 4个测试智能体" -ForegroundColor Yellow

Write-Host "`n🌐 浏览器查看:" -ForegroundColor Cyan
Write-Host "   https://explorer.solana.com/address/$programId?cluster=devnet" -ForegroundColor White

Write-Host "`n🚀 下一步操作:" -ForegroundColor Cyan
Write-Host "   1. 验证程序状态: solana program show $programId" -ForegroundColor White
Write-Host "   2. 运行完整测试: anchor test --provider.cluster devnet" -ForegroundColor White
Write-Host "   3. 注册智能体: 运行实际交易测试" -ForegroundColor White
Write-Host "   4. 查看交易: solana transaction-history --limit 10" -ForegroundColor White

Write-Host "`n💡 提示:" -ForegroundColor Cyan
Write-Host "   - Devnet是真实的测试网络，交易会被广播" -ForegroundColor Yellow
Write-Host "   - 使用测试SOL，没有实际价值" -ForegroundColor Yellow
Write-Host "   - 可以随时重新部署和测试" -ForegroundColor Yellow

Write-Host "`n📁 生成的文件:" -ForegroundColor Cyan
Write-Host "   ✅ devnet_deployment.json - 部署记录" -ForegroundColor Yellow
Write-Host "   ✅ devnet_test_config.json - 测试配置" -ForegroundColor Yellow
Write-Host "   ✅ devnet_test.js - 测试脚本" -ForegroundColor Yellow

Write-Host "`n🎯 实际部署完成! 现在可以开始真正的多智能体注册测试了。" -ForegroundColor Green
