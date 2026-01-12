# Commitment-Reveal 协议使用指南

## 概述

本指南介绍如何使用多智能体预言机系统中的 Commitment-Reveal 协议，该协议旨在防止智能体之间的信息泄露，确保独立思考过程，并防御恶意节点攻击。

## 核心组件

### 1. Commitment-Reveal 协议 (`CommitmentRevealProtocol`)

防止智能体在计算过程中看到彼此的响应，确保独立思考。

**主要特性：**
- 两阶段协议：承诺阶段 → 揭示阶段
- 密码学哈希保证数据完整性
- 随机数防止彩虹表攻击
- 超时机制防止阻塞

### 2. 独立思考保护器 (`IndependentThinkingGuard`)

监控智能体的思考时间，检测异常行为。

**主要特性：**
- 防止思考时间过短（可能抄袭）
- 防止重复提交
- 响应时间异常检测

### 3. 恶意节点防御管理器 (`MaliciousDefenseManager`)

多层次检测和防御各种攻击。

**主要特性：**
- Sybil攻击检测
- 共谋攻击检测
- 模型同质性检测
- 响应时间异常检测
- 信誉系统

## 快速开始

### 基本使用

```rust
use multi_agent_oracle::consensus::{
    CommitmentRevealProtocol, IndependentThinkingGuard, MaliciousDefenseManager, DefenseConfig,
    Commitment, Reveal, compute_commitment_hash, generate_nonce, serialize_data
};

// 1. 创建参与Agent列表
let participating_agents = vec![
    "agent1".to_string(),
    "agent2".to_string(),
    "agent3".to_string(),
];

// 2. 创建协议实例
let mut protocol = CommitmentRevealProtocol::new(
    participating_agents,
    10000, // 承诺超时：10秒
    10000, // 揭示超时：10秒
);

// 3. 创建防御管理器
let defense_config = DefenseConfig::default();
let mut defense_manager = MaliciousDefenseManager::new(defense_config);

// 4. 创建独立思考保护器
let mut thinking_guard = IndependentThinkingGuard::new(60, 1); // 60秒窗口，1秒最小思考时间
```

### Agent参与协议

```rust
// 每个Agent独立执行以下步骤：

// 步骤1: 计算响应（独立思考过程）
let response_data = compute_agent_response(intervention_vector);
let serialized_data = serialize_data(&response_data)?;

// 步骤2: 生成随机数
let nonce = generate_nonce();

// 步骤3: 计算承诺哈希（不泄露实际数据）
let commitment_hash = compute_commitment_hash(&serialized_data, &nonce);

// 步骤4: 提交承诺
let commitment = Commitment {
    agent_id: agent_id.clone(),
    commitment_hash,
    timestamp: current_timestamp_ms(),
    nonce,
};
protocol.submit_commitment(commitment)?;

// 步骤5: 等待所有Agent提交承诺...

// 步骤6: 提交揭示（显示实际数据）
let reveal = Reveal {
    agent_id: agent_id.clone(),
    response_data: serialized_data,
    nonce,
    timestamp: current_timestamp_ms(),
};
protocol.submit_reveal(reveal)?;
```

### 验证和检测

```rust
// 获取验证通过的响应
let verified_responses = protocol.get_verified_responses()?;

// 检测共谋攻击
let commitments: Vec<Commitment> = protocol.commitments.values().cloned().collect();
let collusion_evidence = defense_manager.detect_collusion_attack(&commitments);

// 检测模型同质性
let spectral_entropies = vec![
    ("agent1".to_string(), 0.75),
    ("agent2".to_string(), 0.82),
    ("agent3".to_string(), 0.68),
];
let homogeneity_evidence = defense_manager.detect_model_homogeneity(&spectral_entropies);

// 检测Sybil攻击
let sybil_evidence = defense_manager.detect_sybil_attack();

// 获取恶意节点列表
let malicious_nodes = defense_manager.get_all_malicious_nodes();
```

## 高级用法

### 自定义防御配置

```rust
let defense_config = DefenseConfig {
    // Sybil检测阈值
    sybil_threshold: 0.75,
    
    // 共谋相似度阈值
    collusion_similarity_threshold: 0.85,
    
    // 最小模型多样性
    min_model_diversity: 3,
    
    // 谱熵健康范围
    min_spectral_entropy: 0.6,
    max_spectral_entropy: 0.9,
    
    // 响应时间异常阈值（标准差倍数）
    timing_anomaly_threshold: 2.5,
    
    // 信誉惩罚系数
    reputation_penalty_factor: 0.5,
    
    // 启用即时惩罚
    enable_instant_penalty: true,
};
```

### 监控思考时间

```rust
// 记录Agent开始思考
thinking_guard.record_thinking_start(agent_id.clone())?;

// 计算响应（模拟思考过程）
let thinking_time_ms = measure_thinking_time(|| {
    compute_agent_response(intervention)
});

// 验证思考时间是否足够
thinking_guard.verify_thinking_time(&agent_id, thinking_time_ms)?;

// 检测响应时间异常
if defense_manager.detect_timing_anomalies(&agent_id, thinking_time_ms)? {
    println!("Agent {} 响应时间异常", agent_id);
}
```

