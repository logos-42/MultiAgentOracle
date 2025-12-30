use crate::reputation::ReputationConfig;
use std::f64::consts::E;

/// 信誉算法
pub struct ReputationAlgorithms {
    config: ReputationConfig,
}

impl ReputationAlgorithms {
    /// 创建新的信誉算法
    pub fn new(config: ReputationConfig) -> Self {
        Self { config }
    }
    
    /// 计算数据准确性带来的信誉变化
    pub fn calculate_accuracy_delta(
        &self,
        expected: f64,
        actual: f64,
        tolerance: f64,
        weight: f64,
    ) -> f64 {
        if expected == 0.0 {
            return 0.0;
        }
        
        let error = (actual - expected).abs() / expected;
        let normalized_error = error / tolerance;
        
        // 使用Sigmoid函数计算分数
        // 当误差在容忍范围内时给正分，超出时给负分
        let score = 2.0 / (1.0 + E.powf(5.0 * normalized_error)) - 1.0;
        
        // 应用权重并缩放
        let delta = score * weight * 10.0;
        
        delta
    }
    
    /// 计算响应时间带来的信誉变化
    pub fn calculate_response_time_delta(
        &self,
        expected_ms: u64,
        actual_ms: u64,
        weight: f64,
    ) -> f64 {
        if expected_ms == 0 {
            return 0.0;
        }
        
        let ratio = actual_ms as f64 / expected_ms as f64;
        
        // 使用对数函数，响应越快得分越高
        let score = if ratio <= 1.0 {
            1.0 - ratio.ln() / 10.0  // 比预期快，正分
        } else {
            -ratio.ln() / 5.0  // 比预期慢，负分
        };
        
        // 应用权重并缩放
        let delta = score.clamp(-1.0, 1.0) * weight * 5.0;
        
        delta
    }
    
    /// 计算服务可用性带来的信誉变化
    pub fn calculate_availability_delta(
        &self,
        expected_uptime: f64,
        actual_uptime: f64,
        weight: f64,
    ) -> f64 {
        let difference = actual_uptime - expected_uptime;
        
        // 使用Sigmoid函数，可用性越高得分越高
        let score = 2.0 / (1.0 + E.powf(-10.0 * difference)) - 1.0;
        
        // 应用权重并缩放
        let delta = score * weight * 5.0;
        
        delta
    }
    
    /// 计算综合信誉分
    pub fn calculate_comprehensive_score(
        &self,
        accuracy_score: f64,
        response_time_score: f64,
        availability_score: f64,
    ) -> f64 {
        let total_weight = self.config.accuracy_weight 
            + self.config.response_time_weight 
            + self.config.availability_weight;
        
        if total_weight == 0.0 {
            return 0.0;
        }
        
        let weighted_sum = accuracy_score * self.config.accuracy_weight
            + response_time_score * self.config.response_time_weight
            + availability_score * self.config.availability_weight;
        
        weighted_sum / total_weight
    }
    
    /// 计算信誉衰减
    pub fn calculate_decay(
        &self,
        current_score: f64,
        days_inactive: f64,
    ) -> f64 {
        if days_inactive <= 0.0 {
            return 0.0;
        }
        
        // 指数衰减：分数越高，衰减越快
        let decay_rate = self.config.decay_rate_per_day * (current_score / 1000.0);
        let decay_amount = current_score * (1.0 - (1.0 - decay_rate).powf(days_inactive));
        
        -decay_amount
    }
    
    /// 计算投票权重
    pub fn calculate_voting_weight(
        &self,
        reputation_score: f64,
        staked_amount: u64,
    ) -> f64 {
        // 权重 = 信誉分 × sqrt(质押金额)
        // 这样既考虑信誉又考虑经济承诺，但质押的影响是次线性的
        reputation_score * (staked_amount as f64).sqrt()
    }
    
    /// 计算信誉等级阈值
    pub fn calculate_tier_thresholds(&self) -> Vec<(String, f64, f64)> {
        vec![
            ("Newbie".to_string(), 0.0, 99.9),
            ("Copper".to_string(), 100.0, 299.9),
            ("Iron".to_string(), 300.0, 499.9),
            ("Bronze".to_string(), 500.0, 599.9),
            ("Silver".to_string(), 600.0, 699.9),
            ("Gold".to_string(), 700.0, 799.9),
            ("Diamond".to_string(), 800.0, 899.9),
            ("Platinum".to_string(), 900.0, 1000.0),
        ]
    }
    
    /// 计算信誉变化对投票权重的影响
    pub fn calculate_weight_change(
        &self,
        old_reputation: f64,
        new_reputation: f64,
        staked_amount: u64,
    ) -> f64 {
        let old_weight = self.calculate_voting_weight(old_reputation, staked_amount);
        let new_weight = self.calculate_voting_weight(new_reputation, staked_amount);
        
        new_weight - old_weight
    }
    
    /// 计算惩罚金额
    pub fn calculate_penalty_amount(
        &self,
        severity: crate::reputation::reputation_score::PenaltySeverity,
        current_score: f64,
    ) -> f64 {
        let base_penalty = match severity {
            crate::reputation::reputation_score::PenaltySeverity::Minor => 10.0,
            crate::reputation::reputation_score::PenaltySeverity::Moderate => 50.0,
            crate::reputation::reputation_score::PenaltySeverity::Severe => 100.0,
            crate::reputation::reputation_score::PenaltySeverity::Malicious => 200.0,
        };
        
        // 分数越高，惩罚越重（因为期望更高）
        let multiplier = 1.0 + (current_score / 1000.0);
        
        -base_penalty * multiplier * self.config.penalty_multiplier
    }
    
    /// 计算奖励金额
    pub fn calculate_reward_amount(
        &self,
        contribution: f64,
        current_score: f64,
    ) -> f64 {
        // 基础奖励
        let base_reward = contribution * 10.0;
        
        // 分数越低，奖励越高（鼓励新节点）
        let multiplier = 2.0 - (current_score / 1000.0);
        
        base_reward * multiplier * self.config.reward_multiplier
    }
    
    /// 计算信誉恢复率
    pub fn calculate_recovery_rate(
        &self,
        current_score: f64,
        historical_performance: f64,
    ) -> f64 {
        // 当前分数越低，恢复越快
        let score_factor = 1.0 - (current_score / 1000.0);
        
        // 历史表现越好，恢复越快
        let performance_factor = historical_performance;
        
        // 基础恢复率
        let base_recovery = 0.1;
        
        base_recovery * score_factor * performance_factor
    }
    
    /// 计算信誉稳定性
    pub fn calculate_stability_score(
        &self,
        score_history: &[f64],
    ) -> f64 {
        if score_history.len() < 2 {
            return 1.0;
        }
        
        let mean = score_history.iter().sum::<f64>() / score_history.len() as f64;
        let variance = score_history.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / score_history.len() as f64;
        
        let std_dev = variance.sqrt();
        
        // 标准差越小，稳定性越高
        1.0 / (1.0 + std_dev)
    }
}
