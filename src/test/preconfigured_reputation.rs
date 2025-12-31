//! 预配置信誉等级系统
//! 
//! 为10个测试节点提供预定义的信誉等级配置

use std::collections::HashMap;

/// 信誉等级配置
pub struct ReputationLevel {
    pub name: String,           // 层级名称：core, validator, data
    pub min_score: f64,         // 最低信誉分
    pub max_score: f64,         // 最高信誉分
    pub voting_weight: f64,     // 投票权重乘数
    pub required_stake: f64,    // 要求质押金额
    pub max_connections: usize, // 最大连接数
}

/// 预配置的信誉等级系统
pub struct PreconfiguredReputation {
    pub levels: Vec<ReputationLevel>,
    pub node_reputation: HashMap<String, f64>,
}

impl PreconfiguredReputation {
    /// 创建新的预配置信誉系统
    pub fn new() -> Self {
        let levels = vec![
            ReputationLevel {
                name: "core".to_string(),
                min_score: 800.0,
                max_score: 1000.0,
                voting_weight: 2.0,
                required_stake: 0.5,
                max_connections: 10,
            },
            ReputationLevel {
                name: "validator".to_string(),
                min_score: 500.0,
                max_score: 799.0,
                voting_weight: 1.5,
                required_stake: 0.3,
                max_connections: 8,
            },
            ReputationLevel {
                name: "data".to_string(),
                min_score: 0.0,
                max_score: 499.0,
                voting_weight: 1.0,
                required_stake: 0.1,
                max_connections: 5,
            },
        ];
        
        // 10个测试节点的预配置信誉分
        let node_reputation = HashMap::from([
            ("node1".to_string(), 850.0),
            ("node2".to_string(), 820.0),
            ("node3".to_string(), 650.0),
            ("node4".to_string(), 580.0),
            ("node5".to_string(), 520.0),
            ("node6".to_string(), 350.0),
            ("node7".to_string(), 280.0),
            ("node8".to_string(), 220.0),
            ("node9".to_string(), 150.0),
            ("node10".to_string(), 80.0),
        ]);
        
        Self {
            levels,
            node_reputation,
        }
    }
    
    /// 获取10个测试节点的预配置信誉
    pub fn get_test_nodes_reputation(&self) -> &HashMap<String, f64> {
        &self.node_reputation
    }
    
    /// 根据信誉分确定层级
    pub fn determine_tier(&self, score: f64) -> String {
        for level in &self.levels {
            if score >= level.min_score && score <= level.max_score {
                return level.name.clone();
            }
        }
        "data".to_string() // 默认数据层
    }
    
    /// 获取层级的投票权重
    pub fn get_tier_voting_weight(&self, tier: &str) -> f64 {
        self.levels
            .iter()
            .find(|level| level.name == tier)
            .map(|level| level.voting_weight)
            .unwrap_or(1.0)
    }
    
    /// 获取层级要求的质押金额
    pub fn get_tier_required_stake(&self, tier: &str) -> f64 {
        self.levels
            .iter()
            .find(|level| level.name == tier)
            .map(|level| level.required_stake)
            .unwrap_or(0.1)
    }
    
    /// 获取层级的最大连接数
    pub fn get_tier_max_connections(&self, tier: &str) -> usize {
        self.levels
            .iter()
            .find(|level| level.name == tier)
            .map(|level| level.max_connections)
            .unwrap_or(5)
    }
    
    /// 计算节点的综合权重
    pub fn calculate_node_weight(&self, node_id: &str, stake: f64) -> f64 {
        let reputation = self.node_reputation.get(node_id).copied().unwrap_or(0.0);
        let tier = self.determine_tier(reputation);
        let tier_weight = self.get_tier_voting_weight(&tier);
        
        // 综合权重 = 信誉分 * 层级权重 * (1 + 质押比例)
        reputation * tier_weight * (1.0 + stake)
    }
    
    /// 模拟信誉更新（用于测试层级迁移）
    pub fn simulate_reputation_update(&mut self, node_id: &str, delta: f64) -> Result<String, String> {
        if let Some(current_reputation) = self.node_reputation.get_mut(node_id) {
            let old_tier = self.determine_tier(*current_reputation);
            
            // 更新信誉分
            *current_reputation = (*current_reputation + delta).max(0.0).min(1000.0);
            
            let new_tier = self.determine_tier(*current_reputation);
            
            if old_tier != new_tier {
                Ok(format!(
                    "节点 {} 信誉分更新: {:.1} -> {:.1}, 层级变更: {} -> {}",
                    node_id, *current_reputation - delta, *current_reputation, old_tier, new_tier
                ))
            } else {
                Ok(format!(
                    "节点 {} 信誉分更新: {:.1} -> {:.1}, 层级不变: {}",
                    node_id, *current_reputation - delta, *current_reputation, old_tier
                ))
            }
        } else {
            Err(format!("节点 {} 不存在", node_id))
        }
    }
    
    /// 获取所有节点的层级分布
    pub fn get_tier_distribution(&self) -> HashMap<String, Vec<String>> {
        let mut distribution = HashMap::new();
        
        for (node_id, reputation) in &self.node_reputation {
            let tier = self.determine_tier(*reputation);
            distribution
                .entry(tier)
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }
        
        distribution
    }
    
    /// 显示信誉等级配置
    pub fn show_reputation_levels(&self) {
        println!("📊 信誉等级配置");
        println!("====================");
        
        for level in &self.levels {
            println!("  {}层:", level.name);
            println!("    信誉范围: {:.0} - {:.0}", level.min_score, level.max_score);
            println!("    投票权重: {:.1}x", level.voting_weight);
            println!("    要求质押: {:.2}", level.required_stake);
            println!("    最大连接数: {}", level.max_connections);
            println!();
        }
    }
    
    /// 显示节点信誉状态
    pub fn show_node_reputation_status(&self) {
        println!("📈 节点信誉状态");
        println!("====================");
        
        let mut nodes_by_tier = self.get_tier_distribution();
        
        for tier in ["core", "validator", "data"] {
            if let Some(nodes) = nodes_by_tier.get(tier) {
                println!("  {}层 ({}个节点):", tier, nodes.len());
                
                for node_id in nodes {
                    if let Some(reputation) = self.node_reputation.get(node_id) {
                        let weight = self.calculate_node_weight(node_id, 0.0);
                        println!("    {}: 信誉={:.1}, 权重={:.1}", node_id, reputation, weight);
                    }
                }
                println!();
            }
        }
    }
    
    /// 检查节点是否符合层级要求
    pub fn check_node_tier_requirements(&self, node_id: &str, stake: f64) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if let Some(reputation) = self.node_reputation.get(node_id) {
            let tier = self.determine_tier(*reputation);
            let required_stake = self.get_tier_required_stake(&tier);
            
            if stake < required_stake {
                errors.push(format!(
                    "节点 {} 的质押金额 {:.2} 低于 {} 层要求 {:.2}",
                    node_id, stake, tier, required_stake
                ));
            }
            
            if *reputation < 0.0 || *reputation > 1000.0 {
                errors.push(format!(
                    "节点 {} 的信誉分 {:.1} 超出有效范围 [0, 1000]",
                    node_id, reputation
                ));
            }
        } else {
            errors.push(format!("节点 {} 不存在", node_id));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for PreconfiguredReputation {
    fn default() -> Self {
        Self::new()
    }
}
