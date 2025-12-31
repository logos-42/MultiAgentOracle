use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 信誉指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationMetrics {
    /// 智能体DID
    pub agent_did: String,
    /// 时间窗口开始
    pub window_start: u64,
    /// 时间窗口结束
    pub window_end: u64,
    /// 数据准确性指标
    pub accuracy: AccuracyMetrics,
    /// 响应时间指标
    pub response_time: ResponseTimeMetrics,
    /// 可用性指标
    pub availability: AvailabilityMetrics,
    /// 综合评分
    pub composite_score: f64,
    /// 趋势分析
    pub trend: TrendAnalysis,
}

/// 数据准确性指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyMetrics {
    /// 总数据点数量
    pub total_data_points: u64,
    /// 准确数据点数量
    pub accurate_data_points: u64,
    /// 平均误差率
    pub average_error_rate: f64,
    /// 最大误差率
    pub max_error_rate: f64,
    /// 误差分布
    pub error_distribution: HashMap<String, u64>,
    /// 准确性得分 (0-100)
    pub accuracy_score: f64,
}

/// 响应时间指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    /// 总请求数量
    pub total_requests: u64,
    /// 平均响应时间 (毫秒)
    pub average_response_time_ms: u64,
    /// 第95百分位响应时间
    pub p95_response_time_ms: u64,
    /// 第99百分位响应时间
    pub p99_response_time_ms: u64,
    /// 超时请求数量
    pub timeout_requests: u64,
    /// 响应时间得分 (0-100)
    pub response_time_score: f64,
}

/// 可用性指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityMetrics {
    /// 总检查次数
    pub total_checks: u64,
    /// 可用检查次数
    pub available_checks: u64,
    /// 平均可用率
    pub average_availability: f64,
    /// 最长连续可用时间 (秒)
    pub max_uptime_seconds: u64,
    /// 最长连续不可用时间 (秒)
    pub max_downtime_seconds: u64,
    /// 可用性得分 (0-100)
    pub availability_score: f64,
}

/// 趋势分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// 短期趋势 (最近7天)
    pub short_term: TrendDirection,
    /// 中期趋势 (最近30天)
    pub medium_term: TrendDirection,
    /// 长期趋势 (最近90天)
    pub long_term: TrendDirection,
    /// 趋势强度 (0-1)
    pub trend_strength: f64,
    /// 预测未来评分
    pub predicted_score: Option<f64>,
    /// 置信区间
    pub confidence_interval: (f64, f64),
}

/// 趋势方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    /// 显著上升
    StrongUp,
    /// 轻微上升
    SlightUp,
    /// 稳定
    Stable,
    /// 轻微下降
    SlightDown,
    /// 显著下降
    StrongDown,
}

impl TrendDirection {
    /// 获取趋势描述
    pub fn description(&self) -> &str {
        match self {
            Self::StrongUp => "显著上升",
            Self::SlightUp => "轻微上升",
            Self::Stable => "稳定",
            Self::SlightDown => "轻微下降",
            Self::StrongDown => "显著下降",
        }
    }
    
    /// 获取趋势符号
    pub fn symbol(&self) -> &str {
        match self {
            Self::StrongUp => "↗",
            Self::SlightUp => "↗",
            Self::Stable => "→",
            Self::SlightDown => "↘",
            Self::StrongDown => "↘",
        }
    }
}

/// 性能指标收集器
pub struct PerformanceMetrics {
    /// 按时间窗口存储的指标
    metrics_by_window: HashMap<String, ReputationMetrics>,
    /// 当前窗口指标
    current_window: Option<ReputationMetrics>,
    /// 窗口大小 (秒)
    window_size_secs: u64,
}

impl PerformanceMetrics {
    /// 创建新的性能指标收集器
    pub fn new(window_size_secs: u64) -> Self {
        Self {
            metrics_by_window: HashMap::new(),
            current_window: None,
            window_size_secs,
        }
    }
    
    /// 开始新的时间窗口
    pub fn start_new_window(&mut self, agent_did: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let window_start = now;
        let window_end = window_start + self.window_size_secs;
        
        // 保存上一个窗口
        if let Some(prev_window) = &self.current_window {
            let window_key = format!("{}-{}", prev_window.window_start, prev_window.window_end);
            self.metrics_by_window.insert(window_key, prev_window.clone());
        }
        
        // 创建新窗口
        self.current_window = Some(ReputationMetrics {
            agent_did,
            window_start,
            window_end,
            accuracy: AccuracyMetrics {
                total_data_points: 0,
                accurate_data_points: 0,
                average_error_rate: 0.0,
                max_error_rate: 0.0,
                error_distribution: HashMap::new(),
                accuracy_score: 0.0,
            },
            response_time: ResponseTimeMetrics {
                total_requests: 0,
                average_response_time_ms: 0,
                p95_response_time_ms: 0,
                p99_response_time_ms: 0,
                timeout_requests: 0,
                response_time_score: 0.0,
            },
            availability: AvailabilityMetrics {
                total_checks: 0,
                available_checks: 0,
                average_availability: 0.0,
                max_uptime_seconds: 0,
                max_downtime_seconds: 0,
                availability_score: 0.0,
            },
            composite_score: 0.0,
            trend: TrendAnalysis {
                short_term: TrendDirection::Stable,
                medium_term: TrendDirection::Stable,
                long_term: TrendDirection::Stable,
                trend_strength: 0.0,
                predicted_score: None,
                confidence_interval: (0.0, 0.0),
            },
        });
    }
    
