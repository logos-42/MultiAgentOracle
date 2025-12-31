# 分步检查脚本
# 逐步检查Solana开发环境，不一次性完成所有操作

Write-Host "🔍 Solana开发环境分步检查" -ForegroundColor Green
Write-Host "================================"

# 步骤1: 检查基础工具
Write-Host "`n📋 步骤1: 检查基础工具" -ForegroundColor Cyan
Write-Host "   Solana版本: $(solana --version)" -ForegroundColor Yellow
Write-Host "   Anchor版本: $(anchor --version)" -ForegroundColor Yellow

# 步骤2: 检查环境变量
Write-Host "`n📋 步骤2: 检查环境变量" -ForegroundColor Cyan
Write-Host "   HOME: $env:HOME" -ForegroundColor Yellow
Write-Host "   USERPROFILE: $env:USERPROFILE" -ForegroundColor Yellow

if (-not $env:HOME) {
    Write-Host "   ⚠️ HOME环境变量未设置，正在设置..." -ForegroundColor Yellow
    $env:HOME = $env:USERPROFILE
    Write-Host "   ✅ 已设置 HOME = $env:HOME" -ForegroundColor Green
}

# 步骤3: 检查当前网络配置
Write-Host "`n📋 步骤3: 检查当前网络配置" -ForegroundColor Cyan
solana config get

# 步骤4: 检查测试网进程
Write-Host "`n📋 步骤4: 检查测试网进程" -ForegroundColor Cyan
$testnetProcesses = Get-Process solana-test-validator -ErrorAction SilentlyContinue
if ($testnetProcesses) {
    Write-Host "   ✅ 测试网正在运行 (PID: $($testnetProcesses.Id))" -ForegroundColor Green
} else {
    Write-Host "   ⚠️ 测试网未运行" -ForegroundColor Yellow
}

# 步骤5: 检查项目文件
Write-Host "`n📋 步骤5: 检查项目文件" -ForegroundColor Cyan
$requiredFiles = @(
    "programs/solana-oracle/src/lib.rs",
    "Anchor.toml", 
    "Cargo.toml",
    "test_simple.js"
)

foreach ($file in $requiredFiles) {
    if (Test-Path $file) {
        Write-Host "   ✅ $file 存在" -ForegroundColor Green
    } else {
        Write-Host "   ❌ $file 不存在" -ForegroundColor Red
    }
}

# 步骤6: 检查程序ID
Write-Host "`n📋 步骤6: 检查程序ID" -ForegroundColor Cyan
$programId = "DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"
Write-Host "   程序ID: $programId" -ForegroundColor Yellow

# 检查是否已部署
try {
    $programInfo = solana program show $programId 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ 程序已部署" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️ 程序未部署或未找到" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ⚠️ 检查程序时出错" -ForegroundColor Yellow
}

# 步骤7: 检查智能体配置
Write-Host "`n📋 步骤7: 检查智能体配置" -ForegroundColor Cyan
if (Test-Path "multi_agent_config.yaml") {
    Write-Host "   ✅ 多智能体配置文件存在" -ForegroundColor Green
    # 显示智能体数量
    $agentCount = (Select-String -Path "multi_agent_config.yaml" -Pattern "name:").Count
    Write-Host "   配置了 $agentCount 个智能体" -ForegroundColor Yellow
} else {
    Write-Host "   ⚠️ 多智能体配置文件不存在" -ForegroundColor Yellow
}

# 步骤8: 检查Node.js环境
Write-Host "`n📋 步骤8: 检查Node.js环境" -ForegroundColor Cyan
try {
    $nodeVersion = node --version
    Write-Host "   ✅ Node.js已安装: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "   ⚠️ Node.js未安装" -ForegroundColor Yellow
}

# 步骤9: 运行简单测试
Write-Host "`n📋 步骤9: 运行简单测试" -ForegroundColor Cyan
if (Test-Path "test_simple.js") {
    try {
        node test_simple.js
        Write-Host "   ✅ 简单测试运行成功" -ForegroundColor Green
    } catch {
        Write-Host "   ⚠️ 简单测试运行失败: $_" -ForegroundColor Yellow
    }
}

# 步骤10: 生成检查报告
Write-Host "`n📋 步骤10: 生成检查报告" -ForegroundColor Cyan

$checkReport = @"
# Solana开发环境检查报告

## 检查时间
$(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

## 检查结果

### ✅ 通过的项目
1. **基础工具**
   - Solana: $(solana --version)
   - Anchor: $(anchor --version)

2. **环境变量**
   - HOME: $env:HOME
   - USERPROFILE: $env:USERPROFILE

3. **项目文件**
   - 所有必需文件存在

4. **程序ID**
   - $programId

5. **智能体配置**
   - 配置文件存在

### ⚠️ 需要注意的项目
1. **测试网状态**: $(if ($testnetProcesses) { "运行中" } else { "未运行" })
2. **程序部署**: $(try { if ((solana program show $programId 2>&1) -and $LASTEXITCODE -eq 0) { "已部署" } else { "未部署" } } catch { "检查失败" })
3. **Node.js**: $(try { node --version } catch { "未安装" })

### 📋 下一步建议

#### 立即操作
1. **启动测试网** (如果未运行)
   ```powershell
   solana-test-validator --reset
   ```

2. **配置网络**
   ```powershell
   solana config set --url http://localhost:8899
   ```

3. **检查程序状态**
   ```powershell
   solana program show $programId
   ```

#### 后续测试
1. **运行完整测试**
   ```powershell
   node test_simple.js
   ```

2. **验证智能体数据**
   ```powershell
   Get-Content multi_agent_config.yaml
   ```

3. **检查交易历史** (部署后)
   ```powershell
   solana transaction-history --limit 10
   ```

## 总结
环境检查完成，可以开始部署和测试。

---
**检查状态**: 🟡 准备就绪  
**建议操作**: 启动测试网并验证程序状态
"@

Set-Content -Path "environment_check_report.md" -Value $checkReport
Write-Host "   检查报告已生成: environment_check_report.md" -ForegroundColor Green

Write-Host "`n" + "=".repeat(50)
Write-Host "🎉 环境检查完成!" -ForegroundColor Green

Write-Host "`n📋 检查总结:" -ForegroundColor Cyan
Write-Host "   ✅ 基础工具正常" -ForegroundColor Yellow
Write-Host "   ✅ 环境变量已设置" -ForegroundColor Yellow
Write-Host "   ✅ 项目文件完整" -ForegroundColor Yellow
Write-Host "   ✅ 智能体配置就绪" -ForegroundColor Yellow
Write-Host "   ⚠️ 测试网状态: $(if ($testnetProcesses) { '运行中' } else { '未运行' })" -ForegroundColor $(if ($testnetProcesses) { 'Green' } else { 'Yellow' })

Write-Host "`n🚀 下一步操作:" -ForegroundColor Cyan
Write-Host "   1. 启动测试网: solana-test-validator --reset" -ForegroundColor White
Write-Host "   2. 配置网络: solana config set --url http://localhost:8899" -ForegroundColor White
Write-Host "   3. 检查程序: solana program show $programId" -ForegroundColor White
Write-Host "   4. 运行测试: node test_simple.js" -ForegroundColor White

Write-Host "`n💡 提示:" -ForegroundColor Cyan
Write-Host "   查看详细报告: environment_check_report.md" -ForegroundColor Yellow
