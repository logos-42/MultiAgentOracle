//! DIAP身份管理器
//!
//! 管理智能体的DIAP身份，包括注册、验证、证明生成等功能。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use super::config::{DiapConfig, IdentityType};
use super::{DiapError, AuthResult, IdentityStatus, IdentityPermissions};

/// 智能体身份信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentIdentity {
    /// 身份ID
    pub id: String,
    
    /// 身份名称
    pub name: String,
    
    /// 身份描述
    pub description: Option<String>,
    
    /// 身份类型
    pub identity_type: IdentityType,
    
    /// 公钥
    pub public_key: String,
    
    /// 身份证明哈希
    pub proof_hash: Option<String>,
    
    /// 身份状态
    pub status: IdentityStatus,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    
    /// 身份权限
    pub permissions: IdentityPermissions,
    
    /// 元数据
    pub metadata: serde_json::Value,
}

/// DIAP身份管理器
pub struct DiapIdentityManager {
    /// 配置
    config: DiapConfig,
    
    /// 当前身份
    current_identity: Arc<RwLock<Option<AgentIdentity>>>,
    
    /// 已知身份缓存
    known_identities: Arc<RwLock<HashMap<String, AgentIdentity>>>,
    
    /// DIAP SDK管理器
    #[allow(dead_code)]
    diap_manager: Option<diap_rs_sdk::UniversalNoirManager>,
}

impl DiapIdentityManager {
    /// 创建新的身份管理器
    pub async fn new(config: DiapConfig) -> Result<Self, DiapError> {
        // 验证配置
        config.validate()?;
        
        // 初始化DIAP SDK管理器
        let diap_manager = if config.proof.enable_zkp {
            match diap_rs_sdk::UniversalNoirManager::new().await {
                Ok(manager) => {
                    log::info!("DIAP Noir管理器初始化成功");
                    Some(manager)
                }
                Err(e) => {
                    log::warn!("DIAP Noir管理器初始化失败: {}, 将使用简化模式", e);
                    None
                }
            }
        } else {
            None
        };
        
        // 创建管理器实例
        let manager = Self {
            config,
            current_identity: Arc::new(RwLock::new(None)),
            known_identities: Arc::new(RwLock::new(HashMap::new())),
            diap_manager,
        };
        
        // 加载现有身份
        manager.load_identities().await?;
        
        Ok(manager)
    }
    
