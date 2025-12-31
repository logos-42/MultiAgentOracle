# 快速启动Solana开发环境
# 用法: .\quick_start.ps1

Write-Host "🚀 启动Solana开发环境" -ForegroundColor Green
Write-Host "=".repeat(50)

# 1. 设置环境变量
. .\set_solana_env.ps1

# 2. 检查网络配置
Write-Host "
🌐 检查网络配置..." -ForegroundColor Cyan
solana config get

# 3. 启动本地测试网（如果需要）
Write-Host "
💡 提示: 要启动本地测试网，运行以下命令:" -ForegroundColor Yellow
Write-Host "   solana-test-validator --reset" -ForegroundColor White
Write-Host "   solana config set --url http://localhost:8899" -ForegroundColor White

# 4. 构建项目
Write-Host "
🔨 构建项目..." -ForegroundColor Cyan
Write-Host "   切换到项目目录后运行:" -ForegroundColor Yellow
Write-Host "   anchor build" -ForegroundColor White

Write-Host "
✅ 环境准备完成!" -ForegroundColor Green
