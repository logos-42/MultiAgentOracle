use crate::consensus::voting::Vote;
use crate::consensus::consensus_result::AggregationMethod;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// 聚合算法
pub struct AggregationAlgorithm {
    /// 配置
    config: AggregationConfig,
    /// 算法实现
    algorithms: HashMap<AggregationMethod, Box<dyn AggregationStrategy + Send + Sync>>,
}

impl std::fmt::Debug for AggregationAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AggregationAlgorithm")
            .field("config", &self.config)
            .field("algorithms", &format!("HashMap<AggregationMethod, Box<dyn AggregationStrategy>> with {} entries", self.algorithms.len()))
            .finish()
    }
}

impl Clone for AggregationAlgorithm {
    fn clone(&self) -> Self {
        // 由于 dyn AggregationStrategy 不能克隆，我们需要重新创建算法
        let mut algorithms = HashMap::new();
        
        // 重新注册所有聚合策略
        // 注意：需要实现具体的聚合策略类
        // 暂时使用简单的实现
        algorithms.insert(AggregationMethod::SimpleAverage, Box::new(SimpleAverageStrategy::new()) as Box<dyn AggregationStrategy + Send + Sync>);
        algorithms.insert(AggregationMethod::WeightedAverage, Box::new(WeightedAverageStrategy::new()) as Box<dyn AggregationStrategy + Send + Sync>);
        algorithms.insert(AggregationMethod::Median, Box::new(MedianStrategy::new()) as Box<dyn AggregationStrategy + Send + Sync>);
        algorithms.insert(AggregationMethod::WeightedMedian, Box::new(WeightedMedianStrategy::new()) as Box<dyn AggregationStrategy + Send + Sync>);
        algorithms.insert(AggregationMethod::TrimmedMean, Box::new(TrimmedMeanStrategy::new(self.config.trimmed_mean_trim)) as Box<dyn AggregationStrategy + Send + Sync>);
        algorithms.insert(AggregationMethod::Adaptive, Box::new(AdaptiveStrategy::new(self.config.adaptive_threshold)) as Box<dyn AggregationStrategy + Send + Sync>);
        
        Self {
            config: self.config.clone(),
            algorithms,
        }
    }
}

/// 聚合配置
#[derive(Debug, Clone)]
pub struct AggregationConfig {
    /// 默认聚合方法
    pub default_method: AggregationMethod,
    /// 截尾均值修剪比例
    pub trimmed_mean_trim: f64,
    /// 自适应算法阈值
    pub adaptive_threshold: f64,
    /// 最小投票数
    pub min_votes: usize,
    /// 最大权重差异
    pub max_weight_variance: f64,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            default_method: AggregationMethod::WeightedMedian,
            trimmed_mean_trim: 0.1, // 修剪10%
            adaptive_threshold: 0.3,
            min_votes: 3,
            max_weight_variance: 3.0,
        }
    }
}

/// 聚合结果
#[derive(Debug, Clone)]
pub struct AggregationResult {
    /// 聚合值
    pub value: f64,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 使用的聚合方法
    pub method: AggregationMethod,
    /// 使用的投票数
    pub votes_used: usize,
    /// 总投票数
    pub total_votes: usize,
    /// 权重统计
    pub weight_stats: WeightStatistics,
}

/// 权重统计
#[derive(Debug, Clone)]
pub struct WeightStatistics {
    /// 总权重
    pub total_weight: f64,
    /// 平均权重
    pub average_weight: f64,
    /// 权重标准差
    pub weight_std_dev: f64,
    /// 最小权重
    pub min_weight: f64,
    /// 最大权重
    pub max_weight: f64,
}

/// 聚合策略 trait
pub trait AggregationStrategy {
    /// 聚合投票
    fn aggregate(&self, votes: &[Vote], weights: &[f64]) -> Result<AggregationResult>;
    
    /// 策略名称
    fn name(&self) -> &str;
}

impl AggregationAlgorithm {
    /// 创建新的聚合算法
    pub fn new(config: AggregationConfig) -> Self {
        let mut algorithms = HashMap::new();
        
        // 注册所有聚合策略
        algorithms.insert(
            AggregationMethod::WeightedAverage,
            Box::new(WeightedAverageStrategy) as Box<dyn AggregationStrategy + Send + Sync>
        );
        
        algorithms.insert(
            AggregationMethod::WeightedMedian,
            Box::new(WeightedMedianStrategy) as Box<dyn AggregationStrategy + Send + Sync>
        );
        
        algorithms.insert(
            AggregationMethod::TrimmedMean,
            Box::new(TrimmedMeanStrategy::new(config.trimmed_mean_trim))
        );
        
        algorithms.insert(
            AggregationMethod::Adaptive,
            Box::new(AdaptiveStrategy::new(config.adaptive_threshold))
        );
        
        Self {
            config,
            algorithms,
        }
    }
    
