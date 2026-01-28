# DeepSeek API 集成使用指南

## 概述

ZK 因果指纹实验现已支持 DeepSeek API，可以将智能体接入真实的 LLM 模型，获取更真实和多样化的响应。

## 快速开始

### 1. 设置 API 密钥

在 Windows 上：
```cmd
set DEEPSEEK_API_KEY=your_deepseek_api_key_here
```

或使用提供的批处理文件：
```cmd
setup_deepseek_api.bat sk-your-api-key-here
```

在 Linux/Mac 上：
```bash
export DEEPSEEK_API_KEY=your_deepseek_api_key_here
```

### 2. 运行实验（使用 DeepSeek API）

```cmd
cargo run --example zk_fingerprint_experiment -- --use-api
```

## 命令行选项

### 基本使用

- **模拟模式（默认）**：
  ```cmd
  cargo run --example zk_fingerprint_experiment
  ```

- **使用真实 API（DeepSeek）**：
  ```cmd
  cargo run --example zk_fingerprint_experiment -- --use-api
  ```

- **指定提供商**：
  ```cmd
  --provider deepseek    # 使用 DeepSeek（默认）
  --provider openai      # 使用 OpenAI
  --provider anthropic    # 使用 Anthropic (Claude)
  ```

- **指定模型**：
  ```cmd
  --model deepseek-chat              # DeepSeek 聊天模型（默认）
  --model deepseek-coder            # DeepSeek 代码模型
  --model gpt-4-turbo-preview      # OpenAI GPT-4
  --model claude-3-opus-20240229  # Anthropic Claude
  ```

### 高级选项

- **指定智能体数量和类型**：
  ```cmd
  --agents analytical=3 cautious=3 aggressive=2 neutral=2
  ```

- **多次运行**：
  ```cmd
  --runs 5
  ```

- **禁用回退到模拟模式**：
  ```cmd
  --no-fallback
  ```
  当 API 调用失败时，不会自动切换到模拟模式。

- **使用配置文件**：
  ```cmd
  --config my_config.json
  ```

## 示例用法

### 示例 1：使用 DeepSeek 运行单个实验

```cmd
set DEEPSEEK_API_KEY=sk-xxxxxxxx
cargo run --example zk_fingerprint_experiment -- --use-api
```

### 示例 2：使用 DeepSeek Coder 模型

```cmd
cargo run --example zk_fingerprint_experiment -- --use-api --model deepseek-coder
```

### 示例 3：混合使用不同类型的智能体

```cmd
cargo run --example zk_fingerprint_experiment -- --use-api --agents analytical=4 cautious=3 aggressive=2 neutral=1
```

### 示例 4：统计测试运行 10 次

```cmd
cargo run --example zk_fingerprint_experiment -- --use-api --runs 10
```

### 示例 5：使用 OpenAI 替代 DeepSeek

```cmd
set OPENAI_API_KEY=sk-xxxxxxxx
cargo run --example zk_fingerprint_experiment -- --use-api --provider openai --model gpt-4
```

## 配置文件示例

创建 `config.json`：

```json
{
  "use_real_api": true,
  "llm_provider": "deepseek",
  "llm_model": "deepseek-chat",
  "fallback_to_simulated": true,
  "agents": [
    {
      "agent_id": "agent_1",
      "prompt_type": "analytical",
      "model_characteristics": ["逻辑推理", "数据分析"],
      "sensitivity": 1.0,
      "noise_level": 0.1,
      "llm_provider": "deepseek",
      "llm_model": "deepseek-chat"
    }
  ],
  "intervention_dimensions": 5,
  "consensus_threshold": 0.85,
  "global_fingerprint": [5.0, 3.0, 1.0],
  "test_runs": 1
}
```

使用配置文件运行：
```cmd
cargo run --example zk_fingerprint_experiment -- --config config.json
```

## 环境变量

| 环境变量 | 描述 | 示例 |
|---------|------|------|
| `DEEPSEEK_API_KEY` | DeepSeek API 密钥 | `sk-xxxxxxxxxxxxxxxxxxxxx` |
| `OPENAI_API_KEY` | OpenAI API 密钥 | `sk-xxxxxxxxxxxxxxxxxxxxx` |
| `ANTHROPIC_API_KEY` | Anthropic API 密钥 | `sk-ant-xxxxxxxxxxxxxxxxxxxxx` |

## 智能体类型

