//! 安全的Commitment-Reveal协议示例
//! 
//! 演示如何防止智能体之间的信息泄露，确保独立思考过程
//! 并防御恶意节点攻击

use multi_agent_oracle::consensus::{
    CommitmentRevealProtocol,
    IndependentThinkingGuard,
    MaliciousDefenseManager,
    DefenseConfig,
    MaliciousBehaviorType,
    Commitment,
    Reveal,
    ProtocolPhase,
    compute_commitment_hash,
    generate_nonce,
    current_timestamp_ms,
    serialize_data,
    deserialize_data,
};
use multi_agent_oracle::types::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

/// Agent响应数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentResponse {
    /// Agent ID
    pub agent_id: NodeId,
    /// 干预向量
    pub intervention_vector: Vec<f64>,
    /// 因果响应
    pub causal_response: Vec<f64>,
    /// 谱特征
    pub spectral_features: Vec<f64>,
    /// ZK证明哈希
    pub zk_proof_hash: String,
}

/// 模拟的Agent
struct Agent {
    /// Agent ID
    id: NodeId,
    /// 模型类型（用于模拟不同的AI模型）
    model_type: String,
    /// 思考时间（毫秒）
    thinking_time_ms: u64,
    /// 是否恶意
    is_malicious: bool,
}

impl Agent {
    /// 创建新的Agent
    fn new(id: NodeId, model_type: String, thinking_time_ms: u64, is_malicious: bool) -> Self {
        Self {
            id,
            model_type,
            thinking_time_ms,
            is_malicious,
        }
    }

    /// 处理干预并生成响应
    fn process_intervention(&self, intervention: &[f64]) -> AgentResponse {
        // 模拟思考时间
        thread::sleep(Duration::from_millis(self.thinking_time_ms));

        // 模拟不同的模型响应
        let causal_response = match self.model_type.as_str() {
            "gpt4" => intervention.iter().map(|x| x * 1.2 + 0.1).collect(),
            "claude" => intervention.iter().map(|x| x * 1.1 + 0.05).collect(),
            "llama" => intervention.iter().map(|x| x * 1.3 - 0.05).collect(),
            _ => intervention.to_vec(),
        };

        // 如果是恶意Agent，尝试操控结果
        let final_response = if self.is_malicious {
            // 恶意Agent会尝试让结果偏向某个方向
            causal_response.iter().map(|x| x * 1.5 + 0.2).collect()
        } else {
            causal_response
        };

        // 计算谱特征（简化版）
        let spectral_features = vec![
            final_response.iter().sum::<f64>() / final_response.len() as f64,
            final_response.iter().map(|x| x.powi(2)).sum::<f64>(),
        ];

        AgentResponse {
            agent_id: self.id.clone(),
            intervention_vector: intervention.to_vec(),
            causal_response: final_response,
            spectral_features,
            zk_proof_hash: format!("zk_proof_{}", self.id),
        }
    }

    /// 参与Commitment-Reveal协议
    fn participate_in_protocol(
        &self,
        protocol: &mut CommitmentRevealProtocol,
        intervention: &[f64],
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 Agent {} ({}，恶意: {}) 开始计算响应...", 
            self.id, self.model_type, self.is_malicious);

        // 步骤1: 计算响应
        let response = self.process_intervention(intervention);
        let response_data = serialize_data(&response)?;

        // 步骤2: 生成随机数
        let nonce = generate_nonce();

        // 步骤3: 计算承诺哈希
        let commitment_hash = compute_commitment_hash(&response_data, &nonce);

        // 步骤4: 提交承诺（不泄露实际响应）
        let commitment = Commitment {
            agent_id: self.id.clone(),
            commitment_hash,
            timestamp: current_timestamp_ms(),
            nonce,
        };

        println!("🔒 Agent {} 提交承诺: hash={:?}", 
            self.id, &commitment_hash[..8]);

        protocol.submit_commitment(commitment)?;

        // 步骤5: 等待揭示阶段
        // 在实际系统中，这里会等待协议进入Reveal阶段
        thread::sleep(Duration::from_millis(100));

        // 步骤6: 提交揭示
        let reveal = Reveal {
            agent_id: self.id.clone(),
            response_data,
            nonce,
            timestamp: current_timestamp_ms(),
        };

        println!("📤 Agent {} 提交揭示数据", self.id);

        protocol.submit_reveal(reveal)?;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=".repeat(80));
    println!("🔐 安全的Commitment-Reveal协议演示");
    println!("=".repeat(80));

    // 创建Agent（模拟不同的AI模型）
    let agents = vec![
        Agent::new("agent1".to_string(), "gpt4".to_string(), 150, false),
        Agent::new("agent2".to_string(), "claude".to_string(), 200, false),
        Agent::new("agent3".to_string(), "llama".to_string(), 180, false),
        Agent::new("agent4".to_string(), "gpt4".to_string(), 120, true), // 恶意节点
        Agent::new("agent5".to_string(), "claude".to_string(), 220, false),
    ];

