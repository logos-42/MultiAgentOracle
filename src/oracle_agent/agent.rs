use crate::oracle_agent::{
    OracleAgentConfig, OracleDataType, OracleData, DataSource, DataCollectionResult,
};
use crate::diap::{DiapIdentityManager, DiapConfig, AgentIdentity, DiapError};
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use log::{info, warn, error};

/// 预言机智能体
pub struct OracleAgent {
    /// 配置
    config: OracleAgentConfig,
    /// 当前信誉分
    reputation_score: f64,
    /// 质押金额
    staked_amount: u64,
    /// 数据缓存
    data_cache: HashMap<String, (OracleData, u64)>, // (数据, 过期时间)
    /// 数据采集器
    data_collector: DataCollector,
    /// 智能体DID
    agent_did: Option<String>,
    /// 智能体私钥（用于签名）
    private_key: Option<Vec<u8>>,
    /// DIAP身份管理器
    diap_identity_manager: Option<Arc<DiapIdentityManager>>,
    /// 当前DIAP身份
    current_diap_identity: Option<AgentIdentity>,
    /// DIAP配置
    diap_config: Option<DiapConfig>,
}

impl OracleAgent {
    /// 创建新的预言机智能体
    pub fn new(config: OracleAgentConfig) -> Result<Self> {
        info!("🚀 创建预言机智能体: {}", config.name);
        
        let data_collector = DataCollector::new(config.data_sources.clone());
        
        info!("✅ 预言机智能体创建成功: {}", config.name);
        info!("   支持的数据类型: {} 种", config.supported_data_types.len());
        info!("   数据源数量: {} 个", config.data_sources.len());
        info!("   初始信誉分: {}", config.reputation_score);
        info!("   质押金额: {}", config.staked_amount);
        
        Ok(OracleAgent {
            config,
            reputation_score: 100.0, // 初始信誉分
            staked_amount: 0,
            data_cache: HashMap::new(),
            data_collector,
            agent_did: None,
            private_key: None,
            diap_identity_manager: None,
            current_diap_identity: None,
            diap_config: None,
        })
    }
    
    /// 初始化DIAP身份系统
    pub async fn init_diap_identity(&mut self, diap_config: Option<DiapConfig>) -> Result<()> {
        info!("🔄 初始化DIAP身份系统");
        
        let config = diap_config.unwrap_or_else(|| {
            let mut default_config = DiapConfig::default();
            default_config.identity.name = self.config.name.clone();
            default_config.identity.description = Some(format!("Oracle Agent: {}", self.config.name));
            default_config
        });
        
        // 保存配置
        self.diap_config = Some(config.clone());
        
        // 创建DIAP身份管理器
        match DiapIdentityManager::new(config).await {
            Ok(manager) => {
                let manager_arc = Arc::new(manager);
                self.diap_identity_manager = Some(manager_arc.clone());
                
                // 自动注册身份
                if config.identity.auto_register {
                    match self.register_diap_identity().await {
                        Ok(identity) => {
                            info!("✅ DIAP身份注册成功: {} ({})", identity.name, identity.id);
                            self.current_diap_identity = Some(identity);
                        }
                        Err(e) => {
                            warn!("⚠️ DIAP身份自动注册失败: {}, 将以匿名模式运行", e);
                        }
                    }
                }
                
                info!("✅ DIAP身份系统初始化完成");
                Ok(())
            }
            Err(e) => {
                error!("❌ DIAP身份系统初始化失败: {}", e);
                Err(anyhow!("DIAP身份系统初始化失败: {}", e))
            }
        }
    }
    
    /// 注册DIAP身份
    pub async fn register_diap_identity(&mut self) -> Result<AgentIdentity, DiapError> {
        info!("📝 注册DIAP身份");
        
        let manager = self.diap_identity_manager.as_ref()
            .ok_or_else(|| DiapError::RegistrationFailed("DIAP身份管理器未初始化".to_string()))?;
        
        let identity_name = format!("oracle-agent-{}", self.config.name);
        let description = Some(format!("Multi-Agent Oracle System Agent: {}", self.config.name));
        
        manager.register_identity(&identity_name, description.as_deref()).await
    }
    
    /// 设置DIAP身份
    pub fn set_diap_identity(&mut self, did: String, private_key: Vec<u8>) {
        self.agent_did = Some(did);
        self.private_key = Some(private_key);
        info!("🔐 设置DIAP身份: {}", did);
    }
    
    /// 获取智能体DID
    pub fn get_did(&self) -> Option<&str> {
        self.agent_did.as_deref()
    }
    
