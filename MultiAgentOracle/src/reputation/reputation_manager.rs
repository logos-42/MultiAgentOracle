use crate::reputation::algorithms;
use crate::reputation::reputation_score::{
    ReputationScore, ReputationUpdate, UpdateReason, ReputationTier, ReputationSummary,
};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};
use serde::{Deserialize, Serialize};

/// 信誉管理器
#[derive(Clone)]
pub struct ReputationManager {
    /// 信誉记录
    scores: Arc<RwLock<HashMap<String, ReputationScore>>>,
    /// 配置
    config: ReputationConfig,
    /// 算法模块
    algorithms: algorithms::ReputationAlgorithms,
}

/// 信誉配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    /// 初始信誉分
    pub initial_score: f64,
    /// 最小信誉分
    pub min_score: f64,
    /// 最大信誉分
    pub max_score: f64,
    /// 数据准确性权重
    pub accuracy_weight: f64,
    /// 响应时间权重
    pub response_time_weight: f64,
    /// 可用性权重
    pub availability_weight: f64,
    /// 信誉衰减率（每天）
    pub decay_rate_per_day: f64,
    /// 最小活跃服务次数
    pub min_active_services: u64,
    /// 惩罚系数
    pub penalty_multiplier: f64,
    /// 奖励系数
    pub reward_multiplier: f64,
    /// 自动清理间隔（秒）
    pub auto_cleanup_interval_secs: u64,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            initial_score: 100.0,
            min_score: 0.0,
            max_score: 1000.0,
            accuracy_weight: 0.6,
            response_time_weight: 0.2,
            availability_weight: 0.2,
            decay_rate_per_day: 0.01, // 每天衰减1%
            min_active_services: 10,
            penalty_multiplier: 2.0,
            reward_multiplier: 1.0,
            auto_cleanup_interval_secs: 3600, // 1小时
        }
    }
}

impl ReputationManager {
    /// 创建新的信誉管理器
    pub fn new(config: ReputationConfig) -> Self {
        Self {
            scores: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            algorithms: algorithms::ReputationAlgorithms::new(config),
        }
    }
    
    /// 注册新智能体
    pub async fn register_agent(&self, agent_did: String, staked_amount: u64) -> Result<()> {
        let mut scores = self.scores.write().await;
        
        if scores.contains_key(&agent_did) {
            return Err(anyhow!("智能体已注册: {}", agent_did));
        }
        
        let score = ReputationScore::new(
            agent_did.clone(),
            self.config.initial_score,
            staked_amount,
        );
        
        scores.insert(agent_did.clone(), score);
        
        info!("✅ 注册新智能体: {}, 初始信誉分: {}", 
            agent_did, self.config.initial_score);
        
        Ok(())
    }
    
    /// 获取信誉分
    pub async fn get_score(&self, agent_did: &str) -> Option<ReputationScore> {
        let scores = self.scores.read().await;
        scores.get(agent_did).cloned()
    }
    
    /// 获取信誉摘要
    pub async fn get_summary(&self, agent_did: &str) -> Option<ReputationSummary> {
        let scores = self.scores.read().await;
        scores.get(agent_did).map(|score| score.get_summary())
    }
    
    /// 更新信誉分（基于数据准确性）
    pub async fn update_for_data_accuracy(
        &self,
        agent_did: &str,
        expected: f64,
        actual: f64,
        tolerance: f64,
        data_id: Option<String>,
    ) -> Result<f64> {
        let delta = self.algorithms.calculate_accuracy_delta(
            expected, actual, tolerance, self.config.accuracy_weight
        );
        
        let update = ReputationUpdate::new(
            UpdateReason::DataAccuracy {
                expected,
                actual,
                tolerance,
            },
            delta,
            1,
            if delta >= 0.0 { 1 } else { 0 },
            data_id,
            None,
        );
        
        self.apply_update(agent_did, update).await
    }
    
