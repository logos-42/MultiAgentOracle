use crate::consensus::{
    ConsensusResult, ConsensusStatus, Vote, AggregationAlgorithm,
    aggregation::AggregationConfig,
};
use crate::reputation::ReputationManager;
use crate::oracle_agent::OracleDataType;
use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};
use serde::{Deserialize, Serialize};

/// 共识引擎
pub struct ConsensusEngine {
    /// 信誉管理器
    reputation_manager: Arc<ReputationManager>,
    /// 配置
    config: ConsensusConfig,
    /// 当前共识状态
    state: Arc<RwLock<ConsensusState>>,
    /// 聚合算法
    aggregation_algorithm: AggregationAlgorithm,
}

/// 共识配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// 最小法定人数比例 (0.0-1.0)
    pub min_quorum_ratio: f64,
    /// 最低信誉阈值
    pub min_reputation_threshold: f64,
    /// 最大投票权重差异倍数
    pub max_weight_variance: f64,
    /// 超时时间 (秒)
    pub timeout_secs: u64,
    /// 重试次数
    pub max_retries: u32,
    /// 是否启用自动争议解决
    pub auto_dispute_resolution: bool,
    /// 争议解决阈值
    pub dispute_resolution_threshold: f64,
    /// 共识确认所需轮数
    pub confirmation_rounds: u32,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_quorum_ratio: 0.67, // 2/3多数
            min_reputation_threshold: 100.0,
            max_weight_variance: 3.0,
            timeout_secs: 30,
            max_retries: 3,
            auto_dispute_resolution: true,
            dispute_resolution_threshold: 0.8,
            confirmation_rounds: 2,
        }
    }
}

/// 共识状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    /// 当前共识ID
    pub consensus_id: String,
    /// 数据类型
    pub data_type: OracleDataType,
    /// 开始时间
    pub start_time: u64,
    /// 结束时间
    pub end_time: Option<u64>,
    /// 当前状态
    pub status: ConsensusStatus,
    /// 收到的投票
    pub votes: HashMap<String, Vote>,
    /// 参与智能体
    pub participants: HashSet<String>,
    /// 当前轮数
    pub current_round: u32,
    /// 争议标记
    pub disputes: Vec<Dispute>,
    /// 最终结果
    pub final_result: Option<ConsensusResult>,
}

/// 争议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    /// 争议ID
    pub dispute_id: String,
    /// 提出争议的智能体
    pub proposer: String,
    /// 争议目标值
    pub target_value: f64,
    /// 争议原因
    pub reason: String,
    /// 支持争议的投票
    pub supporting_votes: Vec<String>,
    /// 解决状态
    pub resolved: bool,
    /// 解决结果
    pub resolution: Option<DisputeResolution>,
}

/// 争议解决
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolution {
    /// 解决方式
    pub method: ResolutionMethod,
    /// 最终值
    pub final_value: f64,
    /// 解决时间
    pub resolved_at: u64,
    /// 解决者
    pub resolver: String,
}

/// 解决方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionMethod {
    /// 重新投票
    Revote,
    /// 仲裁
    Arbitration,
    /// 使用备用数据源
    FallbackSource,
    /// 手动解决
    Manual,
}

impl ConsensusEngine {
    /// 创建新的共识引擎
    pub fn new(
        reputation_manager: Arc<ReputationManager>,
        config: ConsensusConfig,
    ) -> Self {
        Self {
            reputation_manager,
            config: config.clone(),
            state: Arc::new(RwLock::new(ConsensusState {
                consensus_id: "".to_string(),
                data_type: OracleDataType::CryptoPrice { symbol: "BTC".to_string() },
                start_time: 0,
                end_time: None,
                status: ConsensusStatus::Idle,
                votes: HashMap::new(),
                participants: HashSet::new(),
                current_round: 0,
                disputes: Vec::new(),
                final_result: None,
            })),
            aggregation_algorithm: AggregationAlgorithm::new(AggregationConfig::default()),
        }
    }
    
    /// 开始新的共识
    pub async fn start_consensus(
        &self,
        consensus_id: String,
        data_type: OracleDataType,
        participants: Vec<String>,
    ) -> Result<()> {
        let mut state = self.state.write().await;
        
        if state.status != ConsensusStatus::Idle {
            return Err(anyhow!("共识引擎忙，当前状态: {:?}", state.status));
        }
        
        // 验证参与者
        let valid_participants = self.validate_participants(&participants).await?;
        if valid_participants.is_empty() {
            return Err(anyhow!("没有有效的参与者"));
        }
        
        // 更新状态
        state.consensus_id = consensus_id;
        state.data_type = data_type;
        state.start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        state.status = ConsensusStatus::Collecting;
        state.votes.clear();
        state.participants = valid_participants.into_iter().collect();
        state.current_round = 1;
        state.disputes.clear();
        state.final_result = None;
        
        info!("🚀 开始共识: {}, 参与者: {} 个", 
            state.consensus_id, state.participants.len());
        
        Ok(())
    }
    
