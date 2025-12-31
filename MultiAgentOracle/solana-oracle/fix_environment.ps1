# 修复Solana环境变量脚本
# 在Windows上，Solana需要HOME环境变量

Write-Host "🔧 修复Solana环境变量" -ForegroundColor Green
Write-Host "=".repeat(50)

# 1. 设置HOME环境变量
Write-Host "`n📝 设置HOME环境变量..." -ForegroundColor Cyan
$env:HOME = $env:USERPROFILE
Write-Host "   HOME = $env:HOME" -ForegroundColor Yellow

# 2. 设置PATH环境变量（如果需要）
Write-Host "`n🛠️  检查PATH环境变量..." -ForegroundColor Cyan
$solanaPath = "C:\Users\$env:USERNAME\.local\share\solana\install\active_release\bin"
if (Test-Path $solanaPath) {
    if ($env:PATH -notlike "*$solanaPath*") {
        $env:PATH = "$solanaPath;$env:PATH"
        Write-Host "   已添加Solana到PATH: $solanaPath" -ForegroundColor Green
    } else {
        Write-Host "   Solana已在PATH中" -ForegroundColor Green
    }
} else {
    Write-Host "   ⚠️  Solana安装路径未找到: $solanaPath" -ForegroundColor Yellow
}

# 3. 检查关键环境变量
Write-Host "`n🔍 检查关键环境变量..." -ForegroundColor Cyan
$envVars = @{
    "HOME" = $env:HOME
    "USERPROFILE" = $env:USERPROFILE
    "APPDATA" = $env:APPDATA
    "LOCALAPPDATA" = $env:LOCALAPPDATA
}

foreach ($key in $envVars.Keys) {
    Write-Host "   $key = $($envVars[$key])" -ForegroundColor Yellow
}

# 4. 创建配置文件目录
Write-Host "`n📁 创建配置文件目录..." -ForegroundColor Cyan
$solanaConfigDir = "$env:HOME\.config\solana"
if (-not (Test-Path $solanaConfigDir)) {
    New-Item -ItemType Directory -Path $solanaConfigDir -Force | Out-Null
    Write-Host "   已创建目录: $solanaConfigDir" -ForegroundColor Green
} else {
    Write-Host "   目录已存在: $solanaConfigDir" -ForegroundColor Green
}

# 5. 测试Solana命令
Write-Host "`n🧪 测试Solana命令..." -ForegroundColor Cyan
try {
    $solanaVersion = solana --version
    Write-Host "   ✅ Solana CLI工作正常: $solanaVersion" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Solana CLI测试失败: $_" -ForegroundColor Red
}

# 6. 测试Anchor命令
Write-Host "`n🧪 测试Anchor命令..." -ForegroundColor Cyan
try {
    $anchorVersion = anchor --version
    Write-Host "   ✅ Anchor工作正常: $anchorVersion" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Anchor测试失败: $_" -ForegroundColor Red
}

# 7. 创建永久环境变量设置脚本
Write-Host "`n💾 创建永久环境变量设置脚本..." -ForegroundColor Cyan
$permanentScript = @"
# 永久设置Solana环境变量
# 将此脚本添加到PowerShell Profile或每次运行前执行

# 设置HOME环境变量
`$env:HOME = `$env:USERPROFILE

# 添加Solana到PATH
`$solanaPath = "C:\Users\`$env:USERNAME\.local\share\solana\install\active_release\bin"
if (Test-Path `$solanaPath) {
    if (`$env:PATH -notlike "*`$solanaPath*") {
        `$env:PATH = "`$solanaPath;`$env:PATH"
    }
}

Write-Host "✅ Solana环境变量已设置" -ForegroundColor Green
"@

Set-Content -Path "set_solana_env.ps1" -Value $permanentScript
Write-Host "   已创建脚本: set_solana_env.ps1" -ForegroundColor Green

# 8. 创建快速启动脚本
Write-Host "`n🚀 创建快速启动脚本..." -ForegroundColor Cyan
$quickStartScript = @"
# 快速启动Solana开发环境
# 用法: .\quick_start.ps1

Write-Host "🚀 启动Solana开发环境" -ForegroundColor Green
Write-Host "=".repeat(50)

# 1. 设置环境变量
. .\set_solana_env.ps1

# 2. 检查网络配置
Write-Host "`n🌐 检查网络配置..." -ForegroundColor Cyan
solana config get

# 3. 启动本地测试网（如果需要）
Write-Host "`n💡 提示: 要启动本地测试网，运行以下命令:" -ForegroundColor Yellow
Write-Host "   solana-test-validator --reset" -ForegroundColor White
Write-Host "   solana config set --url http://localhost:8899" -ForegroundColor White

# 4. 构建项目
Write-Host "`n🔨 构建项目..." -ForegroundColor Cyan
Write-Host "   切换到项目目录后运行:" -ForegroundColor Yellow
Write-Host "   anchor build" -ForegroundColor White

Write-Host "`n✅ 环境准备完成!" -ForegroundColor Green
"@

Set-Content -Path "quick_start.ps1" -Value $quickStartScript
Write-Host "   已创建脚本: quick_start.ps1" -ForegroundColor Green

Write-Host "`n" + "=".repeat(50)
Write-Host "🎉 环境变量修复完成!" -ForegroundColor Green
Write-Host "`n📋 下一步操作:" -ForegroundColor Cyan
Write-Host "   1. 运行快速启动: .\quick_start.ps1" -ForegroundColor White
Write-Host "   2. 或手动设置环境: .\set_solana_env.ps1" -ForegroundColor White
Write-Host "   3. 然后构建项目: anchor build" -ForegroundColor White
Write-Host "`n💡 提示: 将这些命令添加到PowerShell Profile以永久生效" -ForegroundColor Yellow
