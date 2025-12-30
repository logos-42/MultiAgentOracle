//! 共识算法实现
//!
//! 包含各种共识算法的具体实现。

use crate::consensus::{Vote, AggregationResult};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

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