    /// 聚合投票
    pub async fn aggregate(&self, votes: &[Vote]) -> Result<AggregationResult> {
        if votes.len() < self.config.min_votes {
            return Err(anyhow!(
                "投票数不足: {} < {}", 
                votes.len(), self.config.min_votes
            ));
        }
        
        // 计算投票权重（简化：使用置信度作为权重）
        let weights: Vec<f64> = votes.iter()
            .map(|vote| vote.confidence)
            .collect();
        
        // 选择聚合方法
        let method = self.select_method(votes, &weights);
        
        // 获取聚合策略
        let strategy = self.algorithms.get(&method)
            .ok_or_else(|| anyhow!("未找到聚合策略: {:?}", method))?;
        
        // 执行聚合
        strategy.aggregate(votes, &weights)
    }
    
    /// 选择聚合方法
    fn select_method(&self, votes: &[Vote], weights: &[f64]) -> AggregationMethod {
        // 检查数据分布
        let values: Vec<f64> = votes.iter().map(|v| v.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        let cv = if mean != 0.0 { std_dev / mean } else { 0.0 };
        
        // 检查权重分布
        let weight_mean = weights.iter().sum::<f64>() / weights.len() as f64;
        let weight_variance = weights.iter()
            .map(|&w| (w - weight_mean).powi(2))
            .sum::<f64>() / weights.len() as f64;
        let weight_std_dev = weight_variance.sqrt();
        
        let weight_cv = if weight_mean != 0.0 { weight_std_dev / weight_mean } else { 0.0 };
        
        // 自适应选择
        if cv > self.config.adaptive_threshold || weight_cv > self.config.max_weight_variance {
            // 数据或权重分布不均匀，使用稳健的方法
            AggregationMethod::WeightedMedian
        } else {
            // 分布相对均匀，使用默认方法
            self.config.default_method.clone()
        }
    }
    
    /// 获取所有可用的聚合方法
    pub fn get_available_methods(&self) -> Vec<AggregationMethod> {
        self.algorithms.keys().cloned().collect()
    }
    
    /// 设置默认聚合方法
    pub fn set_default_method(&mut self, method: AggregationMethod) {
        self.config.default_method = method;
    }
}

/// 简单平均策略
struct SimpleAverageStrategy;

impl SimpleAverageStrategy {
    fn new() -> Self {
        Self
    }
}

/// 中位数策略
struct MedianStrategy;

impl MedianStrategy {
    fn new() -> Self {
        Self
    }
}

/// 加权平均策略
struct WeightedAverageStrategy;

impl WeightedAverageStrategy {
    fn new() -> Self {
        Self
    }
}

impl AggregationStrategy for SimpleAverageStrategy {
    fn aggregate(&self, votes: &[Vote], weights: &[f64]) -> Result<AggregationResult> {
        if votes.is_empty() {
            return Err(anyhow!("没有投票数据"));
        }
        
        // 计算简单平均值
        let sum: f64 = votes.iter().map(|v| v.value).sum();
        let value = sum / votes.len() as f64;
        
        // 计算平均置信度
        let avg_confidence = votes.iter()
            .map(|v| v.confidence)
            .sum::<f64>() / votes.len() as f64;
        
        // 计算权重统计（对于简单平均，所有权重相等）
        let weight_stats = WeightStatistics {
            total_weight: votes.len() as f64,
            average_weight: 1.0,
            weight_std_dev: 0.0,
            min_weight: 1.0,
            max_weight: 1.0,
        };
        
        Ok(AggregationResult {
            value,
            confidence: avg_confidence,
            method: AggregationMethod::SimpleAverage,
            votes_used: votes.len(),
            total_votes: votes.len(),
            weight_stats,
        })
    }
    