    /// 记录数据准确性
    pub fn record_accuracy(&mut self, expected: f64, actual: f64, tolerance: f64) {
        if let Some(window) = &mut self.current_window {
            window.accuracy.total_data_points += 1;
            
            let error_rate = if expected == 0.0 {
                0.0
            } else {
                (actual - expected).abs() / expected
            };
            
            let is_accurate = error_rate <= tolerance;
            if is_accurate {
                window.accuracy.accurate_data_points += 1;
            }
            
            // 更新平均误差率
            let total = window.accuracy.total_data_points as f64;
            window.accuracy.average_error_rate = 
                (window.accuracy.average_error_rate * (total - 1.0) + error_rate) / total;
            
            // 更新最大误差率
            window.accuracy.max_error_rate = window.accuracy.max_error_rate.max(error_rate);
            
            // 记录误差分布
            let error_range = if error_rate <= 0.01 {
                "0-1%"
            } else if error_rate <= 0.05 {
                "1-5%"
            } else if error_rate <= 0.1 {
                "5-10%"
            } else if error_rate <= 0.2 {
                "10-20%"
            } else {
                ">20%"
            };
            
            *window.accuracy.error_distribution.entry(error_range.to_string())
                .or_insert(0) += 1;
            
            // 计算准确性得分
            let accuracy_rate = window.accuracy.accurate_data_points as f64 
                / window.accuracy.total_data_points as f64;
            window.accuracy.accuracy_score = accuracy_rate * 100.0;
        }
    }
    
    /// 记录响应时间
    pub fn record_response_time(&mut self, response_time_ms: u64, timeout_ms: u64) {
        if let Some(window) = &mut self.current_window {
            window.response_time.total_requests += 1;
            
            // 更新平均响应时间
            let total = window.response_time.total_requests as u64;
            window.response_time.average_response_time_ms = 
                (window.response_time.average_response_time_ms * (total - 1) + response_time_ms) / total;
            
            // 检查是否超时
            if response_time_ms > timeout_ms {
                window.response_time.timeout_requests += 1;
            }
            
            // 计算响应时间得分
            let timeout_rate = window.response_time.timeout_requests as f64 
                / window.response_time.total_requests as f64;
            window.response_time.response_time_score = (1.0 - timeout_rate) * 100.0;
        }
    }
    
    /// 记录可用性检查
    pub fn record_availability(&mut self, is_available: bool, uptime_seconds: u64) {
        if let Some(window) = &mut self.current_window {
            window.availability.total_checks += 1;
            
            if is_available {
                window.availability.available_checks += 1;
                window.availability.max_uptime_seconds = 
                    window.availability.max_uptime_seconds.max(uptime_seconds);
            } else {
                window.availability.max_downtime_seconds = 
                    window.availability.max_downtime_seconds.max(uptime_seconds);
            }
            
            // 计算可用率
            window.availability.average_availability = 
                window.availability.available_checks as f64 
                / window.availability.total_checks as f64;
            
            window.availability.availability_score = window.availability.average_availability * 100.0;
        }
    }
    
    /// 计算综合评分
    pub fn calculate_composite_score(&mut self, weights: (f64, f64, f64)) {
        if let Some(window) = &mut self.current_window {
            let (accuracy_weight, response_time_weight, availability_weight) = weights;
            let total_weight = accuracy_weight + response_time_weight + availability_weight;
            
            if total_weight > 0.0 {
                window.composite_score = (
                    window.accuracy.accuracy_score * accuracy_weight +
                    window.response_time.response_time_score * response_time_weight +
                    window.availability.availability_score * availability_weight
                ) / total_weight;
            }
        }
    }
    
    /// 获取当前窗口指标
    pub fn get_current_metrics(&self) -> Option<&ReputationMetrics> {
        self.current_window.as_ref()
    }
    
    /// 获取历史指标
    pub fn get_historical_metrics(&self, limit: usize) -> Vec<&ReputationMetrics> {
        let mut windows: Vec<&ReputationMetrics> = self.metrics_by_window.values().collect();
        windows.sort_by(|a, b| b.window_start.cmp(&a.window_start));
        windows.truncate(limit);
        windows
    }
    
