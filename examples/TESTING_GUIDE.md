# ZK Causal Fingerprint 测试指南

本文档说明如何使用优化后的 `zk_fingerprint_experiment.rs` 进行不同 prompt 配置的测试。

## 功能特性

### 1. 多种测试模式

- **单轮测试**：运行一次实验，快速验证配置
- **多轮统计测试**：运行多次实验，进行统计分析
- **配置驱动**：通过 JSON 配置文件定义测试场景
- **命令行参数**：快速指定智能体分布

### 2. 灵活的 Prompt 配置

每个智能体现在支持自定义参数：
- `sensitivity`: 响应灵敏度系数（影响因果响应计算）
- `noise_level`: 噪声水平（影响随机性）
- `model_characteristics`: 模型特征描述

### 3. 可配置的共识参数

- `consensus_threshold`: 异常检测阈值
- `global_fingerprint`: 全局参考指纹
- `intervention_dimensions`: 干预向量维度

## 使用方法

### 方法 1：使用默认配置

```bash
cargo run --example zk_fingerprint_experiment
```

运行默认的 10 个智能体配置（2 analytical + 2 cautious + 2 aggressive + 2 neutral + 1 creative + 1 suspicious）。

### 方法 2：使用 JSON 配置文件

```bash
# 使用保守型智能体配置
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_conservative.json

# 使用激进型智能体配置
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_aggressive.json

# 使用混合型智能体配置
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_mixed.json
```

### 方法 3：命令行指定智能体分布

```bash
# 测试 5 个 analytical + 3 个 cautious + 2 个 aggressive
cargo run --example zk_fingerprint_experiment -- --agents analytical=5,cautious=3,aggressive=2

# 测试多种类型，包含可疑智能体
cargo run --example zk_fingerprint_experiment -- --agents analytical=3,cautious=2,aggressive=2,neutral=2,suspicious=1
```

### 方法 4：多轮统计测试

```bash
# 运行 10 次实验，进行统计分析
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_mixed.json --runs 10

# 快速测试 5 轮
cargo run --example zk_fingerprint_experiment -- --agents analytical=4,cautious=3,neutral=3 --runs 5
```

### 组合使用

```bash
# 使用配置文件 + 多轮测试
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_aggressive.json --runs 20
```

## 配置文件说明

### 配置文件结构

```json
{
  "agents": [
    {
      "agent_id": "agent_1",
      "prompt_type": "analytical",
      "model_characteristics": ["逻辑推理", "数据分析", "理性决策"],
      "sensitivity": 1.0,
      "noise_level": 0.1
    }
  ],
  "intervention_dimensions": 5,
  "consensus_threshold": 0.85,
  "global_fingerprint": [5.0, 3.0, 1.0],
  "test_runs": 1
}
```

### 参数说明

| 参数 | 类型 | 说明 | 默认值 |
|------|------|------|--------|
| `sensitivity` | f64 | 响应灵敏度，影响 Δy = sensitivity × δX + noise | 1.0 |
| `noise_level` | f64 | 噪声水平范围 [-noise, noise] | 0.1 |
| `intervention_dimensions` | usize | 干预向量维度 | 5 |
| `consensus_threshold` | f64 | 共识相似度阈值，低于此值标记为异常 | 0.85 |
| `global_fingerprint` | Vec<f64> | 全局参考指纹 | [5.0, 3.0, 1.0] |
| `test_runs` | usize | 测试运行次数（仅用于统计分析） | 1 |

### Prompt 类型参考

| Prompt 类型 | Sensitivity 建议 | Noise 建议 | 说明 |
|-------------|------------------|------------|------|
| `conservative` | 0.2 - 0.4 | 0.02 - 0.05 | 极度保守，低响应 |
| `cautious` | 0.5 - 0.7 | 0.04 - 0.08 | 谨慎型，中等保守 |
| `neutral` | 0.9 - 1.1 | 0.08 - 0.12 | 中性平衡 |
| `analytical` | 1.0 - 1.2 | 0.08 - 0.12 | 分析型，标准响应 |
| `aggressive` | 1.4 - 1.8 | 0.15 - 0.25 | 激进型，高响应 |
| `creative` | 1.3 - 1.6 | 0.18 - 0.28 | 创新型，较高响应和噪声 |
| `suspicious` | -1.0 - -0.5 | 0.15 - 0.30 | 可疑型，反向响应 |

## 测试场景建议

### 场景 1：系统健康度测试

**目标**：验证系统在正常使用情况下的表现

```bash
cargo run --example zk_fingerprint_experiment -- --agents analytical=4,cautious=3,neutral=3 --runs 10
```

**期望结果**：
- Pass Rate > 85%
- Average Consensus Similarity > 0.85
- Spectral Entropy 0.6-0.9

### 场景 2：异常检测测试

**目标**：测试系统检测异常智能体的能力

```bash
cargo run --example zk_fingerprint_experiment -- --agents analytical=7,suspicious=3 --runs 5
```

**期望结果**：
- 正确识别出 suspicious 智能体为异常
- Valid Agents ≈ 7 (仅正常智能体通过)

### 场景 3：负载测试

