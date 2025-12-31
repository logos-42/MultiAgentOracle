# 多智能体预言机项目

这是一个基于DIAP身份的去中心化多智能体预言机系统项目。

## 项目结构

```
.
├── README.md                    # 本文件
├── MultiAgentOracle/            # 主项目：多智能体预言机系统
│   ├── Cargo.toml              # Rust项目配置
│   ├── src/                    # 源代码
│   ├── examples/               # 示例代码
│   ├── tests/                  # 测试代码
│   ├── config/                 # 配置文件
│   ├── scripts/                # 脚本文件
│   ├── solana-oracle/          # Solana预言机程序
│   └── DIAP-Rust-SDK/          # DIAP Rust SDK（子模块）
└── .gitignore                  # Git忽略文件
```

## 快速开始

### 1. 进入主项目目录
```bash
cd MultiAgentOracle
```

### 2. 构建项目
```bash
cargo build
```

### 3. 运行测试
```bash
cargo test
```

### 4. 运行示例
```bash
cargo run --example oracle_agent_demo
```

## 主要组件

### MultiAgentOracle（主项目）
基于DIAP身份的去中心化多智能体预言机网络，包含：
- 智能体节点管理
- 信誉加权共识系统
- 多链数据源支持
- 网络通信层

### DIAP-Rust-SDK
基于零知识证明的去中心化智能体身份协议SDK，提供：
- 去中心化身份验证
- 零知识证明验证
- 身份管理工具
- 加密通信

### Solana预言机程序
Solana区块链上的预言机智能合约，支持：
- 数据提交和验证
- 信誉评分更新
- 奖励分配
- 治理功能

## 开发指南

### 环境要求
- Rust 1.70+ 
- Cargo
- Solana CLI（可选，用于Solana开发）
- Node.js（可选，用于JavaScript测试）

### 代码组织
- `src/` - 核心业务逻辑
- `src/bin/` - 可执行程序入口
- `src/consensus/` - 共识算法
- `src/network/` - 网络通信
- `src/oracle_agent/` - 预言机智能体
- `src/reputation/` - 信誉系统
- `src/solana/` - Solana集成

### 测试
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --test diap_integration_test

# 运行示例
cargo run --example solana_demo
```

## 部署

### 本地测试
```bash
cd MultiAgentOracle
cargo run --bin oracle-agent -- --config config/local_test.toml
```

### Solana部署
```bash
cd MultiAgentOracle/solana-oracle
.\deploy_to_devnet.ps1
```

## 贡献

欢迎提交Issue和Pull Request来改进这个项目。

## 许可证

MIT License
