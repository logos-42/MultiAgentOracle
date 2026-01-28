//! 因果指纹模块
//!
//! 实现核心的因果指纹验证算法：
//! - 增量响应计算：Δy = f(x+δ) - f(x)
//! - 用于共识的余弦相似度聚类
//! - 基于逻辑一致性的异常值检测

#![allow(dead_code, missing_docs)]

use serde::{Deserialize, Serialize};
use rand::Rng;
use std::collections::HashSet;

/// 智能体的因果指纹数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalFingerprint {
    pub agent_id: String,
    pub base_prediction: f64,
    pub delta_response: Vec<f64>,           // Δy = f(x+δ) - f(x)
    pub spectral_features: Vec<f64>,        // 谱特征值
    pub perturbation: Vec<f64>,             // δ
    pub confidence: f64,
    pub timestamp: u64,
}

/// 共识聚合结果
#[derive(Debug, Clone)]
pub struct ConsensusResult {
    pub consensus_value: f64,
    pub valid_agents: Vec<String>,
    pub outliers: Vec<String>,
    pub consensus_similarity: f64,
    pub cluster_quality: f64,
}

/// 因果指纹验证配置
#[derive(Debug, Clone)]
pub struct CausalFingerprintConfig {
    pub cosine_threshold: f64,           // 共识所需的最小相似度
    pub outlier_threshold: f64,          // 被视为内点的最大距离
    pub min_valid_agents: usize,         // 所需的最小智能体数量
    pub spectral_dimensions: usize,      // 谱特征数量
    pub perturbation_dimensions: usize,  // 扰动维度数量
}

impl Default for CausalFingerprintConfig {
    fn default() -> Self {
        Self {
            cosine_threshold: 0.85,
            outlier_threshold: 2.0,
            min_valid_agents: 3,
            spectral_dimensions: 8,
            perturbation_dimensions: 5,
        }
    }
}

/// 计算增量响应：Δy = f(x+δ) - f(x)
pub fn calculate_delta_response(
    base_prediction: f64,
    perturbed_prediction: f64,
) -> f64 {
    perturbed_prediction - base_prediction
}

/// 生成随机扰动向量 δ
pub fn generate_perturbation(dim: usize, magnitude: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-magnitude..magnitude)).collect()
}

/// 计算两个向量之间的余弦相似度
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

/// 计算两个向量之间的欧几里得距离
pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// 从指纹中计算共识质心
pub fn compute_consensus_centroid(fingerprints: &[&CausalFingerprint]) -> Vec<f64> {
    if fingerprints.is_empty() {
        return Vec::new();
    }
    
    let dim = fingerprints[0].delta_response.len();
    let mut centroid = vec![0.0; dim];
    
    for fp in fingerprints {
        for i in 0..dim {
            centroid[i] += fp.delta_response[i];
        }
    }
    
    for val in &mut centroid {
        *val /= fingerprints.len() as f64;
    }
    
    centroid
}

/// 基于与质心的相似度识别异常值
pub fn identify_outliers(
    fingerprints: &[CausalFingerprint],
    centroid: &[f64],
    threshold: f64,
) -> Vec<usize> {
    let mut outliers = Vec::new();
    
    for (i, fp) in fingerprints.iter().enumerate() {
        let similarity = cosine_similarity(&fp.delta_response, centroid);
        if similarity < threshold {
            outliers.push(i);
        }
    }
    
    outliers
}

