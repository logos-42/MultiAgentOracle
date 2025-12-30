# 智能合约

多智能体预言机系统的智能合约，包括身份注册、信誉存储和质押管理。

## 合约列表

### 1. IdentityRegistry.sol
**身份注册合约**
- 管理DIAP身份的注册和验证
- 存储智能体DID和公钥
- 支持身份验证和撤销
- 集成零知识证明验证

### 2. ReputationStorage.sol
**信誉存储合约**
- 存储智能体信誉分数
- 记录信誉更新历史
- 支持信誉查询和验证
- 集成信誉加权计算

### 3. StakingManager.sol
**质押管理合约**
- 管理智能体质押代币
- 处理质押和取回操作
- 计算投票权重
- 处理惩罚和奖励

### 4. OracleService.sol
**预言机服务合约**
- 处理数据请求和响应
- 管理共识过程
- 聚合预言机数据
- 分配奖励和费用

## 开发环境

### 要求
- Node.js 18+
- Hardhat
- Solidity 0.8.20+

### 安装
```bash
cd contracts
npm install
```

### 编译
```bash
npx hardhat compile
```

### 测试
```bash
npx hardhat test
```

### 部署
```bash
npx hardhat run scripts/deploy.js --network <network>
```

## 网络配置

支持的网络：
- 本地开发网络
- Base Sepolia测试网
- Base主网
- 以太坊主网
- 其他EVM兼容链

## 安全考虑

1. **重入攻击防护**：使用Checks-Effects-Interactions模式
2. **整数溢出防护**：使用SafeMath或Solidity 0.8+
3. **权限控制**：严格的访问控制列表
4. **输入验证**：所有外部输入都经过验证
5. **事件记录**：所有重要操作都记录事件
6. **升级模式**：支持可升级合约模式

## 经济模型

### 代币经济
- **质押代币**：用于节点质押和投票权重
- **奖励代币**：用于激励准确的数据提供
- **治理代币**：用于系统治理和参数调整

### 费用结构
- **注册费用**：新智能体注册费用
- **服务费用**：数据请求服务费用
- **惩罚费用**：错误数据提供的惩罚
- **网络费用**：网络维护和运营费用

## 集成指南

### 前端集成
```javascript
import { ethers } from 'ethers';
import IdentityRegistry from './artifacts/IdentityRegistry.json';

const provider = new ethers.providers.Web3Provider(window.ethereum);
const signer = provider.getSigner();
const contract = new ethers.Contract(
  contractAddress,
  IdentityRegistry.abi,
  signer
);
```

### 后端集成
```javascript
const { ethers } = require('ethers');
const IdentityRegistry = require('./artifacts/IdentityRegistry.json');

const provider = new ethers.providers.JsonRpcProvider(rpcUrl);
const wallet = new ethers.Wallet(privateKey, provider);
const contract = new ethers.Contract(
  contractAddress,
  IdentityRegistry.abi,
  wallet
);
```

## 审计和验证

### 代码审计
- [ ] 安全审计
- [ ] 功能审计
- [ ] 性能审计
- [ ] 经济模型审计

### 合约验证
```bash
npx hardhat verify --network <network> <contract_address> <constructor_args>
```

## 许可证

MIT License
