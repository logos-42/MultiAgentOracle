use crate::oracle_agent::{OracleDataType, OracleData};
use super::config::DataSource;
use crate::oracle_agent::http_client::{HttpClient, api_templates, response_parsers};
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::time::{SystemTime, Duration};
use log::{info, warn, debug};

/// 数据采集结果
#[derive(Debug, Clone)]
pub struct DataCollectionResult {
    /// 是否成功
    pub success: bool,
    /// 采集的数据
    pub data: Option<OracleData>,
    /// 错误信息
    pub error: Option<String>,
    /// 使用的数据源
    pub sources_used: Vec<String>,
    /// 采集耗时（毫秒）
    pub collection_time_ms: u64,
}

/// 数据采集器
pub struct DataCollector {
    data_sources: Vec<DataSource>,
    last_used_sources: Vec<String>,
}

impl DataCollector {
    /// 创建新的数据采集器
    pub fn new(data_sources: Vec<DataSource>) -> Self {
        Self {
            data_sources,
            last_used_sources: Vec::new(),
        }
    }
    
    /// 采集数据
    pub async fn collect(&mut self, data_type: &OracleDataType) -> Result<OracleData> {
        let start_time = SystemTime::now();
        
        // 过滤可用的数据源
        let available_sources: Vec<DataSource> = self.data_sources
            .iter()
            .filter(|source| source.is_available())
            .cloned()
            .collect();
        
        if available_sources.is_empty() {
            return Err(anyhow::anyhow!("没有可用的数据源"));
        }
        
        info!("开始采集数据: {:?}, 可用数据源: {}", data_type, available_sources.len());
        
        // 从多个数据源并行采集
        let mut tasks = Vec::new();
        for source in &available_sources {
            let source_clone = source.clone();
            let data_type_clone = data_type.clone();
            tasks.push(tokio::spawn(async move {
                Self::fetch_from_source(&source_clone, &data_type_clone).await
            }));
        }
        
        // 等待所有任务完成
        let mut results = Vec::new();
        let mut used_sources = Vec::new();
        
        for (i, task) in tasks.into_iter().enumerate() {
            match task.await {
                Ok(Ok((data, source_name))) => {
                    results.push(data);
                    used_sources.push(source_name);
                }
                Ok(Err(e)) => {
                    warn!("数据源 {} 采集失败: {}", available_sources[i].name, e);
                }
                Err(e) => {
                    warn!("数据源 {} 任务失败: {}", available_sources[i].name, e);
                }
            }
        }
        
        if results.is_empty() {
            return Err(anyhow::anyhow!("所有数据源采集失败"));
        }
        
        // 记录使用的数据源
        self.last_used_sources = used_sources.clone();
        
        // 聚合结果
        let aggregated_data = Self::aggregate_results(results, data_type)?;
        
        let collection_time = start_time.elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        info!("数据采集完成: {:?}, 耗时: {}ms, 使用数据源: {:?}", 
            data_type, collection_time, used_sources);
        
        Ok(aggregated_data)
    }
    
    /// 从单个数据源获取数据
    async fn fetch_from_source(
        source: &DataSource,
        data_type: &OracleDataType,
    ) -> Result<(OracleData, String)> {
        debug!("从数据源 {} 采集数据: {:?}", source.name, data_type);
        
        // 创建HTTP客户端
        let http_client = HttpClient::new(source.timeout_secs)
            .map_err(|e| anyhow!("创建HTTP客户端失败: {}", e))?;
        
        // 根据数据类型构建请求
        let (url, api_key) = Self::build_request_url(source, data_type);
        
        // 发送HTTP请求
        let json_response = if let Some(key) = api_key {
            http_client.fetch_json_with_auth(&url, &key, Some(source.timeout_secs)).await
        } else {
            http_client.fetch_json(&url, Some(source.timeout_secs)).await
        };
        
        match json_response {
            Ok(json) => {
                // 解析响应
                let (value, confidence) = Self::parse_response(&json, data_type, &source.name)?;
                
                let data = OracleData::new(
                    data_type.clone(),
                    value,
                    confidence,
                    vec![source.name.clone()],
                );
                
                debug!("数据源 {} 采集成功", source.name);
                Ok((data, source.name.clone()))
            }
            Err(e) => {
                warn!("数据源 {} 采集失败: {}", source.name, e);
                Err(anyhow!("数据源 {} 采集失败: {}", source.name, e))
            }
        }
    }
    
