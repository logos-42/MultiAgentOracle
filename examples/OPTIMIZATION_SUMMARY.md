# ZK Causal Fingerprint 实验优化总结

## 优化概述

已对 `examples/zk_fingerprint_experiment.rs` 进行全面优化，使其支持灵活的测试配置和多种测试模式。

## 新增功能

### 1. 配置文件支持
- ✅ JSON 配置文件驱动测试
- ✅ 可自定义智能体参数（sensitivity, noise_level）
- ✅ 可配置共识阈值和全局指纹
- ✅ 支持多种预设配置（保守、激进、混合型）

### 2. 命令行参数
- ✅ `--config`：指定配置文件路径
- ✅ `--agents`：快速指定智能体分布
- ✅ `--runs`：设置测试运行次数

### 3. 多轮统计测试
- ✅ 自动运行多次实验
- ✅ 统计分析（平均值、最小/最大值、标准差）
- ✅ Prompt 类型性能分析
- ✅ 系统稳定性评估

### 4. 增强的输出
- ✅ 指纹创建表（带状态标识）
- ✅ 实验结果摘要
- ✅ 健康度分析报告
- ✅ 统计摘要（多轮测试）

## 文件结构

```
examples/
├── zk_fingerprint_experiment.rs    # 主实验文件（已优化）
├── configs/                         # 配置文件夹
│   ├── test_default.json           # 默认配置（5个智能体）
│   ├── test_conservative.json      # 保守型智能体配置
│   ├── test_aggressive.json        # 激进型智能体配置
│   └── test_mixed.json             # 混合型智能体配置
├── quick_test.bat                   # Windows 快速测试脚本
├── quick_test.sh                    # Linux/Mac 快速测试脚本
├── validate_configs.bat             # 配置验证脚本
├── TESTING_GUIDE.md                 # 详细测试指南
└── OPTIMIZATION_SUMMARY.md          # 本文档
```

## 快速开始

### 1. 运行默认测试

```bash
cargo run --example zk_fingerprint_experiment
```

### 2. 使用预设配置

```bash
# 测试保守型智能体
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_conservative.json

# 测试激进型智能体
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_aggressive.json

# 测试混合型智能体
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_mixed.json
```

### 3. 命令行快速测试

```bash
# 5个analytical + 3个cautious + 2个aggressive
cargo run --example zk_fingerprint_experiment -- --agents analytical=5,cautious=3,aggressive=2
```

### 4. 多轮统计测试

```bash
# 运行10轮统计分析
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_mixed.json --runs 10
```

## 使用快速测试脚本

### Windows

```bash
cd examples
quick_test.bat
```

### Linux/Mac

```bash
cd examples
chmod +x quick_test.sh
./quick_test.sh
```

快速测试脚本会自动运行：
1. 默认配置测试
2. 保守型智能体测试
3. 激进型智能体测试（3轮）
4. 混合型智能体测试（命令行）
5. 统计分析（5轮）

## 创建自定义测试

### 方法1：修改现有配置

复制 `examples/configs/test_default.json` 并修改参数：

```json
{
  "agents": [
    {
      "agent_id": "agent_1",
      "prompt_type": "creative",
      "model_characteristics": ["创新思维", "发散思考"],
      "sensitivity": 1.6,
      "noise_level": 0.2
    }
  ],
  "intervention_dimensions": 8,
  "consensus_threshold": 0.82,
  "global_fingerprint": [6.0, 4.0, 2.0, 1.5],
  "test_runs": 5
}
```

### 方法2：命令行组合

```bash
# 测试新prompt类型组合
cargo run --example zk_fingerprint_experiment -- \
  --agents analytical=3,cautious=2,aggressive=2,creative=2,suspicious=1 \
  --runs 10
```

## 测试策略建议

### 1. 基础验证
```bash
cargo run --example zk_fingerprint_experiment -- --agents analytical=5,neutral=5 --runs 5
```
**目标**：Pass Rate > 90%，验证系统基本功能

### 2. 异常检测能力
```bash
cargo run --example zk_fingerprint_experiment -- --agents analytical=7,suspicious=3 --runs 5
```
**目标**：正确识别出 3 个 suspicious 智能体

### 3. 极端配置测试
```bash
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_aggressive.json --runs 10
```
**目标**：系统在激进配置下保持稳定

### 4. 大规模测试
创建包含 50-100 个智能体的配置，测试扩展性。

## 结果解读

### 单轮测试结果

```
✅ System is very healthy - high pass rate (>85%)
✅ Good model diversity - entropy in healthy range
✅ Strong consensus - high similarity
```

### 多轮统计结果

```
Pass Rate - Avg: 87.0%, Min: 80.0%, Max: 90.0%, Std: 3.5%
Consensus Sim - Avg: 0.873, Min: 0.850, Max: 0.891
```

**健康指标**：
- Pass Rate > 85%
- Standard Deviation < 5%
- Consensus Similarity > 0.85

### Prompt 类型分析

```
✅ analytical: avg similarity 0.892 (15 samples)
⚠️  aggressive: avg similarity 0.783 (10 samples)
❌ suspicious: avg similarity 0.421 (5 samples)
```

## 高级功能

### 1. 动态智能体生成

```rust
// 自动根据计数生成智能体
fn generate_agents_from_counts(counts: &HashMap<String, usize>)
```

### 2. 自定义共识计算

```rust
// 使用配置的全局指纹
fn calculate_consensus_similarity_with_global(
    eigenvalues: &[f64], 
    global_fingerprint: &[f64]
) -> f64
```

### 3. 统计聚合分析

```rust
// 多轮测试结果聚合
fn print_statistical_summary(...)
```

## 故障排查

### 问题：JSON 配置文件解析错误

**解决方案**：
```bash
# 验证 JSON 语法
cd examples
python -m json.tool configs/test_default.json

# 或使用验证脚本
validate_configs.bat
```

### 问题：找不到配置文件

**解决方案**：
- 确保在项目根目录运行命令
- 使用相对路径：`examples/configs/test_default.json`

### 问题：编译错误

**解决方案**：
```bash
# 检查依赖
cargo check --example zk_fingerprint_experiment

# 清理并重新构建
cargo clean
cargo build --example zk_fingerprint_experiment
```

## 性能优化

### 大规模测试建议

对于 >50 个智能体的测试：

1. **Release 模式编译**：
   ```bash
   cargo run --release --example zk_fingerprint_experiment -- --config large_config.json
   ```

2. **并行测试**：使用多个终端运行不同配置

3. **减少输出**：暂时注释掉详细的打印信息

## 后续扩展

可能的扩展方向：

1. **更多 Prompt 类型**：添加 `defensive`, `exploratory`, `adaptive` 等
2. **动态阈值调整**：根据历史数据自动调整 consensus_threshold
3. **可视化工具**：生成图表展示测试结果
4. **集成测试**：与 CI/CD 系统集成
5. **性能分析**：添加运行时间、内存使用等指标

## 文档索引

- **详细测试指南**：`examples/TESTING_GUIDE.md`
- **配置文件**：`examples/configs/`
- **快速测试**：`examples/quick_test.bat` 或 `.sh`

## 总结

优化后的实验文件支持：

✅ **零代码修改测试**：通过 JSON 或命令行参数配置
✅ **灵活的 Prompt 配置**：自定义 sensitivity 和 noise_level
✅ **统计分析**：多轮测试和性能评估
✅ **丰富的输出**：详细的实验报告和健康度分析
✅ **预设场景**：保守型、激进型、混合型配置

无需编写代码即可测试不同的智能体配置，大幅提高了测试效率和灵活性。
