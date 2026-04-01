# SDK 改造计划

## 概述

将多智能体预言机系统改造为易于集成的 SDK，面向智能合约开发者提供高级 API。

## 改造内容

### 1. 新增 SDK 便捷层 (`src/sdk/`)

#### 1.1 `src/sdk/mod.rs` - SDK 入口
- 重新导出核心类型
- 提供统一的 SDK 模块入口

#### 1.2 `src/sdk/types.rs` - SDK 专用类型
- `OracleQuery` - 查询请求封装
- `OracleResponse` - 单个 Agent 响应
- `OracleResult` - 完整查询结果
- `ConsensusOutput` - 共识输出（含链上提交数据）
- `AgentSubmission` - Agent 提交数据
- `SdkError` - SDK 错误类型
- `SdkResult<T>` - 结果别名

#### 1.3 `src/sdk/builder.rs` - 配置构建器
- `SdkConfig` - SDK 配置（Builder 模式）
- `OracleBuilder` - 预言机构建器
- 链式 API 设计

#### 1.4 `src/sdk/oracle.rs` - 核心 Oracle 类
- `Oracle::new(config)` - 创建实例
- `oracle.query(query)` - 发起查询
- `oracle.get_consensus()` - 获取共识结果
- `oracle.submit_to_chain()` - 提交到链上

#### 1.5 `src/sdk/solana.rs` - Solana 集成
- `SolanaIntegration` - Solana 链上交互
- `SolanaConfig` - Solana 配置
- 合约 IDL 辅助函数
- 事件监听器

### 2. 更新 `src/lib.rs`
- 添加 `pub mod sdk;`
- 重新导出 SDK 核心类型

### 3. 更新 `Cargo.toml`
- 添加 crate 元数据（description, documentation, keywords）
- 确保 `sdk` 模块正确导出

### 4. 新增示例 `examples/sdk_quick_start.rs`
- SDK 快速开始示例
- 展示完整工作流程

### 5. 新增文档 `docs/SDK_GUIDE.md`
- SDK 使用指南
- API 参考
- 最佳实践

## 文件清单

### 新增文件
- `src/sdk/mod.rs`
- `src/sdk/types.rs`
- `src/sdk/builder.rs`
- `src/sdk/oracle.rs`
- `src/sdk/solana.rs`
- `examples/sdk_quick_start.rs`
- `docs/SDK_GUIDE.md`

### 修改文件
- `src/lib.rs` - 添加 sdk 模块
- `Cargo.toml` - 更新元数据

## API 设计示例

```rust
use multi_agent_oracle::sdk::{Oracle, SdkConfig};

// 1. 配置
let config = SdkConfig::builder()
    .with_solana_rpc("https://api.devnet.solana.com")
    .with_agent_count(5)
    .with_consensus_threshold(0.67)
    .build();

// 2. 创建预言机
let oracle = Oracle::new(config)?;

// 3. 发起查询
let result = oracle.query("BTC price prediction")
    .with_timeout(Duration::from_secs(30))
    .execute()
    .await?;

// 4. 获取共识结果
let consensus = result.consensus()?;

// 5. 提交到链上
let tx_hash = oracle.submit_to_chain(&consensus).await?;
```
