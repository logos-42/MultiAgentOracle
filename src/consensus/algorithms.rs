//! 共识算法实现
//!
//! 包含各种共识算法的具体实现。

use crate::consensus::{Vote, AggregationResult};
use crate::diap::{DiapIdentityManager, AgentIdentity, DiapError};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;

/// 拜占庭容错算法
pub struct ByzantineFaultTolerance {
    /// 容错节点数
    fault_tolerance: usize,
    /// 总节点数
    total_nodes: usize,
}

impl ByzantineFaultTolerance {
    /// 创建新的BFT算法
    pub fn new(fault_tolerance: usize, total_nodes: usize) -> Result<Self> {
        if fault_tolerance >= total_nodes / 3 {
            return Err(anyhow!("容错节点数不能超过总节点数的1/3"));
        }
        
        Ok(Self {
            fault_tolerance,
            total_nodes,
        })
    }
    
    /// 检查是否达到法定人数
    pub fn check_quorum(&self, received_votes: usize) -> bool {
        received_votes >= 2 * self.fault_tolerance + 1
    }
    
    /// 检查是否达成共识
    pub fn check_consensus(&self, votes: &[Vote]) -> Option<f64> {
        if votes.len() < 2 * self.fault_tolerance + 1 {
            return None;
        }
        
        // 统计每个值的投票数
        let mut value_counts = HashMap::new();
        for vote in votes {
            let key = format!("{:.6}", vote.value); // 使用6位小数精度
            *value_counts.entry(key).or_insert(0) += 1;
        }
        
        // 找到得票最多的值
        let (best_value_str, best_count) = value_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .unwrap_or((String::new(), 0));
        
        // 检查是否达到2f+1
        if best_count >= 2 * self.fault_tolerance + 1 {
            best_value_str.parse::<f64>().ok()
        } else {
            None
        }
    }
    
    /// 计算最小法定人数
    pub fn min_quorum(&self) -> usize {
        2 * self.fault_tolerance + 1
    }
    
    /// 计算最大容错数
    pub fn max_faults(&self) -> usize {
        self.fault_tolerance
    }
}

/// 实用拜占庭容错算法
pub struct PracticalByzantineFaultTolerance {
    /// 视图编号
    view_number: u64,
    /// 序列号
    sequence_number: u64,
    /// 主节点
    primary_node: String,
    /// 备份节点
    backup_nodes: Vec<String>,
}

impl PracticalByzantineFaultTolerance {
    /// 创建新的PBFT算法
    pub fn new(
        view_number: u64,
        sequence_number: u64,
        primary_node: String,
        backup_nodes: Vec<String>,
    ) -> Self {
        Self {
            view_number,
            sequence_number,
            primary_node,
            backup_nodes,
        }
    }
    
