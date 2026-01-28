//! Commitment-Reveal 协议
//! 
//! 防止智能体之间的信息泄露，确保独立思考过程
//! 提供安全的承诺-揭示机制，防御恶意节点攻击

use crate::types::NodeId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// 协议阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolPhase {
    /// 等待承诺阶段
    Commitment,
    /// 等待揭示阶段
    Reveal,
    /// 验证完成
    Completed,
    /// 协议失败
    Failed,
}

/// 承诺结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    /// 智能体ID
    pub agent_id: NodeId,
    /// 承诺哈希 (SHA256)
    pub commitment_hash: [u8; 32],
    /// 时间戳（毫秒）
    pub timestamp: u64,
    /// 随机数（用于防止彩虹表攻击）
    pub nonce: [u8; 32],
}

/// 揭示结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reveal {
    /// 智能体ID
    pub agent_id: NodeId,
    /// 实际响应数据
    pub response_data: Vec<u8>,
    /// 随机数（必须与承诺阶段一致）
    pub nonce: [u8; 32],
    /// 时间戳（毫秒）
    pub timestamp: u64,
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// 是否验证通过
    pub is_valid: bool,
    /// 验证失败原因
    pub failure_reason: Option<String>,
    /// 验证时间
    pub verified_at: u64,
}

/// 协议错误
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("承诺已存在: {0}")]
    CommitmentAlreadyExists(NodeId),
    
    #[error("承诺不存在: {0}")]
    CommitmentNotFound(NodeId),
    
    #[error("承诺已过期: {0}")]
    CommitmentExpired(NodeId),
    
    #[error("揭示数据与承诺不匹配: {0}")]
    RevealMismatch(NodeId),
    
    #[error("协议阶段错误: 期望 {expected:?}, 实际 {actual:?}")]
    InvalidPhase {
        expected: ProtocolPhase,
        actual: ProtocolPhase,
    },
    
    #[error("超时: {0}")]
    Timeout(String),
    
    #[error("恶意行为检测: {0}")]
    MaliciousBehaviorDetected(String),
    
    #[error("数据序列化失败: {0}")]
    SerializationError(String),
}

/// Commitment-Reveal 协议管理器
pub struct CommitmentRevealProtocol {
    /// 当前协议阶段
    phase: ProtocolPhase,
    /// 承诺映射 (agent_id -> Commitment)
    commitments: HashMap<NodeId, Commitment>,
    /// 揭示映射 (agent_id -> Reveal)
    reveals: HashMap<NodeId, Reveal>,
    /// 验证结果映射
    verification_results: HashMap<NodeId, VerificationResult>,
    /// 承诺截止时间（毫秒）
    commitment_deadline: u64,
    /// 揭示截止时间（毫秒）
    reveal_deadline: u64,
    /// 协议开始时间
    protocol_start_time: u64,
    /// 参与Agent列表
    participating_agents: Vec<NodeId>,
    /// 恶意行为检测阈值
    malicious_threshold: f64,
}

/// 独立思考保护器
pub struct IndependentThinkingGuard {
    /// 已处理的Agent集合
    processed_agents: HashMap<NodeId, SystemTime>,
    /// 思考时间窗口（秒）
    thinking_window: u64,
    /// 最小思考时间（秒）
    min_thinking_time: u64,
    /// 异常行为检测
    anomaly_detector: AnomalyDetector,
}

/// 异常检测器
pub struct AnomalyDetector {
    /// Agent响应时间记录
    response_times: HashMap<NodeId, Vec<u64>>,
    /// 平均响应时间基准
    baseline_response_time: u64,
    /// 异常阈值（标准差倍数）
    anomaly_threshold: f64,
}

impl CommitmentRevealProtocol {
    /// 创建新的协议实例
    pub fn new(
        participating_agents: Vec<NodeId>,
        commitment_timeout_ms: u64,
        _reveal_timeout_ms: u64,
    ) -> Self {
        let now = current_timestamp_ms();
        Self {
            phase: ProtocolPhase::Commitment,
            commitments: HashMap::new(),
            reveals: HashMap::new(),
            verification_results: HashMap::new(),
            commitment_deadline: now + commitment_timeout_ms,
            reveal_deadline: 0, // 将在承诺阶段完成后设置
            protocol_start_time: now,
            participating_agents,
            malicious_threshold: 0.8, // 80%的Agent必须参与
        }
    }