    /// 更新信誉分（基于响应时间）
    pub async fn update_for_response_time(
        &self,
        agent_did: &str,
        expected_ms: u64,
        actual_ms: u64,
    ) -> Result<f64> {
        let delta = self.algorithms.calculate_response_time_delta(
            expected_ms, actual_ms, self.config.response_time_weight
        );
        
        let update = ReputationUpdate::new(
            UpdateReason::ResponseTime {
                expected_ms,
                actual_ms,
            },
            delta,
            1,
            1, // 响应时间不影响成功率统计
            None,
            None,
        );
        
        self.apply_update(agent_did, update).await
    }
    
    /// 更新信誉分（基于服务可用性）
    pub async fn update_for_availability(
        &self,
        agent_did: &str,
        expected_uptime: f64,
        actual_uptime: f64,
    ) -> Result<f64> {
        let delta = self.algorithms.calculate_availability_delta(
            expected_uptime, actual_uptime, self.config.availability_weight
        );
        
        let update = ReputationUpdate::new(
            UpdateReason::ServiceAvailability {
                expected_uptime,
                actual_uptime,
            },
            delta,
            1,
            1, // 可用性不影响成功率统计
            None,
            None,
        );
        
        self.apply_update(agent_did, update).await
    }
    
    /// 应用惩罚
    pub async fn apply_penalty(
        &self,
        agent_did: &str,
        reason: String,
        severity: crate::reputation::reputation_score::PenaltySeverity,
        note: Option<String>,
    ) -> Result<f64> {
        let penalty_amount = match severity {
            crate::reputation::reputation_score::PenaltySeverity::Minor => -10.0,
            crate::reputation::reputation_score::PenaltySeverity::Moderate => -50.0,
            crate::reputation::reputation_score::PenaltySeverity::Severe => -100.0,
            crate::reputation::reputation_score::PenaltySeverity::Malicious => -200.0,
        };
        
        let delta = penalty_amount * self.config.penalty_multiplier;
        
        let update = ReputationUpdate::new(
            UpdateReason::Penalty { reason, severity },
            delta,
            0,
            0,
            None,
            note,
        );
        
        self.apply_update(agent_did, update).await
    }
    
    /// 应用奖励
    pub async fn apply_reward(
        &self,
        agent_did: &str,
        reason: String,
        amount: f64,
        note: Option<String>,
    ) -> Result<f64> {
        let delta = amount * self.config.reward_multiplier;
        
        let update = ReputationUpdate::new(
            UpdateReason::Reward { reason, amount },
            delta,
            0,
            0,
            None,
            note,
        );
        
        self.apply_update(agent_did, update).await
    }
    
    /// 更新质押金额
    pub async fn update_stake(
        &self,
        agent_did: &str,
        new_amount: u64,
    ) -> Result<()> {
        let mut scores = self.scores.write().await;
        
        if let Some(score) = scores.get_mut(agent_did) {
            let old_amount = score.staked_amount;
            
            if new_amount > old_amount {
                score.stake(new_amount - old_amount);
            } else {
                score.unstake(old_amount - new_amount)
                    .map_err(|e| anyhow!("减少质押失败: {}", e))?;
            }
            
            // 记录质押变化
            let update = ReputationUpdate::new(
                UpdateReason::StakeChange {
                    old_amount,
                    new_amount,
                },
                0.0, // 质押变化不影响信誉分
                0,
                0,
                None,
                None,
            );
            
            score.update(update);
            
            info!("💰 更新质押: {} -> {} ({}), 智能体: {}", 
                old_amount, new_amount, 
                if new_amount > old_amount { "增加" } else { "减少" },
                agent_did);
            
            Ok(())
        } else {
            Err(anyhow!("智能体未注册: {}", agent_did))
        }
    }
    
    /// 应用信誉更新
    async fn apply_update(&self, agent_did: &str, update: ReputationUpdate) -> Result<f64> {
        let mut scores = self.scores.write().await;
        
        if let Some(score) = scores.get_mut(agent_did) {
            let old_score = score.score;
            score.update(update.clone());
            let new_score = score.score;
            
            info!("📊 信誉更新: {} -> {} (Δ: {:.2}), 原因: {:?}", 
                old_score, new_score, update.delta, update.reason);
            
            Ok(update.delta)
        } else {
            Err(anyhow!("智能体未注册: {}", agent_did))
        }
    }
    
    /// 获取所有信誉分
    pub async fn get_all_scores(&self) -> Vec<ReputationScore> {
        let scores = self.scores.read().await;
        scores.values().cloned().collect()
    }
    
