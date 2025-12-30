//! DIAP身份管理模块
//!
//! 基于DIAP SDK的去中心化智能体身份验证和管理系统。

pub mod config;
pub mod identity_manager;
pub mod network_adapter;

// 重新导出主要类型
pub use config::DiapConfig;
pub use identity_manager::{DiapIdentityManager, AgentIdentity};
pub use network_adapter::DiapNetworkAdapter;

/// DIAP相关错误类型
#[derive(Debug, thiserror::Error)]
pub enum DiapError {
    /// 身份验证失败
    #[error("身份验证失败: {0}")]
    AuthenticationFailed(String),
    
    /// 身份注册失败
    #[error("身份注册失败: {0}")]
    RegistrationFailed(String),
    
    /// 身份证明生成失败
    #[error("身份证明生成失败: {0}")]
    ProofGenerationFailed(String),
    
    /// 身份证明验证失败
    #[error("身份证明验证失败: {0}")]
    ProofVerificationFailed(String),
    
    /// 网络连接失败
    #[error("网络连接失败: {0}")]
    NetworkError(String),
    
    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    /// 内部错误
    #[error("内部错误: {0}")]
    InternalError(String),
}

/// DIAP身份验证结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthResult {
    /// 是否验证成功
    pub authenticated: bool,
    
    /// 身份标识
    pub identity_id: String,
    
    /// 身份证明哈希
    pub proof_hash: Option<String>,
    
    /// 验证时间戳
    pub timestamp: i64,
    
    /// 附加信息
    pub metadata: serde_json::Value,
}

/// 身份状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IdentityStatus {
    /// 未注册
    Unregistered,
    
    /// 注册中
    Registering,
    
    /// 已注册
    Registered,
    
    /// 已验证
    Verified,
    
    /// 已撤销
    Revoked,
    
    /// 已过期
    Expired,
}

/// 身份权限
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdentityPermissions {
    /// 是否可以创建预言机数据
    pub can_create_oracle_data: bool,
    
    /// 是否可以参与共识投票
    pub can_vote_in_consensus: bool,
    
    /// 是否可以管理其他智能体
    pub can_manage_agents: bool,
    
    /// 是否可以访问敏感数据
    pub can_access_sensitive_data: bool,
    
    /// 权限过期时间（Unix时间戳）
    pub expires_at: Option<i64>,
}

/// DIAP模块初始化结果
#[derive(Debug, Clone)]
pub struct DiapInitResult {
    /// 是否初始化成功
    pub success: bool,
    
    /// 身份管理器实例
    pub identity_manager: Option<DiapIdentityManager>,
    
    /// 网络适配器实例
    pub network_adapter: Option<DiapNetworkAdapter>,
    
    /// 错误信息（如果有）
    pub error: Option<String>,
}