**目标**：测试系统在大规模智能体下的表现

```bash
# 使用配置文件定义 50 个智能体
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_large_scale.json
```

### 场景 4：极端值测试

**目标**：测试系统在极端配置下的鲁棒性

```bash
# 全部使用激进型智能体
cargo run --example zk_fingerprint_experiment -- --agents aggressive=10 --runs 3

# 全部使用保守型智能体
cargo run --example zk_fingerprint_experiment -- --agents conservative=10 --runs 3
```

## 结果解读

### 单轮测试输出

```
╔════════════════════════════════════════════════════════════════════════════╗
║                    Fingerpring Creation Table                                    ║
╠════════════╦═══════════╦════════════════╦══════════════╦════════════════╦══════╦═══════╗
║  Agent ID  ║ Prompt    ║ Δy (3 dims)    ║ Eigenvalues  ║ R (Radius)  ║ H(Ent)║ Status║
╠════════════╬═══════════╬════════════════╬══════════════╬════════════════╬══════╬═══════╣
║  agent_1   ║ analytical║ [0.8, -0.3, 0.1] ║ [5.12, 3.05, 1.02] ║ 5.12 ║ 0.85 ║ ✅ Valid ║
╚════════════╩═══════════╩════════════════╩══════════════╩════════════════╩══════╩═══════╝
```

**字段说明**：
- **Δy (3 dims)**：因果响应向量（前3维）
- **Eigenvalues**：特征值
- **R (Radius)**：谱半径（最大特征值绝对值）
- **H(Ent)**：谱熵（系统多样性指标）
- **Status**：验证状态（✅ Valid / ❌ Invalid Proof / ⚠️ Outlier）

### 多轮统计输出

```
╔════════════════════════════════════════════════════════════════════════════╗
║                    Statistical Analysis Summary                              ║
╠════════════════════════════════════════════════════════════════════════════╣
║  Pass Rate - Avg: 87.0%, Min: 80.0%, Max: 90.0%, Std: 3.5%                ║
║  Consensus Sim - Avg: 0.873, Min: 0.850, Max: 0.891                       ║
║  Spectral Entropy - Avg: 0.745, Min: 0.721, Max: 0.768                    ║
╚════════════════════════════════════════════════════════════════════════════╝
```

**统计指标**：
- **Avg**：平均值
- **Min/Max**：最小/最大值
- **Std**：标准差（衡量稳定性）

### 健康度指标

| 指标 | 健康范围 | 警告范围 | 危险范围 |
|------|----------|----------|----------|
| Pass Rate | > 85% | 70-85% | < 70% |
| Consensus Similarity | > 0.85 | 0.70-0.85 | < 0.70 |
| Spectral Entropy | 0.6-0.9 | 0.5-0.6 或 0.9-1.0 | < 0.5 或 > 1.0 |
| Standard Deviation | < 0.1 | 0.1-0.2 | > 0.2 |

## 高级用法

### 创建自定义测试配置

1. 复制 `examples/configs/test_default.json`
2. 修改参数，创建新的测试场景
3. 使用 `--config` 参数运行

### 批量测试脚本

```bash
#!/bin/bash
# test_all_configs.sh

echo "Testing all configurations..."

for config in examples/configs/*.json; do
    echo "Running $config..."
    cargo run --example zk_fingerprint_experiment -- --config "$config" --runs 5
    echo "----------------------------------------"
done
```

### 结果对比分析

运行不同配置后，对比以下指标：
1. **Pass Rate**：系统整体健康度
2. **Consensus Similarity**：共识强度
3. **Spectral Entropy**：模型多样性
4. **Outlier Detection**：异常检测能力

## 故障排查

### 问题 1：所有智能体都被标记为异常

**可能原因**：
- `consensus_threshold` 设置过高
- `global_fingerprint` 配置不当

**解决方案**：
```bash
# 降低阈值测试
cargo run --example zk_fingerprint_experiment -- --agents analytical=5 --runs 1
# 然后在 JSON 中调整 consensus_threshold
```

### 问题 2：Pass Rate 过低

**可能原因**：
- 包含太多 suspicious 智能体
- ZK 证明生成/验证失败

**解决方案**：
```bash
# 先测试全正常智能体
cargo run --example zk_fingerprint_experiment -- --agents analytical=5,neutral=5 --runs 3
```

### 问题 3：统计方差过大

**可能原因**：
- `noise_level` 设置过高
- 智能体数量过少

**解决方案**：
- 增加智能体数量
- 适当降低 noise_level
- 增加 test_runs 次数

## 最佳实践

1. **从小到大**：先测试小规模（5-10个智能体），再逐步扩大
2. **控制变量**：每次只改变一个参数，便于分析影响
3. **多轮测试**：重要配置至少运行 5-10 轮，确保结果稳定
4. **记录基线**：建立健康系统的基准指标，用于后续对比
5. **定期测试**：将测试集成到 CI/CD 流程中

## 扩展建议

- 添加更多 prompt 类型（如 `creative`, `defensive`, `exploratory`）
- 实现动态全局指纹更新机制
- 添加时间序列分析功能
- 集成可视化工具展示结果