    // 创建防御管理器
    let defense_config = DefenseConfig {
        sybil_threshold: 0.75,
        collusion_similarity_threshold: 0.85,
        min_model_diversity: 3,
        min_spectral_entropy: 0.6,
        max_spectral_entropy: 0.9,
        timing_anomaly_threshold: 2.5,
        reputation_penalty_factor: 0.5,
        enable_instant_penalty: true,
    };

    let mut defense_manager = MaliciousDefenseManager::new(defense_config);

    // 注册节点IP（用于Sybil检测）
    defense_manager.register_node_ip("agent1".to_string(), "192.168.1.101".to_string());
    defense_manager.register_node_ip("agent2".to_string(), "192.168.1.102".to_string());
    defense_manager.register_node_ip("agent3".to_string(), "192.168.1.103".to_string());
    defense_manager.register_node_ip("agent4".to_string(), "192.168.1.104".to_string());
    defense_manager.register_node_ip("agent5".to_string(), "192.168.1.105".to_string());

    println!("\n📋 参与Agent信息:");
    for agent in &agents {
        println!("   - Agent {} ({}): 思考时间={}ms, 恶意={}", 
            agent.id, agent.model_type, agent.thinking_time_ms, agent.is_malicious);
    }

    // 生成随机干预向量
    let intervention_vector = vec![0.1, -0.2, 0.3, -0.1, 0.25];
    println!("\n🎲 生成的干预向量: {:?}", intervention_vector);

    // 创建协议实例
    let participating_agents: Vec<NodeId> = agents.iter().map(|a| a.id.clone()).collect();
    let mut protocol = CommitmentRevealProtocol::new(
        participating_agents.clone(),
        10000, // 10秒承诺超时
        10000, // 10秒揭示超时
    );

    // 创建独立思考保护器
    let mut thinking_guard = IndependentThinkingGuard::new(60, 1); // 60秒窗口，1秒最小思考时间

    println!("\n" + &"=".repeat(80));
    println!("🔒 阶段1: 承诺阶段 (Commitment Phase)");
    println!("=".repeat(80));

    // Agent并行计算并提交承诺
    let mut handles = vec![];
    
    for agent in &agents {
        let agent = agent.clone();
        let mut protocol_clone = CommitmentRevealProtocol::new(
            participating_agents.clone(),
            10000,
            10000,
        );
        let intervention = intervention_vector.clone();
        
        let handle = thread::spawn(move || {
            agent.participate_in_protocol(&mut protocol_clone, &intervention)
        });
        
        handles.push((agent.id.clone(), handle));
    }

    // 等待所有Agent完成
    for (agent_id, handle) in handles {
        match handle.join() {
            Ok(Ok(())) => {
                println!("✅ Agent {} 成功完成承诺阶段", agent_id);
            }
            Ok(Err(e)) => {
                println!("❌ Agent {} 承诺阶段失败: {}", agent_id, e);
            }
            Err(_) => {
                println!("💥 Agent {} 线程崩溃", agent_id);
            }
        }
    }

    // 检查协议状态
    let status = protocol.get_status();
    println!("\n📊 协议状态: {:?}", status.phase);
    println!("📊 已收到的承诺: {}/{}", status.commitments_count, status.participating_agents);

    println!("\n" + &"=".repeat(80));
    println!("📤 阶段2: 揭示阶段 (Reveal Phase)");
    println!("=".repeat(80));

    // 所有Agent揭示实际响应
    for agent in &agents {
        let agent_id = agent.id.clone();
        
        // 模拟思考时间
        thread::sleep(Duration::from_millis(agent.thinking_time_ms));
        
        // 记录思考时间
        if let Err(e) = thinking_guard.record_thinking_start(agent_id.clone()) {
            println!("⚠️  Agent {} 思考异常: {}", agent_id, e);
        }
        
        // 验证思考时间
        if let Err(e) = thinking_guard.verify_thinking_time(&agent_id, agent.thinking_time_ms) {
            println!("⚠️  Agent {} 思考时间验证失败: {}", agent_id, e);
            
            // 记录恶意行为
            defense_manager.record_malicious_behavior(
                agent_id.clone(),
                MaliciousBehaviorType::TimingAnomaly,
                0.8,
                vec![e.to_string()],
            );
        }
        
        println!("✅ Agent {} 思考时间: {}ms", agent_id, agent.thinking_time_ms);
    }

    // 检测共谋攻击
    let commitments: Vec<Commitment> = protocol.commitments.values().cloned().collect();
    let collusion_evidence = defense_manager.detect_collusion_attack(&commitments);
    