    /// 聚合多个数据源的结果
    fn aggregate_results(
        results: Vec<OracleData>,
        data_type: &OracleDataType,
    ) -> Result<OracleData> {
        if results.is_empty() {
            return Err(anyhow::anyhow!("没有可聚合的结果"));
        }
        
        // 如果只有一个结果，直接返回
        if results.len() == 1 {
            return Ok(results[0].clone());
        }
        
        // 提取数值数据
        let numeric_values: Vec<f64> = results
            .iter()
            .filter_map(|data| data.get_number())
            .collect();
        
        if numeric_values.is_empty() {
            // 对于非数值数据，返回第一个结果
            return Ok(results[0].clone());
        }
        
        // 计算加权平均值（使用置信度作为权重）
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        let mut sources_used = Vec::new();
        
        for data in &results {
            if let Some(value) = data.get_number() {
                let weight = data.confidence;
                weighted_sum += value * weight;
                total_weight += weight;
                sources_used.extend(data.sources_used.clone());
            }
        }
        
        if total_weight == 0.0 {
            return Err(anyhow::anyhow!("总权重为0"));
        }
        
        let average_value = weighted_sum / total_weight;
        
        // 计算总体置信度（平均值）
        let avg_confidence = results.iter().map(|d| d.confidence).sum::<f64>() / results.len() as f64;
        
        // 去重数据源
        sources_used.sort();
        sources_used.dedup();
        
        Ok(OracleData::new(
            data_type.clone(),
            Value::Number(serde_json::Number::from_f64(average_value).unwrap()),
            avg_confidence,
            sources_used,
        ))
    }
    
    /// 构建请求URL
    fn build_request_url(source: &DataSource, data_type: &OracleDataType) -> (String, Option<String>) {
        match data_type {
            OracleDataType::CryptoPrice { symbol } => {
                // 根据数据源名称选择API
                if source.name.to_lowercase().contains("coingecko") {
                    let url = api_templates::coingecko_price(symbol, "usd");
                    (url, source.api_key.clone())
                } else if source.name.to_lowercase().contains("binance") {
                    let url = api_templates::binance_price(symbol);
                    (url, source.api_key.clone())
                } else {
                    // 使用自定义端点
                    let url = source.endpoint.replace("{symbol}", symbol);
                    (url, source.api_key.clone())
                }
            }
            OracleDataType::StockPrice { symbol, exchange: _ } => {
                if source.name.to_lowercase().contains("alphavantage") {
                    if let Some(api_key) = &source.api_key {
                        let url = api_templates::alpha_vantage_stock(symbol, api_key);
                        (url, None)
                    } else {
                        (source.endpoint.clone(), source.api_key.clone())
                    }
                } else {
                    (source.endpoint.clone(), source.api_key.clone())
                }
            }
            OracleDataType::WeatherData { location, metric } => {
                if source.name.to_lowercase().contains("openweather") {
                    if let Some(api_key) = &source.api_key {
                        let url = api_templates::openweather(location, api_key, metric);
                        (url, None)
                    } else {
                        (source.endpoint.clone(), source.api_key.clone())
                    }
                } else {
                    (source.endpoint.clone(), source.api_key.clone())
                }
            }
            OracleDataType::ForexRate { from, to } => {
                if source.name.to_lowercase().contains("exchangerate") {
                    if let Some(api_key) = &source.api_key {
                        let url = api_templates::exchangerate(from, to, api_key);
                        (url, None)
                    } else {
                        (source.endpoint.clone(), source.api_key.clone())
                    }
                } else {
                    (source.endpoint.clone(), source.api_key.clone())
                }
            }
            OracleDataType::CustomApi { endpoint, params } => {
                // 构建带参数的URL
                let mut url = endpoint.clone();
                if !params.is_empty() {
                    url.push('?');
                    let params_str: Vec<String> = params.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    url.push_str(&params_str.join("&"));
                }
                (url, source.api_key.clone())
            }
            _ => (source.endpoint.clone(), source.api_key.clone()),
        }
    }
    