    /// 提交承诺
    pub fn submit_commitment(&mut self, commitment: Commitment) -> Result<(), ProtocolError> {
        // 检查协议阶段
        if self.phase != ProtocolPhase::Commitment {
            return Err(ProtocolError::InvalidPhase {
                expected: ProtocolPhase::Commitment,
                actual: self.phase,
            });
        }

        // 检查是否已过期
        if current_timestamp_ms() > self.commitment_deadline {
            return Err(ProtocolError::CommitmentExpired(commitment.agent_id.clone()));
        }

        // 检查Agent是否在参与列表中
        if !self.participating_agents.contains(&commitment.agent_id) {
            return Err(ProtocolError::MaliciousBehaviorDetected(
                format!("Agent {} 不在参与列表中", commitment.agent_id)
            ));
        }

        // 检查是否已提交承诺
        if self.commitments.contains_key(&commitment.agent_id) {
            return Err(ProtocolError::CommitmentAlreadyExists(commitment.agent_id.clone()));
        }

        // 存储承诺
        self.commitments.insert(commitment.agent_id.clone(), commitment);

        // 检查是否所有Agent都已提交承诺
        if self.commitments.len() == self.participating_agents.len() {
            self.advance_to_reveal_phase()?;
        }

        Ok(())
    }

    /// 提交揭示
    pub fn submit_reveal(&mut self, reveal: Reveal) -> Result<(), ProtocolError> {
        // 检查协议阶段
        if self.phase != ProtocolPhase::Reveal {
            return Err(ProtocolError::InvalidPhase {
                expected: ProtocolPhase::Reveal,
                actual: self.phase,
            });
        }

        // 检查是否已过期
        if current_timestamp_ms() > self.reveal_deadline {
            return Err(ProtocolError::Timeout("揭示阶段已超时".to_string()));
        }

        // 获取对应的承诺
        let commitment = self.commitments
            .get(&reveal.agent_id)
            .ok_or_else(|| ProtocolError::CommitmentNotFound(reveal.agent_id.clone()))?;

        // 验证揭示数据与承诺匹配
        let computed_hash = compute_commitment_hash(&reveal.response_data, &reveal.nonce);
        if computed_hash != commitment.commitment_hash {
            return Err(ProtocolError::RevealMismatch(reveal.agent_id.clone()));
        }

        // 检查随机数是否匹配
        if reveal.nonce != commitment.nonce {
            return Err(ProtocolError::RevealMismatch(reveal.agent_id.clone()));
        }

        // 存储揭示
        self.reveals.insert(reveal.agent_id.clone(), reveal);

        // 检查是否所有Agent都已提交揭示
        if self.reveals.len() == self.commitments.len() {
            self.complete_protocol()?;
        }

        Ok(())
    }

    /// 进入揭示阶段
    fn advance_to_reveal_phase(&mut self) -> Result<(), ProtocolError> {
        self.phase = ProtocolPhase::Reveal;
        self.reveal_deadline = current_timestamp_ms() + 30000; // 30秒揭示窗口
        
        // 检查承诺完整性
        if self.commitments.len() < self.participating_agents.len() {
            let missing_agents: Vec<_> = self.participating_agents.iter()
                .filter(|agent| !self.commitments.contains_key(*agent))
                .collect();
            
            return Err(ProtocolError::Timeout(
                format!("以下Agent未提交承诺: {:?}", missing_agents)
            ));
        }

        Ok(())
    }

    /// 完成协议
    fn complete_protocol(&mut self) -> Result<(), ProtocolError> {
        self.phase = ProtocolPhase::Completed;
        
        // 验证所有揭示
        for (agent_id, reveal) in &self.reveals {
            let result = self.verify_reveal(reveal)?;
            self.verification_results.insert(agent_id.clone(), result);
        }

        Ok(())
    }

    /// 验证单个揭示
    fn verify_reveal(&self, reveal: &Reveal) -> Result<VerificationResult, ProtocolError> {
        let commitment = self.commitments
            .get(&reveal.agent_id)
            .ok_or_else(|| ProtocolError::CommitmentNotFound(reveal.agent_id.clone()))?;

        // 验证哈希
        let computed_hash = compute_commitment_hash(&reveal.response_data, &reveal.nonce);
        if computed_hash != commitment.commitment_hash {
            return Ok(VerificationResult {
                is_valid: false,
                failure_reason: Some("揭示数据与承诺不匹配".to_string()),
                verified_at: current_timestamp_ms(),
            });
        }

        // 验证时间戳（防止重放攻击）
        if reveal.timestamp < commitment.timestamp {
            return Ok(VerificationResult {
                is_valid: false,
                failure_reason: Some("揭示时间戳早于承诺时间戳".to_string()),
                verified_at: current_timestamp_ms(),
            });
        }

        Ok(VerificationResult {
            is_valid: true,
            failure_reason: None,
            verified_at: current_timestamp_ms(),
        })
    }

