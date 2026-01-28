use serde::{Deserialize, Serialize};
use crate::oracle_agent::OracleDataType;
use std::env;

/// 数据源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// 数据源名称
    pub name: String,
    /// API端点
    pub endpoint: String,
    /// API密钥（可选）
    pub api_key: Option<String>,
    /// 超时时间（秒）
    pub timeout_secs: u64,
    /// 数据源权重（0.0-1.0），用于加权平均
    pub weight: f64,
    /// 是否启用
    pub enabled: bool,
    /// 最后使用时间
    #[serde(skip)]
    pub last_used: Option<std::time::SystemTime>,
    /// 成功率统计
    #[serde(skip)]
    pub success_rate: f64,
}

impl DataSource {
    /// 创建新的数据源
    pub fn new(name: &str, endpoint: &str, weight: f64) -> Self {
        Self {
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            api_key: None,
            timeout_secs: 10,
            weight: weight.clamp(0.0, 1.0),
            enabled: true,
            last_used: None,
            success_rate: 1.0,
        }
    }
    
    /// 设置API密钥
    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }
    
    /// 设置超时时间
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
    
    /// 更新使用统计
    pub fn update_stats(&mut self, success: bool) {
        self.last_used = Some(std::time::SystemTime::now());
        
        // 更新成功率（指数移动平均）
        let alpha = 0.1; // 平滑因子
        let success_value = if success { 1.0 } else { 0.0 };
        self.success_rate = alpha * success_value + (1.0 - alpha) * self.success_rate;
    }
    
    /// 检查数据源是否可用
    pub fn is_available(&self) -> bool {
        self.enabled && self.success_rate > 0.5
    }
}

/// 预言机智能体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAgentConfig {
    /// 智能体名称
    pub name: String,
    /// 数据源列表
    pub data_sources: Vec<DataSource>,
    /// 最小置信度阈值
    pub min_confidence: f64,
    /// 最大超时时间（秒）
    pub max_timeout_secs: u64,
    /// 初始信誉分
    pub initial_reputation: f64,
    /// 初始质押金额
    pub initial_stake: u64,
    /// 支持的数据类型
    pub supported_data_types: Vec<OracleDataType>,
    /// 缓存TTL（秒）
    pub cache_ttl_secs: u64,
    /// 是否启用自动缓存清理
    pub auto_cache_cleanup: bool,
    /// 缓存清理间隔（秒）
    pub cache_cleanup_interval_secs: u64,
}

impl Default for OracleAgentConfig {
    fn default() -> Self {
        Self {
            name: "default_oracle_agent".to_string(),
            data_sources: vec![
                // 加密货币数据源
                DataSource::new("CoinGecko", "https://api.coingecko.com/api/v3/simple/price", 0.8),
                DataSource::new("Binance", "https://api.binance.com/api/v3/ticker/price", 0.9),
                // 股票数据源（需要API密钥）
                DataSource::new("AlphaVantage", "https://www.alphavantage.co/query", 0.7)
                    .with_api_key("demo"), // 使用demo API密钥
                // 天气数据源（需要API密钥）
                DataSource::new("OpenWeather", "https://api.openweathermap.org/data/2.5/weather", 0.8)
                    .with_api_key("demo_key"),
            ],
            min_confidence: 0.7,
            max_timeout_secs: 30,
            initial_reputation: 100.0,
            initial_stake: 1000,
            supported_data_types: vec![
                OracleDataType::CryptoPrice { symbol: "BTC".to_string() },
                OracleDataType::CryptoPrice { symbol: "ETH".to_string() },
                OracleDataType::CryptoPrice { symbol: "SOL".to_string() },
                OracleDataType::StockPrice { symbol: "AAPL".to_string(), exchange: "NASDAQ".to_string() },
                OracleDataType::StockPrice { symbol: "GOOGL".to_string(), exchange: "NASDAQ".to_string() },
                OracleDataType::WeatherData { location: "Beijing".to_string(), metric: "temperature".to_string() },
                OracleDataType::WeatherData { location: "Shanghai".to_string(), metric: "temperature".to_string() },
            ],
            cache_ttl_secs: 300, // 5分钟
            auto_cache_cleanup: true,
            cache_cleanup_interval_secs: 60, // 1分钟
        }
    }
}

impl OracleAgentConfig {
    /// 创建默认配置
    pub fn default_with_name(name: &str) -> Self {
        let mut config = Self::default();
        config.name = name.to_string();
        config
    }
    
    /// 添加数据源
    pub fn add_data_source(&mut self, source: DataSource) {
        self.data_sources.push(source);
    }
    
    /// 添加支持的数据类型
    pub fn add_supported_data_type(&mut self, data_type: OracleDataType) {
        if !self.supported_data_types.contains(&data_type) {
            self.supported_data_types.push(data_type);
        }
    }
    
    /// 检查配置有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("智能体名称不能为空".to_string());
        }
        
        if self.data_sources.is_empty() {
            return Err("至少需要一个数据源".to_string());
        }
        
        if self.supported_data_types.is_empty() {
            return Err("至少需要支持一种数据类型".to_string());
        }
        
        if self.min_confidence < 0.0 || self.min_confidence > 1.0 {
            return Err("最小置信度必须在0.0到1.0之间".to_string());
        }
        
        if self.max_timeout_secs == 0 {
            return Err("超时时间必须大于0".to_string());
        }
        
        if self.initial_reputation < 0.0 {
            return Err("初始信誉分不能为负数".to_string());
        }
        
        Ok(())
    }
    
    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))
    }
    
    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        
        std::fs::write(path, content)
            .map_err(|e| format!("保存配置文件失败: {}", e))
    }
    
    /// 创建使用真实API的配置（从环境变量读取API密钥）
    pub fn with_real_apis() -> Self {
        println!("🔧 创建真实API配置...");
        
        let mut config = Self::default();
        
        // 更新数据源为真实API密钥
        for source in &mut config.data_sources {
            match source.name.as_str() {
                "AlphaVantage" => {
                    if let Ok(api_key) = env::var("ALPHA_VANTAGE_API_KEY") {
                        if !api_key.is_empty() && api_key != "demo" {
                            println!("  ✅ AlphaVantage: 使用真实API密钥");
                            source.api_key = Some(api_key);
                        }
                    }
                }
                "OpenWeather" => {
                    if let Ok(api_key) = env::var("OPENWEATHER_API_KEY") {
                        if !api_key.is_empty() && api_key != "demo_key" {
                            println!("  ✅ OpenWeather: 使用真实API密钥");
                            source.api_key = Some(api_key);
                        }
                    }
                }
                _ => {}
            }
        }
        
        config
    }
}
