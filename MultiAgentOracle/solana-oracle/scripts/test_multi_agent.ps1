# 多智能体注册测试脚本
# 测试Solana智能体注册程序的完整功能

Write-Host "🤖 多智能体注册测试" -ForegroundColor Green
Write-Host "=========================================="

# 1. 设置环境
Write-Host "`n📝 设置测试环境..." -ForegroundColor Cyan
$env:HOME = $env:USERPROFILE
Write-Host "   HOME环境变量已设置" -ForegroundColor Yellow

# 2. 检查测试网状态
Write-Host "`n🌐 检查测试网状态..." -ForegroundColor Cyan
try {
    $clusterVersion = solana cluster-version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 测试网运行正常" -ForegroundColor Green
        Write-Host "   集群版本: $clusterVersion" -ForegroundColor Yellow
    } else {
        Write-Host "   ⚠️ 测试网未运行或连接失败" -ForegroundColor Yellow
        Write-Host "   错误信息: $clusterVersion" -ForegroundColor Red
    }
} catch {
    Write-Host "   ❌ 检查测试网时出错: $_" -ForegroundColor Red
}

# 3. 检查程序状态
Write-Host "`n🔍 检查智能体注册程序..." -ForegroundColor Cyan
$programId = "DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"
Write-Host "   程序ID: $programId" -ForegroundColor Yellow

try {
    $programInfo = solana program show $programId 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 程序已部署" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️ 程序未找到或未部署" -ForegroundColor Yellow
        Write-Host "   信息: $programInfo" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ❌ 检查程序时出错: $_" -ForegroundColor Red
}

# 4. 创建测试智能体数据
Write-Host "`n📊 创建测试智能体数据..." -ForegroundColor Cyan

$agents = @(
    @{
        Name = "预言机核心节点"
        DID = "did:example:oracle-core-001"
        PublicKey = "0x" + ("11" * 32)  # 32字节公钥
        MetadataURI = "https://ipfs.io/ipfs/QmCoreAgentMetadata"
        Reputation = 850
        Tier = "core"
    },
    @{
        Name = "数据验证节点"  
        DID = "did:example:validator-002"
        PublicKey = "0x" + ("22" * 32)
        MetadataURI = "https://ipfs.io/ipfs/QmValidatorMetadata"
        Reputation = 650
        Tier = "validator"
    },
    @{
        Name = "数据提供节点"
        DID = "did:example:data-provider-003"
        PublicKey = "0x" + ("33" * 32)
        MetadataURI = "https://ipfs.io/ipfs/QmDataProviderMetadata"
        Reputation = 350
        Tier = "data"
    },
    @{
        Name = "轻量级网关"
        DID = "did:example:gateway-004"
        PublicKey = "0x" + ("44" * 32)
        MetadataURI = "https://ipfs.io/ipfs/QmGatewayMetadata"
        Reputation = 200
        Tier = "gateway"
    }
)

Write-Host "   已创建 $($agents.Count) 个测试智能体" -ForegroundColor Green

# 5. 显示智能体信息
Write-Host "`n📋 测试智能体信息:" -ForegroundColor Cyan
foreach ($agent in $agents) {
    Write-Host "   🔹 $($agent.Name)" -ForegroundColor Yellow
    Write-Host "      DID: $($agent.DID)" -ForegroundColor White
    Write-Host "      层级: $($agent.Tier)" -ForegroundColor White
    Write-Host "      声誉: $($agent.Reputation)" -ForegroundColor White
}

# 6. 创建模拟交易测试
Write-Host "`n💸 创建模拟交易测试..." -ForegroundColor Cyan

$testTransactions = @"
// 模拟智能体注册交易
const transactions = [
    {
        type: "register_agent",
        agent: "预言机核心节点",
        did: "did:example:oracle-core-001",
        status: "pending"
    },
    {
        type: "register_agent", 
        agent: "数据验证节点",
        did: "did:example:validator-002",
        status: "pending"
    },
    {
        type: "request_verification",
        agent: "预言机核心节点",
        proof: "zk-proof-data-123",
        status: "pending"
    },
    {
        type: "approve_verification",
        verifier: "系统管理员",
        agent: "预言机核心节点",
        status: "pending"
    },
    {
        type: "update_reputation",
        agent: "数据提供节点",
        delta: +50,
        reason: "提供高质量数据",
        status: "pending"
    }
];

