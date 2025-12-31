//! DIAP SDK模拟服务器
//! 
//! 为本地测试提供DIAP身份验证的模拟服务

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

/// DIAP模拟服务器状态
#[derive(Clone)]
struct AppState {
    /// 存储的身份信息
    identities: Arc<RwLock<HashMap<String, IdentityRecord>>>,
    /// 验证记录
    auth_records: Arc<RwLock<Vec<AuthRecord>>>,
    /// 服务器配置
    config: ServerConfig,
}

/// 身份记录
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdentityRecord {
    /// 身份ID
    id: String,
    /// 公钥
    public_key: String,
    /// 身份证明
    proof: String,
    /// 层级
    tier: String,
    /// 信誉分
    reputation: f64,
    /// 是否已验证
    verified: bool,
    /// 创建时间戳
    created_at: u64,
    /// 最后验证时间
    last_verified_at: Option<u64>,
}

/// 验证记录
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthRecord {
    /// 请求ID
    request_id: String,
    /// 身份ID
    identity_id: String,
    /// 验证结果
    success: bool,
    /// 验证时间戳
    timestamp: u64,
    /// 错误信息（如果有）
    error: Option<String>,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerConfig {
    /// 服务器端口
    port: u16,
    /// 模拟延迟（毫秒）
    simulated_delay_ms: u64,
    /// 验证成功率
    verification_success_rate: f64,
    /// 默认层级
    default_tier: String,
}

/// 身份注册请求
#[derive(Debug, Deserialize)]
struct RegisterRequest {
    /// 公钥
    public_key: String,
    /// 身份证明
    proof: String,
    /// 请求的层级
    requested_tier: Option<String>,
}

/// 身份注册响应
#[derive(Debug, Serialize)]
struct RegisterResponse {
    /// 是否成功
    success: bool,
    /// 分配的身份ID
    identity_id: Option<String>,
    /// 分配的层级
    tier: Option<String>,
    /// 错误信息
    error: Option<String>,
}

/// 身份验证请求
#[derive(Debug, Deserialize)]
struct VerifyRequest {
    /// 身份ID
    identity_id: String,
    /// 身份证明
    proof: String,
    /// 请求的层级
    requested_tier: Option<String>,
}

/// 身份验证响应
#[derive(Debug, Serialize)]
struct VerifyResponse {
    /// 是否验证成功
    verified: bool,
    /// 验证的身份ID
    identity_id: String,
    /// 分配的层级
    tier: Option<String>,
    /// 信誉分
    reputation: Option<f64>,
    /// 验证时间戳
    timestamp: u64,
    /// 错误信息
    error: Option<String>,
}

/// 服务器状态响应
#[derive(Debug, Serialize)]
struct StatusResponse {
    /// 服务器版本
    version: String,
    /// 运行时间（秒）
    uptime_seconds: u64,
    /// 总身份数
    total_identities: usize,
    /// 已验证身份数
    verified_identities: usize,
    /// 总验证请求数
    total_verifications: usize,
    /// 成功验证数
    successful_verifications: usize,
    /// 服务器配置
    config: ServerConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            simulated_delay_ms: 50,
            verification_success_rate: 0.95,
            default_tier: "data".to_string(),
        }
    }
}

impl AppState {
    /// 创建新的应用状态
    fn new(config: ServerConfig) -> Self {
        Self {
            identities: Arc::new(RwLock::new(HashMap::new())),
            auth_records: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }
    
    /// 模拟网络延迟
    async fn simulate_delay(&self) {
        if self.config.simulated_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.config.simulated_delay_ms)).await;
        }
    }
    
    /// 模拟验证成功率
    fn should_succeed(&self) -> bool {
        rand::random::<f64>() < self.config.verification_success_rate
    }
    
    /// 根据信誉分确定层级
    fn determine_tier(&self, reputation: f64) -> String {
        if reputation >= 800.0 {
            "core".to_string()
        } else if reputation >= 500.0 {
            "validator".to_string()
        } else {
            "data".to_string()
        }
    }
    
    /// 生成初始信誉分
    fn generate_initial_reputation(&self, requested_tier: Option<&str>) -> f64 {
        match requested_tier {
            Some("core") => 800.0 + rand::random::<f64>() * 200.0, // 800-1000
            Some("validator") => 500.0 + rand::random::<f64>() * 300.0, // 500-800
            _ => rand::random::<f64>() * 500.0, // 0-500
        }
    }
}

/// 处理身份注册
async fn handle_register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> impl IntoResponse {
    println!("📝 处理身份注册请求: {:?}", request);
    
    // 模拟网络延迟
    state.simulate_delay().await;
    
    // 生成身份ID
    let identity_id = format!("diap_{}_{}", rand::random::<u32>(), chrono::Utc::now().timestamp());
    
    // 生成初始信誉分和层级
    let reputation = state.generate_initial_reputation(request.requested_tier.as_deref());
    let tier = state.determine_tier(reputation);
    
    // 创建身份记录
    let identity = IdentityRecord {
        id: identity_id.clone(),
        public_key: request.public_key,
        proof: request.proof,
        tier: tier.clone(),
        reputation,
        verified: true,
        created_at: chrono::Utc::now().timestamp() as u64,
        last_verified_at: Some(chrono::Utc::now().timestamp() as u64),
    };
    
    // 存储身份
    {
        let mut identities = state.identities.write().await;
        identities.insert(identity_id.clone(), identity);
    }
    
    println!("✅ 身份注册成功: ID={}, 层级={}, 信誉={:.1}", identity_id, tier, reputation);
    
    let response = RegisterResponse {
        success: true,
        identity_id: Some(identity_id),
        tier: Some(tier),
        error: None,
    };
    
    (StatusCode::OK, Json(response))
}

