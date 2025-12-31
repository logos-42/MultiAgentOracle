# 本地网络部署脚本
Write-Host "🚀 开始本地网络部署" -ForegroundColor Green

# 设置环境变量
$env:HOME = $env:USERPROFILE
$env:PATH = "$env:PATH;C:\Users\$env:USERNAME\.cargo\bin"

# 检查Anchor是否安装
Write-Host "📋 检查工具..." -ForegroundColor Yellow
try {
    $anchorVersion = anchor --version
    Write-Host "✅ Anchor版本: $anchorVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Anchor未安装或不在PATH中" -ForegroundColor Red
    exit 1
}

# 检查Solana是否安装
try {
    $solanaVersion = solana --version
    Write-Host "✅ Solana版本: $solanaVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Solana未安装或不在PATH中" -ForegroundColor Red
    exit 1
}

# 启动本地测试网络
Write-Host "🌐 启动本地测试网络..." -ForegroundColor Yellow
try {
    # 停止可能正在运行的本地网络
    Write-Host "  停止现有本地网络..." -ForegroundColor Gray
    solana-test-validator --reset 2>&1 | Out-Null
    
    # 启动本地验证器
    Write-Host "  启动本地验证器..." -ForegroundColor Gray
    Start-Process -NoNewWindow -FilePath "solana-test-validator" -ArgumentList "--reset" -PassThru
    
    # 等待验证器启动
    Write-Host "  等待验证器启动..." -ForegroundColor Gray
    Start-Sleep -Seconds 5
    
    # 设置本地网络配置
    Write-Host "  配置本地网络..." -ForegroundColor Gray
    solana config set --url http://localhost:8899
    
    # 创建测试钱包
    Write-Host "  创建测试钱包..." -ForegroundColor Gray
    if (-not (Test-Path "test-wallet.json")) {
        solana-keygen new --outfile test-wallet.json --no-passphrase
    }
    solana config set --keypair test-wallet.json
    
    # 获取测试SOL
    Write-Host "  获取测试SOL..." -ForegroundColor Gray
    solana airdrop 10
    
    Write-Host "✅ 本地测试网络启动成功" -ForegroundColor Green
} catch {
    Write-Host "❌ 启动本地测试网络失败: $_" -ForegroundColor Red
    exit 1
}

# 构建智能合约
Write-Host "🔨 构建智能合约..." -ForegroundColor Yellow
try {
    anchor build
    Write-Host "✅ 智能合约构建成功" -ForegroundColor Green
} catch {
    Write-Host "❌ 智能合约构建失败: $_" -ForegroundColor Red
    exit 1
}

# 部署智能合约
Write-Host "🚀 部署智能合约到本地网络..." -ForegroundColor Yellow
try {
    anchor deploy
    Write-Host "✅ 智能合约部署成功" -ForegroundColor Green
    
    # 获取部署的程序ID
    $programId = (Get-Content "target/deploy/solana_oracle-keypair.json" | ConvertFrom-Json).pubkey
    Write-Host "📋 程序ID: $programId" -ForegroundColor Cyan
    
    # 更新配置文件
    Write-Host "📝 更新配置文件..." -ForegroundColor Gray
    $anchorToml = Get-Content "Anchor.toml" -Raw
    $anchorToml = $anchorToml -replace 'solana_oracle = ".*?"', "solana_oracle = `"$programId`""
    $anchorToml | Set-Content "Anchor.toml"
    
    Write-Host "✅ 配置文件更新完成" -ForegroundColor Green
} catch {
    Write-Host "❌ 智能合约部署失败: $_" -ForegroundColor Red
    exit 1
}

# 运行测试
Write-Host "🧪 运行智能合约测试..." -ForegroundColor Yellow
try {
    anchor test
    Write-Host "✅ 智能合约测试通过" -ForegroundColor Green
} catch {
    Write-Host "❌ 智能合约测试失败: $_" -ForegroundColor Red
    exit 1
}

Write-Host "🎉 本地网络部署完成！" -ForegroundColor Green
Write-Host "📋 部署信息:" -ForegroundColor Cyan
Write-Host "   网络: http://localhost:8899" -ForegroundColor Gray
Write-Host "   程序ID: $programId" -ForegroundColor Gray
Write-Host "   钱包: test-wallet.json" -ForegroundColor Gray
Write-Host "   余额: $(solana balance) SOL" -ForegroundColor Gray

Write-Host "`n🚀 下一步:" -ForegroundColor Yellow
Write-Host "   1. 更新Rust项目中的程序ID" -ForegroundColor Gray
Write-Host "   2. 运行集成测试" -ForegroundColor Gray
Write-Host "   3. 部署到devnet/testnet" -ForegroundColor Gray
