# Solana本地测试网部署指南

## 🚀 快速开始

### 1. 环境准备

```powershell
# 设置环境变量
$env:HOME = $env:USERPROFILE

# 检查工具版本
solana --version
anchor --version
```

### 2. 启动本地测试网

```powershell
# 停止现有测试网
Get-Process solana-test-validator -ErrorAction SilentlyContinue | Stop-Process -Force

# 启动本地测试网
solana-test-validator --reset

# 等待10秒让测试网完全启动
Start-Sleep -Seconds 10

# 配置本地网络
solana config set --url http://localhost:8899
```

### 3. 检查测试网状态

```powershell
# 检查集群版本
solana cluster-version

# 检查余额
solana balance

# 如果余额为0，请求空投
solana airdrop 100
```

## 📋 智能体注册程序信息

### 程序ID
```
DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b
```

### 合约功能
1. **注册智能体** - `register_agent(did, public_key, metadata_uri)`
2. **更新身份** - `update_identity(new_public_key, new_metadata_uri)`
3. **请求验证** - `request_verification(proof_data)`
4. **批准验证** - `approve_verification()`
5. **更新声誉** - `update_reputation(delta)`
6. **停用身份** - `deactivate_identity()`
7. **重新激活** - `reactivate_identity()`

## 🤖 多智能体测试配置

### 配置文件: `multi_agent_config.yaml`

包含4个测试智能体：
1. **预言机核心节点** - 声誉850，核心层
2. **数据验证节点** - 声誉650，验证层  
3. **数据提供节点** - 声誉350，数据层
4. **轻量级网关** - 声誉200，网关层

### 运行JavaScript测试

```bash
node test_agent.js
```

### 运行Rust测试

```bash
# 编译测试客户端
rustc test_client.rs

# 运行测试
.\test_client.exe
```

## 🔧 故障排除

### 常见问题

#### 1. 测试网启动失败
```powershell
# 检查端口占用
netstat -ano | findstr :8899

# 使用不同端口
solana-test-validator --reset --rpc-port 8900 --faucet-port 8901
```

#### 2. 编译权限问题
```powershell
# 以管理员身份运行PowerShell
# 或使用WSL2/Linux环境
```

#### 3. 网络连接问题
```powershell
# 检查防火墙设置
# 确保可以访问外部网络下载依赖
```

#### 4. 环境变量问题
```powershell
# 运行修复脚本
.\fix_environment.ps1

# 或手动设置
$env:HOME = $env:USERPROFILE
```

## 🧪 测试场景

### 场景1: 单个智能体注册
```javascript
// 注册单个智能体
register_agent(
    "did:example:agent001",
    [1,2,3,...,32], // 32字节公钥
    "https://ipfs.io/ipfs/QmExample"
)
```

### 场景2: 多智能体批量注册
```javascript
// 批量注册4个智能体
agents.forEach(agent => {
    register_agent(
        agent.did,
        agent.public_key,
        agent.metadata_uri
    )
})
```

### 场景3: 智能体交互
```javascript
// 智能体A请求验证
request_verification(proof_data)

// 智能体B批准验证  
approve_verification()

// 系统更新声誉
update_reputation(+50) // 奖励
update_reputation(-20) // 惩罚
```

### 场景4: 身份管理
```javascript
// 更新身份信息
update_identity(new_public_key, new_metadata_uri)

// 停用身份
deactivate_identity()

// 重新激活
reactivate_identity()
```

## 📊 监控和调试

### 查看日志
```powershell
# 查看测试网日志
solana logs

# 查看特定程序日志
solana logs --program DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b
```

### 检查账户状态
```powershell
# 查看程序账户
solana program show DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b

# 查看智能体账户
solana account <智能体公钥>
```

### 交易历史
```powershell
# 查看最近交易
solana transaction-history --limit 10
```

## 🔗 与多智能体预言机系统集成

### 1. 配置集成
在 `MultiAgentOracle/config/local_test.toml` 中添加：

```toml
[solana]
program_id = "DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"
rpc_url = "http://localhost:8899"
cluster = "localnet"
enable_solana_integration = true
```

### 2. Rust客户端集成
```rust
use crate::solana::client::SolanaClient;

let client = SolanaClient::new(
    "http://localhost:8899",
    "DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"
);

// 注册智能体
client.register_agent(did, public_key, metadata_uri).await?;
```

### 3. 测试集成
```bash
# 运行集成测试
cargo test --test solana_integration

# 运行完整系统测试
cargo run --bin test_console -- --test-solana
```

## 🎯 下一步计划

### 短期目标
1. ✅ 创建本地测试网环境
2. ✅ 配置多智能体测试
3. 🔄 测试智能体注册功能
4. 🔄 集成到多智能体系统
5. 🔄 性能测试和优化

### 中期目标
1. 添加更多智能体功能
2. 实现跨链身份验证
3. 集成DIAP身份系统
4. 添加治理机制

### 长期目标
1. 部署到测试网
2. 主网部署
3. 生态系统扩展
4. 社区治理

## 📞 支持

### 问题反馈
1. 检查日志文件
2. 查看错误信息
3. 搜索常见问题
4. 提交Issue

### 资源链接
- [Solana文档](https://docs.solana.com/)
- [Anchor框架](https://www.anchor-lang.com/)
- [多智能体预言机项目](../README.md)

---

**最后更新**: 2025-12-31  
**版本**: 1.0.0  
**状态**: 🟢 可运行