    /// 提交投票
    pub async fn submit_vote(&self, vote: Vote) -> Result<()> {
        let mut state = self.state.write().await;
        
        if state.status != ConsensusStatus::Collecting {
            return Err(anyhow!("当前不接受投票，状态: {:?}", state.status));
        }
        
        // 验证投票者
        if !state.participants.contains(&vote.agent_did) {
            return Err(anyhow!("投票者不是共识参与者: {}", vote.agent_did));
        }
        
        // 验证投票有效性
        if !vote.validate() {
            return Err(anyhow!("无效投票"));
        }
        
        // 检查是否已投票
        if state.votes.contains_key(&vote.agent_did) {
            return Err(anyhow!("该智能体已投票"));
        }
        
        // 记录投票
        state.votes.insert(vote.agent_did.clone(), vote.clone());
        
        info!("🗳️ 收到投票: {}, 值: {:.4}, 置信度: {:.2}", 
            vote.agent_did, vote.value, vote.confidence);
        
        // 检查是否达到法定人数
        if self.check_quorum(&state).await {
            info!("✅ 达到法定人数，开始聚合");
            state.status = ConsensusStatus::Aggregating;
            
            // 触发聚合（暂时注释，避免生命周期问题）
            // let engine_clone = self.clone();
            // tokio::spawn(engine_clone.aggregate_votes());
        }
        
        Ok(())
    }
    
    /// 验证参与者
    async fn validate_participants(&self, participants: &[String]) -> Result<Vec<String>> {
        let mut valid_participants = Vec::new();
        
        for agent_did in participants {
            if let Some(score) = self.reputation_manager.get_score(agent_did).await {
                if score.is_active && score.causal_credit >= self.config.min_reputation_threshold {
                    valid_participants.push(agent_did.clone());
                } else {
                    warn!("参与者无效: {} (活跃: {}, 因果信用分: {:.2})", 
                        agent_did, score.is_active, score.causal_credit);
                }
            } else {
                warn!("参与者未注册: {}", agent_did);
            }
        }
        
        Ok(valid_participants)
    }
    
    /// 检查是否达到法定人数
    async fn check_quorum(&self, state: &ConsensusState) -> bool {
        let total_weight = self.calculate_total_weight(state).await;
        let current_weight = self.calculate_current_weight(state).await;
        
        let quorum_ratio = current_weight / total_weight;
        quorum_ratio >= self.config.min_quorum_ratio
    }
    
    /// 计算总权重
    async fn calculate_total_weight(&self, state: &ConsensusState) -> f64 {
        let mut total_weight = 0.0;
        
        for agent_did in &state.participants {
            if let Some(score) = self.reputation_manager.get_score(agent_did).await {
                total_weight += score.voting_weight();
            }
        }
        
        total_weight
    }
    
    /// 计算当前权重
    async fn calculate_current_weight(&self, state: &ConsensusState) -> f64 {
        let mut current_weight = 0.0;
        
        for (agent_did, _) in &state.votes {
            if let Some(score) = self.reputation_manager.get_score(agent_did).await {
                current_weight += score.voting_weight();
            }
        }
        
        current_weight
    }
    
    /// 聚合投票
    async fn aggregate_votes(&self) -> Result<()> {
        let state = self.state.read().await.clone();
        
        if state.votes.is_empty() {
            error!("没有投票可聚合");
            return Ok(());
        }
        
        info!("🔍 开始聚合投票: {} 个投票", state.votes.len());
        
        // 收集投票数据
        let votes: Vec<Vote> = state.votes.values().cloned().collect();
        
        // 应用聚合算法
        let aggregation_result = self.aggregation_algorithm.aggregate(&votes).await;
        
        // 检查争议
        let disputes = match &aggregation_result {
            Ok(result) => self.check_disputes(&votes, result).await,
            Err(_) => Vec::new(),
        };
        
        // 更新状态
        let mut state_write = self.state.write().await;
        
        if disputes.is_empty() || !self.config.auto_dispute_resolution {
            // 没有争议或禁用自动解决，直接完成
            state_write.status = ConsensusStatus::Completed;
            state_write.final_result = match aggregation_result {
                Ok(result) => Some(ConsensusResult {
                    consensus_id: state_write.consensus_id.clone(),
                    data_type: state_write.data_type.clone(),
                    final_value: result.value,
                    confidence: result.confidence,
                    participants: state_write.participants.iter().cloned().collect(),
                    votes_used: votes.len(),
                    total_weight: self.calculate_current_weight(&state_write).await,
                    aggregation_method: result.method,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                }),
                Err(e) => {
                    warn!("聚合失败: {}", e);
                    None
                }
            };
            
            if let Some(result) = state_write.final_result.clone() {
                state_write.end_time = Some(result.timestamp);
                info!("✅ 共识完成: {}, 最终值: {:.4}, 置信度: {:.2}", 
                    state_write.consensus_id, result.final_value, result.confidence);
            }
        } else {
            // 有争议，进入争议解决
            state_write.status = ConsensusStatus::DisputeResolution;
            state_write.disputes = disputes;
            
            info!("⚖️ 进入争议解决: {} 个争议", state_write.disputes.len());
            
            // 触发争议解决（暂时注释，避免生命周期问题）
            // tokio::spawn(self.resolve_disputes());
        }
        
        Ok(())
    }
    