### 处理验证失败

```rust
// 验证揭示数据与承诺匹配
if !defense_manager.verify_hash_match(&agent_id, &commitment, &reveal)? {
    println!("Agent {} 哈希不匹配，可能是恶意行为", agent_id);
}

// 记录恶意行为
defense_manager.record_malicious_behavior(
    agent_id.clone(),
    MaliciousBehaviorType::HashMismatch,
    1.0, // 置信度
    vec!["承诺哈希与揭示数据不匹配".to_string()],
);

// 检查节点信誉
if let Some(score) = defense_manager.get_reputation_score(&agent_id) {
    if score < 0.5 {
        println!("Agent {} 信誉分数过低: {:.2}", agent_id, score);
    }
}
```

## 攻击防御机制

### 1. Sybil攻击防御

检测同一IP下的多个相似节点：

```rust
// 注册节点IP
defense_manager.register_node_ip("node1".to_string(), "192.168.1.1".to_string());
defense_manager.register_node_ip("node2".to_string(), "192.168.1.1".to_string());
defense_manager.register_node_ip("node3".to_string(), "192.168.1.1".to_string());

// 检测Sybil攻击
let evidence = defense_manager.detect_sybil_attack();
if !evidence.is_empty() {
    for sybil in evidence {
        println!("检测到Sybil攻击: IP {} 有 {} 个可疑节点",
            sybil.ip_address, sybil.suspected_nodes.len());
    }
}
```

### 2. 共谋攻击防御

检测高度相似的承诺（可能协同作弊）：

```rust
// 检测共谋
let evidence = defense_manager.detect_collusion_attack(&commitments);
for collusion in evidence {
    println!("检测到共谋: Agent {} 和 Agent {} 相似度 {:.2}%",
        collusion.agent1,
        collusion.agent2,
        collusion.similarity_score * 100.0);
}
```

### 3. 模型同质性检测

检测谱熵异常（可能使用相同模型）：

```rust
let spectral_entropies = vec![
    ("agent1".to_string(), 0.3), // 过低，可能共谋
    ("agent2".to_string(), 0.35),
    ("agent3".to_string(), 0.92),
];

let evidence = defense_manager.detect_model_homogeneity(&spectral_entropies);
if !evidence.is_empty() {
    println!("检测到模型同质性异常");
}
```

### 4. 响应时间异常检测

检测思考时间异常（可能抄袭）：

```rust
// 正常响应时间
for _ in 0..10 {
    defense_manager.detect_timing_anomalies(&agent_id, 1000)?; // 1秒
}

// 异常响应时间（突然变得很快或很慢）
if defense_manager.detect_timing_anomalies(&agent_id, 10000)? { // 10秒
    println!("Agent {} 响应时间异常，可能是恶意行为", agent_id);
}
```

## 完整示例

参见示例文件：`examples/secure_commitment_reveal.rs`

运行示例：

```bash
cargo run --example secure_commitment_reveal
```

## 安全特性

### 密码学保证
- SHA256哈希确保数据完整性
- 随机数防止彩虹表攻击
- 时间戳防止重放攻击

### 统计学保证
- 谱熵检测模型多样性
- Z-score检测响应时间异常
- 相似度分析检测共谋

### 博弈论保证
- 信誉系统激励诚实行为
- 惩罚机制降低作弊收益
- 独立思考要求增加作弊成本

## 性能考虑

- 哈希计算：O(n)，其中n是数据大小
- 共谋检测：O(m²)，其中m是Agent数量
- 建议Agent数量：10-100个
- 响应数据大小：建议 < 10KB

## 最佳实践

1. **设置合理的超时时间**
   - 承诺超时：根据网络延迟设置（建议5-30秒）
   - 揭示超时：根据计算复杂度设置（建议10-60秒）

2. **配置适当的阈值**
   - 共谋相似度：0.85-0.95（越高越严格）
   - 谱熵范围：根据模型多样性调整
   - 响应时间异常：2-3倍标准差

3. **监控信誉分数**
   - 定期检查节点信誉
   - 对低信誉节点增加审查
   - 考虑长期信誉历史

4. **处理协议失败**
   - 记录失败原因
   - 识别恶意节点
   - 考虑重新启动协议

## 常见问题

**Q: 如果某个Agent在承诺阶段超时怎么办？**
A: 协议会标记该Agent为恶意，并在达到阈值时失败。可以设置 `malicious_threshold` 控制容忍度。

**Q: 如何防止Agent在揭示阶段提交虚假数据？**
A: 承诺哈希确保Agent无法更改已承诺的数据。任何不匹配都会被检测到。

**Q: 系统能处理多少个Agent？**
A: 理论上是无限的，但共谋检测是O(n²)复杂度。建议最多100个Agent以获得良好性能。

**Q: 谱熵阈值如何设置？**
A: 根据模型多样性设置。3-5个不同模型时，建议0.6-0.9。

## 相关文档

- [ZK指纹架构](./ZK_ARCHITECTURE.md)
- [安全机制](./safety.md)
- [簇中心安全](./Cluster%20Center.md)
- [因果指纹](./Causal%20Fingerprinting.md)
