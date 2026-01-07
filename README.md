# 多智能体预言机系统

基于DIAP身份的去中心化多智能体预言机网络，每个智能体都是独立的预言机节点，通过信誉加权共识机制提供可靠的数据服务。

## 核心特性

- **完全去中心化**：任何人都可以使用DIAP身份加入网络成为预言机节点
- **身份可信**：基于DIAP零知识证明的去中心化身份系统
- **信誉机制**：基于数据准确性的动态信誉评分系统
- **抗Sybil攻击**：通过质押和信誉机制防止女巫攻击
- **多链支持**：支持以太坊、Solana、Cosmos等多条区块链
- **自动扩展**：智能体节点可自动发现和加入网络

## 系统架构


## 技术栈

### 后端
- **核心语言**: Rust
- **身份系统**: DIAP-Rust-SDK (零知识证明身份)
- **P2P网络**: libp2p + Iroh
- **区块链交互**: ethers-rs, solana-client
- **数据存储**: IPFS, 区块链存储

### 智能合约
- **身份注册合约**: Solidity/Rust
- **信誉存储合约**: 存储节点信誉分
- **预言机服务合约**: 处理数据请求和聚合

### 前端
- **管理面板**: React + TypeScript
- **节点监控**: 实时监控节点状态
- **数据可视化**: 图表展示数据趋势

## 快速开始

### 环境要求
- Rust 1.70+
- Node.js 18+
- Git

### 安装和运行

```bash
# 克隆项目
git clone <repository-url>
cd MultiAgentOracle

# 安装依赖
cargo build

# 运行示例智能体
cargo run --example oracle_agent_demo

# 运行共识节点
cargo run --bin consensus_node
```

## 项目结构

```
MultiAgentOracle/
├── Cargo.toml              # Rust项目配置
├── README.md              # 项目说明
├── src/
│   ├── lib.rs            # 库入口点
│   ├── oracle_agent/     # 预言机智能体实现
│   ├── reputation/       # 信誉系统
│   ├── consensus/        # 共识机制
│   ├── blockchain/       # 区块链交互
│   └── network/         # P2P网络
├── contracts/            # 智能合约
├── examples/            # 示例代码
└── frontend/           # 前端管理面板
```

## 开发计划

### 第一阶段：核心预言机智能体
- [x] 创建项目结构
- [ ] 实现预言机智能体基础功能
- [ ] 集成DIAP身份系统
- [ ] 实现多源数据采集

### 第二阶段：共识和网络层
- [ ] 实现信誉加权共识算法
- [ ] 开发P2P通信框架
- [ ] 创建智能合约

### 第三阶段：系统集成
- [ ] 端到端测试
- [ ] 性能优化
- [ ] 安全审计

## 许可证

MIT License