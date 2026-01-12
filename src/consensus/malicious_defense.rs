//! 恶意节点攻击防御模块
//! 
//! 提供多层次的恶意行为检测和防御机制
//! 包括：Sybil攻击防御、共谋检测、异常行为识别等

use crate::types::NodeId;
use crate::consensus::commitment_reveal::{Commitment, Reveal, ProtocolError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// 恶意行为类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaliciousBehaviorType {
    /// Sybil攻击（女巫攻击）
    SybilAttack,
    /// 共谋攻击
    CollusionAttack,
    /// 响应时间异常
    TimingAnomaly,
    /// 哈希不匹配
    HashMismatch,
    /// 重复承诺
    DuplicateCommitment,
    /// 超时未响应
    Timeout,
    /// 逻辑一致性异常
    LogicalInconsistency,
    /// 谱熵异常（模型同质性）
    SpectralEntropyAnomaly,
}

/// 恶意节点记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaliciousNodeRecord {
    /// 节点ID
    pub node_id: NodeId,
    /// 检测到的恶意行为类型
    pub behavior_type: MaliciousBehaviorType,
    /// 检测时间
    pub detected_at: u64,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f64,
    /// 详细证据
    pub evidence: Vec<String>,
    /// 惩罚分数
    pub penalty_score: f64,
}

/// 防御配置
#[derive(Debug, Clone)]
pub struct DefenseConfig {
    /// Sybil攻击检测阈值
    pub sybil_threshold: f64,
    /// 共谋检测阈值（相似度）
    pub collusion_similarity_threshold: f64,
    /// 最小Agent多样性（不同模型数量）
    pub min_model_diversity: usize,
    /// 谱熵健康范围最小值
    pub min_spectral_entropy: f64,
    /// 谱熵健康范围最大值
    pub max_spectral_entropy: f64,
    /// 响应时间异常阈值（标准差倍数）
    pub timing_anomaly_threshold: f64,
    /// 信誉惩罚系数
    pub reputation_penalty_factor: f64,
    /// 是否启用即时惩罚
    pub enable_instant_penalty: bool,
}

/// 恶意节点防御管理器
pub struct MaliciousDefenseManager {
    /// 配置
    config: DefenseConfig,
    /// 恶意节点记录
    malicious_records: HashMap<NodeId, Vec<MaliciousNodeRecord>>,
    /// 节点行为历史
    behavior_history: HashMap<NodeId, BehaviorHistory>,
    /// IP地址映射（用于Sybil检测）
    ip_mappings: HashMap<String, HashSet<NodeId>>,
    /// 模型指纹映射
    model_fingerprints: HashMap<NodeId, Vec<f64>>,
    /// 共谋检测记录
    collusion_records: HashMap<(NodeId, NodeId), CollusionRecord>,
}

/// 行为历史
#[derive(Debug, Clone)]
struct BehaviorHistory {
    /// 响应时间历史
    response_times: Vec<u64>,
    /// 承诺历史
    commitments: Vec<CommitmentRecord>,
    /// 谱熵历史
    spectral_entropy_history: Vec<f64>,
    /// 信誉分数
    reputation_score: f64,
}

/// 承诺记录
#[derive(Debug, Clone)]
struct CommitmentRecord {
    pub timestamp: u64,
    pub was_verified: bool,
}

/// 共谋记录
#[derive(Debug, Clone)]
struct CollusionRecord {
    pub similarity_score: f64,
    pub detection_count: usize,
    pub last_detected: u64,
}

impl MaliciousDefenseManager {
    /// 创建新的防御管理器
    pub fn new(config: DefenseConfig) -> Self {
        Self {
            config,
            malicious_records: HashMap::new(),
            behavior_history: HashMap::new(),
            ip_mappings: HashMap::new(),
            model_fingerprints: HashMap::new(),
            collusion_records: HashMap::new(),
        }
    }

    /// 注册节点IP地址（用于Sybil检测）
    pub fn register_node_ip(&mut self, node_id: NodeId, ip_address: String) {
        self.ip_mappings.entry(ip_address)
            .or_insert_with(HashSet::new)
            .insert(node_id);
    }