    /// 检查争议
    async fn check_disputes(
        &self,
        votes: &[Vote],
        aggregation_result: &crate::consensus::aggregation::AggregationResult,
    ) -> Vec<Dispute> {
        let mut disputes = Vec::new();
        
        for vote in votes {
            let deviation = (vote.value - aggregation_result.value).abs() / aggregation_result.value;
            
            if deviation > self.config.max_weight_variance {
                let dispute = Dispute {
                    dispute_id: format!("{}_{}", vote.agent_did, vote.timestamp),
                    proposer: vote.agent_did.clone(),
                    target_value: vote.value,
                    reason: format!("偏差过大: {:.2}%", deviation * 100.0),
                    supporting_votes: Vec::new(),
                    resolved: false,
                    resolution: None,
                };
                
                disputes.push(dispute);
            }
        }
        
        disputes
    }
    
    /// 解决争议
    async fn resolve_disputes(&self) -> Result<()> {
        let state = self.state.read().await.clone();
        
        info!("🔄 开始解决争议: {} 个", state.disputes.len());
        
        for dispute in &state.disputes {
            match self.resolve_dispute(dispute).await {
                Ok(resolution) => {
                    info!("✅ 解决争议: {}, 方式: {:?}", 
                        dispute.dispute_id, resolution.method);
                    
                    // 更新状态
                    let mut state_write = self.state.write().await;
                    if let Some(d) = state_write.disputes.iter_mut()
                        .find(|d| d.dispute_id == dispute.dispute_id) 
                    {
                        d.resolved = true;
                        d.resolution = Some(resolution);
                    }
                }
                Err(e) => {
                    error!("解决争议失败 {}: {}", dispute.dispute_id, e);
                }
            }
        }
        
        // 检查是否所有争议都已解决
        let state = self.state.read().await;
        let all_resolved = state.disputes.iter().all(|d| d.resolved);
        
        if all_resolved {
            let mut state_write = self.state.write().await;
            state_write.status = ConsensusStatus::Completed;
            
            info!("✅ 所有争议已解决，共识完成");
        } else {
            warn!("⚠️ 部分争议未解决");
        }
        
        Ok(())
    }
    
    /// 解决单个争议
    async fn resolve_dispute(&self, dispute: &Dispute) -> Result<DisputeResolution> {
        // 简化实现：使用重新投票
        // 实际实现应该更复杂，可能包括仲裁、备用数据源等
        
        let resolution = DisputeResolution {
            method: ResolutionMethod::Revote,
            final_value: 0.0, // 实际应该从重新投票中获取
            resolved_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            resolver: "system".to_string(),
        };
        
        Ok(resolution)
    }
    
    /// 获取共识结果
    pub async fn get_result(&self) -> Option<ConsensusResult> {
        let state = self.state.read().await;
        state.final_result.clone()
    }
    
    /// 获取共识状态
    pub async fn get_state(&self) -> ConsensusState {
        self.state.read().await.clone()
    }
    
    /// 重置共识引擎
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = ConsensusState {
            consensus_id: "".to_string(),
            data_type: OracleDataType::CryptoPrice { symbol: "BTC".to_string() },
            start_time: 0,
            end_time: None,
            status: ConsensusStatus::Idle,
            votes: HashMap::new(),
            participants: HashSet::new(),
            current_round: 0,
            disputes: Vec::new(),
            final_result: None,
        };
        
        info!("🔄 重置共识引擎");
    }
}

impl Clone for ConsensusEngine {
    fn clone(&self) -> Self {
        Self {
            reputation_manager: self.reputation_manager.clone(),
            config: self.config.clone(),
            state: self.state.clone(),
            aggregation_algorithm: self.aggregation_algorithm.clone(),
        }
    }
}
