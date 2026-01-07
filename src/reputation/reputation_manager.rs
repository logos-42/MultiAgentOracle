//! 信誉管理器 - 因果指纹版
//!
//! 基于逻辑一致性的信誉评分系统，用于评估和激励预言机智能体。

use crate::reputation::reputation_score::{
    ReputationScore, ReputationUpdate, UpdateReason, ReputationSummary, ReputationTier,
};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;
use serde::{Deserialize, Serialize};

/// 信誉管理器 - 因果指纹版
#[derive(Clone)]
pub struct ReputationManager {
    /// 信誉记录
    scores: Arc<RwLock<HashMap<String, ReputationScore>>>,
    /// 配置
    config: ReputationConfig,
}

/// 信誉配置 - 因果指纹版
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    /// 初始因果信用分
    pub initial_credit: f64,
    /// 最小信用分
    pub min_credit: f64,
    /// 最大信用分
    pub max_credit: f64,
    /// 逻辑一致性权重
    pub logical_consistency_weight: f64,
    /// 谱一致性权重
    pub spectral_consistency_weight: f64,
    /// 信誉衰减率（每天）
    pub decay_rate_per_day: f64,
    /// 最小活跃任务数
    pub min_active_tasks: u64,
    /// 惩罚系数
    pub penalty_multiplier: f64,
    /// 奖励系数
    pub reward_multiplier: f64,
    /// 自动清理间隔（秒）
    pub auto_cleanup_interval_secs: u64,
    /// 余弦相似度阈值
    pub cosine_threshold: f64,
    /// 谱同质性阈值
    pub homogeneity_threshold: f64,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            initial_credit: 500.0,
            min_credit: 0.0,
            max_credit: 1000.0,
            logical_consistency_weight: 0.6,
            spectral_consistency_weight: 0.4,
            decay_rate_per_day: 0.005, // 每天衰减0.5%
            min_active_tasks: 5,
            penalty_multiplier: 2.0,
            reward_multiplier: 1.0,
            auto_cleanup_interval_secs: 3600,
            cosine_threshold: 0.85,
            homogeneity_threshold: 0.95,
        }
    }
}