    if !collusion_evidence.is_empty() {
        println!("\n🚨 检测到共谋攻击证据:");
        for evidence in &collusion_evidence {
            println!("   - Agent {} 和 Agent {} 相似度: {:.2}%", 
                evidence.agent1, evidence.agent2, evidence.similarity_score * 100.0);
        }
    }

    // 获取验证后的响应
    match protocol.get_verified_responses() {
        Ok(responses) => {
            println!("\n✅ 成功获取 {} 个验证通过的响应", responses.len());
            
            // 分析响应数据
            println!("\n" + &"=".repeat(80));
            println!("📊 响应数据分析");
            println!("=".repeat(80));
            
            for (agent_id, response_data) in &responses {
                let response: AgentResponse = deserialize_data(response_data)?;
                
                println!("\n📈 Agent {} 响应分析:", agent_id);
                println!("   - 模型类型: {}", response.agent_id);
                println!("   - 因果响应: {:?}", &response.causal_response[..3]);
                println!("   - 谱特征: {:?}", response.spectral_features);
                println!("   - ZK证明: {}", &response.zk_proof_hash[..10]);
                
                // 检测谱熵异常
                let spectral_entropy = calculate_spectral_entropy(&response.causal_response);
                println!("   - 谱熵: {:.3}", spectral_entropy);
                
                if spectral_entropy < 0.6 || spectral_entropy > 0.9 {
                    println!("   ⚠️  谱熵异常，可能存在模型同质性或共谋");
                    
                    defense_manager.record_malicious_behavior(
                        agent_id.clone(),
                        MaliciousBehaviorType::SpectralEntropyAnomaly,
                        0.7,
                        vec![format!("谱熵异常: {}", spectral_entropy)],
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ 获取验证响应失败: {}", e);
        }
    }

    // 检测恶意节点
    println!("\n" + &"=".repeat(80));
    println!("🛡️  恶意节点检测结果");
    println!("=".repeat(80));
    
    let malicious_agents = defense_manager.get_all_malicious_nodes();
    
    if malicious_agents.is_empty() {
        println!("✅ 未检测到恶意节点");
    } else {
        println!("🚨 检测到 {} 个恶意节点:", malicious_agents.len());
        for (node_id, behavior_types) in &malicious_agents {
            println!("   - {}: {:?}", node_id, behavior_types);
            
            // 显示信誉分数
            if let Some(score) = defense_manager.get_reputation_score(node_id) {
                println!("     信誉分数: {:.2}", score);
            }
        }
    }

    // 检测Sybil攻击
    let sybil_evidence = defense_manager.detect_sybil_attack();
    if !sybil_evidence.is_empty() {
        println!("\n🚨 检测到Sybil攻击证据:");
        for evidence in &sybil_evidence {
            println!("   - IP {} 有 {} 个可疑节点，相似度: {:.2}%", 
                evidence.ip_address, 
                evidence.suspected_nodes.len(),
                evidence.similarity_score * 100.0);
        }
    }

    println!("\n" + &"=".repeat(80));
    println!("📋 最终协议状态");
    println!("=".repeat(80));
    
    let final_status = protocol.get_status();
    println!("协议阶段: {:?}", final_status.phase);
    println!("承诺数量: {}/{}", final_status.commitments_count, final_status.participating_agents);
    println!("揭示数量: {}/{}", final_status.reveals_count, final_status.commitments_count);
    
    match final_status.phase {
        ProtocolPhase::Completed => println!("✅ 协议成功完成"),
        ProtocolPhase::Failed => println!("❌ 协议失败"),
        _ => println!("⏳ 协议进行中"),
    }

    println!("\n" + &"=".repeat(80));
    println!("🎉 演示完成");
    println!("=".repeat(80));
    
    println!("\n💡 关键特性演示:");
    println!("   ✅ 承诺-揭示机制防止信息泄露");
    println!("   ✅ 独立思考保护确保自主计算");
    println!("   ✅ 异常检测识别恶意行为");
    println!("   ✅ 共谋检测防止协同攻击");
    println!("   ✅ Sybil攻击检测识别虚假身份");
    println!("   ✅ 信誉系统惩罚恶意节点");

    Ok(())
}

/// 克隆Agent结构体
impl Clone for Agent {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            model_type: self.model_type.clone(),
            thinking_time_ms: self.thinking_time_ms,
            is_malicious: self.is_malicious,
        }
    }
}

/// 计算谱熵（简化版）
fn calculate_spectral_entropy(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let sum: f64 = data.iter().map(|x| x.abs()).sum();
    if sum == 0.0 {
        return 0.0;
    }

    let mut entropy = 0.0;
    for &value in data {
        let p = value.abs() / sum;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }

    // 归一化到0-1范围
    let max_entropy = (data.len() as f64).log2();
    if max_entropy > 0.0 {
        entropy / max_entropy
    } else {
        0.0
    }
}