    /// 获取当前信誉分
    pub fn get_reputation_score(&self) -> f64 {
        self.reputation_score
    }
    
    /// 更新信誉分
    pub fn update_reputation(&mut self, delta: f64) {
        let old_score = self.reputation_score;
        let new_score = self.reputation_score + delta;
        self.reputation_score = new_score.max(0.0).min(1000.0); // 限制在0-1000之间
        
        info!("📊 信誉分更新: {:.2} -> {:.2} (Δ: {:.2})", 
            old_score, self.reputation_score, delta);
    }
    
    /// 获取质押金额
    pub fn get_staked_amount(&self) -> u64 {
        self.staked_amount
    }
    
    /// 增加质押
    pub fn stake(&mut self, amount: u64) {
        self.staked_amount += amount;
        info!("💰 增加质押: {} -> {}", self.staked_amount - amount, self.staked_amount);
    }
    
    /// 减少质押
    pub fn unstake(&mut self, amount: u64) -> Result<()> {
        if amount > self.staked_amount {
            return Err(anyhow!("质押金额不足"));
        }
        self.staked_amount -= amount;
        info!("💰 减少质押: {} -> {}", self.staked_amount + amount, self.staked_amount);
        Ok(())
    }
    
    /// 获取支持的数据类型
    pub fn get_supported_data_types(&self) -> &Vec<OracleDataType> {
        &self.config.supported_data_types
    }
    
    /// 检查是否支持特定数据类型
    pub fn supports_data_type(&self, data_type: &OracleDataType) -> bool {
        self.config.supported_data_types.iter().any(|dt| dt == data_type)
    }
    
    /// 采集数据
    pub async fn collect_data(&self, data_type: &OracleDataType) -> Result<DataCollectionResult> {
        if !self.supports_data_type(data_type) {
            return Ok(DataCollectionResult {
                success: false,
                data: None,
                error: Some(format!("不支持的数据类型: {:?}", data_type)),
                sources_used: vec![],
                collection_time_ms: 0,
            });
        }
        
        // 检查缓存
        let cache_key = format!("{:?}", data_type);
        if let Some(cached) = self.get_cached_data(&cache_key) {
            info!("📦 使用缓存数据: {}", cache_key);
            return Ok(DataCollectionResult {
                success: true,
                data: Some(cached.clone()),
                error: None,
                sources_used: vec!["cache".to_string()],
                collection_time_ms: 0,
            });
        }
        
        // 从数据源采集
        let start_time = SystemTime::now();
        let result = self.data_collector.collect(data_type).await;
        let collection_time = start_time.elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        match result {
            Ok(data) => {
                // 签名数据
                let signed_data = self.sign_data(data)?;
                
                // 缓存数据
                // 注意：这里不能直接修改self，所以缓存需要在外部处理
                
                Ok(DataCollectionResult {
                    success: true,
                    data: Some(signed_data),
                    error: None,
                    sources_used: self.data_collector.get_last_used_sources(),
                    collection_time_ms: collection_time,
                })
            }
            Err(e) => {
                error!("数据采集失败: {}", e);
                Ok(DataCollectionResult {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                    sources_used: vec![],
                    collection_time_ms: collection_time,
                })
            }
        }
    }
    