    fn name(&self) -> &str {
        "简单平均"
    }
}

impl AggregationStrategy for MedianStrategy {
    fn aggregate(&self, votes: &[Vote], weights: &[f64]) -> Result<AggregationResult> {
        if votes.is_empty() {
            return Err(anyhow!("没有投票数据"));
        }
        
        // 提取值并排序
        let mut values: Vec<f64> = votes.iter().map(|v| v.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // 计算中位数
        let value = if values.len() % 2 == 0 {
            // 偶数个值，取中间两个的平均值
            let mid = values.len() / 2;
            (values[mid - 1] + values[mid]) / 2.0
        } else {
            // 奇数个值，取中间值
            values[values.len() / 2]
        };
        
        // 计算平均置信度
        let avg_confidence = votes.iter()
            .map(|v| v.confidence)
            .sum::<f64>() / votes.len() as f64;
        
        // 计算权重统计
        let weight_stats = calculate_weight_statistics(weights);
        
        Ok(AggregationResult {
            value,
            confidence: avg_confidence,
            method: AggregationMethod::Median,
            votes_used: votes.len(),
            total_votes: votes.len(),
            weight_stats,
        })
    }
    
    fn name(&self) -> &str {
        "中位数"
    }
}

impl AggregationStrategy for WeightedAverageStrategy {
    fn aggregate(&self, votes: &[Vote], weights: &[f64]) -> Result<AggregationResult> {
        if votes.len() != weights.len() {
            return Err(anyhow!("投票和权重数量不匹配"));
        }
        
        let total_weight: f64 = weights.iter().sum();
        if total_weight == 0.0 {
            return Err(anyhow!("总权重为0"));
        }
        
        // 计算加权平均值
        let weighted_sum: f64 = votes.iter()
            .zip(weights.iter())
            .map(|(vote, &weight)| vote.value * weight)
            .sum();
        
        let value = weighted_sum / total_weight;
        
        // 计算平均置信度
        let avg_confidence = votes.iter()
            .map(|v| v.confidence)
            .sum::<f64>() / votes.len() as f64;
        
        // 计算权重统计
        let weight_stats = calculate_weight_statistics(weights);
        
        Ok(AggregationResult {
            value,
            confidence: avg_confidence,
            method: AggregationMethod::WeightedAverage,
            votes_used: votes.len(),
            total_votes: votes.len(),
            weight_stats,
        })
    }
    
    fn name(&self) -> &str {
        "加权平均"
    }
}

/// 加权中位数策略
struct WeightedMedianStrategy;

impl WeightedMedianStrategy {
    fn new() -> Self {
        Self
    }
}

/// 截尾均值策略
struct TrimmedMeanStrategy {
    trim_percentage: f64,
}

impl TrimmedMeanStrategy {
    fn new(trim_percentage: f64) -> Self {
        Self {
            trim_percentage: trim_percentage.clamp(0.0, 0.5),
        }
    }
}

/// 自适应聚合策略
struct AdaptiveStrategy {
    threshold: f64,
}

impl AdaptiveStrategy {
    fn new(threshold: f64) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
        }
    }
}

impl AggregationStrategy for WeightedMedianStrategy {
    fn aggregate(&self, votes: &[Vote], weights: &[f64]) -> Result<AggregationResult> {
        if votes.len() != weights.len() {
            return Err(anyhow!("投票和权重数量不匹配"));
        }
        
        // 将投票和权重组合并排序
        let mut weighted_votes: Vec<(f64, f64)> = votes.iter()
            .zip(weights.iter())
            .map(|(vote, &weight)| (vote.value, weight))
            .collect();
        
        weighted_votes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        let total_weight: f64 = weights.iter().sum();
        if total_weight == 0.0 {
            return Err(anyhow!("总权重为0"));
        }
        
        // 计算加权中位数
        let mut cumulative_weight = 0.0;
        let target_weight = total_weight / 2.0;
        
        for (value, weight) in &weighted_votes {
            cumulative_weight += weight;
            if cumulative_weight >= target_weight {
                // 计算平均置信度
                let avg_confidence = votes.iter()
                    .map(|v| v.confidence)
                    .sum::<f64>() / votes.len() as f64;
                
                // 计算权重统计
                let weight_stats = calculate_weight_statistics(weights);
                
                return Ok(AggregationResult {
                    value: *value,
                    confidence: avg_confidence,
                    method: AggregationMethod::WeightedMedian,
                    votes_used: votes.len(),
                    total_votes: votes.len(),
                    weight_stats,
                });
            }
        }
        
        // 如果没找到（理论上不会发生），返回最后一个值
        let last_value = weighted_votes.last()
            .map(|(value, _)| *value)
            .unwrap_or(0.0);
        
        let avg_confidence = votes.iter()
            .map(|v| v.confidence)
            .sum::<f64>() / votes.len() as f64;
        
        let weight_stats = calculate_weight_statistics(weights);
        
        Ok(AggregationResult {
            value: last_value,
            confidence: avg_confidence,
            method: AggregationMethod::WeightedMedian,
            votes_used: votes.len(),
            total_votes: votes.len(),
            weight_stats,
        })
    }
    
