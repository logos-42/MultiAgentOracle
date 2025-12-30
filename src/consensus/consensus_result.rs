use crate::oracle_agent::OracleDataType;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 共识结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// 共识ID
    pub consensus_id: String,
    /// 数据类型
    pub data_type: OracleDataType,
    /// 最终值
    pub final_value: f64,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 参与者列表
    pub participants: Vec<String>,
    /// 使用的投票数
    pub votes_used: usize,
    /// 总权重
    pub total_weight: f64,
    /// 聚合方法
    pub aggregation_method: AggregationMethod,
    /// 时间戳
    pub timestamp: u64,
}

/// 共识状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsensusStatus {
    /// 空闲
    Idle,
    /// 收集投票中
    Collecting,
    /// 聚合中
    Aggregating,
    /// 争议解决中
    DisputeResolution,
    /// 已完成
    Completed,
    /// 超时
    Timeout,
    /// 失败
    Failed,
}

/// 聚合方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationMethod {
    /// 加权平均值
    WeightedAverage,
    /// 加权中位数
    WeightedMedian,
    /// 截尾均值
    TrimmedMean,
    /// 自适应聚合
    Adaptive,
    /// 自定义算法
    Custom(String),
}

/// 共识错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusError {
    /// 未达到法定人数
    InsufficientQuorum {
        required: f64,
        actual: f64,
        participants: usize,
        votes: usize,
    },
    /// 超时
    Timeout {
        timeout_secs: u64,
        elapsed_secs: u64,
    },
    /// 争议未解决
    UnresolvedDisputes {
        dispute_count: usize,
        unresolved: Vec<String>,
    },
    /// 无效投票
    InvalidVotes {
        count: usize,
        reasons: Vec<String>,
    },
    /// 参与者不足
    InsufficientParticipants {
        required: usize,
        actual: usize,
    },
    /// 内部错误
    InternalError {
        message: String,
        details: Option<String>,
    },
}

impl ConsensusResult {
    /// 创建新的共识结果
    pub fn new(
        consensus_id: String,
        data_type: OracleDataType,
        final_value: f64,
        confidence: f64,
        participants: Vec<String>,
        votes_used: usize,
        total_weight: f64,
        aggregation_method: AggregationMethod,
    ) -> Self {
        Self {
            consensus_id,
            data_type,
            final_value,
            confidence: confidence.clamp(0.0, 1.0),
            participants,
            votes_used,
            total_weight,
            aggregation_method,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// 验证结果有效性
    pub fn validate(&self) -> bool {
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return false;
        }
        
        if self.votes_used == 0 {
            return false;
        }
        
        if self.participants.is_empty() {
            return false;
        }
        
        if self.total_weight <= 0.0 {
            return false;
        }
        
        true
    }
    
    /// 获取格式化时间
    pub fn formatted_timestamp(&self) -> String {
        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(self.timestamp as i64, 0)
            .unwrap_or(chrono::Utc::now());
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    /// 计算参与率
    pub fn participation_rate(&self) -> f64 {
        if self.participants.is_empty() {
            return 0.0;
        }
        self.votes_used as f64 / self.participants.len() as f64
    }
    
    /// 获取简要信息
    pub fn get_summary(&self) -> ConsensusSummary {
        ConsensusSummary {
            consensus_id: self.consensus_id.clone(),
            data_type: format!("{:?}", self.data_type),
            final_value: self.final_value,
            confidence: self.confidence,
            participants: self.participants.len(),
            votes_used: self.votes_used,
            participation_rate: self.participation_rate(),
            timestamp: self.formatted_timestamp(),
            aggregation_method: format!("{:?}", self.aggregation_method),
        }
    }
}

/// 共识摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusSummary {
    /// 共识ID
    pub consensus_id: String,
    /// 数据类型
    pub data_type: String,
    /// 最终值
    pub final_value: f64,
    /// 置信度
    pub confidence: f64,
    /// 参与者数量
    pub participants: usize,
    /// 使用的投票数
    pub votes_used: usize,
    /// 参与率
    pub participation_rate: f64,
    /// 时间戳
    pub timestamp: String,
    /// 聚合方法
    pub aggregation_method: String,
}

/// 共识统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStats {
    /// 总共识次数
    pub total_consensus: u64,
    /// 成功次数
    pub successful: u64,
    /// 失败次数
    pub failed: u64,
    /// 平均置信度
    pub average_confidence: f64,
    /// 平均参与率
    pub average_participation_rate: f64,
    /// 平均处理时间 (秒)
    pub average_processing_time_secs: f64,
    /// 按数据类型的统计
    pub by_data_type: std::collections::HashMap<String, DataTypeStats>,
    /// 最近共识
    pub recent_consensus: Vec<ConsensusSummary>,
}

/// 数据类型统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTypeStats {
    /// 共识次数
    pub count: u64,
    /// 成功率
    pub success_rate: f64,
    /// 平均置信度
    pub average_confidence: f64,
    /// 平均处理时间
    pub average_processing_time_secs: f64,
}

/// 共识性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    /// 开始时间
    pub start_time: u64,
    /// 结束时间
    pub end_time: u64,
    /// 投票收集时间 (秒)
    pub vote_collection_time_secs: f64,
    /// 聚合时间 (秒)
    pub aggregation_time_secs: f64,
    /// 争议解决时间 (秒)
    pub dispute_resolution_time_secs: f64,
    /// 总处理时间 (秒)
    pub total_processing_time_secs: f64,
    /// 网络延迟统计
    pub network_latency: NetworkLatencyStats,
    /// 资源使用统计
    pub resource_usage: ResourceUsageStats,
}

/// 网络延迟统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLatencyStats {
    /// 平均延迟 (毫秒)
    pub average_latency_ms: u64,
    /// 第95百分位延迟
    pub p95_latency_ms: u64,
    /// 第99百分位延迟
    pub p99_latency_ms: u64,
    /// 最大延迟
    pub max_latency_ms: u64,
    /// 超时次数
    pub timeout_count: u64,
}

/// 资源使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageStats {
    /// 平均CPU使用率 (%)
    pub average_cpu_usage: f64,
    /// 平均内存使用 (MB)
    pub average_memory_usage_mb: f64,
    /// 平均网络带宽 (KB/s)
    pub average_network_bandwidth_kbps: f64,
    /// 峰值CPU使用率
    pub peak_cpu_usage: f64,
    /// 峰值内存使用
    pub peak_memory_usage_mb: f64,
}

/// 共识配置验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 警告信息
    pub warnings: Vec<String>,
    /// 错误信息
    pub errors: Vec<String>,
    /// 建议
    pub suggestions: Vec<String>,
}

impl ConfigValidationResult {
    /// 创建有效的验证结果
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    /// 创建无效的验证结果
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            warnings: Vec::new(),
            errors,
            suggestions: Vec::new(),
        }
    }
    
    /// 添加警告
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    /// 添加错误
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }
    
    /// 添加建议
    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }
    
    /// 合并多个验证结果
    pub fn merge(&mut self, other: ConfigValidationResult) {
        self.is_valid = self.is_valid && other.is_valid;
        self.warnings.extend(other.warnings);
        self.errors.extend(other.errors);
        self.suggestions.extend(other.suggestions);
    }
}
