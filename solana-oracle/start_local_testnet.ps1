# Solana本地测试网启动脚本
# 用于启动本地测试网并部署智能体注册程序

Write-Host "🚀 启动Solana本地测试网" -ForegroundColor Green
Write-Host "=".repeat(60)

# 1. 检查Solana CLI是否安装
Write-Host "`n🔍 步骤1: 检查Solana CLI..." -ForegroundColor Cyan
try {
    $solanaVersion = solana --version
    Write-Host "✅ Solana CLI已安装: $solanaVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Solana CLI未安装" -ForegroundColor Red
    Write-Host "   请先安装Solana CLI: https://docs.solana.com/cli/install-solana-cli-tools" -ForegroundColor Yellow
    exit 1
}

# 2. 检查Anchor是否安装
Write-Host "`n🔍 步骤2: 检查Anchor..." -ForegroundColor Cyan
try {
    $anchorVersion = anchor --version
    Write-Host "✅ Anchor已安装: $anchorVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Anchor未安装" -ForegroundColor Red
    Write-Host "   请先安装Anchor: https://www.anchor-lang.com/docs/installation" -ForegroundColor Yellow
    exit 1
}

# 3. 停止可能存在的本地测试网
Write-Host "`n🛑 步骤3: 停止现有本地测试网..." -ForegroundColor Cyan
try {
    solana-test-validator --reset 2>&1 | Out-Null
    Write-Host "✅ 已停止现有测试网" -ForegroundColor Green
} catch {
    Write-Host "⚠️  无法停止测试网: $_" -ForegroundColor Yellow
}

# 4. 启动本地测试网
Write-Host "`n🌐 步骤4: 启动本地测试网..." -ForegroundColor Cyan
Write-Host "   启动本地测试网（带日志）..." -ForegroundColor Yellow

# 启动测试网（后台进程）
$testnetProcess = Start-Process -NoNewWindow -PassThru -FilePath "solana-test-validator" -ArgumentList "--reset", "--log"

# 等待测试网启动
Write-Host "   等待测试网启动..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# 5. 配置本地网络
Write-Host "`n⚙️  步骤5: 配置本地网络..." -ForegroundColor Cyan
solana config set --url http://localhost:8899
Write-Host "✅ 已配置本地网络: http://localhost:8899" -ForegroundColor Green

# 6. 创建测试钱包
Write-Host "`n💰 步骤6: 创建测试钱包..." -ForegroundColor Cyan
$walletPath = "test-wallet.json"
if (-not (Test-Path $walletPath)) {
    solana-keygen new --outfile $walletPath --no-passphrase --force
    Write-Host "✅ 已创建测试钱包: $walletPath" -ForegroundColor Green
} else {
    Write-Host "✅ 测试钱包已存在: $walletPath" -ForegroundColor Green
}

# 设置默认钱包
solana config set --keypair $walletPath
Write-Host "✅ 已设置默认钱包" -ForegroundColor Green

# 7. 获取测试SOL
Write-Host "`n💸 步骤7: 获取测试SOL..." -ForegroundColor Cyan
$balance = solana balance
Write-Host "   当前余额: $balance" -ForegroundColor Yellow

if ($balance -eq "0 SOL") {
    Write-Host "   请求空投..." -ForegroundColor Yellow
    solana airdrop 100
    Start-Sleep -Seconds 2
    $balance = solana balance
    Write-Host "   新余额: $balance" -ForegroundColor Green
}

# 8. 构建智能合约
Write-Host "`n🔨 步骤8: 构建智能体注册程序..." -ForegroundColor Cyan
try {
    Set-Location "."
    anchor build
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ 构建成功!" -ForegroundColor Green
    } else {
        Write-Host "❌ 构建失败" -ForegroundColor Red
        Stop-Process -Id $testnetProcess.Id -Force
        exit 1
    }
} catch {
    Write-Host "❌ 构建错误: $_" -ForegroundColor Red
    Stop-Process -Id $testnetProcess.Id -Force
    exit 1
}

# 9. 获取程序ID
Write-Host "`n📝 步骤9: 获取程序ID..." -ForegroundColor Cyan
$programId = solana address -k target/deploy/solana_oracle-keypair.json
Write-Host "   程序ID: $programId" -ForegroundColor Yellow

# 10. 更新程序ID
Write-Host "`n🔄 步骤10: 更新程序ID..." -ForegroundColor Cyan
$libRsPath = "programs\solana-oracle\src\lib.rs"
$content = Get-Content $libRsPath -Raw
$updatedContent = $content -replace 'declare_id\(".*"\)', "declare_id(`"$programId`")"
Set-Content $libRsPath -Value $updatedContent
Write-Host "✅ 已更新程序ID" -ForegroundColor Green

# 11. 重新构建
Write-Host "`n🔨 步骤11: 重新构建..." -ForegroundColor Cyan
anchor build
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ 重新构建成功!" -ForegroundColor Green
} else {
    Write-Host "❌ 重新构建失败" -ForegroundColor Red
    Stop-Process -Id $testnetProcess.Id -Force
    exit 1
}