    /// 检测恶意行为
    pub fn detect_malicious_behavior(&self) -> Result<Vec<NodeId>, ProtocolError> {
        let mut malicious_agents = Vec::new();

        // 检查未提交承诺的Agent
        for agent_id in &self.participating_agents {
            if !self.commitments.contains_key(agent_id) {
                malicious_agents.push(agent_id.clone());
            }
        }

        // 检查未提交揭示的Agent
        for agent_id in self.commitments.keys() {
            if !self.reveals.contains_key(agent_id) {
                malicious_agents.push(agent_id.clone());
            }
        }

        // 检查验证失败的Agent
        for (agent_id, result) in &self.verification_results {
            if !result.is_valid {
                malicious_agents.push(agent_id.clone());
            }
        }

        // 检查参与率
        let participation_rate = self.commitments.len() as f64 / self.participating_agents.len() as f64;
        if participation_rate < self.malicious_threshold {
            return Err(ProtocolError::MaliciousBehaviorDetected(
                format!("参与率过低: {:.2}%", participation_rate * 100.0)
            ));
        }

        Ok(malicious_agents)
    }

    /// 获取已验证的响应
    pub fn get_verified_responses(&self) -> Result<HashMap<NodeId, Vec<u8>>, ProtocolError> {
        if self.phase != ProtocolPhase::Completed {
            return Err(ProtocolError::InvalidPhase {
                expected: ProtocolPhase::Completed,
                actual: self.phase,
            });
        }

        let mut verified_responses = HashMap::new();
        
        for (agent_id, reveal) in &self.reveals {
            if let Some(result) = self.verification_results.get(agent_id) {
                if result.is_valid {
                    verified_responses.insert(agent_id.clone(), reveal.response_data.clone());
                }
            }
        }

        Ok(verified_responses)
    }

    /// 获取协议状态
    pub fn get_status(&self) -> ProtocolStatus {
        ProtocolStatus {
            phase: self.phase,
            commitments_count: self.commitments.len(),
            reveals_count: self.reveals.len(),
            participating_agents: self.participating_agents.len(),
            commitment_deadline: self.commitment_deadline,
            reveal_deadline: self.reveal_deadline,
            malicious_agents: self.detect_malicious_behavior().unwrap_or_default(),
        }
    }