    /// 从缓存获取数据
    pub fn get_cached_data(&self, key: &str) -> Option<&OracleData> {
        self.data_cache.get(key).and_then(|(data, expiry)| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if now < *expiry {
                Some(data)
            } else {
                None
            }
        })
    }
    
    /// 缓存数据（内部使用）
    pub(crate) fn cache_data_internal(&mut self, key: String, data: OracleData, ttl_secs: u64) {
        let expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + ttl_secs;
        
        self.data_cache.insert(key, (data, expiry));
        info!("💾 缓存数据: {} (TTL: {}s)", key, ttl_secs);
    }
    
    /// 清理过期缓存
    pub fn cleanup_cache(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let before = self.data_cache.len();
        self.data_cache.retain(|_, (_, expiry)| now < *expiry);
        let after = self.data_cache.len();
        
        if before > after {
            info!("🧹 清理缓存: {} -> {} 个条目", before, after);
        }
    }
    
    /// 签名数据
    fn sign_data(&self, mut data: OracleData) -> Result<OracleData> {
        if let (Some(did), Some(private_key)) = (&self.agent_did, &self.private_key) {
            // 这里应该使用实际的签名算法
            // 简化版本：使用base64编码的伪签名
            let signature = base64::encode(format!("{}-{:?}-{}", 
                did, data.data_type, data.timestamp));
            
            data.agent_did = Some(did.clone());
            data.signature = Some(signature);
        }
        
        Ok(data)
    }
    
    /// 获取智能体信息
    pub fn get_info(&self) -> OracleAgentInfo {
        OracleAgentInfo {
            name: self.config.name.clone(),
            did: self.agent_did.clone().unwrap_or_default(),
            reputation_score: self.reputation_score,
            staked_amount: self.staked_amount,
            supported_data_types: self.config.supported_data_types.clone(),
            data_source_count: self.config.data_sources.len(),
            cache_size: self.data_cache.len(),
        }
    }
    
    /// 获取当前DIAP身份
    pub async fn get_current_diap_identity(&self) -> Option<AgentIdentity> {
        if let Some(manager) = &self.diap_identity_manager {
            manager.get_current_identity().await
        } else {
            None
        }
    }
    
    /// 验证DIAP身份
    pub async fn verify_diap_identity(&self, identity_id: &str, proof: Option<&str>) -> Result<bool, DiapError> {
        let manager = self.diap_identity_manager.as_ref()
            .ok_or_else(|| DiapError::AuthenticationFailed("DIAP身份管理器未初始化".to_string()))?;
        
        let auth_result = manager.verify_identity(identity_id, proof).await?;
        Ok(auth_result.authenticated)
    }
    
    /// 获取DIAP身份状态
    pub async fn get_diap_identity_status(&self) -> String {
        match &self.current_diap_identity {
            Some(identity) => {
                format!("已注册: {} ({}) - 状态: {:?}", 
                    identity.name, identity.id, identity.status)
            }
            None => {
                if self.diap_identity_manager.is_some() {
                    "已初始化但未注册身份".to_string()
                } else {
                    "未初始化DIAP身份系统".to_string()
                }
            }
        }
    }
    
    /// 使用DIAP身份签名数据
    pub async fn sign_data_with_diap(&self, data: &[u8]) -> Result<String, DiapError> {
        let identity = self.current_diap_identity.as_ref()
            .ok_or_else(|| DiapError::AuthenticationFailed("当前无DIAP身份".to_string()))?;
        
        // 这里应该使用DIAP SDK进行签名
        // 暂时使用简化版本
        let signature = format!("{}-{:x}", identity.id, md5::compute(data));
        Ok(signature)
    }
    
    /// 验证DIAP身份签名
    pub async fn verify_diap_signature(&self, data: &[u8], signature: &str, identity_id: &str) -> Result<bool, DiapError> {
        let manager = self.diap_identity_manager.as_ref()
            .ok_or_else(|| DiapError::AuthenticationFailed("DIAP身份管理器未初始化".to_string()))?;
        
        // 验证身份
        let auth_result = manager.verify_identity(identity_id, None).await?;
        if !auth_result.authenticated {
            return Ok(false);
        }
        
        // 验证签名（简化版本）
        let expected_signature = format!("{}-{:x}", identity_id, md5::compute(data));
        Ok(signature == expected_signature)
    }
}

/// 智能体信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAgentInfo {
    pub name: String,
    pub did: String,
    pub reputation_score: f64,
    pub staked_amount: u64,
    pub supported_data_types: Vec<OracleDataType>,
    pub data_source_count: usize,
    pub cache_size: usize,
}

/// 数据采集器
pub struct DataCollector {
    data_sources: Vec<DataSource>,
    last_used_sources: Vec<String>,
}

impl DataCollector {
    pub fn new(data_sources: Vec<DataSource>) -> Self {
        Self {
            data_sources,
            last_used_sources: Vec::new(),
        }
    }
    
    pub async fn collect(&mut self, data_type: &OracleDataType) -> Result<OracleData> {
        // 简化实现：模拟数据采集
        // 实际实现应该从多个数据源采集并验证
        
        self.last_used_sources = vec!["mock_source".to_string()];
        
        let value = match data_type {
            OracleDataType::CryptoPrice { symbol } => {
                Value::Number((1000 + rand::random::<u16>() % 1000).into())
            }
            OracleDataType::StockPrice { symbol, exchange } => {
                Value::Number((50 + rand::random::<u16>() % 100).into())
            }
            OracleDataType::WeatherData { location, metric } => {
                Value::Number((20 + rand::random::<u8>() % 20).into())
            }
            _ => Value::String("mock_data".to_string()),
        };
        
        Ok(OracleData {
            data_type: data_type.clone(),
            value,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            confidence: 0.9,
            sources_used: self.last_used_sources.clone(),
            signature: None,
            agent_did: None,
        })
    }
    
    pub fn get_last_used_sources(&self) -> Vec<String> {
        self.last_used_sources.clone()
    }
}
