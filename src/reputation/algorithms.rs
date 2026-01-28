//! 信誉算法 - 因果指纹版
//!
//! 基于逻辑一致性的信誉评分算法

use crate::reputation::ReputationConfig;

/// 信誉算法 - 因果指纹版
#[derive(Clone)]
pub struct ReputationAlgorithms {
    config: ReputationConfig,
}

impl ReputationAlgorithms {
    /// 创建新的信誉算法
    pub fn new(config: ReputationConfig) -> Self {
        Self { config }
    }
    
    /// 计算逻辑一致性带来的信誉变化
    pub fn calculate_logical_consistency_delta(
        &self,
        cosine_similarity: f64,
        is_outlier: bool,
    ) -> f64 {
        let threshold = self.config.cosine_threshold;
        
        if is_outlier {
            // 离群点：惩罚
            let penalty = (threshold - cosine_similarity).max(0.0);
            -50.0 * penalty * self.config.penalty_multiplier
        } else {
            // 一致：奖励
            let reward = (cosine_similarity - threshold).max(0.0);
            50.0 * reward * self.config.reward_multiplier
        }
    }
    
    /// 计算谱一致性带来的信誉变化
    pub fn calculate_spectral_consistency_delta(
        &self,
        consistency_score: f64,
    ) -> f64 {
        if consistency_score > 0.9 {
            20.0 * self.config.reward_multiplier
        } else if consistency_score > 0.8 {
            10.0 * self.config.reward_multiplier
        } else if consistency_score < 0.5 {
            -20.0 * self.config.penalty_multiplier
        } else {
            0.0
        }
    }
    
    /// 计算综合信誉分（因果指纹版）
    pub fn calculate_comprehensive_score(
        &self,
        logical_score: f64,
        spectral_score: f64,
    ) -> f64 {
        let total_weight = self.config.logical_consistency_weight 
            + self.config.spectral_consistency_weight;
        
        if total_weight == 0.0 {
            return 0.0;
        }
        
        let weighted_sum = logical_score * self.config.logical_consistency_weight
            + spectral_score * self.config.spectral_consistency_weight;
        
        weighted_sum / total_weight
    }
    
    /// 计算信誉衰减
    pub fn calculate_decay(
        &self,
        current_credit: f64,
        days_inactive: f64,
    ) -> f64 {
        if days_inactive <= 0.0 {
            return 0.0;
        }
        
        // 指数衰减：分数越高，衰减越快
        let decay_rate = self.config.decay_rate_per_day * (current_credit / 1000.0);
        let decay_amount = current_credit * (1.0 - (1.0 - decay_rate).powf(days_inactive));
        
        -decay_amount
    }
    
    /// 计算投票权重（基于因果信用分）
    pub fn calculate_voting_weight(
        &self,
        causal_credit: f64,
    ) -> f64 {
        // 权重与因果信用分成正比
        causal_credit / 100.0
    }
    
    /// 计算信誉等级阈值
    pub fn calculate_tier_thresholds(&self) -> Vec<(String, f64, f64)> {
        vec![
            ("Newbie".to_string(), 0.0, 199.9),
            ("Copper".to_string(), 200.0, 399.9),
            ("Iron".to_string(), 400.0, 499.9),
            ("Bronze".to_string(), 500.0, 599.9),
            ("Silver".to_string(), 600.0, 699.9),
            ("Gold".to_string(), 700.0, 799.9),
            ("Diamond".to_string(), 800.0, 899.9),
            ("Platinum".to_string(), 900.0, 1000.0),
        ]
    }
    
    /// 计算惩罚金额
    pub fn calculate_penalty_amount(
        &self,
        severity: crate::reputation::reputation_score::PenaltySeverity,
        current_credit: f64,
    ) -> f64 {
        let base_penalty = match severity {
            crate::reputation::reputation_score::PenaltySeverity::Minor => 10.0,
            crate::reputation::reputation_score::PenaltySeverity::Moderate => 50.0,
            crate::reputation::reputation_score::PenaltySeverity::Severe => 100.0,
            crate::reputation::reputation_score::PenaltySeverity::Malicious => 200.0,
        };
        
        // 分数越高，惩罚越重
        let multiplier = 1.0 + (current_credit / 1000.0);
        
        -base_penalty * multiplier * self.config.penalty_multiplier
    }
    
    /// 计算奖励金额
    pub fn calculate_reward_amount(
        &self,
        contribution: f64,
        current_credit: f64,
    ) -> f64 {
        // 基础奖励
        let base_reward = contribution * 10.0;
        
        // 分数越低，奖励越高（鼓励新节点）
        let multiplier = 2.0 - (current_credit / 1000.0);
        
        base_reward * multiplier * self.config.reward_multiplier
    }
    
    /// 计算信誉恢复率
    pub fn calculate_recovery_rate(
        &self,
        current_credit: f64,
        historical_performance: f64,
    ) -> f64 {
        // 当前分数越低，恢复越快
        let score_factor = 1.0 - (current_credit / 1000.0);
        
        // 历史表现越好，恢复越快
        let performance_factor = historical_performance;
        
        // 基础恢复率
        let base_recovery = 0.1;
        
        base_recovery * score_factor * performance_factor
    }
    
    /// 计算指纹稳定性分数
    pub fn calculate_stability_score(
        &self,
        fingerprint_history: &[f64],
    ) -> f64 {
        if fingerprint_history.len() < 2 {
            return 1.0;
        }
        
        let mean = fingerprint_history.iter().sum::<f64>() / fingerprint_history.len() as f64;
        let variance = fingerprint_history.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / fingerprint_history.len() as f64;
        
        let std_dev = variance.sqrt();
        
        // 标准差越小，稳定性越高
        1.0 / (1.0 + std_dev)
    }
    
    /// 计算谱距离（用于同质性检测）
    pub fn calculate_spectral_distance(
        &self,
        features_a: &[f64; 16],
        features_b: &[f64; 16],
    ) -> f64 {
        features_a.iter()
            .zip(features_b.iter())
            .map(|(&a, &b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
    
    /// 判断是否为同源模型
    pub fn is_homogeneous(&self, distance: f64) -> bool {
        distance < self.config.homogeneity_threshold
    }
}