/// 用于共识的余弦相似度聚类
pub fn cluster_by_consensus(
    fingerprints: &[CausalFingerprint],
    config: &CausalFingerprintConfig,
) -> ConsensusResult {
    if fingerprints.len() < config.min_valid_agents {
        return ConsensusResult {
            consensus_value: 0.0,
            valid_agents: Vec::new(),
            outliers: fingerprints.iter().map(|f| f.agent_id.clone()).collect(),
            consensus_similarity: 0.0,
            cluster_quality: 0.0,
        };
    }
    
    // 计算成对相似度
    let mut similarities: Vec<(usize, usize, f64)> = Vec::new();
    for i in 0..fingerprints.len() {
        for j in (i + 1)..fingerprints.len() {
            let sim = cosine_similarity(&fingerprints[i].delta_response, &fingerprints[j].delta_response);
            similarities.push((i, j, sim));
        }
    }
    
    // 寻找具有高内部相似度的最大聚类
    let threshold = config.cosine_threshold;
    let mut agent_clusters: Vec<Vec<usize>> = Vec::new();
    let mut used: HashSet<usize> = HashSet::new();
    
    for (i, j, sim) in &similarities {
        if *sim >= threshold {
            if used.contains(i) {
                if !used.contains(j) {
                    // 找到 i 的聚类并添加 j
                    for cluster in &mut agent_clusters {
                        if cluster.contains(i) {
                            cluster.push(*j);
                            used.insert(*j);
                            break;
                        }
                    }
                }
            } else if !used.contains(j) {
                // 创建新聚类
                let new_cluster = vec![*i, *j];
                agent_clusters.push(new_cluster);
                used.insert(*i);
                used.insert(*j);
            }
        }
    }
    
    // 寻找最大聚类
    let mut largest_cluster = Vec::new();
    for cluster in &agent_clusters {
        if cluster.len() > largest_cluster.len() {
            largest_cluster = cluster.clone();
        }
    }
    
    // 从最大聚类中计算共识
    let mut consensus_value = 0.0;
    let mut valid_agents = Vec::new();
    let mut cluster_quality = 0.0;
    
    if !largest_cluster.is_empty() {
        let cluster_fps: Vec<&CausalFingerprint> = largest_cluster.iter()
            .map(|&i| &fingerprints[i])
            .collect();
        
        let centroid = compute_consensus_centroid(&cluster_fps);
        
        // 共识值是按相似度加权的基础预测平均值
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        
        for &i in &largest_cluster {
            let sim = cosine_similarity(&fingerprints[i].delta_response, &centroid);
            weighted_sum += fingerprints[i].base_prediction * sim;
            weight_sum += sim;
            valid_agents.push(fingerprints[i].agent_id.clone());
        }
        
        consensus_value = if weight_sum > 0.0 { weighted_sum / weight_sum } else { 0.0 };
        
        // 计算聚类质量（平均内部相似度）
        cluster_quality = similarities.iter()
            .filter(|(i, j, _)| largest_cluster.contains(i) && largest_cluster.contains(j))
            .map(|(_, _, s)| s)
            .sum::<f64>() / largest_cluster.len().max(1) as f64;
    }
    
    // 识别异常值
    let mut outliers = Vec::new();
    for (i, fp) in fingerprints.iter().enumerate() {
        if !largest_cluster.contains(&i) {
            outliers.push(fp.agent_id.clone());
        }
    }
    
    let consensus_similarity = if !valid_agents.is_empty() {
        let cluster_fps: Vec<&CausalFingerprint> = valid_agents.iter()
            .zip(fingerprints.iter())
            .filter_map(|(name, fp)| if fp.agent_id == *name { Some(fp) } else { None })
            .collect();
        
        let centroid = compute_consensus_centroid(&cluster_fps);
        fingerprints.iter()
            .filter(|fp| valid_agents.contains(&fp.agent_id))
            .map(|fp| cosine_similarity(&fp.delta_response, &centroid))
            .sum::<f64>() / valid_agents.len() as f64
    } else {
        0.0
    };
    
    ConsensusResult {
        consensus_value,
        valid_agents,
        outliers,
        consensus_similarity,
        cluster_quality,
    }
}

