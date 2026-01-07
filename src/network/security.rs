//! 网络安全模块
//! 
//! 提供网络通信的安全功能，包括加密、认证和访问控制

use crate::types::NodeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 安全配置
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// 是否启用加密
    pub enable_encryption: bool,
    /// 加密算法
    pub encryption_algorithm: EncryptionAlgorithm,
    /// 是否启用消息认证
    pub enable_message_auth: bool,
    /// 认证算法
    pub auth_algorithm: AuthAlgorithm,
    /// 是否启用访问控制
    pub enable_access_control: bool,
    /// 会话超时时间（秒）
    pub session_timeout: u64,
    /// 最大失败尝试次数
    pub max_failed_attempts: u32,
}

/// 加密算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
    /// XChaCha20-Poly1305
    XChaCha20Poly1305,
    /// 无加密
    None,
}

/// 认证算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthAlgorithm {
    /// HMAC-SHA256
    HmacSha256,
    /// HMAC-SHA512
    HmacSha512,
    /// Poly1305
    Poly1305,
    /// 无认证
    None,
}

/// 安全会话
#[derive(Debug, Clone)]
pub struct SecuritySession {
    /// 会话ID
    pub session_id: String,
    /// 远程节点ID
    pub remote_node_id: NodeId,
    /// 会话密钥
    pub session_key: Option<Vec<u8>>,
    /// 建立时间
    pub established_at: std::time::SystemTime,
    /// 最后活动时间
    pub last_activity: std::time::SystemTime,
    /// 加密状态
    pub encryption_enabled: bool,
    /// 认证状态
    pub auth_enabled: bool,
    /// 失败尝试次数
    pub failed_attempts: u32,
}

/// 安全管理器
pub struct SecurityManager {
    /// 配置
    config: SecurityConfig,
    /// 本地节点ID
    local_node_id: NodeId,
    /// 活跃会话
    sessions: Arc<RwLock<HashMap<NodeId, SecuritySession>>>,
    /// 可信节点列表
    trusted_nodes: Arc<RwLock<Vec<NodeId>>>,
    /// 安全统计
    stats: Arc<RwLock<SecurityStats>>,
}

/// 安全统计
#[derive(Debug, Clone, Default)]
pub struct SecurityStats {
    /// 总会话数
    pub total_sessions: u64,
    /// 当前活跃会话数
    pub active_sessions: usize,
    /// 加密消息数
    pub encrypted_messages: u64,
    /// 认证消息数
    pub authenticated_messages: u64,
    /// 安全违规次数
    pub security_violations: u64,
    /// 失败认证尝试次数
    pub failed_auth_attempts: u64,
}