    /// 检测Sybil攻击
    pub fn detect_sybil_attack(&self) -> Vec<SybilAttackEvidence> {
        let mut evidence = Vec::new();

        for (ip_address, node_ids) in &self.ip_mappings {
            if node_ids.len() >= 3 {
                // 同一IP有3个或以上节点
                let similarity_score = self.calculate_node_similarity(node_ids);
                
                if similarity_score > self.config.sybil_threshold {
                    evidence.push(SybilAttackEvidence {
                        ip_address: ip_address.clone(),
                        suspected_nodes: node_ids.clone(),
                        similarity_score,
                        confidence: self.calculate_sybil_confidence(node_ids),
                    });
                }
            }
        }

        evidence
    }

    /// 检测共谋攻击
    pub fn detect_collusion_attack(&mut self, commitments: &[Commitment]) -> Vec<CollusionEvidence> {
        let mut evidence = Vec::new();
        let now = current_timestamp_ms();

        // 分析承诺的相似性
        for i in 0..commitments.len() {
            for j in (i + 1)..commitments.len() {
                let agent1 = &commitments[i].agent_id;
                let agent2 = &commitments[j].agent_id;
                
                // 计算承诺哈希的相似性
                let hash_similarity = self.calculate_hash_similarity(
                    &commitments[i].commitment_hash,
                    &commitments[j].commitment_hash,
                );
                
                // 计算时间戳的相似性
                let time_similarity = self.calculate_time_similarity(
                    commitments[i].timestamp,
                    commitments[j].timestamp,
                );
                
                // 综合相似度
                let total_similarity = (hash_similarity + time_similarity) / 2.0;
                
                if total_similarity > self.config.collusion_similarity_threshold {
                    let collusion_key = self.get_collusion_key(agent1, agent2);
                    let record = self.collusion_records.entry(collusion_key).or_insert(CollusionRecord {
                        similarity_score: 0.0,
                        detection_count: 0,
                        last_detected: 0,
                    });
                    
                    record.similarity_score = total_similarity;
                    record.detection_count += 1;
                    record.last_detected = now;
                    
                    evidence.push(CollusionEvidence {
                        agent1: agent1.clone(),
                        agent2: agent2.clone(),
                        similarity_score: total_similarity,
                        detection_count: record.detection_count,
                        evidence_type: CollusionEvidenceType::HighSimilarity,
                    });
                }
            }
        }

        evidence
    }

    /// 检测模型同质性（谱熵异常）
    pub fn detect_model_homogeneity(&self, spectral_entropies: &[(NodeId, f64)]) -> Vec<HomogeneityEvidence> {
        let mut evidence = Vec::new();

        for (node_id, entropy) in spectral_entropies {
            if *entropy < self.config.min_spectral_entropy || *entropy > self.config.max_spectral_entropy {
                evidence.push(HomogeneityEvidence {
                    node_id: node_id.clone(),
                    spectral_entropy: *entropy,
                    expected_range: (self.config.min_spectral_entropy, self.config.max_spectral_entropy),
                    evidence_type: if *entropy < self.config.min_spectral_entropy {
                        HomogeneityEvidenceType::TooLow // 谱熵过低，可能共谋
                    } else {
                        HomogeneityEvidenceType::TooHigh
                    },
                });
            }
        }

        // 检查整体多样性
        if spectral_entropies.len() >= self.config.min_model_diversity {
            let unique_models = self.count_unique_models(spectral_entropies);
            if unique_models < self.config.min_model_diversity {
                evidence.push(HomogeneityEvidence {
                    node_id: "ALL".to_string(),
                    spectral_entropy: 0.0,
                    expected_range: (self.config.min_spectral_entropy, self.config.max_spectral_entropy),
                    evidence_type: HomogeneityEvidenceType::InsufficientDiversity,
                });
            }
        }

        evidence
    }

