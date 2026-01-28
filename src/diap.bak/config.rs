//! DIAP配置模块
//!
//! 定义DIAP身份系统的配置参数和结构。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// DIAP系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiapConfig {
    /// 身份配置
    pub identity: IdentityConfig,
    
    /// 网络配置
    pub network: NetworkConfig,
    
    /// 证明配置
    pub proof: ProofConfig,
    
    /// 存储配置
    pub storage: StorageConfig,
}

/// 身份配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// 身份名称
    pub name: String,
    
    /// 身份描述
    pub description: Option<String>,
    
    /// 身份类型
    pub identity_type: IdentityType,
    
    /// 是否启用自动注册
    pub auto_register: bool,
    
    /// 身份过期时间（秒，None表示永不过期）
    pub expires_in: Option<u64>,
    
    /// 身份权限
    pub permissions: IdentityPermissions,
}

/// 身份类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityType {
    /// 个人身份
    Personal,
    
    /// 组织身份
    Organization,
    
    /// 设备身份
    Device,
    
    /// 服务身份
    Service,
    
    /// 智能体身份
    Agent,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 是否启用P2P网络
    pub enable_p2p: bool,
    
    /// P2P网络类型
    pub p2p_type: P2pType,
    
    /// 引导节点地址列表
    pub bootstrap_nodes: Vec<String>,
    
    /// 监听地址
    pub listen_address: String,
    
    /// 公告地址
    pub announce_address: Option<String>,
    
    /// 最大连接数
    pub max_connections: u32,
    
    /// 是否启用中继
    pub enable_relay: bool,
}

/// P2P网络类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum P2pType {
    /// libp2p网络
    Libp2p,
    
    /// Iroh网络
    Iroh,
    
    /// 混合网络
    Hybrid,
}

/// 证明配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofConfig {
    /// 是否启用零知识证明
    pub enable_zkp: bool,
    
    /// 证明类型
    pub proof_type: ProofType,
    
    /// 证明难度级别
    pub proof_difficulty: ProofDifficulty,
    
    /// 证明过期时间（秒）
    pub proof_expires_in: u64,
    
    /// 是否启用证明缓存
    pub enable_proof_cache: bool,
    
    /// 证明缓存大小
    pub proof_cache_size: usize,
}

/// 证明类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofType {
    /// Noir电路证明
    Noir,
    
    /// Arkworks Groth16证明
    Groth16,
    
    /// 简单签名证明
    Signature,
}

/// 证明难度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofDifficulty {
    /// 低难度（快速验证）
    Low,
    
    /// 中等难度（平衡）
    Medium,
    
    /// 高难度（高安全性）
    High,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 身份存储路径
    pub identity_store_path: PathBuf,
    
    /// 证明存储路径
    pub proof_store_path: PathBuf,
    
    /// 网络状态存储路径
    pub network_store_path: PathBuf,
    
    /// 是否启用加密存储
    pub enable_encryption: bool,
    
    /// 加密密钥（base64编码）
    pub encryption_key: Option<String>,
    
    /// 存储备份间隔（秒）
    pub backup_interval: u64,
}

/// 身份权限（从mod.rs导入）
use super::IdentityPermissions;

impl Default for DiapConfig {
    fn default() -> Self {
        Self {
            identity: IdentityConfig {
                name: "oracle-agent".to_string(),
                description: Some("Multi-Agent Oracle System Agent".to_string()),
                identity_type: IdentityType::Agent,
                auto_register: true,
                expires_in: None,
                permissions: IdentityPermissions {
                    can_create_oracle_data: true,
                    can_vote_in_consensus: true,
                    can_manage_agents: false,
                    can_access_sensitive_data: false,
                    expires_at: None,
                },
            },
            network: NetworkConfig {
                enable_p2p: true,
                p2p_type: P2pType::Hybrid,
                bootstrap_nodes: vec![
                    "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ".to_string(),
                    "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN".to_string(),
                ],
                listen_address: "/ip4/0.0.0.0/tcp/0".to_string(),
                announce_address: None,
                max_connections: 100,
                enable_relay: true,
            },
            proof: ProofConfig {
                enable_zkp: true,
                proof_type: ProofType::Noir,
                proof_difficulty: ProofDifficulty::Medium,
                proof_expires_in: 3600, // 1小时
                enable_proof_cache: true,
                proof_cache_size: 1000,
            },
            storage: StorageConfig {
                identity_store_path: PathBuf::from("./data/diap/identities"),
                proof_store_path: PathBuf::from("./data/diap/proofs"),
                network_store_path: PathBuf::from("./data/diap/network"),
                enable_encryption: true,
                encryption_key: None,
                backup_interval: 86400, // 24小时
            },
        }
    }
}

impl DiapConfig {
    /// 从文件加载配置
    pub fn from_file(path: &PathBuf) -> Result<Self, super::DiapError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| super::DiapError::ConfigError(format!("读取配置文件失败: {}", e)))?;
        
        let config: Self = toml::from_str(&content)
            .map_err(|e| super::DiapError::ConfigError(format!("解析配置文件失败: {}", e)))?;
        
        Ok(config)
    }
    
    /// 保存配置到文件
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), super::DiapError> {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| super::DiapError::ConfigError(format!("创建目录失败: {}", e)))?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| super::DiapError::ConfigError(format!("序列化配置失败: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| super::DiapError::ConfigError(format!("写入配置文件失败: {}", e)))?;
        
        Ok(())
    }
    
    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), super::DiapError> {
        // 验证身份配置
        if self.identity.name.trim().is_empty() {
            return Err(super::DiapError::ConfigError("身份名称不能为空".to_string()));
        }
        
        // 验证网络配置
        if self.network.enable_p2p && self.network.bootstrap_nodes.is_empty() {
            log::warn!("P2P网络已启用但没有引导节点，可能无法连接到网络");
        }
        
        // 验证存储配置
        if self.storage.enable_encryption && self.storage.encryption_key.is_none() {
            return Err(super::DiapError::ConfigError("启用加密存储但未提供加密密钥".to_string()));
        }
        
        Ok(())
    }
    
    /// 获取身份标识符
    pub fn get_identity_id(&self) -> String {
        let uuid_str = uuid::Uuid::new_v4().to_string();
        format!("{}-{}", self.identity.name, &uuid_str[..8])
    }
}