| 类型 | 描述 | 特点 | sensitivity |
|-----|------|------|------------|
| `analytical` | 分析型 | 逻辑推理能力强，数据分析严谨 | 1.0 |
| `cautious` | 谨慎型 | 保守估计，注重安全性，低风险容忍度 | 0.5 |
| `aggressive` | 激进型 | 乐观估计，追求高收益，高风险容忍度 | 1.5 |
| `neutral` | 中立型 | 平衡分析，综合考虑，中庸策略 | 1.0 |
| `suspicious` | 可疑型 | 异常行为，逻辑不一致，可能的攻击者 | -1.0 |

## 工作流程

### API 模式

1. **初始化 LLM 客户端**
   - 检测 API 密钥
   - 验证连接
   - 选择提供商和模型

2. **生成干预向量**
   - 创建随机 δX 向量

3. **处理每个智能体**
   - 构建特定于智能体类型的 Prompt
   - 调用 LLM API 获取响应
   - 解析响应为数值向量
   - 应用智能体特征（sensitivity 和 noise）

4. **计算谱特征**
   - 提取特征值
   - 计算谱半径和熵

5. **生成 ZK 证明**
   - 使用 Nori 电路
   - 生成证明和验证

6. **异常检测**
   - 计算余弦相似度
   - 标记异常值

### 回退机制

如果 API 调用失败且启用了 `fallback_to_simulated`：
- 自动切换到模拟模式
- 使用数学公式生成响应
- 继续实验流程

## 输出示例

```
🧪 ZK Causal Fingerprint Experiment
==========================================
Architecture: Flat P2P Oracle Network (No Aggregation Agent)
ZK Verification: Enabled (Nori Circuit)

📋 Configuration loaded: 10 agents, 1 test runs
🤖 Using Real API Mode: DeepSeek (deepseek-chat)
✅ Initialized LLM client: DeepSeek (deepseek-chat)

🔄 Running single experiment with 10 agents...
✅ Generated intervention vector δX: [-0.5237, -0.0985, 0.6348, ...]

✅ Initialized ZKP generator

🔄 Processing agent agent_1 (analytical)...
   🤖 LLM response: [-0.487, -0.069, 0.680, ...]
   ✓ Causal response Δy: [-0.512, -0.078, 0.692, ...]
   ✓ Eigenvalues: [0.053, 0.038, 0.015]
   ✓ Spectral radius: 0.0533, Entropy: 0.9602
   ✓ ZK proof generated (1024 bytes)
   ✓ Proof verification: ✅ Valid
```

## 故障排除

### 问题：找不到 API 密钥

```
⚠️  No API key found, falling back to simulated mode
   💡 提示: 请设置环境变量 DEEPSEEK_API_KEY
```

**解决方案**：
```cmd
set DEEPSEEK_API_KEY=your_key
```

### 问题：API 调用失败

```
⚠️  LLM API call failed: ..., using simulated response
```

**解决方案**：
1. 检查 API 密钥是否正确
2. 检查网络连接
3. 验证 API 额度是否充足
4. 使用 `--no-fallback` 查看完整错误信息

### 问题：无法解析 LLM 响应

```
⚠️  Failed to parse LLM response: ..., using simulated response
```

**解决方案**：
- 这通常发生在 LLM 返回格式不正确时
- 回退机制会自动切换到模拟模式
- 可以调整 prompt 或 temperature 参数

## 支持的 DeepSeek 模型

| 模型 | 描述 | 推荐用途 |
|-----|------|---------|
| `deepseek-chat` | 通用聊天模型 | 通用对话，推理任务 |
| `deepseek-coder` | 代码专用模型 | 编程，技术分析 |

## 性能考虑

### API 调用延迟

- DeepSeek API 响应时间通常在 1-5 秒
- 10 个智能体的实验可能需要 10-50 秒
- 建议：使用较小的智能体数量进行测试

### 成本优化

- `deepseek-chat` 通常比 `gpt-4` 便宜得多
- 可以先使用小规模测试验证逻辑
- 启用缓存功能（如果需要）

## 下一步

1. **自定义 Prompt**：修改 `build_agent_prompt` 函数
2. **添加更多智能体类型**：在 `create_default_agent_prompt_identities` 中添加
3. **集成其他提供商**：参考 `llm_client.rs` 的实现
4. **优化性能**：实现并行 API 调用

## 参考资源

- DeepSeek API 文档：https://platform.deepseek.com/api-docs/
- 项目 README：../../README.md
- ZK 证明文档：../../docs/zkp/
