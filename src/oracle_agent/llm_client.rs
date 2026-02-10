//! LLM API 客户端模块
//!
//! 支持调用 OpenAI、Claude 等 LLM API 来获取智能体响应
//!
//! 使用方法：
//!   1. 设置环境变量：
//!      export OPENAI_API_KEY=your_key
//!      export ANTHROPIC_API_KEY=your_key
//!      export DEEPSEEK_API_KEY=your_key
//!
//!   2. 创建客户端：
//!      let client = LlmClient::new(Provider::OpenAI)?;
//!
//!   3. 调用 API：
//!      let response = client.generate_response(prompt).await?;

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::env;
use reqwest::Client;
use std::time::Duration;
use log::{info, warn, debug};

/// LLM API 提供商
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProvider {
    /// OpenAI (GPT-4, GPT-3.5, etc.)
    OpenAI,
    /// Anthropic (Claude)
    Anthropic,
    /// DeepSeek (deepseek-chat, deepseek-coder, etc.)
    DeepSeek,
    /// Minimax (minimax-chat)
    Minimax,
    /// 本地 LLM (通过 HTTP API)
    Local,
}

/// LLM 客户端配置
#[derive(Debug, Clone)]
pub struct LlmClientConfig {
    /// API 提供商
    pub provider: LlmProvider,
    /// 模型名称
    pub model: String,
    /// API 密钥
    pub api_key: Option<String>,
    /// API 端点
    pub api_endpoint: String,
    /// 最大重试次数
    pub max_retries: u32,
    /// 超时时间（秒）
    pub timeout_secs: u64,
    /// 温度参数（0.0-2.0）
    pub temperature: f32,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 强制 JSON 输出模式（仅部分提供商支持，如 OpenAI、DeepSeek）
    pub response_format_json: bool,
}

impl Default for LlmClientConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model: "gpt-3.5-turbo".to_string(),
            api_key: None,
            api_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            max_retries: 3,
            timeout_secs: 30,
            temperature: 0.7,
            max_tokens: 500,
            response_format_json: false,
        }
    }
}

impl LlmClientConfig {
    /// 创建 OpenAI 配置
    pub fn openai(model: &str) -> Self {
        let mut config = Self::default();
        config.provider = LlmProvider::OpenAI;
        config.model = model.to_string();
        config.api_endpoint = "https://api.openai.com/v1/chat/completions".to_string();
        
        // 从环境变量读取 API 密钥
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            config.api_key = Some(key);
        }
        
        config
    }
    
    /// 创建 Anthropic (Claude) 配置
    pub fn anthropic(model: &str) -> Self {
        let mut config = Self::default();
        config.provider = LlmProvider::Anthropic;
        config.model = model.to_string();
        config.api_endpoint = "https://api.anthropic.com/v1/messages".to_string();
        
        // 从环境变量读取 API 密钥
        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            config.api_key = Some(key);
        }
        
        config
    }
    
    /// 创建本地 LLM 配置
    pub fn local(endpoint: &str, model: &str) -> Self {
        let mut config = Self::default();
        config.provider = LlmProvider::Local;
        config.model = model.to_string();
        config.api_endpoint = endpoint.to_string();
        config
    }

    /// 创建 DeepSeek 配置
    pub fn deepseek(model: &str) -> Self {
        let mut config = Self::default();
        config.provider = LlmProvider::DeepSeek;
        config.model = model.to_string();
        config.api_endpoint = "https://api.deepseek.com/v1/chat/completions".to_string();

        // 从环境变量读取 API 密钥
        if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
            config.api_key = Some(key);
        }

        config
    }

    /// 创建 Minimax 配置
    pub fn minimax(model: &str) -> Self {
        let mut config = Self::default();
        config.provider = LlmProvider::Minimax;
        config.model = model.to_string();
        config.api_endpoint = "https://api.minimax.chat/v1/text/chatcompletion_v2".to_string();

        // 从环境变量读取 API 密钥
        if let Ok(key) = env::var("Minimax_API_KEY") {
            config.api_key = Some(key);
        }

        config
    }

    /// 设置 API 密钥
    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }
    
    /// 设置温度
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 2.0);
        self
    }
    
    /// 设置最大 token 数
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// 启用 JSON 输出模式（强制返回 JSON 格式）
    pub fn with_json_mode(mut self) -> Self {
        self.response_format_json = true;
        self
    }
}

/// LLM 响应
#[derive(Debug, Clone, Deserialize)]
pub struct LlmResponse {
    /// 响应文本
    pub text: String,
    /// 使用的 token 数
    pub usage: Usage,
    /// 模型名称
    pub model: String,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
}

/// Token 使用情况
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    /// 输入 token 数
    pub prompt_tokens: u32,
    /// 输出 token 数
    pub completion_tokens: u32,
    /// 总 token 数
    pub total_tokens: u32,
}

/// LLM 客户端
#[derive(Clone)]
pub struct LlmClient {
    config: LlmClientConfig,
    http_client: Client,
}

impl LlmClient {
    /// 创建新的 LLM 客户端
    pub fn new(config: LlmClientConfig) -> Result<Self> {
        // 验证配置
        if config.api_key.is_none() && matches!(config.provider, LlmProvider::OpenAI | LlmProvider::Anthropic) {
            warn!("⚠️ 未配置 API 密钥，某些功能可能无法使用");
        }
        
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent("MultiAgentOracle/1.0")
            .build()
            .map_err(|e| anyhow!("创建 HTTP 客户端失败: {}", e))?;
        
        info!("🤖 创建 LLM 客户端: {:?}, 模型: {}", config.provider, config.model);
        
        Ok(Self {
            config,
            http_client,
        })
    }
    