    /// 注册新身份
    pub async fn register_identity(&self, name: &str, description: Option<&str>) -> Result<AgentIdentity, DiapError> {
        log::info!("开始注册新身份: {}", name);
        
        // 检查是否已存在同名身份
        {
            let identities = self.known_identities.read().await;
            for identity in identities.values() {
                if identity.name == name && identity.status != IdentityStatus::Revoked {
                    return Err(DiapError::RegistrationFailed(format!("身份名称 '{}' 已存在", name)));
                }
            }
        }
        
        // 生成身份ID
        let identity_id = self.config.get_identity_id();
        
        // 生成密钥对
        use ed25519_dalek::{SigningKey, SecretKey};
        use rand::rngs::OsRng;
        use rand::RngCore;
        
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        csprng.fill_bytes(&mut secret_bytes);
        
        let secret_key = SecretKey::from(secret_bytes);
        let signing_key = SigningKey::from(&secret_key);
        let public_key = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
        
        // 创建身份证明
        let proof_hash = if self.config.proof.enable_zkp {
            match self.generate_identity_proof(&identity_id, &public_key).await {
                Ok(hash) => {
                    log::info!("身份证明生成成功: {}", hash);
                    Some(hash)
                }
                Err(e) => {
                    log::warn!("身份证明生成失败: {}, 将继续使用无证明身份", e);
                    None
                }
            }
        } else {
            None
        };
        
        // 创建身份对象
        let now = Utc::now();
        let identity = AgentIdentity {
            id: identity_id.clone(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            identity_type: self.config.identity.identity_type,
            public_key,
            proof_hash: proof_hash.clone(),
            status: IdentityStatus::Registered,
            created_at: now,
            updated_at: now,
            expires_at: self.config.identity.expires_in.map(|seconds| now + chrono::Duration::seconds(seconds as i64)),
            permissions: self.config.identity.permissions.clone(),
            metadata: serde_json::json!({
                "registered_with_proof": proof_hash.is_some(),
                "config_version": "0.2.11",
                "platform": std::env::consts::OS,
            }),
        };
        
        // 保存身份
        self.save_identity(&identity).await?;
        
        // 更新缓存
        {
            let mut identities = self.known_identities.write().await;
            identities.insert(identity_id.clone(), identity.clone());
        }
        
        // 设置为当前身份
        {
            let mut current = self.current_identity.write().await;
            *current = Some(identity.clone());
        }
        
        log::info!("身份注册成功: {} ({})", name, identity_id);
        
        Ok(identity)
    }
    
    /// 验证身份
    pub async fn verify_identity(&self, identity_id: &str, proof: Option<&str>) -> Result<AuthResult, DiapError> {
        log::debug!("验证身份: {}", identity_id);
        
        // 获取身份信息
        let identity = {
            let identities = self.known_identities.read().await;
            identities.get(identity_id).cloned()
        };
        
        let identity = match identity {
            Some(id) => id,
            None => return Err(DiapError::AuthenticationFailed(format!("身份不存在: {}", identity_id))),
        };
        
        // 检查身份状态
        match identity.status {
            IdentityStatus::Revoked => return Err(DiapError::AuthenticationFailed("身份已被撤销".to_string())),
            IdentityStatus::Expired => return Err(DiapError::AuthenticationFailed("身份已过期".to_string())),
            _ => {}
        }
        
        // 检查过期时间
        if let Some(expires_at) = identity.expires_at {
            if Utc::now() > expires_at {
                return Err(DiapError::AuthenticationFailed("身份已过期".to_string()));
            }
        }
        
        // 验证证明（如果提供了证明）
        let proof_valid = if let Some(provided_proof) = proof {
            if let Some(stored_proof) = &identity.proof_hash {
                // 验证证明哈希
                provided_proof == stored_proof
            } else {
                // 身份没有存储证明，无法验证
                false
            }
        } else {
            // 没有提供证明，跳过证明验证
            true
        };
        
        // 创建验证结果
        let auth_result = AuthResult {
            authenticated: proof_valid,
            identity_id: identity.id.clone(),
            proof_hash: identity.proof_hash.clone(),
            timestamp: Utc::now().timestamp(),
            metadata: serde_json::json!({
                "identity_name": identity.name,
                "identity_type": format!("{:?}", identity.identity_type),
                "proof_verified": proof_valid,
                "permissions": identity.permissions,
            }),
        };
        
        if auth_result.authenticated {
            log::info!("身份验证成功: {}", identity_id);
        } else {
            log::warn!("身份验证失败: {}", identity_id);
        }
        
        Ok(auth_result)
    }
    
    /// 获取当前身份
    pub async fn get_current_identity(&self) -> Option<AgentIdentity> {
        let current = self.current_identity.read().await;
        current.clone()
    }
    
    /// 设置当前身份
    pub async fn set_current_identity(&self, identity_id: &str) -> Result<(), DiapError> {
        let identity = {
            let identities = self.known_identities.read().await;
            identities.get(identity_id).cloned()
        };
        
        match identity {
            Some(id) => {
                let mut current = self.current_identity.write().await;
                *current = Some(id);
                log::info!("当前身份已设置为: {}", identity_id);
                Ok(())
            }
            None => Err(DiapError::AuthenticationFailed(format!("身份不存在: {}", identity_id))),
        }
    }
    
    /// 获取所有已知身份
    pub async fn get_all_identities(&self) -> Vec<AgentIdentity> {
        let identities = self.known_identities.read().await;
        identities.values().cloned().collect()
    }
    
    /// 生成身份证明
    async fn generate_identity_proof(&self, identity_id: &str, public_key: &str) -> Result<String, DiapError> {
        // 这里应该调用DIAP SDK生成零知识证明
        // 由于DIAP SDK的具体API需要进一步研究，这里先实现一个简化版本
        
        // 创建证明数据
        let proof_data = serde_json::json!({
            "identity_id": identity_id,
            "public_key": public_key,
            "timestamp": Utc::now().timestamp(),
            "nonce": rand::random::<u64>(),
        });
        
        // 计算证明哈希
        let proof_string = serde_json::to_string(&proof_data)
            .map_err(|e| DiapError::ProofGenerationFailed(format!("序列化证明数据失败: {}", e)))?;
        
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(proof_string.as_bytes());
        let hash = hasher.finalize();
        let proof_hash = hex::encode(hash);
        
        Ok(proof_hash)
    }
    
    /// 保存身份到存储
    async fn save_identity(&self, identity: &AgentIdentity) -> Result<(), DiapError> {
        // 确保存储目录存在
        std::fs::create_dir_all(&self.config.storage.identity_store_path)
            .map_err(|e| DiapError::InternalError(format!("创建存储目录失败: {}", e)))?;
        
        // 构建文件路径
        let file_path = self.config.storage.identity_store_path.join(format!("{}.json", identity.id));
        
        // 序列化身份数据
        let identity_json = serde_json::to_string_pretty(identity)
            .map_err(|e| DiapError::InternalError(format!("序列化身份数据失败: {}", e)))?;
        
        // 加密数据（如果启用）
        let data_to_write = if self.config.storage.enable_encryption {
            // 这里应该实现加密逻辑
            // 暂时先不加密
            identity_json
        } else {
            identity_json
        };
        
        // 写入文件
        std::fs::write(&file_path, data_to_write)
            .map_err(|e| DiapError::InternalError(format!("写入身份文件失败: {}", e)))?;
        
        log::debug!("身份已保存到: {:?}", file_path);
        
        Ok(())
    }
    
    /// 从存储加载身份
    async fn load_identities(&self) -> Result<(), DiapError> {
        let identity_dir = &self.config.storage.identity_store_path;
        
        // 检查目录是否存在
        if !identity_dir.exists() {
            log::info!("身份存储目录不存在，将创建新目录: {:?}", identity_dir);
            return Ok(());
        }
        
        // 读取目录中的所有身份文件
        let entries = std::fs::read_dir(identity_dir)
            .map_err(|e| DiapError::InternalError(format!("读取身份目录失败: {}", e)))?;
        
        let mut loaded_count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| DiapError::InternalError(format!("读取目录条目失败: {}", e)))?;
            let path = entry.path();
            
            // 只处理.json文件
            if path.extension().map(|ext| ext == "json").unwrap_or(false) {
                match self.load_identity_from_file(&path).await {
                    Ok(identity) => {
                        let mut identities = self.known_identities.write().await;
                        identities.insert(identity.id.clone(), identity);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        log::warn!("加载身份文件失败 {:?}: {}", path, e);
                    }
                }
            }
        }
        
