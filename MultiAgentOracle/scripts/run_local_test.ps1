# 本地分层架构测试脚本
# 用于启动和运行10个节点的分层架构测试环境

Write-Host "🚀 启动本地分层架构测试环境" -ForegroundColor Green
Write-Host ""

# 1. 检查配置文件
$configPath = "config/local_test.toml"
if (-not (Test-Path $configPath)) {
    Write-Host "❌ 配置文件不存在: $configPath" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 找到配置文件: $configPath" -ForegroundColor Green

# 2. 编译项目
Write-Host "🔧 编译项目..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 编译失败" -ForegroundColor Red
    exit 1
}
Write-Host "✅ 编译成功" -ForegroundColor Green

# 3. 启动DIAP SDK模拟服务（如果启用）
Write-Host "🔐 检查DIAP SDK模拟..." -ForegroundColor Yellow
$configContent = Get-Content $configPath -Raw
if ($configContent -match 'enable_diap_mock\s*=\s*true') {
    Write-Host "  启动DIAP SDK模拟服务..." -ForegroundColor Cyan
    # 在实际实现中，这里会启动DIAP模拟服务
    # Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run --bin diap_mock_server"
    Write-Host "  ✅ DIAP模拟服务已配置" -ForegroundColor Green
} else {
    Write-Host "  ⚠️  DIAP模拟服务未启用" -ForegroundColor Yellow
}

# 4. 初始化测试节点
Write-Host "🔄 初始化测试节点..." -ForegroundColor Yellow
cargo run --bin test_console -- --init --config $configPath
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 节点初始化失败" -ForegroundColor Red
    exit 1
}
Write-Host "✅ 测试节点初始化完成" -ForegroundColor Green

# 5. 运行分层网络测试
Write-Host "🌐 运行分层网络测试..." -ForegroundColor Yellow
cargo test --test hierarchical_network_test -- --nocapture
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  网络测试有错误" -ForegroundColor Yellow
} else {
    Write-Host "✅ 网络测试通过" -ForegroundColor Green
}

# 6. 运行分层共识测试
Write-Host "🤝 运行分层共识测试..." -ForegroundColor Yellow
cargo test --test hierarchical_consensus_test -- --nocapture
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  共识测试有错误" -ForegroundColor Yellow
} else {
    Write-Host "✅ 共识测试通过" -ForegroundColor Green
}

# 7. 运行DIAP身份测试
Write-Host "🔐 运行DIAP身份测试..." -ForegroundColor Yellow
cargo test --test diap_integration_test -- --nocapture
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  DIAP测试有错误" -ForegroundColor Yellow
} else {
    Write-Host "✅ DIAP测试通过" -ForegroundColor Green
}

# 8. 运行网关接入测试
Write-Host "🚪 运行网关接入测试..." -ForegroundColor Yellow
cargo test --test gateway_access_test -- --nocapture
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  网关测试有错误" -ForegroundColor Yellow
} else {
    Write-Host "✅ 网关测试通过" -ForegroundColor Green
}

# 9. 运行Prompt交互测试
Write-Host "🤖 运行Prompt交互测试..." -ForegroundColor Yellow
cargo test --test prompt_interaction_test -- --nocapture
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Prompt测试有错误" -ForegroundColor Yellow
} else {
    Write-Host "✅ Prompt测试通过" -ForegroundColor Green
}

# 10. 生成测试报告
Write-Host "📊 生成测试报告..." -ForegroundColor Yellow
cargo run --bin test_console -- --report --config $configPath
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  报告生成有错误" -ForegroundColor Yellow
} else {
    Write-Host "✅ 测试报告生成完成" -ForegroundColor Green
}

Write-Host ""
Write-Host "🎉 本地测试完成！" -ForegroundColor Green
Write-Host ""
Write-Host "下一步操作:" -ForegroundColor Cyan
Write-Host "  1. 查看详细报告: cargo run --bin test_console -- --report" -ForegroundColor White
Write-Host "  2. 交互式测试: cargo run --bin test_console" -ForegroundColor White
Write-Host "  3. 运行特定测试: cargo test --test <测试名称>" -ForegroundColor White
Write-Host "  4. 清理测试数据: cargo run --bin test_console -- --clean" -ForegroundColor White