/// 从响应矩阵生成谱特征（简化版）
pub fn extract_spectral_features(responses: &[Vec<f64>]) -> Vec<f64> {
    if responses.is_empty() || responses[0].is_empty() {
        return vec![0.0; 8];
    }
    
    let n = responses.len();
    let m = responses[0].len();
    
    // 计算协方差矩阵（简化版）
    let mut means = vec![0.0; m];
    for response in responses {
        for (j, val) in response.iter().enumerate() {
            means[j] += val;
        }
    }
    for mean in &mut means {
        *mean /= n as f64;
    }
    
    let mut features = Vec::with_capacity(8);
    
    // 特征 1-4：主成分的简单统计
    let mut variances: Vec<f64> = Vec::with_capacity(m);
    for j in 0..m {
        let var: f64 = responses.iter()
            .map(|r| (r[j] - means[j]).powi(2))
            .sum::<f64>() / n as f64;
        variances.push(var);
    }
    
    // 对方差进行排序（主成分）
    variances.sort_by(|a, b| b.partial_cmp(a).unwrap());
    
    // 添加前 4 个方差作为特征
    for i in 0..4.min(variances.len()) {
        features.push(variances[i]);
    }
    
    // 用统计值填充剩余特征
    while features.len() < 8 {
        let sum: f64 = features.iter().sum();
        let count = features.len() as f64;
        features.push(if count > 0.0 { sum / count } else { 0.0 });
    }
    
    features
}

/// 检测多个智能体是否使用相同的基础模型
pub fn detect_model_homogeneity(fingerprints: &[CausalFingerprint], threshold: f64) -> bool {
    if fingerprints.len() < 2 {
        return false;
    }
    
    for i in 0..fingerprints.len() {
        for j in (i + 1)..fingerprints.len() {
            let spec_sim = cosine_similarity(
                &fingerprints[i].spectral_features,
                &fingerprints[j].spectral_features,
            );
            if spec_sim > threshold {
                return true;
            }
        }
    }
    
    false
}

/// 计算智能体的逻辑一致性分数
pub fn logical_consistency_score(
    agent_fp: &CausalFingerprint,
    global_fingerprint: &[f64],
) -> f64 {
    cosine_similarity(&agent_fp.spectral_features, global_fingerprint)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);
        
        let c = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &c), 0.0);
        
        let d = vec![-1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &d), -1.0);
    }
    
    #[test]
    fn test_delta_response() {
        assert_eq!(calculate_delta_response(100.0, 105.0), 5.0);
        assert_eq!(calculate_delta_response(100.0, 95.0), -5.0);
    }
    
    #[test]
    fn test_perturbation_generation() {
        let perturbation = generate_perturbation(5, 10.0);
        assert_eq!(perturbation.len(), 5);
        for val in &perturbation {
            assert!(*val >= -10.0 && *val <= 10.0);
        }
    }
    
    #[test]
    fn test_cluster_by_consensus() {
        let config = CausalFingerprintConfig::default();
        
        let fingerprints = vec![
            CausalFingerprint {
                agent_id: "agent1".to_string(),
                base_prediction: 100.0,
                delta_response: vec![1.0, 2.0, 3.0],
                spectral_features: vec![0.1, 0.2, 0.3],
                perturbation: vec![0.1, 0.1, 0.1],
                confidence: 0.9,
                timestamp: 0,
            },
            CausalFingerprint {
                agent_id: "agent2".to_string(),
                base_prediction: 101.0,
                delta_response: vec![1.1, 2.1, 3.1],
                spectral_features: vec![0.11, 0.21, 0.31],
                perturbation: vec![0.1, 0.1, 0.1],
                confidence: 0.85,
                timestamp: 0,
            },
            CausalFingerprint {
                agent_id: "agent3".to_string(),
                base_prediction: 50.0,
                delta_response: vec![-5.0, -10.0, -15.0],
                spectral_features: vec![0.5, 0.6, 0.7],
                perturbation: vec![0.1, 0.1, 0.1],
                confidence: 0.7,
                timestamp: 0,
            },
        ];
        
        let result = cluster_by_consensus(&fingerprints, &config);
        
        // agent1 和 agent2 应该达成共识
        assert_eq!(result.valid_agents.len(), 2);
        assert!(result.valid_agents.contains(&"agent1".to_string()));
        assert!(result.valid_agents.contains(&"agent2".to_string()));
        assert_eq!(result.outliers.len(), 1);
        assert!(result.outliers.contains(&"agent3".to_string()));
    }
}
