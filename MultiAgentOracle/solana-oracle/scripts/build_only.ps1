# 仅构建脚本（不需要管理员权限）
Write-Host "🔨 开始构建智能合约" -ForegroundColor Green

# 设置环境变量
$env:HOME = $env:USERPROFILE

# 检查当前目录
Write-Host "📁 当前目录: $(Get-Location)" -ForegroundColor Yellow

# 检查文件
Write-Host "📋 检查项目文件..." -ForegroundColor Yellow
if (Test-Path "Anchor.toml") {
    Write-Host "✅ 找到Anchor.toml" -ForegroundColor Green
} else {
    Write-Host "❌ 未找到Anchor.toml" -ForegroundColor Red
    exit 1
}

if (Test-Path "programs/solana-oracle/src/lib.rs") {
    Write-Host "✅ 找到智能合约源代码" -ForegroundColor Green
} else {
    Write-Host "❌ 未找到智能合约源代码" -ForegroundColor Red
    exit 1
}

# 尝试构建
Write-Host "🔨 尝试构建智能合约..." -ForegroundColor Yellow
try {
    # 先清理
    Write-Host "  清理构建缓存..." -ForegroundColor Gray
    Remove-Item -Path "target" -Recurse -Force -ErrorAction SilentlyContinue
    
    # 构建
    Write-Host "  开始构建..." -ForegroundColor Gray
    anchor build
    
    Write-Host "✅ 智能合约构建成功！" -ForegroundColor Green
    
    # 显示构建结果
    if (Test-Path "target/deploy/solana_oracle.so") {
        $fileSize = (Get-Item "target/deploy/solana_oracle.so").Length / 1MB
        Write-Host "📦 构建结果:" -ForegroundColor Cyan
        Write-Host "   程序文件: target/deploy/solana_oracle.so" -ForegroundColor Gray
        Write-Host "   文件大小: $fileSize MB" -ForegroundColor Gray
        
        # 显示程序ID
        if (Test-Path "target/deploy/solana_oracle-keypair.json") {
            $keypair = Get-Content "target/deploy/solana_oracle-keypair.json" | ConvertFrom-Json
            Write-Host "   程序公钥: $($keypair.pubkey)" -ForegroundColor Gray
        }
    }
    
} catch {
    Write-Host "❌ 构建失败: $_" -ForegroundColor Red
    
    # 尝试替代构建方法
    Write-Host "🔄 尝试替代构建方法..." -ForegroundColor Yellow
    try {
        # 使用cargo直接构建
        Write-Host "  使用cargo构建..." -ForegroundColor Gray
        cd programs/solana-oracle
        cargo build-sbf --sbf-out-dir ../../target/deploy
        
        Write-Host "✅ 使用cargo构建成功！" -ForegroundColor Green
    } catch {
        Write-Host "❌ 所有构建方法都失败了" -ForegroundColor Red
        Write-Host "💡 建议:" -ForegroundColor Yellow
        Write-Host "   1. 检查Rust工具链: rustup toolchain list" -ForegroundColor Gray
        Write-Host "   2. 安装Solana工具链: solana-install init" -ForegroundColor Gray
        Write-Host "   3. 检查环境变量" -ForegroundColor Gray
        exit 1
    }
}

Write-Host "🎉 构建完成！" -ForegroundColor Green
Write-Host "`n🚀 下一步:" -ForegroundColor Yellow
Write-Host "   1. 启动本地测试网络: solana-test-validator" -ForegroundColor Gray
Write-Host "   2. 部署智能合约: anchor deploy" -ForegroundColor Gray
Write-Host "   3. 运行测试: anchor test" -ForegroundColor Gray