/// 处理身份验证
async fn handle_verify(
    State(state): State<AppState>,
    Json(request): Json<VerifyRequest>,
) -> impl IntoResponse {
    println!("🔐 处理身份验证请求: ID={}", request.identity_id);
    
    // 模拟网络延迟
    state.simulate_delay().await;
    
    let request_id = format!("req_{}_{}", rand::random::<u32>(), chrono::Utc::now().timestamp());
    let timestamp = chrono::Utc::now().timestamp() as u64;
    
    // 检查身份是否存在
    let identities = state.identities.read().await;
    let identity = identities.get(&request.identity_id);
    
    let (verified, tier, reputation, error) = if let Some(identity) = identity {
        // 模拟验证成功率
        let should_succeed = state.should_succeed();
        
        if should_succeed {
            println!("✅ 身份验证成功: ID={}", request.identity_id);
            (true, Some(identity.tier.clone()), Some(identity.reputation), None)
        } else {
            let error_msg = "模拟验证失败".to_string();
            println!("❌ 身份验证失败: ID={}, 错误: {}", request.identity_id, error_msg);
            (false, None, None, Some(error_msg))
        }
    } else {
        let error_msg = format!("身份 {} 不存在", request.identity_id);
        println!("❌ {}", error_msg);
        (false, None, None, Some(error_msg))
    };
    
    // 记录验证结果
    {
        let mut auth_records = state.auth_records.write().await;
        auth_records.push(AuthRecord {
            request_id,
            identity_id: request.identity_id.clone(),
            success: verified,
            timestamp,
            error: error.clone(),
        });
        
        // 限制记录大小
        if auth_records.len() > 1000 {
            auth_records.remove(0);
        }
    }
    
    let response = VerifyResponse {
        verified,
        identity_id: request.identity_id,
        tier,
        reputation,
        timestamp,
        error,
    };
    
    (StatusCode::OK, Json(response))
}

/// 处理服务器状态查询
async fn handle_status(State(state): State<AppState>) -> impl IntoResponse {
    let identities = state.identities.read().await;
    let auth_records = state.auth_records.read().await;
    
    let total_identities = identities.len();
    let verified_identities = identities.values().filter(|id| id.verified).count();
    let total_verifications = auth_records.len();
    let successful_verifications = auth_records.iter().filter(|r| r.success).count();
    
    let response = StatusResponse {
        version: "1.0.0".to_string(),
        uptime_seconds: 0, // 在实际实现中会计算运行时间
        total_identities,
        verified_identities,
        total_verifications,
        successful_verifications,
        config: state.config.clone(),
    };
    
    (StatusCode::OK, Json(response))
}

/// 处理健康检查
async fn handle_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "healthy", "timestamp": chrono::Utc::now().timestamp()})))
}

/// 主函数
#[tokio::main]
async fn main() {
    println!("🚀 启动DIAP SDK模拟服务器");
    
    // 加载配置
    let config = ServerConfig::default();
    println!("📋 服务器配置:");
    println!("  端口: {}", config.port);
    println!("  模拟延迟: {}ms", config.simulated_delay_ms);
    println!("  验证成功率: {:.1}%", config.verification_success_rate * 100.0);
    println!("  默认层级: {}", config.default_tier);
    
    // 创建应用状态
    let state = AppState::new(config.clone());
    
    // 预创建一些测试身份
    initialize_test_identities(state.clone()).await;
    
    // 创建路由
    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/status", get(handle_status))
        .route("/register", post(handle_register))
        .route("/verify", post(handle_verify))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    // 启动服务器
    let addr = format!("0.0.0.0:{}", config.port);
    println!("🌐 服务器监听地址: http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// 初始化测试身份
async fn initialize_test_identities(state: AppState) {
    println!("🔧 初始化测试身份...");
    
    let test_identities = vec![
        ("node1", "core", 850.0),
        ("node2", "core", 820.0),
        ("node3", "validator", 650.0),
        ("node4", "validator", 580.0),
        ("node5", "validator", 520.0),
        ("node6", "data", 350.0),
        ("node7", "data", 280.0),
        ("node8", "data", 220.0),
        ("node9", "data", 150.0),
        ("node10", "data", 80.0),
    ];
    
    let mut identities = state.identities.write().await;
    
    for (node_id, tier, reputation) in test_identities {
        let identity_id = format!("test_{}", node_id);
        
        let identity = IdentityRecord {
            id: identity_id.clone(),
            public_key: format!("pk_{}", node_id),
            proof: format!("proof_{}", node_id),
            tier: tier.to_string(),
            reputation,
            verified: true,
            created_at: chrono::Utc::now().timestamp() as u64,
            last_verified_at: Some(chrono::Utc::now().timestamp() as u64),
        };
        
        identities.insert(identity_id, identity);
    }
    
    println!("✅ 初始化了 {} 个测试身份", identities.len());
}

/// 测试客户端
#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    
    #[tokio::test]
    async fn test_mock_server() {
        // 注意：这个测试需要服务器运行
        // 在实际测试中，应该启动服务器然后进行测试
        println!("测试DIAP模拟服务器功能");
    }
}
