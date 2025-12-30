use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 投票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// 智能体DID
    pub agent_did: String,
    /// 投票值
    pub value: f64,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 时间戳
    pub timestamp: u64,
    /// 数据源列表
    pub sources: Vec<String>,
    /// 签名
    pub signature: Option<String>,
    /// 元数据
    pub metadata: Option<serde_json::Value>,
}

impl Vote {
    /// 创建新的投票
    pub fn new(
        agent_did: String,
        value: f64,
        confidence: f64,
        sources: Vec<String>,
    ) -> Self {
        Self {
            agent_did,
            value,
            confidence: confidence.clamp(0.0, 1.0),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            sources,
            signature: None,
            metadata: None,
        }
    }
    
    /// 验证投票有效性
    pub fn validate(&self) -> bool {
        if self.agent_did.is_empty() {
            return false;
        }
        
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return false;
        }
        
        if self.sources.is_empty() {
            return false;
        }
        
        // 检查时间戳（不能是未来时间，不能太旧）
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if self.timestamp > now + 300 { // 不能是未来5分钟的时间
            return false;
        }
        
        if now - self.timestamp > 3600 { // 不能是1小时前的数据
            return false;
        }
        
        true
    }
    
    /// 设置签名
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }
    
    /// 设置元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// 获取投票权重
    pub fn get_weight(&self, reputation_score: f64, staked_amount: u64) -> f64 {
        // 权重 = 信誉分 × sqrt(质押金额) × 置信度
        reputation_score * (staked_amount as f64).sqrt() * self.confidence
    }
    
    /// 转换为JSON字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
    
    /// 从JSON字符串解析
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// 投票结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    /// 共识ID
    pub consensus_id: String,
    /// 总投票数
    pub total_votes: usize,
    /// 有效投票数
    pub valid_votes: usize,
    /// 总权重
    pub total_weight: f64,
    /// 投票统计
    pub statistics: VotingStatistics,
    /// 异常投票
    pub anomalies: Vec<VoteAnomaly>,
    /// 时间窗口
    pub time_window: (u64, u64),
}

/// 投票统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingStatistics {
    /// 平均值
    pub mean: f64,
    /// 中位数
    pub median: f64,
    /// 标准差
    pub standard_deviation: f64,
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 第25百分位
    pub percentile_25: f64,
    /// 第75百分位
    pub percentile_75: f64,
    /// 置信度分布
    pub confidence_distribution: Vec<(f64, usize)>, // (置信度范围, 数量)
    /// 数据源分布
    pub source_distribution: Vec<(String, usize)>, // (数据源, 数量)
}

/// 投票异常
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteAnomaly {
    /// 投票
    pub vote: Vote,
    /// 异常类型
    pub anomaly_type: AnomalyType,
    /// 异常分数 (0.0-1.0)
    pub anomaly_score: f64,
    /// 原因
    pub reason: String,
}

/// 异常类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    /// 值异常（离群值）
    ValueOutlier,
    /// 时间异常（投票时间异常）
    TimeAnomaly,
    /// 置信度异常
    ConfidenceAnomaly,
    /// 数据源异常
    SourceAnomaly,
    /// 签名异常
    SignatureAnomaly,
    /// 复合异常
    Composite,
}

/// 投票权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingWeight {
    /// 智能体DID
    pub agent_did: String,
    /// 信誉分
    pub reputation_score: f64,
    /// 质押金额
    pub staked_amount: u64,
    /// 基础权重
    pub base_weight: f64,
    /// 调整因子
    pub adjustment_factor: f64,
    /// 最终权重
    pub final_weight: f64,
}

impl VotingWeight {
    /// 计算投票权重
    pub fn calculate(
        agent_did: String,
        reputation_score: f64,
        staked_amount: u64,
        adjustment_factor: f64,
    ) -> Self {
        let base_weight = reputation_score * (staked_amount as f64).sqrt();
        let final_weight = base_weight * adjustment_factor;
        
        Self {
            agent_did,
            reputation_score,
            staked_amount,
            base_weight,
            adjustment_factor,
            final_weight,
        }
    }
    
    /// 应用惩罚
    pub fn apply_penalty(&mut self, penalty_factor: f64) {
        self.adjustment_factor *= penalty_factor;
        self.final_weight = self.base_weight * self.adjustment_factor;
    }
    
    /// 应用奖励
    pub fn apply_reward(&mut self, reward_factor: f64) {
        self.adjustment_factor *= reward_factor;
        self.final_weight = self.base_weight * self.adjustment_factor;
    }
}

/// 投票收集器
pub struct VoteCollector {
    /// 收集的投票
    votes: Vec<Vote>,
    /// 投票权重
    weights: Vec<VotingWeight>,
    /// 时间窗口开始
    window_start: u64,
    /// 时间窗口结束
    window_end: u64,
    /// 最大投票数
    max_votes: usize,
}

