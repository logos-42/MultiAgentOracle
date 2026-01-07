use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 预言机数据类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OracleDataType {
    /// 加密货币价格
    CryptoPrice {
        symbol: String,
    },
    /// 股票价格
    StockPrice {
        symbol: String,
        exchange: String,
    },
    /// 天气数据
    WeatherData {
        location: String,
        metric: String, // temperature, humidity, etc.
    },
    /// 体育赛事数据
    SportsData {
        sport: String,
        event_id: String,
    },
    /// 外汇汇率
    ForexRate {
        from: String,
        to: String,
    },
    /// 商品价格
    CommodityPrice {
        commodity: String, // gold, oil, etc.
        unit: String,
    },
    /// 自定义API数据
    CustomApi {
        endpoint: String,
        params: HashMap<String, String>,
    },
}

/// 数据值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<DataValue>),
    Object(HashMap<String, DataValue>),
}

impl From<f64> for DataValue {
    fn from(value: f64) -> Self {
        DataValue::Number(value)
    }
}

impl From<String> for DataValue {
    fn from(value: String) -> Self {
        DataValue::String(value)
    }
}

impl From<bool> for DataValue {
    fn from(value: bool) -> Self {
        DataValue::Boolean(value)
    }
}

/// 预言机数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleData {
    /// 数据类型
    pub data_type: OracleDataType,
    /// 数据值
    pub value: Value,
    /// 时间戳（Unix时间戳，秒）
    pub timestamp: u64,
    /// 置信度 0.0-1.0
    pub confidence: f64,
    /// 使用的数据源
    pub sources_used: Vec<String>,
    /// 智能体签名
    pub signature: Option<String>,
    /// 智能体DID
    pub agent_did: Option<String>,
}

impl OracleData {
    /// 创建新的预言机数据
    pub fn new(
        data_type: OracleDataType,
        value: Value,
        confidence: f64,
        sources_used: Vec<String>,
    ) -> Self {
        Self {
            data_type,
            value,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            confidence: confidence.clamp(0.0, 1.0),
            sources_used,
            signature: None,
            agent_did: None,
        }
    }
    
    /// 验证数据完整性
    pub fn validate(&self) -> bool {
        // 检查时间戳（不能是未来时间，不能太旧）
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if self.timestamp > now + 300 { // 不能是未来5分钟的时间
            return false;
        }
        
        if now - self.timestamp > 3600 { // 不能是1小时前的数据
            return false;
        }
        
        // 检查置信度
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return false;
        }
        
        // 检查数据源
        if self.sources_used.is_empty() {
            return false;
        }
        
        true
    }
    
    /// 获取数值（如果是数字类型）
    pub fn get_number(&self) -> Option<f64> {
        match &self.value {
            Value::Number(n) => n.as_f64(),
            _ => None,
        }
    }
    
    /// 获取字符串值
    pub fn get_string(&self) -> Option<&str> {
        match &self.value {
            Value::String(s) => Some(s),
            _ => None,
        }
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