    fn name(&self) -> &str {
        "加权中位数"
    }
}

impl AggregationStrategy for TrimmedMeanStrategy {
    fn aggregate(&self, votes: &[Vote], weights: &[f64]) -> Result<AggregationResult> {
        if votes.len() != weights.len() {
            return Err(anyhow!("投票和权重数量不匹配"));
        }
        
        // 将投票和权重组合并排序
        let mut weighted_votes: Vec<(f64, f64)> = votes.iter()
            .zip(weights.iter())
            .map(|(vote, &weight)| (vote.value, weight))
            .collect();
        
        weighted_votes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        // 计算要修剪的数量
        let trim_count = (weighted_votes.len() as f64 * self.trim_percentage).floor() as usize;
        let start_idx = trim_count;
        let end_idx = weighted_votes.len() - trim_count;
        
        if start_idx >= end_idx {
            return Err(anyhow!("修剪后没有剩余数据"));
        }
        
        // 计算修剪后的加权平均值
        let trimmed_votes = &weighted_votes[start_idx..end_idx];
        let total_weight: f64 = trimmed_votes.iter()
            .map(|(_, weight)| weight)
            .sum();
        
        if total_weight == 0.0 {
            return Err(anyhow!("修剪后总权重为0"));
        }
        
        let weighted_sum: f64 = trimmed_votes.iter()
            .map(|(value, weight)| value * weight)
            .sum();
        
        let value = weighted_sum / total_weight;
        
        // 计算平均置信度（使用所有投票）
        let avg_confidence = votes.iter()
            .map(|v| v.confidence)
            .sum::<f64>() / votes.len() as f64;
        
        // 计算权重统计
        let weight_stats = calculate_weight_statistics(weights);
        
        Ok(AggregationResult {
            value,
            confidence: avg_confidence,
            method: AggregationMethod::TrimmedMean,
            votes_used: trimmed_votes.len(),
            total_votes: votes.len(),
            weight_stats,
        })
    }
    
    fn name(&self) -> &str {
        "截尾均值"
    }
}

impl AggregationStrategy for AdaptiveStrategy {
    fn aggregate(&self, votes: &[Vote], weights: &[f64]) -> Result<AggregationResult> {
        // 计算变异系数
        let values: Vec<f64> = votes.iter().map(|v| v.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        let cv = if mean != 0.0 { std_dev / mean } else { 0.0 };
        
        // 根据变异系数选择策略
        if cv > self.threshold {
            // 变异大，使用加权中位数
            WeightedMedianStrategy.aggregate(votes, weights)
        } else {
            // 变异小，使用加权平均值
            WeightedAverageStrategy.aggregate(votes, weights)
        }
    }
    
    fn name(&self) -> &str {
        "自适应聚合"
    }
}

/// 计算权重统计
fn calculate_weight_statistics(weights: &[f64]) -> WeightStatistics {
    if weights.is_empty() {
        return WeightStatistics {
            total_weight: 0.0,
            average_weight: 0.0,
            weight_std_dev: 0.0,
            min_weight: 0.0,
            max_weight: 0.0,
        };
    }
    
    let total_weight: f64 = weights.iter().sum();
    let average_weight = total_weight / weights.len() as f64;
    
    let variance = weights.iter()
        .map(|&w| (w - average_weight).powi(2))
        .sum::<f64>() / weights.len() as f64;
    let weight_std_dev = variance.sqrt();
    
    let min_weight = weights.iter()
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let max_weight = weights.iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    WeightStatistics {
        total_weight,
        average_weight,
        weight_std_dev,
        min_weight,
        max_weight,
    }
}

/// 自定义聚合策略
pub struct CustomAggregationStrategy {
    name: String,
    aggregator: Box<dyn Fn(&[Vote], &[f64]) -> Result<AggregationResult> + Send + Sync>,
}

impl CustomAggregationStrategy {
    /// 创建自定义聚合策略
    pub fn new<F>(name: &str, aggregator: F) -> Self
    where
        F: Fn(&[Vote], &[f64]) -> Result<AggregationResult> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            aggregator: Box::new(aggregator),
        }
    }
}

impl AggregationStrategy for CustomAggregationStrategy {
    fn aggregate(&self, votes: &[Vote], weights: &[f64]) -> Result<AggregationResult> {
        (self.aggregator)(votes, weights)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}