impl VoteCollector {
    /// 创建新的投票收集器
    pub fn new(max_votes: usize) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            votes: Vec::new(),
            weights: Vec::new(),
            window_start: now,
            window_end: now + 300, // 5分钟窗口
            max_votes,
        }
    }
    
    /// 添加投票
    pub fn add_vote(&mut self, vote: Vote, weight: VotingWeight) -> bool {
        if self.votes.len() >= self.max_votes {
            return false;
        }
        
        if !vote.validate() {
            return false;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 检查是否在时间窗口内
        if vote.timestamp < self.window_start || vote.timestamp > self.window_end {
            return false;
        }
        
        // 检查重复投票
        if self.votes.iter().any(|v| v.agent_did == vote.agent_did) {
            return false;
        }
        
        self.votes.push(vote);
        self.weights.push(weight);
        
        true
    }
    
    /// 获取投票统计
    pub fn get_statistics(&self) -> VotingStatistics {
        if self.votes.is_empty() {
            return VotingStatistics {
                mean: 0.0,
                median: 0.0,
                standard_deviation: 0.0,
                min: 0.0,
                max: 0.0,
                percentile_25: 0.0,
                percentile_75: 0.0,
                confidence_distribution: Vec::new(),
                source_distribution: Vec::new(),
            };
        }
        
        // 计算基本统计
        let values: Vec<f64> = self.votes.iter().map(|v| v.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if sorted_values.len() % 2 == 0 {
            let mid = sorted_values.len() / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[sorted_values.len() / 2]
        };
        
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let standard_deviation = variance.sqrt();
        
        let min = *sorted_values.first().unwrap_or(&0.0);
        let max = *sorted_values.last().unwrap_or(&0.0);
        
        let percentile_25 = if sorted_values.len() >= 4 {
            let idx = (sorted_values.len() as f64 * 0.25).floor() as usize;
            sorted_values[idx]
        } else {
            median
        };
        
        let percentile_75 = if sorted_values.len() >= 4 {
            let idx = (sorted_values.len() as f64 * 0.75).floor() as usize;
            sorted_values[idx]
        } else {
            median
        };
        
        // 计算置信度分布
        let mut confidence_distribution = Vec::new();
        for range in &[(0.0, 0.2), (0.2, 0.4), (0.4, 0.6), (0.6, 0.8), (0.8, 1.0)] {
            let count = self.votes.iter()
                .filter(|v| v.confidence >= range.0 && v.confidence < range.1)
                .count();
            confidence_distribution.push(((range.0 + range.1) / 2.0, count));
        }
        
        // 计算数据源分布
        let mut source_counts = std::collections::HashMap::new();
        for vote in &self.votes {
            for source in &vote.sources {
                *source_counts.entry(source.clone()).or_insert(0) += 1;
            }
        }
        
        let mut source_distribution: Vec<(String, usize)> = source_counts.into_iter().collect();
        source_distribution.sort_by(|a, b| b.1.cmp(&a.1));
        
        VotingStatistics {
            mean,
            median,
            standard_deviation,
            min,
            max,
            percentile_25,
            percentile_75,
            confidence_distribution,
            source_distribution,
        }
    }
    
    /// 检测异常投票
    pub fn detect_anomalies(&self) -> Vec<VoteAnomaly> {
        let mut anomalies = Vec::new();
        let stats = self.get_statistics();
        
        for vote in &self.votes {
            let mut anomaly_score = 0.0;
            let mut anomaly_types = Vec::new();
            let mut reasons = Vec::new();
            
            // 检查值异常
            let z_score = (vote.value - stats.mean).abs() / stats.standard_deviation;
            if stats.standard_deviation > 0.0 && z_score > 3.0 {
                anomaly_score += 0.4;
                anomaly_types.push(AnomalyType::ValueOutlier);
                reasons.push(format!("Z-score: {:.2}", z_score));
            }
            
            // 检查置信度异常
            if vote.confidence < 0.3 {
                anomaly_score += 0.3;
                anomaly_types.push(AnomalyType::ConfidenceAnomaly);
                reasons.push(format!("低置信度: {:.2}", vote.confidence));
            }
            
            // 检查时间异常
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if vote.timestamp > now {
                anomaly_score += 0.5;
                anomaly_types.push(AnomalyType::TimeAnomaly);
                reasons.push("未来时间戳".to_string());
            }
            
            // 检查数据源异常
            if vote.sources.len() == 1 && vote.sources[0] == "unknown" {
                anomaly_score += 0.2;
                anomaly_types.push(AnomalyType::SourceAnomaly);
                reasons.push("未知数据源".to_string());
            }
            
            if anomaly_score > 0.3 {
                let anomaly_type = if anomaly_types.len() > 1 {
                    AnomalyType::Composite
                } else {
                    anomaly_types.first().cloned().unwrap_or(AnomalyType::ValueOutlier)
                };
                
                anomalies.push(VoteAnomaly {
                    vote: vote.clone(),
                    anomaly_type,
                    anomaly_score: anomaly_score.clamp(0.0, 1.0),
                    reason: reasons.join(", "),
                });
            }
        }
        
        anomalies
    }
    
    /// 获取投票结果
    pub fn get_result(&self, consensus_id: String) -> VotingResult {
        let statistics = self.get_statistics();
        let anomalies = self.detect_anomalies();
        let valid_votes = self.votes.len() - anomalies.len();
        
        // 计算总权重
        let total_weight = self.weights.iter()
            .map(|w| w.final_weight)
            .sum();
        
        VotingResult {
            consensus_id,
            total_votes: self.votes.len(),
            valid_votes,
            total_weight,
            statistics,
            anomalies,
            time_window: (self.window_start, self.window_end),
        }
    }
    
    /// 清理过期投票
    pub fn cleanup_expired(&mut self) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let before = self.votes.len();
        
        // 移除过期投票
        let mut i = 0;
        while i < self.votes.len() {
            if self.votes[i].timestamp < now - 3600 { // 1小时前
                self.votes.remove(i);
                self.weights.remove(i);
            } else {
                i += 1;
            }
        }
        
        before - self.votes.len()
    }
}