    /// 解析API响应
    fn parse_response(
        json: &Value,
        data_type: &OracleDataType,
        source_name: &str,
    ) -> Result<(Value, f64)> {
        let (value, confidence) = match data_type {
            OracleDataType::CryptoPrice { symbol } => {
                if source_name.to_lowercase().contains("coingecko") {
                    let price = response_parsers::parse_coingecko_response(json, symbol, "usd")?;
                    (Value::Number(serde_json::Number::from_f64(price).unwrap()), 0.9)
                } else if source_name.to_lowercase().contains("binance") {
                    let price = response_parsers::parse_binance_response(json)?;
                    (Value::Number(serde_json::Number::from_f64(price).unwrap()), 0.95)
                } else {
                    // 通用解析：尝试从JSON中提取数字
                    if let Some(num) = json.as_f64() {
                        (Value::Number(serde_json::Number::from_f64(num).unwrap()), 0.8)
                    } else if let Some(num) = json.as_u64() {
                        (Value::Number(serde_json::Number::from(num)), 0.8)
                    } else {
                        return Err(anyhow!("无法解析加密货币价格响应"));
                    }
                }
            }
            OracleDataType::StockPrice { symbol: _, exchange: _ } => {
                if source_name.to_lowercase().contains("alphavantage") {
                    let price = response_parsers::parse_alpha_vantage_response(json)?;
                    (Value::Number(serde_json::Number::from_f64(price).unwrap()), 0.85)
                } else {
                    // 通用解析
                    if let Some(num) = json.as_f64() {
                        (Value::Number(serde_json::Number::from_f64(num).unwrap()), 0.8)
                    } else {
                        return Err(anyhow!("无法解析股票价格响应"));
                    }
                }
            }
            OracleDataType::WeatherData { location: _, metric } => {
                if source_name.to_lowercase().contains("openweather") {
                    let value = response_parsers::parse_openweather_response(json, metric)?;
                    (Value::Number(serde_json::Number::from_f64(value).unwrap()), 0.9)
                } else {
                    // 通用解析
                    if let Some(num) = json.as_f64() {
                        (Value::Number(serde_json::Number::from_f64(num).unwrap()), 0.8)
                    } else {
                        return Err(anyhow!("无法解析天气数据响应"));
                    }
                }
            }
            OracleDataType::ForexRate { from: _, to: _ } => {
                if source_name.to_lowercase().contains("exchangerate") {
                    let rate = response_parsers::parse_exchangerate_response(json)?;
                    (Value::Number(serde_json::Number::from_f64(rate).unwrap()), 0.95)
                } else {
                    // 通用解析
                    if let Some(num) = json.as_f64() {
                        (Value::Number(serde_json::Number::from_f64(num).unwrap()), 0.8)
                    } else {
                        return Err(anyhow!("无法解析汇率响应"));
                    }
                }
            }
            _ => {
                // 对于其他数据类型，返回原始JSON
                (json.clone(), 0.7)
            }
        };
        
        Ok((value, confidence))
    }
    
    /// 获取最后使用的数据源
    pub fn get_last_used_sources(&self) -> Vec<String> {
        self.last_used_sources.clone()
    }
    
    /// 更新数据源统计
    pub fn update_source_stats(&mut self, source_name: &str, success: bool) {
        if let Some(source) = self.data_sources.iter_mut().find(|s| s.name == source_name) {
            source.update_stats(success);
        }
    }
}