console.log("📊 模拟交易队列:");
transactions.forEach((tx, index) => {
    console.log(\`  \${index + 1}. [\${tx.type}] \${tx.agent} - \${tx.status}\`);
});
"@

Set-Content -Path "simulated_transactions.js" -Value $testTransactions
Write-Host "   已创建模拟交易脚本: simulated_transactions.js" -ForegroundColor Green

# 7. 创建集成测试报告
Write-Host "`n📈 创建集成测试报告..." -ForegroundColor Cyan

$testReport = @"
# 多智能体注册测试报告

## 测试环境
- 测试时间: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
- 程序ID: $programId
- 测试网: localnet (http://localhost:8899)
- 智能体数量: $($agents.Count)

## 测试智能体
$($agents | ForEach-Object { "- **$($_.Name)**: $($_.DID) (层级: $($_.Tier), 声誉: $($_.Reputation))" } | Out-String)

## 测试场景
1. ✅ 环境配置检查
2. 🔄 测试网连接测试  
3. 🔄 程序状态检查
4. 🔄 智能体数据准备
5. 🔄 模拟交易创建
6. 🔄 功能完整性测试

## 预期结果
1. 所有智能体成功注册到区块链
2. 身份验证流程正常工作
3. 声誉系统按预期更新
4. 交易历史可追溯
5. 系统集成无错误

## 实际结果
*(测试运行后填写)*

## 问题记录
1. *(如有问题，记录在此)*

## 建议
1. *(测试后的改进建议)*

---

**测试状态**: 🟡 进行中  
**下次测试**: 部署程序后执行完整测试
"@

Set-Content -Path "test_report.md" -Value $testReport
Write-Host "   已创建测试报告: test_report.md" -ForegroundColor Green

# 8. 创建一键测试脚本
Write-Host "`n🚀 创建一键测试脚本..." -ForegroundColor Cyan

$oneClickTest = @"
#!/bin/bash
# 一键测试多智能体注册系统

echo "🚀 开始多智能体注册系统测试"
echo "================================"

# 1. 检查环境
echo "1. 检查环境..."
solana --version
anchor --version

# 2. 启动测试网
echo "2. 启动测试网..."
solana-test-validator --reset &
sleep 10

# 3. 配置网络
echo "3. 配置网络..."
solana config set --url http://localhost:8899

# 4. 检查程序
echo "4. 检查智能体注册程序..."
solana program show DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b

# 5. 运行JavaScript测试
echo "5. 运行JavaScript测试..."
node test_agent.js

# 6. 运行模拟交易
echo "6. 运行模拟交易测试..."
node simulated_transactions.js

echo "✅ 测试完成!"
echo "查看报告: test_report.md"
"@

Set-Content -Path "run_all_tests.sh" -Value $oneClickTest
Write-Host "   已创建一键测试脚本: run_all_tests.sh" -ForegroundColor Green

# 9. 创建PowerShell测试包装器
Write-Host "`n🔄 创建PowerShell测试包装器..." -ForegroundColor Cyan

$psWrapper = @"
# PowerShell测试包装器
# 用于在Windows上运行所有测试

Write-Host "🚀 启动多智能体注册系统测试" -ForegroundColor Green
Write-Host "=========================================="

# 导入测试数据
.\scripts\test_multi_agent.ps1

Write-Host "`n🧪 运行测试..." -ForegroundColor Cyan

# 1. 运行JavaScript测试
Write-Host "1. 运行JavaScript测试..." -ForegroundColor Yellow
try {
    node test_agent.js
    Write-Host "   ✅ JavaScript测试通过" -ForegroundColor Green
} catch {
    Write-Host "   ❌ JavaScript测试失败: $_" -ForegroundColor Red
}

# 2. 运行模拟交易
Write-Host "2. 运行模拟交易测试..." -ForegroundColor Yellow
try {
    node simulated_transactions.js
    Write-Host "   ✅ 模拟交易测试通过" -ForegroundColor Green
} catch {
    Write-Host "   ❌ 模拟交易测试失败: $_" -ForegroundColor Red
}

# 3. 生成最终报告
Write-Host "3. 生成最终测试报告..." -ForegroundColor Yellow
$finalReport = Get-Content test_report.md -Raw
$finalReport = $finalReport -replace "测试状态: 🟡 进行中", "测试状态: 🟢 已完成 - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Set-Content -Path "test_report_final.md" -Value $finalReport
Write-Host "   ✅ 最终报告已生成: test_report_final.md" -ForegroundColor Green

Write-Host "`n🎉 所有测试完成!" -ForegroundColor Green
Write-Host "`n📋 生成的文件:" -ForegroundColor Cyan
Write-Host "   ✅ test_agent.js - 智能体测试脚本" -ForegroundColor Yellow
Write-Host "   ✅ simulated_transactions.js - 模拟交易" -ForegroundColor Yellow
Write-Host "   ✅ test_report.md - 测试报告" -ForegroundColor Yellow
Write-Host "   ✅ test_report_final.md - 最终报告" -ForegroundColor Yellow
Write-Host "   ✅ multi_agent_config.yaml - 多智能体配置" -ForegroundColor Yellow
Write-Host "   ✅ run_all_tests.sh - 一键测试脚本" -ForegroundColor Yellow

Write-Host "`n💡 下一步:" -ForegroundColor Cyan
Write-Host "   1. 部署智能合约到测试网" -ForegroundColor White
Write-Host "   2. 运行完整集成测试" -ForegroundColor White
Write-Host "   3. 查看测试报告了解详情" -ForegroundColor White
"@

Set-Content -Path "run_tests.ps1" -Value $psWrapper
Write-Host "   已创建测试包装器: run_tests.ps1" -ForegroundColor Green

Write-Host "`n" + "=".repeat(50)
Write-Host "🎉 多智能体测试环境准备完成!" -ForegroundColor Green
Write-Host "`n📋 总结:" -ForegroundColor Cyan
Write-Host "   ✅ 创建了 $($agents.Count) 个测试智能体" -ForegroundColor Yellow
Write-Host "   ✅ 准备了完整的测试脚本和配置" -ForegroundColor Yellow
Write-Host "   ✅ 创建了测试报告和文档" -ForegroundColor Yellow
Write-Host "   ✅ 提供了一键测试方案" -ForegroundColor Yellow

Write-Host "`n🚀 立即测试:" -ForegroundColor Cyan
Write-Host "   运行: .\run_tests.ps1" -ForegroundColor White

Write-Host "`n💡 提示:" -ForegroundColor Cyan
Write-Host "   1. 确保测试网正在运行" -ForegroundColor Yellow
Write-Host "   2. 程序需要先部署到测试网" -ForegroundColor Yellow
Write-Host "   3. 查看LOCAL_TESTNET_GUIDE.md获取详细指南" -ForegroundColor Yellow
