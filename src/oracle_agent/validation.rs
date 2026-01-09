use crate::oracle_agent::OracleData;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// 数据验证器
#[allow(dead_code)]
pub struct DataValidator {
    /// 验证规则
    rules: HashMap<String, ValidationRule>,
    /// 历史数据记录（用于异常检测）
    history: HashMap<String, Vec<HistoricalData>>,
    /// 最大历史记录数
    max_history_size: usize,
}

/// 验证规则
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ValidationRule {
    /// 规则名称
    pub name: String,
    /// 最小值（可选）
    pub min_value: Option<f64>,
    /// 最大值（可选）
    pub max_value: Option<f64>,
    /// 允许的变化率（百分比）
    pub max_change_percent: Option<f64>,
    /// 必须包含的字段
    pub required_fields: Vec<String>,
}

/// 历史数据
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct HistoricalData {
    timestamp: u64,
    value: f64,
    confidence: f64,
}

impl DataValidator {
    /// 创建新的数据验证器
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            history: HashMap::new(),
            max_history_size: 100,
        }
    }
    
    /// 添加验证规则
    pub fn add_rule(&mut self, key: String, rule: ValidationRule) {
        self.rules.insert(key, rule);
    }
    
    /// 验证数据
    #[allow(dead_code)]
    pub fn validate(&mut self, data: &OracleData) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            confidence_adjustment: 0.0,
        };
        
        // 基本验证
        if !data.validate() {
            result.is_valid = false;
            result.errors.push("基本验证失败".to_string());
            return Ok(result);
        }
        
        // 获取数值（如果是数值数据）
        if let Some(value) = data.get_number() {
            let data_key = format!("{:?}", data.data_type);
            
            // 应用验证规则
            if let Some(rule) = self.rules.get(&data_key) {
                self.apply_rule(&rule, value, data, &mut result);
            }
            
            // 异常检测
            self.detect_anomalies(&data_key, value, data, &mut result);
            
            // 记录历史数据
            self.record_history(&data_key, value, data);
        }
        
        // 检查数据源
        if data.sources_used.is_empty() {
            result.warnings.push("没有数据源信息".to_string());
            result.confidence_adjustment -= 0.1;
        }
        
        // 检查时间戳新鲜度
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age = now - data.timestamp;
        
        if age > 300 { // 5分钟
            result.warnings.push(format!("数据较旧: {}秒前", age));
            result.confidence_adjustment -= 0.05 * (age / 300) as f64;
        }
        
        // 调整置信度
        result.confidence_adjustment = result.confidence_adjustment.clamp(-0.5, 0.2);
        
        Ok(result)
    }
    
    /// 应用验证规则
    fn apply_rule(
        &self,
        rule: &ValidationRule,
        value: f64,
        data: &OracleData,
        result: &mut ValidationResult,
    ) {
        // 检查最小值
        if let Some(min) = rule.min_value {
            if value < min {
                result.errors.push(format!("值 {} 小于最小值 {}", value, min));
                result.is_valid = false;
            }
        }
        
        // 检查最大值
        if let Some(max) = rule.max_value {
            if value > max {
                result.errors.push(format!("值 {} 大于最大值 {}", value, max));
                result.is_valid = false;
            }
        }
        
        // 检查必需字段
        if let Value::Object(obj) = &data.value {
            for field in &rule.required_fields {
                if !obj.contains_key(field) {
                    result.warnings.push(format!("缺少必需字段: {}", field));
                    result.confidence_adjustment -= 0.05;
                }
            }
        }
    }
    
    /// 检测异常
    fn detect_anomalies(
        &self,
        data_key: &str,
        value: f64,
        _data: &OracleData,
        result: &mut ValidationResult,
    ) {
        if let Some(history) = self.history.get(data_key) {
            if history.len() >= 3 {
                // 计算移动平均值
                let recent: Vec<&HistoricalData> = history
                    .iter()
                    .rev()
                    .take(5)
                    .collect();
                
                let avg: f64 = recent.iter().map(|h| h.value).sum::<f64>() / recent.len() as f64;
                let std_dev = self.calculate_std_dev(&recent, avg);
                
                // 检查是否在3个标准差范围内
                if (value - avg).abs() > 3.0 * std_dev {
                    result.warnings.push(format!(
                        "检测到异常值: {} (平均: {}, 标准差: {})",
                        value, avg, std_dev
                    ));
                    result.confidence_adjustment -= 0.2;
                }
                
                // 检查变化率
                if let Some(last) = history.last() {
                    let change_percent = ((value - last.value) / last.value).abs() * 100.0;
                    
                    if change_percent > 50.0 { // 50%的变化
                        result.warnings.push(format!(
                            "检测到大幅变化: {:.2}%",
                            change_percent
                        ));
                        result.confidence_adjustment -= 0.15;
                    }
                }
            }
        }
    }
    
    /// 记录历史数据
    fn record_history(&mut self, data_key: &str, value: f64, data: &OracleData) {
        let entry = HistoricalData {
            timestamp: data.timestamp,
            value,
            confidence: data.confidence,
        };
        
        let history = self.history.entry(data_key.to_string()).or_insert_with(Vec::new);
        history.push(entry);
        
        // 限制历史记录大小
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }
    
    /// 计算标准差
    fn calculate_std_dev(&self, data: &[&HistoricalData], mean: f64) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        
        let variance: f64 = data
            .iter()
            .map(|h| (h.value - mean).powi(2))
            .sum::<f64>() / (data.len() - 1) as f64;
        
        variance.sqrt()
    }
    
    /// 获取验证统计
    #[allow(dead_code)]
    pub fn get_stats(&self) -> ValidationStats {
        let mut stats = ValidationStats {
            total_validations: 0,
            successful_validations: 0,
            failed_validations: 0,
            warning_count: 0,
            error_count: 0,
            average_confidence_adjustment: 0.0,
            history_sizes: HashMap::new(),
        };
        
        for (key, history) in &self.history {
            stats.history_sizes.insert(key.clone(), history.len());
        }
        
        stats
    }
}