    /// 生成响应
    pub async fn generate_response(&self, prompt: &str) -> Result<LlmResponse> {
        let start_time = std::time::Instant::now();

        debug!("发送请求到 LLM: {}...", &prompt[..prompt.len().min(100)]);

        let response_text = match self.config.provider {
            LlmProvider::OpenAI => self.call_openai(prompt).await?,
            LlmProvider::Anthropic => self.call_anthropic(prompt).await?,
            LlmProvider::DeepSeek => self.call_deepseek(prompt).await?,
            LlmProvider::Minimax => self.call_minimax(prompt).await?,
            LlmProvider::Local => self.call_local(prompt).await?,
        };

        let response_time = start_time.elapsed().as_millis() as u64;

        info!("✅ LLM 响应完成，耗时: {}ms, 长度: {} 字符", response_time, response_text.len());

        Ok(LlmResponse {
            text: response_text,
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            model: self.config.model.clone(),
            response_time_ms: response_time,
        })
    }
    
    /// 调用 OpenAI API
    async fn call_openai(&self, prompt: &str) -> Result<String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow!("未配置 OpenAI API 密钥"))?;

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });

        let response = self.http_client
            .post(&self.config.api_endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("发送 OpenAI 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI API 错误: {} - {}", status, error_text));
        }

        let json: Value = response.json().await
            .map_err(|e| anyhow!("解析 OpenAI 响应失败: {}", e))?;

        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("无法从 OpenAI 响应中提取文本"))?;

        Ok(text.to_string())
    }
    
    /// 调用 Anthropic (Claude) API
    async fn call_anthropic(&self, prompt: &str) -> Result<String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow!("未配置 Anthropic API 密钥"))?;

        let request_body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self.http_client
            .post(&self.config.api_endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("发送 Anthropic 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Anthropic API 错误: {} - {}", status, error_text));
        }

        let json: Value = response.json().await
            .map_err(|e| anyhow!("解析 Anthropic 响应失败: {}", e))?;

        let text = json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("无法从 Anthropic 响应中提取文本"))?;

        Ok(text.to_string())
    }
    
    /// 调用本地 LLM API
    async fn call_local(&self, prompt: &str) -> Result<String> {
        let request_body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });

        let response = self.http_client
            .post(&self.config.api_endpoint)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("发送本地 LLM 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("本地 LLM API 错误: {} - {}", status, error_text));
        }

        let json: Value = response.json().await
            .map_err(|e| anyhow!("解析本地 LLM 响应失败: {}", e))?;

        // 尝试多种可能的响应格式
        let text = json["response"]
            .as_str()
            .or_else(|| json["output"].as_str())
            .or_else(|| json["text"].as_str())
            .or_else(|| json["completion"].as_str())
            .ok_or_else(|| anyhow!("无法从本地 LLM 响应中提取文本"))?;

        Ok(text.to_string())
    }
    
    /// 检查 API 密钥是否配置
    pub fn has_api_key(&self) -> bool {
        self.config.api_key.is_some()
    }

    /// 调用 DeepSeek API (兼容 OpenAI 格式)
    async fn call_deepseek(&self, prompt: &str) -> Result<String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow!("未配置 DeepSeek API 密钥，请设置环境变量 DEEPSEEK_API_KEY"))?;

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });

        let response = self.http_client
            .post(&self.config.api_endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("发送 DeepSeek 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("DeepSeek API 错误: {} - {}", status, error_text));
        }

        let json: Value = response.json().await
            .map_err(|e| anyhow!("解析 DeepSeek 响应失败: {}", e))?;

        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("无法从 DeepSeek 响应中提取文本"))?;

        Ok(text.to_string())
    }

    /// 调用 Minimax API
    async fn call_minimax(&self, prompt: &str) -> Result<String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow!("未配置 Minimax API 密钥，请设置环境变量 Minimax_API_KEY"))?;

        // Minimax API 参数说明：
        // - tokens_to_generate: 最大输出token数
        // - max_tokens: 也是有效的参数名（某些模型使用）
        let max_tokens_val = std::cmp::max(self.config.max_tokens, 4000); // 确保至少4000 tokens，避免JSON截断

        let mut request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": self.config.temperature,
            "tokens_to_generate": max_tokens_val,
            "max_tokens": max_tokens_val,  // Minimax 可能使用不同的参数名
        });

        // 如果启用了 JSON 模式，添加提示（Minimax 不支持 response_format 参数）
        if self.config.response_format_json {
            // Minimax 不支持 OpenAI 风格的 response_format 参数
            // 需要在 prompt 中明确要求 JSON 格式
        }

        let response = self.http_client
            .post(&self.config.api_endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("发送 Minimax 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Minimax API 错误: {} - {}", status, error_text));
        }

        let json: Value = response.json().await
            .map_err(|e| anyhow!("解析 Minimax 响应失败: {}", e))?;

        // Minimax 响应格式: {"choices": [{"message": {"content": "..."}}], "usage": {...}}
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("无法从 Minimax 响应中提取文本"))?;

        Ok(text.to_string())
    }

    /// 获取提供商信息
    pub fn get_provider_info(&self) -> String {
        format!("{:?} ({})", self.config.provider, self.config.model)
    }
}
