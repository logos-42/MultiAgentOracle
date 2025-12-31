# Devnet实际部署指南

## 🚀 概述
由于Windows权限问题导致本地测试网启动失败，我们可以直接部署到Solana Devnet进行实际测试。Devnet是Solana的公共测试网络，不需要本地权限。

## 📋 前提条件

### 已满足的条件
1. ✅ Solana CLI 1.18.26 已安装
2. ✅ Anchor 0.32.1 已安装  
3. ✅ 环境变量已配置
4. ✅ 项目文件完整
5. ✅ 4个测试智能体配置就绪

### 需要准备
1. Devnet SOL（测试代币）
2. 部署钱包
3. 网络连接

## 🔧 部署步骤

### 步骤1: 切换到Devnet
```powershell
# 配置Devnet网络
solana config set --url https://api.devnet.solana.com

# 检查当前配置
solana config get
```

### 步骤2: 检查Devnet连接
```powershell
# 检查Devnet状态
solana cluster-version

# 检查当前钱包余额
solana balance

# 如果没有SOL，请求空投
solana airdrop 1
```

### 步骤3: 构建项目
```powershell
# 设置环境变量（如果需要）
$env:HOME = $env:USERPROFILE

# 构建智能合约
anchor build
```

### 步骤4: 获取程序ID
```powershell
# 获取构建后的程序ID
$programId = solana address -k target/deploy/solana_oracle-keypair.json
Write-Host "程序ID: $programId"
```

### 步骤5: 更新源代码中的程序ID
```powershell
# 更新lib.rs中的程序ID
$libRsPath = "programs/solana-oracle/src/lib.rs"
$content = Get-Content $libRsPath -Raw
$updatedContent = $content -replace 'declare_id\(".*"\)', "declare_id(`"$programId`")"
Set-Content $libRsPath -Value $updatedContent
```

### 步骤6: 重新构建
```powershell
# 使用新程序ID重新构建
anchor build
```

### 步骤7: 部署到Devnet
```powershell
# 部署智能合约到Devnet
anchor deploy --provider.cluster devnet
```

### 步骤8: 验证部署
```powershell
# 检查程序是否已部署
solana program show $programId

# 查看部署详情
solana program show $programId --verbose
```

## 🤖 多智能体测试

### 测试智能体数据
我们已经准备了4个测试智能体：
1. **预言机核心节点** - did:example:oracle-core-001
2. **数据验证节点** - did:example:validator-002  
3. **数据提供节点** - did:example:data-provider-003
4. **轻量级网关** - did:example:gateway-004

### 测试脚本
```powershell
# 运行测试脚本
node test_simple.js

# 或运行完整测试
anchor test --provider.cluster devnet
```

## 📊 监控和验证

### 查看交易
```powershell
# 查看最近交易
solana transaction-history --limit 10

# 查看程序相关交易
solana program show $programId --transactions
```

### 浏览器查看
1. 打开Solana Explorer: https://explorer.solana.com
2. 切换到Devnet网络
3. 搜索你的程序ID: `$programId`
4. 查看交易和账户状态

## 🔧 故障排除

### 常见问题

#### 1. 余额不足
```powershell
# 请求更多SOL
solana airdrop 2

# 检查余额
solana balance
```

#### 2. 部署失败
```powershell
# 检查网络连接
solana cluster-version

# 检查gas费用
solana fees

# 重新尝试部署
anchor deploy --provider.cluster devnet --force
```

#### 3. 程序验证失败
```powershell
# 重新构建
anchor clean
anchor build

# 重新部署
anchor deploy --provider.cluster devnet
```

### 网络问题
如果Devnet连接有问题，可以尝试：
1. 使用不同的RPC端点
2. 检查防火墙设置
3. 等待网络恢复

## 🎯 集成到多智能体系统

### 配置更新
在 `MultiAgentOracle/config/local_test.toml` 中添加：

```toml
[solana]
program_id = "你的程序ID"
rpc_url = "https://api.devnet.solana.com"
cluster = "devnet"
enable_solana_integration = true
```

### Rust客户端集成
```rust
use crate::solana::client::SolanaClient;

// 创建Devnet客户端
let client = SolanaClient::new(
    "https://api.devnet.solana.com",
    "你的程序ID"
);

// 注册智能体
client.register_agent(did, public_key, metadata_uri).await?;
```

### 测试集成
```bash
# 运行集成测试
cargo test --test solana_integration

# 运行完整系统测试
cargo run --bin test_console -- --test-solana-devnet
```

## 📈 成本估算

### Devnet部署成本
- 程序部署: ~0.5-1 SOL（测试代币，免费获取）
- 智能体注册: ~0.01 SOL/每个
- 交易费用: 极低（测试网络）

### 获取测试SOL
```powershell
# 每次最多可获取2 SOL
solana airdrop 2

# 可以多次请求（有频率限制）
Start-Sleep -Seconds 30
solana airdrop 1
```

## 🚀 一键部署脚本

创建 `deploy_devnet.ps1`：
```powershell
# Devnet一键部署脚本
Write-Host "🚀 开始Devnet部署..." -ForegroundColor Green

# 1. 配置网络
solana config set --url https://api.devnet.solana.com

# 2. 获取测试SOL
solana airdrop 1
Start-Sleep -Seconds 5

# 3. 构建项目
anchor build

# 4. 部署
anchor deploy --provider.cluster devnet

Write-Host "🎉 部署完成!" -ForegroundColor Green
```

## 📞 支持资源

### 官方文档
- [Solana Devnet文档](https://docs.solana.com/clusters#devnet)
- [Anchor部署指南](https://www.anchor-lang.com/docs/deployment)
- [Solana Explorer](https://explorer.solana.com)

### 社区支持
- Solana Discord: #devnet 频道
- Anchor GitHub: Issues 页面
- Stack Overflow: solana 标签

### 监控工具
- Solana Beach: 实时网络监控
- Solscan: 交易浏览器
- Solana CLI: 本地监控工具

## 🎉 成功标准

### 部署成功标志
1. ✅ 程序成功部署到Devnet
2. ✅ 程序ID在Explorer中可查
3. ✅ 智能体注册交易成功
4. ✅ 交易在区块链上确认
5. ✅ 集成测试通过

### 验证方法
```powershell
# 验证程序状态
solana program show <程序ID>

# 验证交易
solana confirm <交易哈希>

# 验证账户
solana account <智能体账户地址>
```

## 💡 最佳实践

### 开发建议
1. **小步测试**: 先测试单个功能，再测试完整流程
2. **监控费用**: 注意测试SOL的使用情况
3. **备份密钥**: 妥善保管部署钱包
4. **版本控制**: 记录每次部署的程序ID

### 安全建议
1. 使用单独的测试钱包
2. 不要在主网使用测试密钥
3. 定期检查程序权限
4. 验证所有交易

---

**最后更新**: 2025-12-31  
**状态**: 🟢 准备部署  
**建议**: 立即开始Devnet部署测试

> 💡 **提示**: Devnet是真实的测试网络，交易会被广播到全球节点，但使用的是测试代币，没有实际价值。