    /// 准备阶段
    pub fn prepare_phase(&self, proposal: &Proposal) -> PrepareMessage {
        PrepareMessage {
            view_number: self.view_number,
            sequence_number: self.sequence_number,
            proposal: proposal.clone(),
            node_id: self.primary_node.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// 提交阶段
    pub fn commit_phase(&self, prepare_messages: &[PrepareMessage]) -> Option<CommitMessage> {
        if prepare_messages.len() < 2 * self.backup_nodes.len() / 3 + 1 {
            return None;
        }
        
        // 检查prepare消息的一致性
        let first_proposal = &prepare_messages[0].proposal;
        let all_same = prepare_messages.iter()
            .all(|msg| msg.proposal == *first_proposal);
        
        if !all_same {
            return None;
        }
        
        Some(CommitMessage {
            view_number: self.view_number,
            sequence_number: self.sequence_number,
            proposal: first_proposal.clone(),
            node_id: self.primary_node.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    /// 回复阶段
    pub fn reply_phase(&self, commit_messages: &[CommitMessage]) -> Option<ReplyMessage> {
        if commit_messages.len() < 2 * self.backup_nodes.len() / 3 + 1 {
            return None;
        }
        
        // 检查commit消息的一致性
        let first_proposal = &commit_messages[0].proposal;
        let all_same = commit_messages.iter()
            .all(|msg| msg.proposal == *first_proposal);
        
        if !all_same {
            return None;
        }
        
        Some(ReplyMessage {
            view_number: self.view_number,
            sequence_number: self.sequence_number,
            result: first_proposal.value,
            node_id: self.primary_node.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    /// 视图变更
    pub fn view_change(&mut self, new_view_number: u64, new_primary: String) {
        self.view_number = new_view_number;
        self.primary_node = new_primary;
    }
}

/// 提案
#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    /// 值
    pub value: f64,
    /// 时间戳
    pub timestamp: u64,
    /// 客户端ID
    pub client_id: String,
    /// 操作ID
    pub operation_id: String,
}

/// 准备消息
#[derive(Debug, Clone)]
pub struct PrepareMessage {
    /// 视图编号
    pub view_number: u64,
    /// 序列号
    pub sequence_number: u64,
    /// 提案
    pub proposal: Proposal,
    /// 节点ID
    pub node_id: String,
    /// 时间戳
    pub timestamp: u64,
}

/// 提交消息
#[derive(Debug, Clone)]
pub struct CommitMessage {
    /// 视图编号
    pub view_number: u64,
    /// 序列号
    pub sequence_number: u64,
    /// 提案
    pub proposal: Proposal,
    /// 节点ID
    pub node_id: String,
    /// 时间戳
    pub timestamp: u64,
}

/// 回复消息
#[derive(Debug, Clone)]
pub struct ReplyMessage {
    /// 视图编号
    pub view_number: u64,
    /// 序列号
    pub sequence_number: u64,
    /// 结果
    pub result: f64,
    /// 节点ID
    pub node_id: String,
    /// 时间戳
    pub timestamp: u64,
}

/// 信誉加权拜占庭容错算法
pub struct ReputationWeightedBFT {
    /// 基础BFT算法
    bft: ByzantineFaultTolerance,
    /// 信誉阈值
    reputation_threshold: f64,
    /// 权重计算函数
    weight_calculator: Box<dyn Fn(f64, u64) -> f64 + Send + Sync>,
}

impl ReputationWeightedBFT {
    /// 创建新的信誉加权BFT算法
    pub fn new(
        fault_tolerance: usize,
        total_nodes: usize,
        reputation_threshold: f64,
        weight_calculator: Box<dyn Fn(f64, u64) -> f64 + Send + Sync>,
    ) -> Result<Self> {
        let bft = ByzantineFaultTolerance::new(fault_tolerance, total_nodes)?;
        
        Ok(Self {
            bft,
            reputation_threshold,
            weight_calculator,
        })
    }
    
    /// 检查是否达到加权法定人数
    pub fn check_weighted_quorum(
        &self,
        votes: &[Vote],
        reputations: &HashMap<String, f64>,
        stakes: &HashMap<String, u64>,
    ) -> bool {
        let mut total_weight = 0.0;
        let mut current_weight = 0.0;
        
        // 计算总权重
        for (node_id, &reputation) in reputations {
            if reputation >= self.reputation_threshold {
                let stake = stakes.get(node_id).copied().unwrap_or(0);
                let weight = (self.weight_calculator)(reputation, stake);
                total_weight += weight;
            }
        }
        
        // 计算当前权重
        for vote in votes {
            if let (Some(&reputation), Some(&stake)) = (reputations.get(&vote.agent_did), stakes.get(&vote.agent_did)) {
                if reputation >= self.reputation_threshold {
                    let weight = (self.weight_calculator)(reputation, stake);
                    current_weight += weight;
                }
            }
        }
        
        // 需要达到总权重的2/3
        current_weight >= total_weight * 2.0 / 3.0
    }
    
    /// 加权共识检查
    pub fn check_weighted_consensus(
        &self,
        votes: &[Vote],
        reputations: &HashMap<String, f64>,
        stakes: &HashMap<String, u64>,
    ) -> Option<AggregationResult> {
        if !self.check_weighted_quorum(votes, reputations, stakes) {
            return None;
        }
        
        // 按值分组并计算加权投票数
        let mut weighted_value_counts = HashMap::new();
        let mut total_weight = 0.0;
        
        for vote in votes {
            if let (Some(&reputation), Some(&stake)) = (reputations.get(&vote.agent_did), stakes.get(&vote.agent_did)) {
                if reputation >= self.reputation_threshold {
                    let weight = (self.weight_calculator)(reputation, stake);
                    let key = format!("{:.6}", vote.value);
                    *weighted_value_counts.entry(key).or_insert(0.0) += weight;
                    total_weight += weight;
                }
            }
        }
        
        if total_weight == 0.0 {
            return None;
        }
        
        // 找到加权得票最多的值
        let (best_value_str, best_weight) = weighted_value_counts
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap_or((String::new(), 0.0));
        
        // 检查是否达到加权法定人数
        if best_weight >= total_weight * 2.0 / 3.0 {
            if let Ok(value) = best_value_str.parse::<f64>() {
                // 计算平均置信度
                let avg_confidence = votes.iter()
                    .map(|v| v.confidence)
                    .sum::<f64>() / votes.len() as f64;
                
                return Some(AggregationResult {
                    value,
                    confidence: avg_confidence,
                    method: crate::consensus::AggregationMethod::WeightedAverage,
                    votes_used: votes.len(),
                    total_votes: votes.len(),
                    weight_stats: crate::consensus::aggregation::WeightStatistics {
                        total_weight,
                        average_weight: total_weight / votes.len() as f64,
                        weight_std_dev: 0.0, // 简化计算
                        min_weight: 0.0,
                        max_weight: 0.0,
                    },
                });
            }
        }
        
        None
    }
}

/// 简单多数算法
pub struct SimpleMajority {
    /// 法定人数比例
    quorum_ratio: f64,
}

impl SimpleMajority {
    /// 创建新的简单多数算法
    pub fn new(quorum_ratio: f64) -> Result<Self> {
        if quorum_ratio <= 0.5 || quorum_ratio > 1.0 {
            return Err(anyhow!("法定人数比例必须在0.5到1.0之间"));
        }
        
        Ok(Self { quorum_ratio })
    }
    
    /// 检查是否达成共识
    pub fn check_consensus(&self, votes: &[Vote]) -> Option<f64> {
        if votes.len() == 0 {
            return None;
        }
        
        // 统计每个值的投票数
        let mut value_counts = HashMap::new();
        for vote in votes {
            let key = format!("{:.6}", vote.value);
            *value_counts.entry(key).or_insert(0) += 1;
        }
        
        // 找到得票最多的值
        let (best_value_str, best_count) = value_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .unwrap_or((String::new(), 0));
        
        // 检查是否达到法定人数
        if best_count as f64 >= votes.len() as f64 * self.quorum_ratio {
            best_value_str.parse::<f64>().ok()
        } else {
            None
        }
    }
    
    /// 计算所需法定人数
    pub fn required_quorum(&self, total_votes: usize) -> usize {
        (total_votes as f64 * self.quorum_ratio).ceil() as usize
    }
}

/// 时间窗口共识算法
pub struct TimeWindowConsensus {
    /// 时间窗口大小（秒）
    window_size_secs: u64,
    /// 最小投票数
    min_votes: usize,
    /// 最大时间偏差（秒）
    max_time_deviation_secs: u64,
}

impl TimeWindowConsensus {
    /// 创建新的时间窗口共识算法
    pub fn new(window_size_secs: u64, min_votes: usize, max_time_deviation_secs: u64) -> Self {
        Self {
            window_size_secs,
            min_votes,
            max_time_deviation_secs,
        }
    }
    
    /// 在时间窗口内检查共识
    pub fn check_consensus_in_window(&self, votes: &[Vote]) -> Option<AggregationResult> {
        if votes.len() < self.min_votes {
            return None;
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 过滤在时间窗口内的投票
        let window_votes: Vec<&Vote> = votes.iter()
            .filter(|vote| {
                let time_diff = (vote.timestamp as i64 - now as i64).abs() as u64;
                time_diff <= self.window_size_secs
            })
            .collect();
        
        if window_votes.len() < self.min_votes {
            return None;
        }
        
        // 检查时间一致性
        let avg_timestamp = window_votes.iter()
            .map(|v| v.timestamp)
            .sum::<u64>() as f64 / window_votes.len() as f64;
        
        let time_variance = window_votes.iter()
            .map(|v| (v.timestamp as f64 - avg_timestamp).powi(2))
            .sum::<f64>() / window_votes.len() as f64;
        
        let time_std_dev = time_variance.sqrt();
        
        if time_std_dev > self.max_time_deviation_secs as f64 {
            return None;
        }
        
        // 使用简单多数算法
        let simple_majority = SimpleMajority::new(0.51).unwrap();
        let values: Vec<f64> = window_votes.iter().map(|v| v.value).collect();
        
        // 计算加权平均值（使用置信度作为权重）
        let total_weight: f64 = window_votes.iter()
            .map(|v| v.confidence)
            .sum();
        
        if total_weight == 0.0 {
            return None;
        }
        
        let weighted_sum: f64 = window_votes.iter()
            .map(|v| v.value * v.confidence)
            .sum();
        
        let value = weighted_sum / total_weight;
        
        // 计算平均置信度
        let avg_confidence = window_votes.iter()
            .map(|v| v.confidence)
            .sum::<f64>() / window_votes.len() as f64;
        
        Some(AggregationResult {
            value,
            confidence: avg_confidence,
            method: crate::consensus::AggregationMethod::WeightedAverage,
            votes_used: window_votes.len(),
            total_votes: votes.len(),
            weight_stats: crate::consensus::aggregation::WeightStatistics {
                total_weight,
                average_weight: total_weight / window_votes.len() as f64,
                weight_std_dev: 0.0,
                min_weight: 0.0,
                max_weight: 0.0,
            },
        })
    }
    
    /// 获取有效时间窗口
    pub fn get_valid_window(&self) -> (u64, u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let start = now.saturating_sub(self.window_size_secs);
        let end = now + self.window_size_secs;
        
        (start, end)
    }
}

/// DIAP增强的拜占庭容错算法
pub struct DiapEnhancedBFT {
    /// 基础BFT算法
    base_bft: ByzantineFaultTolerance,
    /// DIAP身份管理器
    diap_identity_manager: Option<Arc<DiapIdentityManager>>,
    /// 是否要求DIAP身份验证
    require_diap_auth: bool,
    /// DIAP身份权重增强因子
    diap_weight_boost: f64,
}

impl DiapEnhancedBFT {
    /// 创建新的DIAP增强BFT算法
    pub fn new(
        fault_tolerance: usize,
        total_nodes: usize,
        diap_identity_manager: Option<Arc<DiapIdentityManager>>,
        require_diap_auth: bool,
    ) -> Result<Self> {
        let base_bft = ByzantineFaultTolerance::new(fault_tolerance, total_nodes)?;
        
        Ok(Self {
            base_bft,
            diap_identity_manager,
            require_diap_auth,
            diap_weight_boost: 1.2, // DIAP身份投票权重增加20%
        })
    }
    
    /// 检查是否达到法定人数（考虑DIAP身份）
    pub async fn check_quorum_with_diap(&self, votes: &[Vote]) -> Result<bool> {
        let mut valid_votes = 0;
        let mut diap_authenticated_votes = 0;
        
        for vote in votes {
            // 基础验证
            if !vote.validate() {
                continue;
            }
            
            // DIAP身份验证
            if let Some(manager) = &self.diap_identity_manager {
                if let Some(identity_id) = &vote.diap_identity_id {
                    match manager.verify_identity(identity_id, vote.diap_proof_hash.as_deref()).await {
                        Ok(auth_result) if auth_result.authenticated => {
                            diap_authenticated_votes += 1;
                            valid_votes += 1;
                            continue;
                        }
                        Ok(_) => {
                            // DIAP身份验证失败
                            if self.require_diap_auth {
                                continue; // 如果要求DIAP身份，跳过此投票
                            }
                        }
                        Err(e) => {
                            log::warn!("DIAP身份验证错误: {}, 跳过投票", e);
                            if self.require_diap_auth {
                                continue;
                            }
                        }
                    }
                }
            }
            
            // 如果没有DIAP身份或不需要DIAP身份验证
            if !self.require_diap_auth {
                valid_votes += 1;
            }
        }
        
        log::debug!("有效投票: {}, DIAP认证投票: {}", valid_votes, diap_authenticated_votes);
        
        // 计算考虑DIAP身份的法定人数
        let effective_votes = if self.require_diap_auth {
            diap_authenticated_votes
        } else {
            // DIAP认证投票有更高权重
            let weighted_votes = diap_authenticated_votes as f64 * self.diap_weight_boost;
            (valid_votes as f64 + weighted_votes - diap_authenticated_votes as f64) as usize
        };
        
        Ok(self.base_bft.check_quorum(effective_votes))
    }
    
    /// 检查是否达成共识（考虑DIAP身份）
    pub async fn check_consensus_with_diap(&self, votes: &[Vote]) -> Result<Option<f64>> {
        // 过滤有效投票
        let mut valid_votes = Vec::new();
        let mut vote_weights = Vec::new();
        
        for vote in votes {
            // 基础验证
            if !vote.validate() {
                continue;
            }
            
            let mut weight = 1.0;
            
            // DIAP身份验证和权重增强
            if let Some(manager) = &self.diap_identity_manager {
                if let Some(identity_id) = &vote.diap_identity_id {
                    match manager.verify_identity(identity_id, vote.diap_proof_hash.as_deref()).await {
                        Ok(auth_result) if auth_result.authenticated => {
                            // DIAP身份验证通过，增加权重
                            weight *= self.diap_weight_boost;
                            valid_votes.push(vote.clone());
                            vote_weights.push(weight);
                            continue;
                        }
                        Ok(_) => {
                            // DIAP身份验证失败
                            if self.require_diap_auth {
                                continue;
                            }
                        }
                        Err(e) => {
                            log::warn!("DIAP身份验证错误: {}, 跳过投票", e);
                            if self.require_diap_auth {
                                continue;
                            }
                        }
                    }
                }
            }
            
            // 如果没有DIAP身份或不需要DIAP身份验证
            if !self.require_diap_auth {
                valid_votes.push(vote.clone());
                vote_weights.push(weight);
            }
        }
        
        if valid_votes.is_empty() {
            return Ok(None);
        }
        
        // 使用加权投票计算共识
        let mut weighted_value_counts = HashMap::new();
        let mut total_weight = 0.0;
        
        for (i, vote) in valid_votes.iter().enumerate() {
            let key = format!("{:.6}", vote.value);
            let weight = vote_weights[i];
            *weighted_value_counts.entry(key).or_insert(0.0) += weight;
            total_weight += weight;
        }
        
        // 找到加权得票最多的值
        let (best_value_str, best_weight) = weighted_value_counts
            .into_iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        
        let best_value: f64 = best_value_str.parse().unwrap();
        
        // 检查是否达到加权法定人数
        let required_weight = total_weight * 2.0 / 3.0; // 需要2/3的加权投票
        
        if best_weight >= required_weight {
            Ok(Some(best_value))
        } else {
            Ok(None)
        }
    }
    
    /// 获取DIAP身份统计信息
    pub async fn get_diap_statistics(&self, votes: &[Vote]) -> DiapConsensusStats {
        let mut stats = DiapConsensusStats::default();
        
        for vote in votes {
            stats.total_votes += 1;
            
            if vote.diap_identity_id.is_some() {
                stats.diap_votes += 1;
                
                if let Some(manager) = &self.diap_identity_manager {
                    if let Some(identity_id) = &vote.diap_identity_id {
                        match manager.verify_identity(identity_id, vote.diap_proof_hash.as_deref()).await {
                            Ok(auth_result) if auth_result.authenticated => {
                                stats.authenticated_diap_votes += 1;
                            }
                            _ => {
                                stats.failed_diap_auth_votes += 1;
                            }
                        }
                    }
                }
            } else {
                stats.non_diap_votes += 1;
            }
        }
        
        stats
    }
}

/// DIAP共识统计信息
#[derive(Debug, Clone, Default)]
pub struct DiapConsensusStats {
    /// 总投票数
    pub total_votes: usize,
    /// DIAP投票数
    pub diap_votes: usize,
    /// 非DIAP投票数
    pub non_diap_votes: usize,
    /// 已认证的DIAP投票数
    pub authenticated_diap_votes: usize,
    /// DIAP认证失败的投票数
    pub failed_diap_auth_votes: usize,
}

impl DiapConsensusStats {
    /// 计算DIAP投票比例
    pub fn diap_vote_ratio(&self) -> f64 {
        if self.total_votes == 0 {
            return 0.0;
        }
        self.diap_votes as f64 / self.total_votes as f64
    }
    
    /// 计算DIAP认证成功率
    pub fn diap_auth_success_rate(&self) -> f64 {
        if self.diap_votes == 0 {
            return 0.0;
        }
        self.authenticated_diap_votes as f64 / self.diap_votes as f64
    }
    
    /// 获取统计摘要
    pub fn summary(&self) -> String {
        format!(
            "总投票: {}, DIAP投票: {} ({:.1}%), 认证成功: {} ({:.1}%)",
            self.total_votes,
            self.diap_votes,
            self.diap_vote_ratio() * 100.0,
            self.authenticated_diap_votes,
            self.diap_auth_success_rate() * 100.0
        )
    }
}
