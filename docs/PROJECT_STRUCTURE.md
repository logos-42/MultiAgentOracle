# Solana Oracle 项目结构（简洁版）

## 📁 项目目录结构

```
solana-oracle/
├── 📄 INSTALLATION_PATHS.md          # 安装路径记录
├── 📄 PROJECT_STRUCTURE.md           # 本项目结构文档
├── 📄 setup_env.sh                   # 环境设置脚本
├── 📄 test_simple.js                 # 简化测试脚本
├── 📄 multi_agent_config.yaml        # 多智能体配置
├── 📄 Anchor.toml                    # Anchor框架配置
├── 📄 Cargo.toml                     # Rust项目配置
├── 📄 Cargo.lock                     # 依赖锁定文件
├── 📄 .gitignore                     # Git忽略文件
├── 📄 .prettierignore                # 代码格式化忽略
├── 📄 rust-toolchain.toml            # Rust工具链配置
├── 📄 test-wallet.json               # 测试钱包
│
├── 📂 programs/                      # 智能合约程序
│   └── 📂 solana-oracle/             # 主合约程序
│       ├── 📄 Cargo.toml             # 合约依赖配置
│       └── 📂 src/                   # 合约源代码
│           └── 📄 lib.rs             # 主合约代码
│               ├── 身份注册功能
│               ├── 验证系统
│               ├── 声誉管理
│               ├── 批量注册（新功能）
│               └── 层级计算（新功能）
│
├── 📂 scripts/                       # 脚本目录
│   ├── 📄 build_only.ps1             # 构建脚本
│   ├── 📄 deploy_local.ps1           # 本地部署脚本
│   ├── 📄 deploy_wsl.sh              # WSL部署脚本
│   └── 📄 test_multi_agent.ps1       # 多智能体测试
│
├── 📂 target/                        # 构建输出目录
│   └── 📂 deploy/                    # 部署文件
│       └── 📄 solana_oracle-keypair.json  # 程序密钥对
│
└── 📂 test-ledger/                   # 测试账本数据
    ├── 📄 validator-keypair.json     # 验证器密钥
    ├── 📄 genesis.bin                # 创世文件
    └── 📂 rocksdb/                   # 数据库文件
```

## 🎯 核心文件说明

### 1. 智能合约 (`programs/solana-oracle/src/lib.rs`)
- **功能**: 多智能体身份注册系统
- **主要指令**:
  - `register_agent()` - 注册新智能体
  - `update_identity()` - 更新身份信息
  - `request_verification()` - 请求验证
  - `approve_verification()` - 批准验证
  - `update_reputation()` - 更新声誉
  - `batch_register_agents()` - 批量注册（新功能）
  - `get_agent_tier()` - 获取智能体层级（新功能）

### 2. 配置文件
- **`Anchor.toml`**: Anchor框架配置，包含程序ID和网络设置
- **`multi_agent_config.yaml`**: 多智能体测试配置
- **`Cargo.toml`**: Rust工作区配置

### 3. 测试文件
- **`test_simple.js`**: 简化版智能体注册测试
- **`test-wallet.json`**: 测试用钱包文件

### 4. 工具脚本
- **`setup_env.sh`**: 环境快速设置脚本
- **`scripts/`**: 各种构建和部署脚本

## 🔧 开发工作流

### 1. 环境设置
```bash
# 运行环境设置脚本
./setup_env.sh

# 加载别名
source /tmp/solana_aliases.sh
```

### 2. 本地开发
```bash
# 启动本地测试网
solana-test-validator --reset --quiet &

# 构建项目
anchor build

# 部署合约
anchor deploy

# 运行测试
node test_simple.js
```

### 3. 测试验证
```bash
# 检查网络状态
solana cluster-version
solana config get
solana balance

# 检查程序状态
solana program show DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b
```

## 📊 项目状态

### ✅ 已完成
1. Rust和Solana环境安装
2. 智能合约基础功能开发
3. 批量注册和层级计算新功能
4. 环境配置文档
5. 测试脚本简化

### 🚧 进行中
1. 智能合约测试完善
2. 前端交互界面
3. 多智能体系统集成

### 📋 待完成
1. 完整的测试套件
2. 部署到测试网
3. 性能优化
4. 安全审计

## 🎨 设计原则

### 1. 简洁性
- 移除冗余测试文件
- 保持核心功能清晰
- 最小化依赖

### 2. 可维护性
- 完整的文档记录
- 清晰的代码结构
- 自动化脚本

### 3. 可扩展性
- 模块化设计
- 易于添加新功能
- 支持多智能体场景

## 🔗 相关文档

1. **安装指南**: `INSTALLATION_PATHS.md`
2. **部署指南**: `DEPLOYMENT_SUMMARY.md`
3. **本地测试网**: `LOCAL_TESTNET_GUIDE.md`
4. **开发指南**: 本项目文档

## 📞 支持

### 常见问题
1. **环境变量问题**: 运行 `source ~/.bashrc`
2. **构建失败**: 运行 `cargo clean && anchor clean`
3. **部署失败**: 检查测试网状态和余额

### 故障排除
```bash
# 重置环境
./setup_env.sh

# 清理构建
cargo clean
rm -rf target/

# 重新开始
anchor build
anchor deploy
```

---

**项目状态**: ✅ 运行正常  
**最后更新**: 2025年12月31日  
**维护目标**: 保持简洁、可维护、可扩展