/// 验证结果
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 警告信息
    pub warnings: Vec<String>,
    /// 错误信息
    pub errors: Vec<String>,
    /// 置信度调整（-0.5到0.2）
    pub confidence_adjustment: f64,
}

impl ValidationResult {
    /// 获取调整后的置信度
    #[allow(dead_code)]
    pub fn get_adjusted_confidence(&self, original_confidence: f64) -> f64 {
        (original_confidence + self.confidence_adjustment).clamp(0.0, 1.0)
    }
    
    /// 是否有警告
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
    
    /// 是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// 验证统计
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ValidationStats {
    /// 总验证次数
    pub total_validations: usize,
    /// 成功验证次数
    pub successful_validations: usize,
    /// 失败验证次数
    pub failed_validations: usize,
    /// 警告数量
    pub warning_count: usize,
    /// 错误数量
    pub error_count: usize,
    /// 平均置信度调整
    pub average_confidence_adjustment: f64,
    /// 历史记录大小
    pub history_sizes: HashMap<String, usize>,
}

/// 预定义的验证规则
pub mod predefined_rules {
    use super::ValidationRule;
    
    /// 比特币价格验证规则
    pub fn btc_price_rule() -> ValidationRule {
        ValidationRule {
            name: "BTC价格验证".to_string(),
            min_value: Some(1000.0),    // 最低1000美元
            max_value: Some(1000000.0), // 最高100万美元
            max_change_percent: Some(50.0), // 最大50%变化
            required_fields: vec![],
        }
    }
    
    /// 以太坊价格验证规则
    pub fn eth_price_rule() -> ValidationRule {
        ValidationRule {
            name: "ETH价格验证".to_string(),
            min_value: Some(10.0),      // 最低10美元
            max_value: Some(100000.0),  // 最高10万美元
            max_change_percent: Some(50.0),
            required_fields: vec![],
        }
    }
    
    /// 股票价格验证规则
    pub fn stock_price_rule(symbol: &str) -> ValidationRule {
        let (min, max) = match symbol {
            "AAPL" => (50.0, 500.0),
            "GOOGL" => (50.0, 5000.0),
            "TSLA" => (50.0, 1000.0),
            _ => (1.0, 10000.0),
        };
        
        ValidationRule {
            name: format!("{}价格验证", symbol),
            min_value: Some(min),
            max_value: Some(max),
            max_change_percent: Some(20.0), // 股票变化较小
            required_fields: vec![],
        }
    }
    
    /// 温度验证规则
    pub fn temperature_rule() -> ValidationRule {
        ValidationRule {
            name: "温度验证".to_string(),
            min_value: Some(-100.0),    // 最低-100°C
            max_value: Some(100.0),     // 最高100°C
            max_change_percent: Some(30.0),
            required_fields: vec!["temp".to_string(), "feels_like".to_string()],
        }
    }
}