# 12. 部署到本地测试网
Write-Host "`n🚀 步骤12: 部署到本地测试网..." -ForegroundColor Cyan
Write-Host "   部署中，请稍候..." -ForegroundColor Yellow

try {
    $deployOutput = anchor deploy 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ 部署成功!" -ForegroundColor Green
        Write-Host "   程序地址: $programId" -ForegroundColor Cyan
    } else {
        Write-Host "❌ 部署失败" -ForegroundColor Red
        Write-Host "   错误信息:" -ForegroundColor Red
        Write-Host $deployOutput -ForegroundColor Red
        Stop-Process -Id $testnetProcess.Id -Force
        exit 1
    }
} catch {
    Write-Host "❌ 部署错误: $_" -ForegroundColor Red
    Stop-Process -Id $testnetProcess.Id -Force
    exit 1
}

# 13. 更新Anchor.toml
Write-Host "`n📋 步骤13: 更新Anchor.toml..." -ForegroundColor Cyan
$anchorTomlPath = "Anchor.toml"
$anchorContent = Get-Content $anchorTomlPath -Raw
$updatedAnchorContent = $anchorContent -replace 'solana_oracle = ".*"', "solana_oracle = `"$programId`""
Set-Content $anchorTomlPath -Value $updatedAnchorContent
Write-Host "✅ 已更新Anchor.toml" -ForegroundColor Green

# 14. 验证部署
Write-Host "`n🔍 步骤14: 验证部署..." -ForegroundColor Cyan
try {
    $programInfo = solana program show $programId
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ 程序验证成功!" -ForegroundColor Green
        Write-Host $programInfo -ForegroundColor Yellow
    } else {
        Write-Host "⚠️  程序验证失败" -ForegroundColor Yellow
    }
} catch {
    Write-Host "⚠️  验证错误: $_" -ForegroundColor Yellow
}

# 15. 创建测试脚本
Write-Host "`n🧪 步骤15: 创建测试脚本..." -ForegroundColor Cyan
$testScript = @"
# 本地测试网智能体注册测试脚本
# 程序ID: $programId
# 测试钱包: $walletPath

Write-Host "🧪 智能体注册测试" -ForegroundColor Green
Write-Host "=".repeat(50)

# 1. 检查程序状态
Write-Host "`n🔍 检查程序状态..." -ForegroundColor Cyan
solana program show $programId

# 2. 运行测试
Write-Host "`n🚀 运行测试..." -ForegroundColor Cyan
anchor test

# 3. 运行演示
Write-Host "`n🎮 运行演示..." -ForegroundColor Cyan
Write-Host "   使用以下命令运行演示:" -ForegroundColor Yellow
Write-Host "   cargo run --example solana_demo" -ForegroundColor White
"@

Set-Content -Path "test_local_network.ps1" -Value $testScript
Write-Host "✅ 已创建测试脚本: test_local_network.ps1" -ForegroundColor Green

# 16. 显示成功信息
Write-Host "`n" + "=".repeat(60)
Write-Host "🎉 Solana本地测试网启动完成!" -ForegroundColor Green
Write-Host "`n📋 本地测试网信息:" -ForegroundColor Cyan
Write-Host "   RPC端点: http://localhost:8899" -ForegroundColor Yellow
Write-Host "   WebSocket: ws://localhost:8900" -ForegroundColor Yellow
Write-Host "   程序ID: $programId" -ForegroundColor Yellow
Write-Host "   测试钱包: $walletPath" -ForegroundColor Yellow
Write-Host "   当前余额: $balance" -ForegroundColor Yellow

Write-Host "`n🚀 下一步操作:" -ForegroundColor Cyan
Write-Host "   1. 运行测试: .\test_local_network.ps1" -ForegroundColor White
Write-Host "   2. 运行演示: cargo run --example solana_demo" -ForegroundColor White
Write-Host "   3. 停止测试网: Stop-Process -Id $($testnetProcess.Id)" -ForegroundColor White
Write-Host "   4. 查看日志: solana logs" -ForegroundColor White

Write-Host "`n💡 提示:" -ForegroundColor Cyan
Write-Host "   - 保持此窗口运行以维持本地测试网" -ForegroundColor Yellow
Write-Host "   - 在新窗口中运行测试和演示" -ForegroundColor Yellow
Write-Host "   - 按Ctrl+C停止测试网" -ForegroundColor Yellow

# 保存进程ID
$processInfo = @{
    ProcessId = $testnetProcess.Id
    ProgramId = $programId
    WalletPath = $walletPath
    StartTime = Get-Date
} | ConvertTo-Json

Set-Content -Path "testnet_info.json" -Value $processInfo
Write-Host "`n📁 测试网信息已保存到: testnet_info.json" -ForegroundColor Green
