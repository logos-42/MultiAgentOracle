//! 共识验证模块
//! 
//! 提供共识过程中的数据验证功能

use crate::types::{ValidationResult, DataHash, NodeId, Timestamp, current_timestamp};

/// 数据验证器
pub struct DataValidator {
    /// 验证器节点ID
    validator_id: NodeId,
    /// 验证阈值
    validation_threshold: f64,
    /// 验证历史
    validation_history: Vec<ValidationResult>,
}

impl DataValidator {
    /// 创建新的数据验证器
    pub fn new(validator_id: NodeId, validation_threshold: f64) -> Self {
        Self {
            validator_id,
            validation_threshold,
            validation_history: Vec::new(),
        }
    }
    
    /// 验证数据
    pub fn validate_data(&mut self, data_hash: DataHash, data: &[u8]) -> ValidationResult {
        let validation_time = current_timestamp();
        
        // 简单的数据验证逻辑
        // 在实际实现中，这里会有更复杂的验证规则
        let valid = !data.is_empty() && data.len() <= 1024 * 1024; // 不超过1MB
        
        let result = ValidationResult {
            data_hash: data_hash.clone(),
            valid,
            validator_id: self.validator_id.clone(),
            validation_time,
            error: if valid {
                None
            } else {
                Some("数据无效或超过大小限制".to_string())
            },
            signature: format!("sig_{}_{}", self.validator_id, validation_time),
        };
        
        // 记录验证历史
        self.validation_history.push(result.clone());
        
        // 限制历史记录大小
        if self.validation_history.len() > 1000 {
            self.validation_history.remove(0);
        }
        
        result
    }
    
    /// 批量验证数据
    pub fn validate_batch(&mut self, data_entries: Vec<(DataHash, Vec<u8>)>) -> Vec<ValidationResult> {
        data_entries
            .into_iter()
            .map(|(hash, data)| self.validate_data(hash, &data))
            .collect()
    }
    
    /// 获取验证成功率
    pub fn get_validation_success_rate(&self) -> f64 {
        if self.validation_history.is_empty() {
            return 1.0;
        }
        
        let success_count = self.validation_history
            .iter()
            .filter(|result| result.valid)
            .count();
        
        success_count as f64 / self.validation_history.len() as f64
    }
    
    /// 获取最近的验证结果
    pub fn get_recent_validations(&self, limit: usize) -> &[ValidationResult] {
        let start = if self.validation_history.len() > limit {
            self.validation_history.len() - limit
        } else {
            0
        };
        
        &self.validation_history[start..]
    }
    
    /// 检查数据是否通过验证阈值
    pub fn check_validation_threshold(&self, validations: &[ValidationResult]) -> bool {
        if validations.is_empty() {
            return false;
        }
        
        let valid_count = validations.iter().filter(|v| v.valid).count();
        let valid_ratio = valid_count as f64 / validations.len() as f64;
        
        valid_ratio >= self.validation_threshold
    }
    
    /// 验证签名
    pub fn verify_signature(&self, _data_hash: &DataHash, signature: &str, timestamp: Timestamp) -> bool {
        // 简单的签名验证逻辑
        // 在实际实现中，这里会有真正的加密签名验证
        signature == format!("sig_{}_{}", self.validator_id, timestamp)
    }
}

/// 共识提案验证器
pub struct ProposalValidator {
    /// 最小投票节点数
    min_voters: u32,
    /// 共识阈值
    consensus_threshold: f64,
}

impl ProposalValidator {
    /// 创建新的提案验证器
    pub fn new(min_voters: u32, consensus_threshold: f64) -> Self {
        Self {
            min_voters,
            consensus_threshold,
        }
    }
    
    /// 验证共识提案
    pub fn validate_proposal(
        &self,
        _proposal_id: &str,
        yes_votes: u32,
        no_votes: u32,
        total_voters: u32,
        weights: Option<&[f64]>,
    ) -> bool {
        // 检查是否有足够多的投票者
        if total_voters < self.min_voters {
            return false;
        }
        
        // 计算通过率
        let total_votes = yes_votes + no_votes;
        if total_votes == 0 {
            return false;
        }
        
        let approval_rate = if let Some(weights) = weights {
            // 加权投票
            let total_weight: f64 = weights.iter().sum();
            let yes_weight: f64 = weights[..yes_votes as usize].iter().sum();
            yes_weight / total_weight
        } else {
            // 简单多数
            yes_votes as f64 / total_votes as f64
        };
        
        approval_rate >= self.consensus_threshold
    }
    