    /// 检测响应时间异常
    pub fn detect_timing_anomalies(&mut self, agent_id: &NodeId, response_time_ms: u64) -> Result<bool, ProtocolError> {
        let history = self.behavior_history.entry(agent_id.clone()).or_insert_with(|| {
            BehaviorHistory {
                response_times: Vec::new(),
                commitments: Vec::new(),
                spectral_entropy_history: Vec::new(),
                reputation_score: 1.0,
            }
        });

        history.response_times.push(response_time_ms);
        
        // 限制历史记录数量
        if history.response_times.len() > 100 {
            history.response_times.remove(0);
        }

        // 使用统计方法检测异常
        if history.response_times.len() >= 10 {
            let mean = history.response_times.iter().sum::<u64>() as f64 / history.response_times.len() as f64;
            let variance = history.response_times.iter()
                .map(|t| (*t as f64 - mean).powi(2))
                .sum::<f64>() / history.response_times.len() as f64;
            let std_dev = variance.sqrt();

            let z_score = (response_time_ms as f64 - mean).abs() / std_dev;
            
            if z_score > self.config.timing_anomaly_threshold {
                // 记录恶意行为
                self.record_malicious_behavior(
                    agent_id.clone(),
                    MaliciousBehaviorType::TimingAnomaly,
                    0.9,
                    vec![format!("响应时间异常: {}ms, 均值: {:.2}ms, 标准差: {:.2}ms, Z-score: {:.2}", 
                        response_time_ms, mean, std_dev, z_score)],
                );
                
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 验证哈希匹配
    pub fn verify_hash_match(
        &mut self,
        agent_id: &NodeId,
        commitment: &Commitment,
        reveal: &Reveal,
    ) -> Result<bool, ProtocolError> {
        use crate::consensus::commitment_reveal::compute_commitment_hash;
        
        let computed_hash = compute_commitment_hash(&reveal.response_data, &reveal.nonce);
        
        if computed_hash != commitment.commitment_hash {
            // 记录恶意行为
            self.record_malicious_behavior(
                agent_id.clone(),
                MaliciousBehaviorType::HashMismatch,
                1.0,
                vec![
                    "承诺哈希与揭示数据不匹配".to_string(),
                    format!("预期哈希: {:?}", commitment.commitment_hash),
                    format!("实际哈希: {:?}", computed_hash),
                ],
            );
            
            return Ok(false);
        }

        // 记录成功的承诺
        let history = self.behavior_history.entry(agent_id.clone()).or_insert_with(|| {
            BehaviorHistory {
                response_times: Vec::new(),
                commitments: Vec::new(),
                spectral_entropy_history: Vec::new(),
                reputation_score: 1.0,
            }
        });

        history.commitments.push(CommitmentRecord {
            timestamp: current_timestamp_ms(),
            was_verified: true,
        });

        Ok(true)
    }

    /// 记录恶意行为
    fn record_malicious_behavior(
        &mut self,
        node_id: NodeId,
        behavior_type: MaliciousBehaviorType,
        confidence: f64,
        evidence: Vec<String>,
    ) {
        let penalty_score = self.calculate_penalty_score(behavior_type, confidence);
        
        let record = MaliciousNodeRecord {
            node_id: node_id.clone(),
            behavior_type,
            detected_at: current_timestamp_ms(),
            confidence,
            evidence: evidence.clone(),
            penalty_score,
        };

        self.malicious_records.entry(node_id.clone())
            .or_insert_with(Vec::new)
            .push(record);

        // 更新信誉分数
        if let Some(history) = self.behavior_history.get_mut(&node_id) {
            history.reputation_score *= (1.0 - penalty_score).max(0.0);
        }

        // 如果启用了即时惩罚
        if self.config.enable_instant_penalty && penalty_score > 0.5 {
            println!("⚠️  节点 {} 检测到恶意行为，置信度: {:.2}%，立即惩罚分数: {:.2}", 
                node_id, confidence * 100.0, penalty_score);
        }
    }

    /// 计算惩罚分数
    fn calculate_penalty_score(&self, behavior_type: MaliciousBehaviorType, confidence: f64) -> f64 {
        let base_penalty = match behavior_type {
            MaliciousBehaviorType::SybilAttack => 0.8,
            MaliciousBehaviorType::CollusionAttack => 0.7,
            MaliciousBehaviorType::HashMismatch => 0.9,
            MaliciousBehaviorType::TimingAnomaly => 0.3,
            MaliciousBehaviorType::DuplicateCommitment => 0.4,
            MaliciousBehaviorType::Timeout => 0.2,
            MaliciousBehaviorType::LogicalInconsistency => 0.6,
            MaliciousBehaviorType::SpectralEntropyAnomaly => 0.5,
        };

        base_penalty * confidence * self.config.reputation_penalty_factor
    }

    /// 计算节点相似度（用于Sybil检测）
    fn calculate_node_similarity(&self, node_ids: &HashSet<NodeId>) -> f64 {
        if node_ids.len() < 2 {
            return 0.0;
        }

        // 这里简化处理，实际应该分析节点的行为模式、响应时间分布等
        // 返回一个0-1之间的相似度分数
        0.85 // 假设高度相似
    }

    /// 计算Sybil攻击置信度
    fn calculate_sybil_confidence(&self, node_ids: &HashSet<NodeId>) -> f64 {
        let count = node_ids.len() as f64;
        (count / 5.0).min(1.0) // 节点越多，置信度越高（上限5个节点）
    }

    /// 计算哈希相似度
    fn calculate_hash_similarity(&self, hash1: &[u8; 32], hash2: &[u8; 32]) -> f64 {
        // 计算两个哈希的汉明距离
        let mut distance = 0;
        for (b1, b2) in hash1.iter().zip(hash2.iter()) {
            distance += (b1 ^ b2).count_ones();
        }

        // 转换为相似度（0-1）
        let max_distance = 32.0 * 8.0; // 256位
        1.0 - (distance as f64 / max_distance)
    }

    /// 计算时间相似度
    fn calculate_time_similarity(&self, time1: u64, time2: u64) -> f64 {
        let diff = (time1 as i64 - time2 as i64).abs() as f64;
        let threshold = 1000.0; // 1秒阈值
        
        if diff > threshold {
            0.0
        } else {
            1.0 - (diff / threshold)
        }
    }

    /// 获取共谋键
    fn get_collusion_key(&self, agent1: &NodeId, agent2: &NodeId) -> (NodeId, NodeId) {
        if agent1 < agent2 {
            (agent1.clone(), agent2.clone())
        } else {
            (agent2.clone(), agent1.clone())
        }
    }

    /// 计算唯一模型数量
    fn count_unique_models(&self, spectral_entropies: &[(NodeId, f64)]) -> usize {
        // 基于谱熵聚类，计算不同的模型数量
        if spectral_entropies.is_empty() {
            return 0;
        }

        // 简化：如果谱熵差异大于0.1，认为是不同模型
        let mut unique_models = 1;
        let mut last_entropy = spectral_entropies[0].1;

        for (_, entropy) in spectral_entropies.iter().skip(1) {
            if (entropy - last_entropy).abs() > 0.1 {
                unique_models += 1;
                last_entropy = *entropy;
            }
        }

        unique_models
    }

    /// 获取恶意节点记录
    pub fn get_malicious_records(&self, node_id: &NodeId) -> Option<&Vec<MaliciousNodeRecord>> {
        self.malicious_records.get(node_id)
    }

    /// 获取所有恶意节点
    pub fn get_all_malicious_nodes(&self) -> Vec<(NodeId, Vec<MaliciousBehaviorType>)> {
        self.malicious_records.iter()
            .map(|(node_id, records)| {
                let behavior_types: Vec<MaliciousBehaviorType> = records.iter()
                    .map(|r| r.behavior_type)
                    .collect::<HashSet<_>>() // 去重
                    .into_iter()
                    .collect();
                (node_id.clone(), behavior_types)
            })
            .collect()
    }

    /// 获取节点信誉分数
    pub fn get_reputation_score(&self, node_id: &NodeId) -> Option<f64> {
        self.behavior_history.get(node_id).map(|h| h.reputation_score)
    }

    /// 清除节点记录（用于测试或节点恢复）
    pub fn clear_node_record(&mut self, node_id: &NodeId) {
        self.malicious_records.remove(node_id);
        self.behavior_history.remove(node_id);
    }
}

/// Sybil攻击证据
#[derive(Debug, Clone)]
pub struct SybilAttackEvidence {
    /// IP地址
    pub ip_address: String,
    /// 可疑节点
    pub suspected_nodes: HashSet<NodeId>,
    /// 相似度分数
    pub similarity_score: f64,
    /// 置信度
    pub confidence: f64,
}

/// 共谋证据
#[derive(Debug, Clone)]
pub struct CollusionEvidence {
    /// Agent 1
    pub agent1: NodeId,
    /// Agent 2
    pub agent2: NodeId,
    /// 相似度分数
    pub similarity_score: f64,
    /// 检测次数
    pub detection_count: usize,
    /// 证据类型
    pub evidence_type: CollusionEvidenceType,
}

/// 共谋证据类型
#[derive(Debug, Clone)]
pub enum CollusionEvidenceType {
    /// 高相似度
    HighSimilarity,
    /// 相同IP
    SameIpAddress,
    /// 同时提交
    SimultaneousSubmission,
    /// 响应模式相似
    SimilarResponsePattern,
}

/// 同质性证据
#[derive(Debug, Clone)]
pub struct HomogeneityEvidence {
    /// 节点ID
    pub node_id: NodeId,
    /// 谱熵值
    pub spectral_entropy: f64,
    /// 预期范围
    pub expected_range: (f64, f64),
    /// 证据类型
    pub evidence_type: HomogeneityEvidenceType,
}

/// 同质性证据类型
#[derive(Debug, Clone)]
pub enum HomogeneityEvidenceType {
    /// 谱熵过低
    TooLow,
    /// 谱熵过高
    TooHigh,
    /// 多样性不足
    InsufficientDiversity,
}

/// 获取当前时间戳
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

impl Default for DefenseConfig {
    fn default() -> Self {
        Self {
            sybil_threshold: 0.75,
            collusion_similarity_threshold: 0.9,
            min_model_diversity: 3,
            min_spectral_entropy: 0.6,
            max_spectral_entropy: 0.9,
            timing_anomaly_threshold: 2.5,
            reputation_penalty_factor: 0.5,
            enable_instant_penalty: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sybil_attack_detection() {
        let config = DefenseConfig::default();
        let mut defense = MaliciousDefenseManager::new(config);

        // 注册同一IP的多个节点
        defense.register_node_ip("node1".to_string(), "192.168.1.1".to_string());
        defense.register_node_ip("node2".to_string(), "192.168.1.1".to_string());
        defense.register_node_ip("node3".to_string(), "192.168.1.1".to_string());
        defense.register_node_ip("node4".to_string(), "192.168.1.1".to_string());

        let evidence = defense.detect_sybil_attack();
        assert!(!evidence.is_empty());
        assert_eq!(evidence[0].suspected_nodes.len(), 4);
    }

    #[test]
    fn test_collusion_detection() {
        let config = DefenseConfig::default();
        let mut defense = MaliciousDefenseManager::new(config);

        let nonce = [0u8; 32];
        let commitments = vec![
            Commitment {
                agent_id: "agent1".to_string(),
                commitment_hash: [1u8; 32],
                timestamp: 1000,
                nonce,
            },
            Commitment {
                agent_id: "agent2".to_string(),
                commitment_hash: [1u8; 32], // 相同的哈希
                timestamp: 1001,            // 相近的时间
                nonce,
            },
        ];

        let evidence = defense.detect_collusion_attack(&commitments);
        assert!(!evidence.is_empty());
    }

    #[test]
    fn test_model_homogeneity_detection() {
        let config = DefenseConfig::default();
        let defense = MaliciousDefenseManager::new(config);

        let spectral_entropies = vec![
            ("agent1".to_string(), 0.3), // 过低
            ("agent2".to_string(), 0.4), // 过低
            ("agent3".to_string(), 0.95), // 过高
        ];

        let evidence = defense.detect_model_homogeneity(&spectral_entropies);
        assert_eq!(evidence.len(), 3);
    }

    #[test]
    fn test_timing_anomaly_detection() {
        let config = DefenseConfig::default();
        let mut defense = MaliciousDefenseManager::new(config);

        let agent_id = "agent1".to_string();

        // 添加正常响应时间
        for _ in 0..10 {
            assert!(!defense.detect_timing_anomalies(&agent_id, 1000).unwrap());
        }

        // 添加异常响应时间
        assert!(defense.detect_timing_anomalies(&agent_id, 10000).unwrap());

        // 检查信誉分数
        let score = defense.get_reputation_score(&agent_id).unwrap();
        assert!(score < 1.0); // 应该被惩罚
    }

    #[test]
    fn test_reputation_penalty() {
        let config = DefenseConfig::default();
        let mut defense = MaliciousDefenseManager::new(config);

        let agent_id = "agent1".to_string();
        
        defense.record_malicious_behavior(
            agent_id.clone(),
            MaliciousBehaviorType::HashMismatch,
            1.0,
            vec!["哈希不匹配".to_string()],
        );

        let records = defense.get_malicious_records(&agent_id).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].behavior_type, MaliciousBehaviorType::HashMismatch);

        let score = defense.get_reputation_score(&agent_id).unwrap();
        assert!(score < 1.0);
    }
}