    /// 获取活跃智能体列表
    pub async fn get_active_agents(&self) -> Vec<ReputationSummary> {
        let scores = self.scores.read().await;
        scores.values()
            .filter(|score| score.is_active && score.total_services >= self.config.min_active_services)
            .map(|score| score.get_summary())
            .collect()
    }
    
    /// 获取信誉排名
    pub async fn get_rankings(&self, limit: usize) -> Vec<ReputationSummary> {
        let mut summaries: Vec<ReputationSummary> = self.get_active_agents().await;
        summaries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        summaries.truncate(limit);
        summaries
    }
    
    /// 应用信誉衰减
    pub async fn apply_decay(&self) -> Result<usize> {
        let mut scores = self.scores.write().await;
        let mut updated_count = 0;
        
        for score in scores.values_mut() {
            if score.is_active {
                let days_since_update = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - score.last_updated) as f64 / 86400.0;
                
                if days_since_update >= 1.0 {
                    let decay_amount = score.score * self.config.decay_rate_per_day * days_since_update;
                    if decay_amount > 0.1 { // 只有衰减超过0.1分才记录
                        let update = ReputationUpdate::new(
                            UpdateReason::ManualAdjustment {
                                admin: "system".to_string(),
                                reason: "信誉衰减".to_string(),
                            },
                            -decay_amount,
                            0,
                            0,
                            None,
                            Some(format!("{}天未活跃", days_since_update as u64)),
                        );
                        
                        score.update(update);
                        updated_count += 1;
                    }
                }
            }
        }
        
        if updated_count > 0 {
            info!("🧹 应用信誉衰减: {}个智能体受影响", updated_count);
        }
        
        Ok(updated_count)
    }
    
    /// 清理不活跃智能体
    pub async fn cleanup_inactive(&self, max_inactive_days: u64) -> Result<usize> {
        let mut scores = self.scores.write().await;
        let mut removed_count = 0;
        let max_inactive_secs = max_inactive_days * 86400;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        scores.retain(|agent_did, score| {
            let inactive_for = now - score.last_updated;
            let should_remove = !score.is_active && inactive_for > max_inactive_secs;
            
            if should_remove {
                info!("🗑️ 清理不活跃智能体: {} ({}天未活跃)", 
                    agent_did, inactive_for / 86400);
                removed_count += 1;
            }
            
            !should_remove
        });
        
        Ok(removed_count)
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> ReputationStats {
        let scores = self.scores.read().await;
        
        let mut stats = ReputationStats {
            total_agents: scores.len(),
            active_agents: 0,
            average_score: 0.0,
            total_staked: 0,
            tier_distribution: HashMap::new(),
            total_services: 0,
            successful_services: 0,
        };
        
        for score in scores.values() {
            if score.is_active {
                stats.active_agents += 1;
            }
            
            stats.average_score += score.score;
            stats.total_staked += score.staked_amount;
            stats.total_services += score.total_services;
            stats.successful_services += score.successful_services;
            
            *stats.tier_distribution.entry(score.tier.name().to_string())
                .or_insert(0) += 1;
        }
        
        if !scores.is_empty() {
            stats.average_score /= scores.len() as f64;
        }
        
        stats
    }
}

/// 信誉统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationStats {
    /// 总智能体数
    pub total_agents: usize,
    /// 活跃智能体数
    pub active_agents: usize,
    /// 平均信誉分
    pub average_score: f64,
    /// 总质押金额
    pub total_staked: u64,
    /// 等级分布
    pub tier_distribution: HashMap<String, usize>,
    /// 总服务次数
    pub total_services: u64,
    /// 成功服务次数
    pub successful_services: u64,
}

impl ReputationStats {
    /// 计算总体成功率
    pub fn overall_success_rate(&self) -> f64 {
        if self.total_services == 0 {
            return 0.0;
        }
        self.successful_services as f64 / self.total_services as f64
    }
    
    /// 计算活跃率
    pub fn active_rate(&self) -> f64 {
        if self.total_agents == 0 {
            return 0.0;
        }
        self.active_agents as f64 / self.total_agents as f64
    }
}