    /// 验证投票权重
    pub fn validate_voting_weights(&self, weights: &[f64], voters: &[NodeId]) -> bool {
        // 检查权重数量与投票者数量匹配
        if weights.len() != voters.len() {
            return false;
        }
        
        // 检查权重有效性
        weights.iter().all(|&w| w >= 0.0 && w <= 1.0)
    }
}

/// 层级验证器
pub struct TierValidator {
    /// 层级配置
    tier_config: Vec<TierConfig>,
}

/// 层级配置
#[derive(Debug, Clone)]
pub struct TierConfig {
    /// 层级名称
    pub name: String,
    /// 最小信誉分
    pub min_reputation: f64,
    /// 最大信誉分
    pub max_reputation: f64,
    /// 投票权重
    pub voting_weight: f64,
    /// 最大连接数
    pub max_connections: u32,
}

impl TierValidator {
    /// 创建新的层级验证器
    pub fn new(tier_config: Vec<TierConfig>) -> Self {
        Self { tier_config }
    }
    
    /// 验证节点层级
    pub fn validate_node_tier(&self, _node_id: &str, reputation: f64, current_tier: &str) -> Option<String> {
        for config in &self.tier_config {
            if reputation >= config.min_reputation && reputation <= config.max_reputation {
                if config.name != current_tier {
                    return Some(config.name.clone());
                }
                break;
            }
        }
        
        None
    }
    
    /// 获取层级的投票权重
    pub fn get_tier_voting_weight(&self, tier: &str) -> Option<f64> {
        self.tier_config
            .iter()
            .find(|config| config.name == tier)
            .map(|config| config.voting_weight)
    }
    
    /// 检查节点是否符合层级要求
    pub fn check_tier_requirements(&self, tier: &str, reputation: f64) -> bool {
        self.tier_config
            .iter()
            .find(|config| config.name == tier)
            .map(|config| reputation >= config.min_reputation && reputation <= config.max_reputation)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_validation() {
        let mut validator = DataValidator::new("validator1".to_string(), 0.7);
        
        let data = b"test data";
        let hash = "hash123".to_string();
        
        let result = validator.validate_data(hash.clone(), data);
        
        assert!(result.valid);
        assert_eq!(result.data_hash, hash);
        assert_eq!(result.validator_id, "validator1");
    }
    
    #[test]
    fn test_proposal_validation() {
        let validator = ProposalValidator::new(3, 0.67);
        
        // 测试通过的情况
        assert!(validator.validate_proposal("prop1", 7, 3, 10, None));
        
        // 测试不通过的情况（投票者不足）
        assert!(!validator.validate_proposal("prop2", 2, 1, 2, None));
        
        // 测试不通过的情况（通过率不足）
        assert!(!validator.validate_proposal("prop3", 5, 5, 10, None));
    }
    
    #[test]
    fn test_tier_validation() {
        let tier_config = vec![
            TierConfig {
                name: "core".to_string(),
                min_reputation: 800.0,
                max_reputation: 1000.0,
                voting_weight: 2.0,
                max_connections: 10,
            },
            TierConfig {
                name: "validator".to_string(),
                min_reputation: 500.0,
                max_reputation: 799.0,
                voting_weight: 1.5,
                max_connections: 8,
            },
        ];
        
        let validator = TierValidator::new(tier_config);
        
        // 测试层级验证
        assert_eq!(
            validator.validate_node_tier("node1", 850.0, "validator"),
            Some("core".to_string())
        );
        
        // 测试投票权重获取
        assert_eq!(validator.get_tier_voting_weight("core"), Some(2.0));
        
        // 测试层级要求检查
        assert!(validator.check_tier_requirements("core", 850.0));
        assert!(!validator.check_tier_requirements("core", 750.0));
    }
}
