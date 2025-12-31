use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use log::{debug, warn};

/// HTTP客户端
pub struct HttpClient {
    client: Client,
    default_timeout_secs: u64,
}

impl HttpClient {
    /// 创建新的HTTP客户端
    pub fn new(default_timeout_secs: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(default_timeout_secs))
            .user_agent("MultiAgentOracle/0.1.0")
            .build()
            .map_err(|e| anyhow!("创建HTTP客户端失败: {}", e))?;
        
        Ok(Self {
            client,
            default_timeout_secs,
        })
    }
    
    /// 获取JSON数据
    pub async fn fetch_json(&self, url: &str, timeout_secs: Option<u64>) -> Result<Value> {
        let timeout = timeout_secs.unwrap_or(self.default_timeout_secs);
        
        debug!("HTTP请求: {}, 超时: {}s", url, timeout);
        
        let response = self.client
            .get(url)
            .timeout(Duration::from_secs(timeout))
            .send()
            .await
            .map_err(|e| anyhow!("HTTP请求失败: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP错误: {}", response.status()));
        }
        
        let json = response.json::<Value>()
            .await
            .map_err(|e| anyhow!("JSON解析失败: {}", e))?;
        
        Ok(json)
    }
    
    /// 获取JSON数据（带API密钥）
    pub async fn fetch_json_with_auth(
        &self,
        url: &str,
        api_key: &str,
        timeout_secs: Option<u64>,
    ) -> Result<Value> {
        let timeout = timeout_secs.unwrap_or(self.default_timeout_secs);
        
        debug!("HTTP请求(带认证): {}, 超时: {}s", url, timeout);
        
        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .timeout(Duration::from_secs(timeout))
            .send()
            .await
            .map_err(|e| anyhow!("HTTP请求失败: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP错误: {}", response.status()));
        }
        
        let json = response.json::<Value>()
            .await
            .map_err(|e| anyhow!("JSON解析失败: {}", e))?;
        
        Ok(json)
    }
    
    /// 获取文本数据
    pub async fn fetch_text(&self, url: &str, timeout_secs: Option<u64>) -> Result<String> {
        let timeout = timeout_secs.unwrap_or(self.default_timeout_secs);
        
        let response = self.client
            .get(url)
            .timeout(Duration::from_secs(timeout))
            .send()
            .await
            .map_err(|e| anyhow!("HTTP请求失败: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP错误: {}", response.status()));
        }
        
        let text = response.text()
            .await
            .map_err(|e| anyhow!("读取响应文本失败: {}", e))?;
        
        Ok(text)
    }
    
    /// 健康检查
    pub async fn health_check(&self, url: &str) -> bool {
        match self.client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(e) => {
                warn!("健康检查失败 {}: {}", url, e);
                false
            }
        }
    }
}

/// 常用的API端点模板
pub mod api_templates {
    /// CoinGecko加密货币价格API
    pub fn coingecko_price(symbol: &str, vs_currency: &str) -> String {
        format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
            symbol.to_lowercase(),
            vs_currency.to_lowercase()
        )
    }
    
    /// Binance加密货币价格API
    pub fn binance_price(symbol: &str) -> String {
        format!(
            "https://api.binance.com/api/v3/ticker/price?symbol={}",
            symbol.to_uppercase()
        )
    }
    
    /// Alpha Vantage股票价格API
    pub fn alpha_vantage_stock(symbol: &str, api_key: &str) -> String {
        format!(
            "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            symbol, api_key
        )
    }
    
    /// OpenWeather天气API
    pub fn openweather(location: &str, api_key: &str, metric: &str) -> String {
        let units = match metric {
            "temperature" => "metric",
            _ => "standard",
        };
        format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&units={}&appid={}",
            location, units, api_key
        )
    }
    
    /// 外汇汇率API
    pub fn exchangerate(from: &str, to: &str, api_key: &str) -> String {
        format!(
            "https://v6.exchangerate-api.com/v6/{}/pair/{}/{}",
            api_key, from, to
        )
    }
}

/// API响应解析器
pub mod response_parsers {
    use serde_json::Value;
    use anyhow::{Result, anyhow};
    
    /// 解析CoinGecko响应
    pub fn parse_coingecko_response(json: &Value, symbol: &str, vs_currency: &str) -> Result<f64> {
        let price = json
            .get(symbol.to_lowercase())
            .and_then(|obj| obj.get(vs_currency.to_lowercase()))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("无法解析CoinGecko响应"))?;
        
        Ok(price)
    }
    
    /// 解析Binance响应
    pub fn parse_binance_response(json: &Value) -> Result<f64> {
        let price_str = json
            .get("price")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("无法解析Binance响应"))?;
        
        price_str.parse::<f64>()
            .map_err(|e| anyhow!("解析价格失败: {}", e))
    }
    
    /// 解析Alpha Vantage响应
    pub fn parse_alpha_vantage_response(json: &Value) -> Result<f64> {
        let price_str = json
            .get("Global Quote")
            .and_then(|obj| obj.get("05. price"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("无法解析Alpha Vantage响应"))?;
        
        price_str.parse::<f64>()
            .map_err(|e| anyhow!("解析价格失败: {}", e))
    }
    
    /// 解析OpenWeather响应
    pub fn parse_openweather_response(json: &Value, metric: &str) -> Result<f64> {
        let value = match metric {
            "temperature" => json
                .get("main")
                .and_then(|obj| obj.get("temp"))
                .and_then(|v| v.as_f64()),
            "humidity" => json
                .get("main")
                .and_then(|obj| obj.get("humidity"))
                .and_then(|v| v.as_f64()),
            "pressure" => json
                .get("main")
                .and_then(|obj| obj.get("pressure"))
                .and_then(|v| v.as_f64()),
            _ => None,
        };
        
        value.ok_or_else(|| anyhow!("无法解析OpenWeather响应中的{}", metric))
    }
    
    /// 解析ExchangeRate API响应
    pub fn parse_exchangerate_response(json: &Value) -> Result<f64> {
        let rate = json
            .get("conversion_rate")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("无法解析ExchangeRate API响应"))?;
        
        Ok(rate)
    }
}