    /// 分析趋势
    pub fn analyze_trends(&mut self) {
        let historical = self.get_historical_metrics(10);
        
        if let Some(window) = &mut self.current_window {
            if historical.len() >= 3 {
                // 简单趋势分析：比较最近几个窗口的评分
                let recent_scores: Vec<f64> = historical.iter()
                    .take(3)
                    .map(|m| m.composite_score)
                    .collect();
                
                let avg_recent = recent_scores.iter().sum::<f64>() / recent_scores.len() as f64;
                let current_score = window.composite_score;
                
                let change = current_score - avg_recent;
                let change_percent = change / avg_recent * 100.0;
                
                window.trend.short_term = Self::classify_trend(change_percent);
                window.trend.trend_strength = change.abs() / 100.0;
                
                // 简单预测：线性外推
                if historical.len() >= 5 {
                    let all_scores: Vec<f64> = historical.iter()
                        .map(|m| m.composite_score)
                        .collect();
                    
                    // 简单线性回归
                    let n = all_scores.len() as f64;
                    let sum_x: f64 = (0..all_scores.len()).map(|i| i as f64).sum();
                    let sum_y: f64 = all_scores.iter().sum();
                    let sum_xy: f64 = all_scores.iter().enumerate()
                        .map(|(i, &score)| i as f64 * score)
                        .sum();
                    let sum_x2: f64 = (0..all_scores.len())
                        .map(|i| (i as f64).powi(2))
                        .sum();
                    
                    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
                    let intercept = (sum_y - slope * sum_x) / n;
                    
                    // 预测下一个窗口
                    window.trend.predicted_score = Some(intercept + slope * n);
                    
                    // 计算置信区间（简化版）
                    let std_err = all_scores.iter()
                        .enumerate()
                        .map(|(i, &score)| {
                            let predicted = intercept + slope * i as f64;
                            (score - predicted).powi(2)
                        })
                        .sum::<f64>()
                        .sqrt() / (n - 2.0).sqrt();
                    
                    window.trend.confidence_interval = (
                        window.trend.predicted_score.unwrap() - 1.96 * std_err,
                        window.trend.predicted_score.unwrap() + 1.96 * std_err,
                    );
                }
            }
        }
    }
    
    /// 分类趋势
    fn classify_trend(change_percent: f64) -> TrendDirection {
        match change_percent {
            c if c >= 10.0 => TrendDirection::StrongUp,
            c if c >= 2.0 => TrendDirection::SlightUp,
            c if c <= -10.0 => TrendDirection::StrongDown,
            c if c <= -2.0 => TrendDirection::SlightDown,
            _ => TrendDirection::Stable,
        }
    }
    
    /// 生成报告
    pub fn generate_report(&self) -> Option<PerformanceReport> {
        self.current_window.as_ref().map(|metrics| {
            PerformanceReport {
                agent_did: metrics.agent_did.clone(),
                window_start: metrics.window_start,
                window_end: metrics.window_end,
                accuracy_score: metrics.accuracy.accuracy_score,
                response_time_score: metrics.response_time.response_time_score,
                availability_score: metrics.availability.availability_score,
                composite_score: metrics.composite_score,
                trend: metrics.trend.short_term.clone(),
                recommendations: self.generate_recommendations(metrics),
            }
        })
    }
    
    /// 生成改进建议
    fn generate_recommendations(&self, metrics: &ReputationMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if metrics.accuracy.accuracy_score < 90.0 {
            recommendations.push("提高数据准确性：检查数据源质量，增加数据验证".to_string());
        }
        
        if metrics.response_time.response_time_score < 80.0 {
            recommendations.push("优化响应时间：检查网络连接，优化数据处理逻辑".to_string());
        }
        
        if metrics.availability.availability_score < 95.0 {
            recommendations.push("提高可用性：确保节点稳定运行，设置自动恢复机制".to_string());
        }
        
        if metrics.composite_score < 70.0 {
            recommendations.push("综合表现需要提升：关注上述各项指标，制定改进计划".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("表现优秀，继续保持！".to_string());
        }
        
        recommendations
    }
}

/// 性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// 智能体DID
    pub agent_did: String,
    /// 时间窗口开始
    pub window_start: u64,
    /// 时间窗口结束
    pub window_end: u64,
    /// 准确性得分
    pub accuracy_score: f64,
    /// 响应时间得分
    pub response_time_score: f64,
    /// 可用性得分
    pub availability_score: f64,
    /// 综合得分
    pub composite_score: f64,
    /// 趋势
    pub trend: TrendDirection,
    /// 改进建议
    pub recommendations: Vec<String>,
}