impl ReputationManager {
    /// 创建新的信誉管理器
    pub fn new(config: ReputationConfig) -> Self {
        Self {
            scores: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// 注册新智能体
    pub async fn register_agent(&self, agent_did: String) -> Result<()> {
        let mut scores = self.scores.write().await;
        
        if scores.contains_key(&agent_did) {
            return Err(anyhow!("智能体已注册: {}", agent_did));
        }
        
        let score = ReputationScore::new(agent_did.clone());
        scores.insert(agent_did.clone(), score);
        
        info!("✅ 注册新智能体: {}, 初始因果信用分: 500", agent_did);
        
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
    
    /// 更新信誉分（基于逻辑一致性）
    pub async fn update_for_logical_consistency(
        &self,
        agent_did: &str,
        cosine_similarity: f64,
        is_outlier: bool,
        cluster_position: usize,
    ) -> Result<f64> {
        // 根据余弦相似度计算奖励/惩罚
        let delta = if is_outlier {
            // 离群点：惩罚
            let penalty = (self.config.cosine_threshold - cosine_similarity).max(0.0);
            -50.0 * penalty * self.config.penalty_multiplier
        } else {
            // 一致：奖励
            let reward = (cosine_similarity - self.config.cosine_threshold).max(0.0);
            50.0 * reward * self.config.reward_multiplier
        };
        
        let update = ReputationUpdate::new(
            UpdateReason::LogicalConsistency {
                cosine_similarity,
                cluster_position,
            },
            delta,
            1,
            if !is_outlier { 1 } else { 0 },
            None,
            None,
        );
        
        self.apply_update(agent_did, update).await
    }
    
    /// 更新信誉分（基于谱一致性）
    pub async fn update_for_spectral_consistency(
        &self,
        agent_did: &str,
        consistency_score: f64,
    ) -> Result<f64> {
        let delta = if consistency_score > 0.9 {
            20.0 * self.config.reward_multiplier
        } else if consistency_score > 0.8 {
            10.0 * self.config.reward_multiplier
        } else if consistency_score < 0.5 {
            -20.0 * self.config.penalty_multiplier
        } else {
            0.0
        };
        
        let update = ReputationUpdate::new(
            UpdateReason::SpectralConsistency { consistency_score },
            delta,
            1,
            if delta > 0.0 { 1 } else { 0 },
            None,
            None,
        );
        
        self.apply_update(agent_did, update).await
    }
    
    /// 处理逻辑同质性检测（供应商一致攻击）
    pub async fn handle_logic_homogeneity(
        &self,
        agent_did: &str,
        cluster_size: usize,
        penalty_applied: bool,
    ) -> Result<f64> {
        let delta = if penalty_applied {
            // 大聚类中检测到同质性：惩罚
            -100.0 * self.config.penalty_multiplier
        } else {
            0.0
        };
        
        let update = ReputationUpdate::new(
            UpdateReason::LogicHomogeneity { cluster_size, penalty_applied },
            delta,
            1,
            0,
            None,
            Some(format!("聚类大小: {}, 惩罚: {}", cluster_size, penalty_applied)),
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
    
    /// 更新全局指纹
    pub async fn update_global_fingerprint(
        &self,
        agent_did: &str,
        new_features: &[f64; 16],
    ) -> Result<()> {
        let mut scores = self.scores.write().await;
        
        if let Some(score) = scores.get_mut(agent_did) {
            score.update_global_fingerprint(new_features, 0.1);
            info!("🔢 更新全局指纹: {}", agent_did);
            Ok(())
        } else {
            Err(anyhow!("智能体未注册: {}", agent_did))
        }
    }
    
    /// 应用信誉更新
    async fn apply_update(&self, agent_did: &str, update: ReputationUpdate) -> Result<f64> {
        let mut scores = self.scores.write().await;
        
        if let Some(score) = scores.get_mut(agent_did) {
            let old_credit = score.causal_credit;
            score.update_for_logical_consistency(update.clone());
            let new_credit = score.causal_credit;
            
            info!("📊 信誉更新: {} -> {} (Δ: {:.2}), 原因: {:?}", 
                old_credit, new_credit, update.delta, update.reason);
            
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
            .filter(|score| score.is_active && score.total_tasks >= self.config.min_active_tasks)
            .map(|score| score.get_summary())
            .collect()
    }
    
    /// 获取信誉排名
    pub async fn get_rankings(&self, limit: usize) -> Vec<ReputationSummary> {
        let mut summaries: Vec<ReputationSummary> = self.get_active_agents().await;
        summaries.sort_by(|a, b| b.causal_credit.partial_cmp(&a.causal_credit).unwrap());
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
                    let decay_amount = score.causal_credit * self.config.decay_rate_per_day * days_since_update;
                    if decay_amount > 0.1 {
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
                        
                        score.update_for_logical_consistency(update);
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
            average_credit: 0.0,
            tier_distribution: HashMap::new(),
            total_tasks: 0,
            successful_tasks: 0,
            avg_stability: 0.0,
            avg_outlier_rate: 0.0,
        };
        
        let mut total_stability = 0.0f64;
        let mut total_outlier_rate = 0.0f64;
        
        for score in scores.values() {
            if score.is_active {
                stats.active_agents += 1;
            }
            
            stats.average_credit += score.causal_credit;
            stats.total_tasks += score.total_tasks;
            stats.successful_tasks += score.successful_tasks;
            total_stability += score.fingerprint_stability;
            total_outlier_rate += score.outlier_count as f64 / score.total_tasks.max(1) as f64;
            
            *stats.tier_distribution.entry(score.tier.name().to_string())
                .or_insert(0) += 1;
        }
        
        if !scores.is_empty() {
            stats.average_credit /= scores.len() as f64;
            stats.avg_stability = total_stability / scores.len() as f64;
            stats.avg_outlier_rate = total_outlier_rate / scores.len() as f64;
        }
        
        stats
    }
}

/// 信誉统计 - 因果指纹版
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationStats {
    /// 总智能体数
    pub total_agents: usize,
    /// 活跃智能体数
    pub active_agents: usize,
    /// 平均因果信用分
    pub average_credit: f64,
    /// 等级分布
    pub tier_distribution: HashMap<String, usize>,
    /// 总任务数
    pub total_tasks: u64,
    /// 成功任务数
    pub successful_tasks: u64,
    /// 平均指纹稳定性
    pub avg_stability: f64,
    /// 平均离群率
    pub avg_outlier_rate: f64,
}

impl ReputationStats {
    /// 计算总体成功率
    pub fn overall_success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            return 0.0;
        }
        self.successful_tasks as f64 / self.total_tasks as f64
    }
    
    /// 计算活跃率
    pub fn active_rate(&self) -> f64 {
        if self.total_agents == 0 {
            return 0.0;
        }
        self.active_agents as f64 / self.total_agents as f64
    }
}
