# WSL Solana开发环境安装路径记录

## 📅 安装时间
2025年12月31日

## 🖥️ 系统环境
- **操作系统**: Windows 11 + WSL2
- **WSL发行版**: Ubuntu 24.04.3 LTS
- **用户名**: logos
- **项目路径**: `/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle`

## 🛠️ 工具安装路径

### 1. Rust 工具链
```
安装路径: /home/logos/.cargo/
二进制文件: /home/logos/.cargo/bin/
├── rustc -> rustup (符号链接)
├── cargo
├── rustup
└── anchor -> /home/logos/.avm/bin/avm (符号链接)

版本信息:
- Rust: 1.92.0 (ded5c06cf 2025-12-08)
- Cargo: 1.92.0 (344c4567c 2025-10-21)
- 工具链: stable-x86_64-unknown-linux-gnu
```

### 2. Solana 工具链
```
安装路径: /home/logos/.local/share/solana/install/
当前版本: /home/logos/.local/share/solana/install/active_release/
二进制文件: /home/logos/.local/share/solana/install/active_release/bin/
├── solana (28.3 MB)
├── solana-test-validator (73.9 MB)
├── solana-keygen (2.8 MB)
├── cargo-build-sbf (19.2 MB)
├── cargo-test-sbf (4.1 MB)
├── agave-install (12.0 MB)
├── agave-ledger-tool (57.7 MB)
└── spl-token (23.1 MB)

版本信息:
- Solana CLI: 3.0.13 (src:f5a29bf6; feat:3604001754, client:Agave)
- 安装版本: v1.18.26
```

### 3. Anchor 框架
```
安装路径: /home/logos/.cargo/bin/anchor
实际路径: /home/logos/.avm/bin/avm (通过avm管理)
版本信息:
- Anchor CLI: 0.32.1
- 管理工具: AVM (Anchor Version Manager)
```

## 🔧 环境变量配置

### 永久配置 (~/.bashrc)
```bash
# Rust and Solana development environment
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

### 临时配置 (当前会话)
```bash
# 设置环境变量
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 验证安装
rustc --version
cargo --version
solana --version
anchor --version
```

## 📁 项目相关路径

### 1. 项目目录
```
主项目: /mnt/d/AI/预言机多智能体/MultiAgentOracle/
Solana项目: /mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle/
智能合约: /mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle/programs/solana-oracle/
```

### 2. 构建输出
```
构建目录: solana-oracle/target/
部署文件: solana-oracle/target/deploy/solana_oracle-keypair.json
程序ID: DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b
```

### 3. 测试文件
```
测试钱包: solana-oracle/test-wallet.json
配置文件: solana-oracle/multi_agent_config.yaml
Anchor配置: solana-oracle/Anchor.toml
```

## 🚀 快速启动命令

### 1. 启动开发环境
```bash
# 进入项目目录
cd /mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle

# 设置环境变量
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 启动本地测试网
solana-test-validator --reset --quiet &
```

### 2. 构建和部署
```bash
# 构建项目
anchor build

# 部署到本地测试网
anchor deploy

# 检查部署状态
solana program show DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b
```

### 3. 测试命令
```bash
# 运行JavaScript测试
node test_simple.js

# 检查网络状态
solana cluster-version
solana config get
solana balance
```

## ⚠️ 注意事项

### 1. 路径访问
- WSL中可以通过 `/mnt/d/` 访问Windows D盘
- 确保文件权限正确：`chmod +x *.sh`
- 符号链接可能需要重新加载环境变量

### 2. 常见问题
- **环境变量未生效**: 运行 `source ~/.bashrc`
- **权限问题**: 使用 `sudo` 或检查文件权限
- **网络问题**: 检查WSL网络配置和防火墙

### 3. 维护建议
1. 定期更新工具链：`rustup update`，`solana-install update`
2. 备份重要配置文件
3. 使用版本控制管理项目代码

## 📞 故障排除

### 工具不可用
```bash
# 检查路径
echo $PATH | tr ':' '\n' | grep -E '(cargo|solana)'

# 重新安装
source ~/.cargo/env
source ~/.bashrc
```

### 构建失败
```bash
# 清理构建缓存
cargo clean
rm -rf target/

# 重新构建
anchor clean
anchor build
```

### 部署问题
```bash
# 检查测试网状态
solana cluster-version

# 请求测试代币
solana airdrop 100

# 重新部署
anchor deploy --provider.cluster localnet
```

---

**文档最后更新**: 2025年12月31日  
**维护者**: 系统自动生成  
**状态**: ✅ 安装完成并验证通过