impl SecurityManager {
    /// 创建新的安全管理器
    pub fn new(config: SecurityConfig, local_node_id: NodeId) -> Self {
        Self {
            config,
            local_node_id,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            trusted_nodes: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(SecurityStats::default())),
        }
    }
    
    /// 建立安全会话
    pub async fn establish_session(&self, remote_node_id: &NodeId) -> Result<String, String> {
        println!("🔐 与节点 {} 建立安全会话", remote_node_id);
        
        // 检查节点是否可信
        if self.config.enable_access_control {
            let trusted_nodes = self.trusted_nodes.read().await;
            if !trusted_nodes.contains(remote_node_id) {
                println!("❌ 节点 {} 不在可信列表中", remote_node_id);
                
                let mut stats = self.stats.write().await;
                stats.security_violations += 1;
                
                return Err("节点不可信".to_string());
            }
        }
        
        let session_id = format!("sess_{}_{}_{}", 
            self.local_node_id, remote_node_id, 
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        
        // 生成会话密钥（模拟）
        let session_key = if self.config.enable_encryption {
            Some(vec![0u8; 32]) // 32字节密钥
        } else {
            None
        };
        
        let session = SecuritySession {
            session_id: session_id.clone(),
            remote_node_id: remote_node_id.clone(),
            session_key,
            established_at: std::time::SystemTime::now(),
            last_activity: std::time::SystemTime::now(),
            encryption_enabled: self.config.enable_encryption,
            auth_enabled: self.config.enable_message_auth,
            failed_attempts: 0,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(remote_node_id.clone(), session);
        
        let mut stats = self.stats.write().await;
        stats.total_sessions += 1;
        stats.active_sessions = sessions.len();
        
        println!("✅ 安全会话建立成功: {}", session_id);
        Ok(session_id)
    }
    
    /// 加密消息
    pub async fn encrypt_message(&self, remote_node_id: &NodeId, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        if !self.config.enable_encryption {
            return Ok(plaintext.to_vec()); // 不加密
        }
        
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(remote_node_id) {
            if !session.encryption_enabled {
                return Ok(plaintext.to_vec());
            }
            
            // 模拟加密过程
            let mut ciphertext = plaintext.to_vec();
            
            // 在实际实现中，这里会使用真正的加密算法
            // 这里只是简单地在数据前后添加标记
            let mut result = Vec::new();
            result.extend_from_slice(b"[ENC]");
            result.extend_from_slice(&ciphertext);
            result.extend_from_slice(b"[/ENC]");
            
            let mut stats = self.stats.write().await;
            stats.encrypted_messages += 1;
            
            println!("🔒 加密 {} 字节消息到节点 {}", plaintext.len(), remote_node_id);
            Ok(result)
        } else {
            Err("安全会话未建立".to_string())
        }
    }
    
    /// 解密消息
    pub async fn decrypt_message(&self, remote_node_id: &NodeId, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        if !self.config.enable_encryption {
            return Ok(ciphertext.to_vec()); // 不解密
        }
        
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(remote_node_id) {
            if !session.encryption_enabled {
                return Ok(ciphertext.to_vec());
            }
            
            // 模拟解密过程
            // 检查是否是加密格式
            if ciphertext.starts_with(b"[ENC]") && ciphertext.ends_with(b"[/ENC]") {
                let plaintext = &ciphertext[5..ciphertext.len()-6]; // 去掉标记
                Ok(plaintext.to_vec())
            } else {
                Err("消息格式错误".to_string())
            }
        } else {
            Err("安全会话未建立".to_string())
        }
    }
    
    /// 认证消息
    pub async fn authenticate_message(&self, remote_node_id: &NodeId, message: &[u8], auth_tag: &[u8]) -> Result<bool, String> {
        if !self.config.enable_message_auth {
            return Ok(true); // 不认证
        }
        
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(remote_node_id) {
            if !session.auth_enabled {
                return Ok(true);
            }
            
            // 模拟认证过程
            // 在实际实现中，这里会验证消息认证码
            let is_authentic = auth_tag == b"AUTH_TAG"; // 简单的模拟
            
            if is_authentic {
                session.last_activity = std::time::SystemTime::now();
                session.failed_attempts = 0;
                
                let mut stats = self.stats.write().await;
                stats.authenticated_messages += 1;
                
                println!("✅ 消息认证成功: 来自节点 {}", remote_node_id);
                Ok(true)
            } else {
                session.failed_attempts += 1;
                
                let mut stats = self.stats.write().await;
                stats.failed_auth_attempts += 1;
                
                println!("❌ 消息认证失败: 来自节点 {}", remote_node_id);
                
                // 检查是否超过最大失败尝试次数
                if session.failed_attempts >= self.config.max_failed_attempts {
                    println!("⚠️  节点 {} 认证失败次数过多，终止会话", remote_node_id);
                    sessions.remove(remote_node_id);
                    
                    stats.active_sessions = sessions.len();
                    stats.security_violations += 1;
                }
                
                Ok(false)
            }
        } else {
            Err("安全会话未建立".to_string())
        }
    }
    
    /// 生成认证标签
    pub async fn generate_auth_tag(&self, remote_node_id: &NodeId, message: &[u8]) -> Result<Vec<u8>, String> {
        if !self.config.enable_message_auth {
            return Ok(Vec::new()); // 不生成认证标签
        }
        
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(remote_node_id) {
            if !session.auth_enabled {
                return Ok(Vec::new());
            }
            
            // 模拟生成认证标签
            // 在实际实现中，这里会生成真正的消息认证码
            Ok(b"AUTH_TAG".to_vec())
        } else {
            Err("安全会话未建立".to_string())
        }
    }
    
    /// 添加可信节点
    pub async fn add_trusted_node(&self, node_id: NodeId) {
        let mut trusted_nodes = self.trusted_nodes.write().await;
        if !trusted_nodes.contains(&node_id) {
            trusted_nodes.push(node_id.clone());
            println!("✅ 添加可信节点: {}", node_id);
        }
    }
    
    /// 移除可信节点
    pub async fn remove_trusted_node(&self, node_id: &NodeId) {
        let mut trusted_nodes = self.trusted_nodes.write().await;
        if let Some(pos) = trusted_nodes.iter().position(|id| id == node_id) {
            trusted_nodes.remove(pos);
            println!("🗑️  移除可信节点: {}", node_id);
        }
    }
    
    /// 获取会话信息
    pub async fn get_session_info(&self, remote_node_id: &NodeId) -> Option<SecuritySession> {
        let sessions = self.sessions.read().await;
        sessions.get(remote_node_id).cloned()
    }
    
    /// 获取所有会话信息
    pub async fn get_all_sessions(&self) -> Vec<SecuritySession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }
    
    /// 获取安全统计
    pub async fn get_stats(&self) -> SecurityStats {
        self.stats.read().await.clone()
    }
    
    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        
        let now = std::time::SystemTime::now();
        
        sessions.retain(|_, session| {
            if let Ok(duration) = now.duration_since(session.last_activity) {
                duration.as_secs() <= self.config.session_timeout
            } else {
                true // 如果时间计算失败，保留会话
            }
        });
        
        let removed_count = initial_count - sessions.len();
        if removed_count > 0 {
            println!("🧹 清理了 {} 个过期会话", removed_count);
            
            let mut stats = self.stats.write().await;
            stats.active_sessions = sessions.len();
        }
        
        removed_count
    }
    
    /// 检查节点是否可信
    pub async fn is_node_trusted(&self, node_id: &NodeId) -> bool {
        let trusted_nodes = self.trusted_nodes.read().await;
        trusted_nodes.contains(node_id)
    }
    
    /// 获取加密算法
    pub fn encryption_algorithm(&self) -> EncryptionAlgorithm {
        self.config.encryption_algorithm
    }
    
    /// 获取认证算法
    pub fn auth_algorithm(&self) -> AuthAlgorithm {
        self.config.auth_algorithm
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_encryption: true,
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            enable_message_auth: true,
            auth_algorithm: AuthAlgorithm::HmacSha256,
            enable_access_control: true,
            session_timeout: 3600, // 1小时
            max_failed_attempts: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_security_manager() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config, "local_node".to_string());
        
        // 添加可信节点
        manager.add_trusted_node("node1".to_string()).await;
        
        // 测试建立会话
        let result = manager.establish_session(&"node1".to_string()).await;
        assert!(result.is_ok());
        
        let session_id = result.unwrap();
        assert!(!session_id.is_empty());
        
        // 测试加密消息
        let plaintext = b"secret message";
        let encrypted = manager.encrypt_message(&"node1".to_string(), plaintext).await;
        assert!(encrypted.is_ok());
        
        let ciphertext = encrypted.unwrap();
        assert_ne!(ciphertext, plaintext);
        
        // 测试解密消息
        let decrypted = manager.decrypt_message(&"node1".to_string(), &ciphertext).await;
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap(), plaintext);
        
        // 测试生成认证标签
        let auth_tag = manager.generate_auth_tag(&"node1".to_string(), plaintext).await;
        assert!(auth_tag.is_ok());
        
        // 测试认证消息
        let authenticated = manager.authenticate_message(&"node1".to_string(), plaintext, b"AUTH_TAG").await;
        assert!(authenticated.is_ok());
        assert!(authenticated.unwrap());
        
        // 测试获取统计
        let stats = manager.get_stats().await;
        assert!(stats.total_sessions >= 1);
        assert!(stats.encrypted_messages >= 1);
        assert!(stats.authenticated_messages >= 1);
    }
}