        log::info!("已加载 {} 个身份", loaded_count);
        
        Ok(())
    }
    
    /// 从文件加载单个身份
    async fn load_identity_from_file(&self, path: &PathBuf) -> Result<AgentIdentity, DiapError> {
        // 读取文件内容
        let content = std::fs::read_to_string(path)
            .map_err(|e| DiapError::InternalError(format!("读取身份文件失败: {}", e)))?;
        
        // 解密数据（如果启用加密）
        let decrypted_content = if self.config.storage.enable_encryption {
            // 这里应该实现解密逻辑
            // 暂时假设数据未加密
            content
        } else {
            content
        };
        
        // 解析身份数据
        let identity: AgentIdentity = serde_json::from_str(&decrypted_content)
            .map_err(|e| DiapError::InternalError(format!("解析身份数据失败: {}", e)))?;
        
        Ok(identity)
    }
    
    /// 更新身份状态
    pub async fn update_identity_status(&self, identity_id: &str, status: IdentityStatus) -> Result<(), DiapError> {
        let mut identities = self.known_identities.write().await;
        
        if let Some(identity) = identities.get_mut(identity_id) {
            identity.status = status;
            identity.updated_at = Utc::now();
            
            // 保存更新
            self.save_identity(identity).await?;
            
            log::info!("身份状态已更新: {} -> {:?}", identity_id, status);
            Ok(())
        } else {
            Err(DiapError::AuthenticationFailed(format!("身份不存在: {}", identity_id)))
        }
    }
    
    /// 更新身份权限
    pub async fn update_identity_permissions(&self, identity_id: &str, permissions: IdentityPermissions) -> Result<(), DiapError> {
        let mut identities = self.known_identities.write().await;
        
        if let Some(identity) = identities.get_mut(identity_id) {
            identity.permissions = permissions;
            identity.updated_at = Utc::now();
            
            // 保存更新
            self.save_identity(identity).await?;
            
            log::info!("身份权限已更新: {}", identity_id);
            Ok(())
        } else {
            Err(DiapError::AuthenticationFailed(format!("身份不存在: {}", identity_id)))
        }
    }
}

impl Default for DiapIdentityManager {
    fn default() -> Self {
        // 注意：这个默认实现主要用于测试，实际使用时应通过new()方法创建
        let config = DiapConfig::default();
        
        // 由于new()是async的，我们不能在这里调用它
        // 所以创建一个简化版本
        Self {
            config,
            current_identity: Arc::new(RwLock::new(None)),
            known_identities: Arc::new(RwLock::new(HashMap::new())),
            diap_manager: None,
        }
    }
}

impl std::fmt::Debug for DiapIdentityManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiapIdentityManager")
            .field("config", &self.config)
            .field("current_identity", &self.current_identity)
            .field("known_identities", &self.known_identities)
            .field("diap_manager", &"<UniversalNoirManager>")
            .finish()
    }
}

impl Clone for DiapIdentityManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            current_identity: Arc::clone(&self.current_identity),
            known_identities: Arc::clone(&self.known_identities),
            diap_manager: None, // 不能克隆 UniversalNoirManager，所以设置为 None
        }
    }
}