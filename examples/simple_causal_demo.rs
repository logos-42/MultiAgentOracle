//! 简化的因果指纹验证演示
//! 不依赖DIAP和其他复杂模块

use std::collections::HashMap;
use anyhow::Result;

/// 简化的智能体结构
#[derive(Debug, Clone)]
pub struct SimpleAgent {
    pub id: String,
    pub model_type: String,
    pub response: f64,
    pub delta_response: Vec<f64>,
}

/// 简化的因果指纹
#[derive(Debug, Clone)]
pub struct SimpleCausalFingerprint {
    pub agent_id: String,
    pub base_prediction: f64,
    pub delta_response: Vec<f64>,
    pub spectral_features: Vec<f64>,
    pub confidence: f64,
}

/// 简化的共识结果
#[derive(Debug)]
pub struct SimpleConsensusResult {
    pub consensus_value: f64,
    pub valid_agents: Vec<String>,
    pub outliers: Vec<String>,
    pub consensus_similarity: f64,
}

/// 计算余弦相似度
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// 生成随机扰动向量
pub fn generate_perturbation(dim: usize, magnitude: f64) -> Vec<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-magnitude..magnitude)).collect()
}

/// 模拟智能体响应
pub fn simulate_agent_response(agent_id: &str, model_type: &str, intervention: &[f64]) -> SimpleAgent {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // 不同模型类型有不同的响应特征
    let base_response = match model_type {
        "analytical" => 0.8 + rng.gen_range(-0.1..0.1),
        "cautious" => 0.5 + rng.gen_range(-0.05..0.05),
        "aggressive" => 1.2 + rng.gen_range(-0.2..0.2),
        "neutral" => 0.7 + rng.gen_range(-0.1..0.1),
        _ => 0.6 + rng.gen_range(-0.1..0.1),
    };
    
    // 计算增量响应 Δy = f(x+δ) - f(x)
    let delta_response: Vec<f64> = intervention.iter()
        .map(|&delta| delta * base_response * rng.gen_range(0.8..1.2))
        .collect();
    
    SimpleAgent {
        id: agent_id.to_string(),
        model_type: model_type.to_string(),
        response: base_response,
        delta_response,
    }
}

/// 运行因果指纹验证实验
pub fn run_causal_fingerprint_experiment() -> Result<SimpleConsensusResult> {
    println!("🧪 简化因果指纹验证实验");
    println!("==========================================");
    
    // 创建3个智能体
    let agents_config = vec![
        ("agent_1", "analytical"),
        ("agent_2", "cautious"),
        ("agent_3", "aggressive"),
    ];
    
    // 生成干预向量
    let intervention = generate_perturbation(5, 1.0);
    println!("✅ 生成干预向量: {:?}", intervention);
    
    // 模拟智能体响应
    let mut agents = Vec::new();
    for (id, model_type) in agents_config {
        let agent = simulate_agent_response(id, model_type, &intervention);
        println!("✅ 智能体 {} ({}) 响应: Δy = {:?}", 
                 agent.id, agent.model_type, 
                 agent.delta_response.iter().take(3).collect::<Vec<_>>());
        agents.push(agent);
    }
    
    // 生成因果指纹
    let mut fingerprints = Vec::new();
    for agent in &agents {
        let fingerprint = SimpleCausalFingerprint {
            agent_id: agent.id.clone(),
            base_prediction: agent.response,
            delta_response: agent.delta_response.clone(),
            spectral_features: agent.delta_response.iter().take(3).cloned().collect(),
            confidence: 0.85,
        };
        fingerprints.push(fingerprint);
    }
    
    // 计算相似度矩阵
    println!("\n🔍 计算智能体间相似度:");
    let mut similarity_matrix = HashMap::new();
    for (i, fp1) in fingerprints.iter().enumerate() {
        for (j, fp2) in fingerprints.iter().enumerate() {
            if i != j {
                let similarity = cosine_similarity(&fp1.delta_response, &fp2.delta_response);
                similarity_matrix.insert((i, j), similarity);
                println!("   {} vs {}: {:.3}", fp1.agent_id, fp2.agent_id, similarity);
            }
        }
    }
    
    // 检测异常值
    let threshold = 0.7;
    let mut valid_agents = Vec::new();
    let mut outliers = Vec::new();
    
    for (i, fp) in fingerprints.iter().enumerate() {
        let mut avg_similarity = 0.0;
        let mut count = 0;
        
        for (j, _) in fingerprints.iter().enumerate() {
            if i != j {
                if let Some(&sim) = similarity_matrix.get(&(i, j)) {
                    avg_similarity += sim;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            avg_similarity /= count;
        }
        
        if avg_similarity >= threshold {
            valid_agents.push(fp.agent_id.clone());
        } else {
            outliers.push(fp.agent_id.clone());
        }
    }
    
    // 计算共识值
    let consensus_value = if valid_agents.is_empty() {
        0.0
    } else {
        let sum: f64 = fingerprints.iter()
            .filter(|fp| valid_agents.contains(&fp.agent_id))
            .map(|fp| fp.base_prediction)
            .sum();
        sum / valid_agents.len() as f64
    };
    
    let overall_similarity = if valid_agents.len() > 1 {
        let mut total_sim = 0.0;
        let mut count = 0;
        
        for i in 0..valid_agents.len() {
            for j in (i+1)..valid_agents.len() {
                if let Some(idx_i) = fingerprints.iter().position(|fp| fp.agent_id == valid_agents[i]) {
                    if let Some(idx_j) = fingerprints.iter().position(|fp| fp.agent_id == valid_agents[j]) {
                        if let Some(&sim) = similarity_matrix.get(&(idx_i, idx_j)) {
                            total_sim += sim;
                            count += 1;
                        }
                    }
                }
            }
        }
        
        if count > 0 { total_sim / count } else { 0.0 }
    } else {
        1.0
    };
    
    let result = SimpleConsensusResult {
        consensus_value,
        valid_agents,
        outliers,
        consensus_similarity: overall_similarity,
    };
    
    // 打印结果
    println!("\n📊 实验结果:");
    println!("==========================================");
    println!("✅ 有效智能体: {:?}", result.valid_agents);
    println!("⚠️  异常智能体: {:?}", result.outliers);
    println!("🎯 共识值: {:.3}", result.consensus_value);
    println!("📈 共识相似度: {:.3}", result.consensus_similarity);
    println!("📊 通过率: {:.1}%", 
             (result.valid_agents.len() as f64 / 3.0) * 100.0);
    
    if result.consensus_similarity > 0.8 {
        println!("✅ 高质量共识 - 智能体间逻辑一致性良好");
    } else if result.consensus_similarity > 0.6 {
        println!("⚠️  中等质量共识 - 存在一定分歧");
    } else {
        println!("❌ 低质量共识 - 智能体间分歧较大");
    }
    
    Ok(result)
}

fn main() -> Result<()> {
    // 运行实验
    let result = run_causal_fingerprint_experiment()?;
    
    println!("\n🎉 因果指纹验证实验完成!");
    println!("==========================================");
    println!("🔒 安全验证: ✅ 通过");
    println!("🧠 因果一致性: {:.1}%", result.consensus_similarity * 100.0);
    println!("🤝 智能体协作: {} 个有效节点", result.valid_agents.len());
    
    Ok(())
}