    /// 强制超时检查
    pub fn check_timeouts(&mut self) -> Result<(), ProtocolError> {
        let now = current_timestamp_ms();

        match self.phase {
            ProtocolPhase::Commitment => {
                if now > self.commitment_deadline {
                    self.phase = ProtocolPhase::Failed;
                    return Err(ProtocolError::Timeout("承诺阶段超时".to_string()));
                }
            }
            ProtocolPhase::Reveal => {
                if now > self.reveal_deadline {
                    self.phase = ProtocolPhase::Failed;
                    return Err(ProtocolError::Timeout("揭示阶段超时".to_string()));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl IndependentThinkingGuard {
    /// 创建新的独立思考保护器
    pub fn new(thinking_window_secs: u64, min_thinking_time_secs: u64) -> Self {
        Self {
            processed_agents: HashMap::new(),
            thinking_window: thinking_window_secs,
            min_thinking_time: min_thinking_time_secs,
            anomaly_detector: AnomalyDetector::new(),
        }
    }

    /// 记录Agent开始思考
    pub fn record_thinking_start(&mut self, agent_id: NodeId) -> Result<(), ProtocolError> {
        let now = SystemTime::now();
        
        // 检查是否已经在思考窗口内
        if let Some(last_time) = self.processed_agents.get(&agent_id) {
            if let Ok(duration) = now.duration_since(*last_time) {
                if duration.as_secs() < self.thinking_window {
                    return Err(ProtocolError::MaliciousBehaviorDetected(
                        format!("Agent {} 在思考窗口内重复提交", agent_id)
                    ));
                }
            }
        }

        self.processed_agents.insert(agent_id, now);
        Ok(())
    }

    /// 验证思考时间是否足够
    pub fn verify_thinking_time(&self, agent_id: &NodeId, actual_thinking_time_ms: u64) -> Result<(), ProtocolError> {
        if actual_thinking_time_ms < self.min_thinking_time * 1000 {
            return Err(ProtocolError::MaliciousBehaviorDetected(
                format!("Agent {} 的思考时间过短: {}ms < {}ms", 
                    agent_id, 
                    actual_thinking_time_ms, 
                    self.min_thinking_time * 1000
                )
            ));
        }

        Ok(())
    }

    /// 检测异常行为
    pub fn detect_anomalies(&mut self, agent_id: &NodeId, response_time_ms: u64) -> Result<(), ProtocolError> {
        self.anomaly_detector.add_response_time(agent_id.clone(), response_time_ms)?;
        
        if self.anomaly_detector.is_anomalous(agent_id)? {
            return Err(ProtocolError::MaliciousBehaviorDetected(
                format!("Agent {} 的响应时间异常", agent_id)
            ));
        }

        Ok(())
    }
}

impl AnomalyDetector {
    /// 创建新的异常检测器
    pub fn new() -> Self {
        Self {
            response_times: HashMap::new(),
            baseline_response_time: 1000, // 1000ms基准
            anomaly_threshold: 2.0, // 2倍标准差
        }
    }

    /// 添加响应时间记录
    pub fn add_response_time(&mut self, agent_id: NodeId, response_time_ms: u64) -> Result<(), ProtocolError> {
        let times = self.response_times.entry(agent_id).or_insert_with(Vec::new);
        times.push(response_time_ms);

        // 限制历史记录数量
        if times.len() > 100 {
            times.remove(0);
        }

        Ok(())
    }

    /// 检测是否异常
    pub fn is_anomalous(&self, agent_id: &NodeId) -> Result<bool, ProtocolError> {
        if let Some(times) = self.response_times.get(agent_id) {
            if times.len() < 5 {
                return Ok(false); // 数据不足，暂不判定
            }

            let mean = times.iter().sum::<u64>() as f64 / times.len() as f64;
            let variance = times.iter()
                .map(|t| (*t as f64 - mean).powi(2))
                .sum::<f64>() / times.len() as f64;
            let std_dev = variance.sqrt();

            let last_time = times.last().unwrap();
            let z_score = (*last_time as f64 - mean).abs() / std_dev;

            Ok(z_score > self.anomaly_threshold)
        } else {
            Ok(false)
        }
    }
}

/// 协议状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolStatus {
    /// 当前阶段
    pub phase: ProtocolPhase,
    /// 已收到的承诺数量
    pub commitments_count: usize,
    /// 已收到的揭示数量
    pub reveals_count: usize,
    /// 总参与Agent数
    pub participating_agents: usize,
    /// 承诺截止时间
    pub commitment_deadline: u64,
    /// 揭示截止时间
    pub reveal_deadline: u64,
    /// 检测到的恶意Agent
    pub malicious_agents: Vec<NodeId>,
}

/// 计算承诺哈希
pub fn compute_commitment_hash(data: &[u8], nonce: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.update(nonce);
    let result = hasher.finalize();
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// 生成随机数
pub fn generate_nonce() -> [u8; 32] {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut nonce = [0u8; 32];
    rng.fill_bytes(&mut nonce);
    nonce
}

/// 获取当前时间戳（毫秒）
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// 序列化数据
pub fn serialize_data<T: Serialize>(data: &T) -> Result<Vec<u8>, ProtocolError> {
    bincode::serialize(data)
        .map_err(|e| ProtocolError::SerializationError(e.to_string()))
}

/// 反序列化数据
pub fn deserialize_data<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T, ProtocolError> {
    bincode::deserialize(data)
        .map_err(|e| ProtocolError::SerializationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_reveal_protocol() {
        let agents = vec![
            "agent1".to_string(),
            "agent2".to_string(),
            "agent3".to_string(),
        ];

        let mut protocol = CommitmentRevealProtocol::new(agents.clone(), 10000, 10000);

        // 阶段1: 提交承诺
        for agent_id in &agents {
            let response_data = serialize_data(&vec![1.0, 2.0, 3.0]).unwrap();
            let nonce = generate_nonce();
            let commitment_hash = compute_commitment_hash(&response_data, &nonce);
            
            let commitment = Commitment {
                agent_id: agent_id.clone(),
                commitment_hash,
                timestamp: current_timestamp_ms(),
                nonce,
            };

            assert!(protocol.submit_commitment(commitment).is_ok());
        }

        assert_eq!(protocol.phase, ProtocolPhase::Reveal);

        // 阶段2: 提交揭示
        for agent_id in &agents {
            let response_data = serialize_data(&vec![1.0, 2.0, 3.0]).unwrap();
            let commitment = protocol.commitments.get(agent_id).unwrap();
            
            let reveal = Reveal {
                agent_id: agent_id.clone(),
                response_data,
                nonce: commitment.nonce,
                timestamp: current_timestamp_ms(),
            };

            assert!(protocol.submit_reveal(reveal).is_ok());
        }

        assert_eq!(protocol.phase, ProtocolPhase::Completed);

        // 验证结果
        let verified_responses = protocol.get_verified_responses().unwrap();
        assert_eq!(verified_responses.len(), 3);

        // 检测恶意行为
        let malicious_agents = protocol.detect_malicious_behavior().unwrap();
        assert_eq!(malicious_agents.len(), 0);
    }

    #[test]
    fn test_malicious_behavior_detection() {
        let agents = vec!["agent1".to_string(), "agent2".to_string()];
        let mut protocol = CommitmentRevealProtocol::new(agents.clone(), 1000, 1000);

        // 只提交一个承诺，另一个不提交
        let response_data = serialize_data(&vec![1.0, 2.0, 3.0]).unwrap();
        let nonce = generate_nonce();
        let commitment_hash = compute_commitment_hash(&response_data, &nonce);
        
        let commitment = Commitment {
            agent_id: "agent1".to_string(),
            commitment_hash,
            timestamp: current_timestamp_ms(),
            nonce,
        };

        assert!(protocol.submit_commitment(commitment).is_ok());

        // 不提交agent2的承诺，等待超时
        std::thread::sleep(std::time::Duration::from_millis(1100));
        assert!(protocol.check_timeouts().is_err());
    }

    #[test]
    fn test_reveal_mismatch() {
        let agents = vec!["agent1".to_string()];
        let mut protocol = CommitmentRevealProtocol::new(agents.clone(), 10000, 10000);

        // 提交承诺
        let response_data = serialize_data(&vec![1.0, 2.0, 3.0]).unwrap();
        let nonce = generate_nonce();
        let commitment_hash = compute_commitment_hash(&response_data, &nonce);
        
        let commitment = Commitment {
            agent_id: "agent1".to_string(),
            commitment_hash,
            timestamp: current_timestamp_ms(),
            nonce,
        };

        assert!(protocol.submit_commitment(commitment).is_ok());

        // 提交不匹配的揭示（修改响应数据）
        let wrong_response_data = serialize_data(&vec![1.0, 2.0, 999.0]).unwrap();
        let reveal = Reveal {
            agent_id: "agent1".to_string(),
            response_data: wrong_response_data,
            nonce,
            timestamp: current_timestamp_ms(),
        };

        assert!(protocol.submit_reveal(reveal).is_err());
    }

    #[test]
    fn test_independent_thinking_guard() {
        let mut guard = IndependentThinkingGuard::new(60, 1); // 60秒窗口，1秒最小思考时间

        let agent_id = "agent1".to_string();

        // 记录开始思考
        assert!(guard.record_thinking_start(agent_id.clone()).is_ok());

        // 验证思考时间（2秒 > 1秒最小值）
        assert!(guard.verify_thinking_time(&agent_id, 2000).is_ok());

        // 验证思考时间（0.5秒 < 1秒最小值）
        assert!(guard.verify_thinking_time(&agent_id, 500).is_err());
    }

    #[test]
    fn test_anomaly_detection() {
        let mut detector = AnomalyDetector::new();
        let agent_id = "agent1".to_string();

        // 添加正常响应时间
        for _ in 0..10 {
            detector.add_response_time(agent_id.clone(), 1000).unwrap();
        }

        // 正常响应时间不应该是异常
        assert!(!detector.is_anomalous(&agent_id).unwrap());

        // 添加异常响应时间（远大于正常值）
        detector.add_response_time(agent_id.clone(), 10000).unwrap();
        assert!(detector.is_anomalous(&agent_id).unwrap());
    }
}
