use crate::network::{PeerDiscovery, MessageHandler, Protocol};
use crate::diap::{DiapNetworkAdapter, DiapConfig, DiapIdentityManager, AgentIdentity};
use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};
use serde::{Deserialize, Serialize};

/// 网络管理器
pub struct NetworkManager {
    /// 节点ID
    node_id: String,
    /// 配置
    config: NetworkConfig,
    /// 对等节点发现
    peer_discovery: Arc<PeerDiscovery>,
    /// 消息处理器
    message_handler: Arc<MessageHandler>,
    /// 协议处理器
    protocols: HashMap<String, Arc<Protocol>>,
    /// 连接状态
    connections: Arc<RwLock<HashMap<String, ConnectionStatus>>>,
    /// 网络状态
    status: Arc<RwLock<NetworkStatus>>,
    /// DIAP网络适配器
    diap_network_adapter: Option<Arc<DiapNetworkAdapter>>,
    /// DIAP身份管理器
    diap_identity_manager: Option<Arc<DiapIdentityManager>>,
    /// 是否启用DIAP身份验证
    enable_diap_auth: bool,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 监听地址
    pub listen_address: String,
    /// 监听端口
    pub listen_port: u16,
    /// 引导节点列表
    pub bootstrap_nodes: Vec<String>,
    /// 最大连接数
    pub max_connections: usize,
    /// 连接超时时间（秒）
    pub connection_timeout_secs: u64,
    /// 心跳间隔（秒）
    pub heartbeat_interval_secs: u64,
    /// 是否启用NAT穿透
    pub enable_nat_traversal: bool,
    /// 是否启用中继
    pub enable_relay: bool,
    /// 中继节点列表
    pub relay_nodes: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0".to_string(),
            listen_port: 4001,
            bootstrap_nodes: vec![
                "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ".to_string(),
                "/ip4/104.131.131.82/udp/4001/quic/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ".to_string(),
            ],
            max_connections: 100,
            connection_timeout_secs: 30,
            heartbeat_interval_secs: 60,
            enable_nat_traversal: true,
            enable_relay: true,
            relay_nodes: vec![],
        }
    }
}

/// 网络状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// 是否运行中
    pub is_running: bool,
    /// 开始时间
    pub start_time: u64,
    /// 总连接数
    pub total_connections: usize,
    /// 活跃连接数
    pub active_connections: usize,
    /// 发送消息数
    pub messages_sent: u64,
    /// 接收消息数
    pub messages_received: u64,
    /// 发现的节点数
    pub discovered_peers: usize,
    /// 网络带宽（KB/s）
    pub network_bandwidth_kbps: f64,
    /// 最后错误
    pub last_error: Option<String>,
}

/// 连接状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    /// 对等节点ID
    pub peer_id: String,
    /// 连接地址
    pub address: String,
    /// 连接时间
    pub connected_at: u64,
    /// 最后活动时间
    pub last_activity: u64,
    /// 是否活跃
    pub is_active: bool,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 接收字节数
    pub bytes_received: u64,
    /// 最后错误
    pub last_error: Option<String>,
    /// 连接类型
    pub connection_type: ConnectionType,
}

/// 连接类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    /// 直接连接
    Direct,
    /// 中继连接
    Relay,
    /// WebRTC连接
    WebRTC,
    /// 其他
    Other(String),
}

impl NetworkManager {
    /// 创建新的网络管理器
    pub fn new(node_id: String, config: NetworkConfig) -> Result<Self> {
        let peer_discovery = PeerDiscovery::new(config.clone());
        let message_handler = MessageHandler::new();
        
        Ok(Self {
            node_id,
            config,
            peer_discovery: Arc::new(peer_discovery),
            message_handler: Arc::new(message_handler),
            protocols: HashMap::new(),
            connections: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(NetworkStatus {
                is_running: false,
                start_time: 0,
                total_connections: 0,
                active_connections: 0,
                messages_sent: 0,
                messages_received: 0,
                discovered_peers: 0,
                network_bandwidth_kbps: 0.0,
                last_error: None,
            })),
            diap_network_adapter: None,
            diap_identity_manager: None,
            enable_diap_auth: false,
        })
    }
    
    /// 初始化DIAP网络适配器
    pub async fn init_diap_network(
        &mut self, 
        diap_config: Option<DiapConfig>,
        identity_manager: Option<Arc<DiapIdentityManager>>
    ) -> Result<()> {
        info!("🔄 初始化DIAP网络适配器");
        
        let config = diap_config.unwrap_or_else(|| {
            let mut default_config = DiapConfig::default();
            default_config.network.listen_address = format!("/ip4/0.0.0.0/tcp/{}", self.config.listen_port);
            default_config.network.bootstrap_nodes = self.config.bootstrap_nodes.clone();
            default_config.network.max_connections = self.config.max_connections as u32;
            default_config.network.enable_relay = self.config.enable_relay;
            default_config
        });
        
        // 创建DIAP网络适配器
        match DiapNetworkAdapter::new(config, identity_manager.unwrap_or_else(|| {
            Arc::new(DiapIdentityManager::default())
        })).await {
            Ok(adapter) => {
                self.diap_network_adapter = Some(Arc::new(adapter));
                self.diap_identity_manager = identity_manager;
                self.enable_diap_auth = true;
                info!("✅ DIAP网络适配器初始化完成");
                Ok(())
            }
            Err(e) => {
                warn!("⚠️ DIAP网络适配器初始化失败: {}, 将使用传统网络模式", e);
                self.enable_diap_auth = false;
                Ok(())
            }
        }
    }
    
    /// 启动DIAP网络
    pub async fn start_diap_network(&self) -> Result<()> {
        if let Some(adapter) = &self.diap_network_adapter {
            info!("🚀 启动DIAP网络");
            adapter.start().await.map_err(|e| anyhow!("启动DIAP网络失败: {}", e))
        } else {
            Err(anyhow!("DIAP网络适配器未初始化"))
        }
    }
    
    /// 停止DIAP网络
    pub async fn stop_diap_network(&self) -> Result<()> {
        if let Some(adapter) = &self.diap_network_adapter {
            info!("🛑 停止DIAP网络");
            adapter.stop().await.map_err(|e| anyhow!("停止DIAP网络失败: {}", e))
        } else {
            Ok(())
        }
    }
    
    /// 启动网络
    pub async fn start(&mut self) -> Result<()> {
        let mut status = self.status.write().await;
        
        if status.is_running {
            return Err(anyhow!("网络已经在运行中"));
        }
        
        info!("🚀 启动网络管理器，节点ID: {}", self.node_id);
        info!("📡 监听地址: {}:{}", self.config.listen_address, self.config.listen_port);
        
        // 更新状态
        status.is_running = true;
        status.start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 启动对等节点发现
        self.peer_discovery.start().await?;
        
        // 连接到引导节点
        self.connect_to_bootstrap_nodes().await?;
        
        info!("✅ 网络启动成功");
        
        Ok(())
    }
    
    /// 停止网络
    pub async fn stop(&mut self) -> Result<()> {
        let mut status = self.status.write().await;
        
        if !status.is_running {
            return Ok(());
        }
        
        info!("🛑 停止网络管理器");
        
        // 停止对等节点发现
        self.peer_discovery.stop().await?;
        
        // 关闭所有连接
        self.close_all_connections().await?;
        
        // 更新状态
        status.is_running = false;
        
        info!("✅ 网络停止成功");
        
        Ok(())
    }
    
    /// 连接到引导节点
    async fn connect_to_bootstrap_nodes(&self) -> Result<()> {
        info!("🔗 连接到引导节点: {} 个", self.config.bootstrap_nodes.len());
        
        let mut connected_count = 0;
        
        for bootstrap_node in &self.config.bootstrap_nodes {
            match self.connect_to_peer(bootstrap_node).await {
                Ok(_) => {
                    connected_count += 1;
                    info!("✅ 连接到引导节点: {}", bootstrap_node);
                }
                Err(e) => {
                    warn!("❌ 连接引导节点失败 {}: {}", bootstrap_node, e);
                }
            }
        }
        
        info!("📊 引导节点连接结果: {}/{} 成功", 
            connected_count, self.config.bootstrap_nodes.len());
        
        Ok(())
    }
    
    /// 连接到对等节点
    pub async fn connect_to_peer(&self, peer_address: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        // 检查是否已连接
        if connections.contains_key(peer_address) {
            return Err(anyhow!("已经连接到该节点"));
        }
        
        // 创建连接状态
        let connection = ConnectionStatus {
            peer_id: peer_address.to_string(),
            address: peer_address.to_string(),
            connected_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_active: true,
            bytes_sent: 0,
            bytes_received: 0,
            last_error: None,
            connection_type: ConnectionType::Direct,
        };
        
        // 添加到连接列表
        connections.insert(peer_address.to_string(), connection);
        
        // 更新网络状态
        let mut status = self.status.write().await;
        status.total_connections += 1;
        status.active_connections += 1;
        
        info!("🔗 连接到对等节点: {}", peer_address);
        
        Ok(())
    }
    
    /// 断开与对等节点的连接
    pub async fn disconnect_from_peer(&self, peer_address: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.remove(peer_address) {
            // 更新网络状态
            let mut status = self.status.write().await;
            status.active_connections -= 1;
            
            info!("🔌 断开与对等节点的连接: {}", peer_address);
            
            Ok(())
        } else {
            Err(anyhow!("未找到该节点的连接"))
        }
    }
    
    /// 关闭所有连接
    async fn close_all_connections(&self) -> Result<()> {
        let mut connections = self.connections.write().await;
        let count = connections.len();
        
        connections.clear();
        
        // 更新网络状态
        let mut status = self.status.write().await;
        status.active_connections = 0;
        
        info!("🔌 关闭所有连接: {} 个", count);
        
        Ok(())
    }
    
    /// 发送消息到对等节点
    pub async fn send_message(&self, peer_address: &str, message: Vec<u8>) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(peer_address) {
            if !connection.is_active {
                return Err(anyhow!("连接不活跃"));
            }
            
            // 更新连接状态
            connection.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            connection.bytes_sent += message.len() as u64;
            
            // 更新网络状态
            let mut status = self.status.write().await;
            status.messages_sent += 1;
            
            info!("📤 发送消息到 {}: {} 字节", peer_address, message.len());
            
            // 这里应该实现实际的消息发送逻辑
            // 简化实现：记录日志
            
            Ok(())
        } else {
            Err(anyhow!("未找到该节点的连接"))
        }
    }
    
    /// 广播消息到所有对等节点
    pub async fn broadcast_message(&self, message: Vec<u8>) -> Result<usize> {
        let connections = self.connections.read().await;
        let mut sent_count = 0;
        
        for (peer_address, connection) in connections.iter() {
            if connection.is_active {
                match self.send_message(peer_address, message.clone()).await {
                    Ok(_) => sent_count += 1,
                    Err(e) => {
                        warn!("广播消息失败 {}: {}", peer_address, e);
                    }
                }
            }
        }
        
        info!("📢 广播消息: {}/{} 个节点成功", 
            sent_count, connections.len());
        
        Ok(sent_count)
    }
    
    /// 注册协议处理器
    pub fn register_protocol(&mut self, protocol: Protocol) -> Result<()> {
        let protocol_name = protocol.config.name.clone();
        
        if self.protocols.contains_key(&protocol_name) {
            return Err(anyhow!("协议已注册: {}", protocol_name));
        }
        
        self.protocols.insert(protocol_name.clone(), Arc::new(protocol));
        
        info!("📝 注册协议: {}", protocol_name);
        
        Ok(())
    }
    
    /// 处理接收到的消息
    pub async fn handle_received_message(&self, peer_address: &str, message: Vec<u8>) -> Result<()> {
        // 更新连接状态
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(peer_address) {
            connection.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            connection.bytes_received += message.len() as u64;
        }
        
        // 更新网络状态
        let mut status = self.status.write().await;
        status.messages_received += 1;
        
        info!("📥 从 {} 接收消息: {} 字节", peer_address, message.len());
        
        // 这里应该实现实际的消息处理逻辑
        // 简化实现：记录日志
        
        Ok(())
    }
    
    /// 获取网络状态
    pub async fn get_status(&self) -> NetworkStatus {
        self.status.read().await.clone()
    }
    
    /// 获取DIAP网络状态
    pub async fn get_diap_network_status(&self) -> Result<String> {
        if let Some(adapter) = &self.diap_network_adapter {
            let status = adapter.check_network_status().await;
            Ok(format!(
                "DIAP网络状态: 运行中={}, 总节点={}, 已连接={}, 活跃连接={}",
                status.is_running, status.total_nodes, status.connected_nodes, status.active_connections
            ))
        } else {
            Ok("DIAP网络未启用".to_string())
        }
    }
    
    /// 使用DIAP身份发送消息
    pub async fn send_message_with_diap_identity(
        &self,
        message: &str,
        receiver_identity_id: Option<&str>,
        require_auth: bool,
    ) -> Result<()> {
        if !self.enable_diap_auth {
            return Err(anyhow!("DIAP身份验证未启用"));
        }
        
        let adapter = self.diap_network_adapter.as_ref()
            .ok_or_else(|| anyhow!("DIAP网络适配器未初始化"))?;
        
        // 获取当前身份
        let current_identity = if let Some(manager) = &self.diap_identity_manager {
            manager.get_current_identity().await
        } else {
            None
        };
        
        let sender_id = current_identity.as_ref()
            .map(|id| id.id.clone())
            .unwrap_or_else(|| "anonymous".to_string());
        
        // 创建网络消息
        use crate::diap::network_adapter::{NetworkMessage, MessageType};
        
        let network_message = NetworkMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            sender_id,
            receiver_id: receiver_identity_id.map(|s| s.to_string()),
            message_type: if require_auth {
                MessageType::AuthRequest
            } else {
                MessageType::Custom
            },
            payload: serde_json::json!({
                "content": message,
                "timestamp": chrono::Utc::now().timestamp(),
                "require_auth": require_auth,
            }),
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
        };
        
        // 发送消息
        adapter.send_message(network_message).await
            .map_err(|e| anyhow!("发送DIAP消息失败: {}", e))?;
        
        info!("📤 通过DIAP发送消息: {} -> {:?}", 
            network_message.sender_id, network_message.receiver_id);
        
        Ok(())
    }
    
    /// 接收DIAP消息
    pub async fn receive_diap_messages(&self, limit: usize) -> Result<Vec<String>> {
        if let Some(adapter) = &self.diap_network_adapter {
            let messages = adapter.receive_messages(limit).await;
            let contents: Vec<String> = messages.into_iter()
                .map(|msg| {
                    format!("[{}] {}: {}", 
                        msg.sender_id,
                        format!("{:?}", msg.message_type),
                        msg.payload.get("content").and_then(|v| v.as_str()).unwrap_or("")
                    )
                })
                .collect();
            
            Ok(contents)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// 验证DIAP身份连接
    pub async fn verify_diap_connection(&self, identity_id: &str) -> Result<bool> {
        if let Some(manager) = &self.diap_identity_manager {
            use crate::diap::DiapError;
            match manager.verify_identity(identity_id, None).await {
                Ok(auth_result) => Ok(auth_result.authenticated),
                Err(DiapError::AuthenticationFailed(msg)) => {
                    warn!("DIAP身份验证失败: {}", msg);
                    Ok(false)
                }
                Err(e) => Err(anyhow!("DIAP身份验证错误: {}", e)),
            }
        } else {
            Err(anyhow!("DIAP身份管理器未初始化"))
        }
    }
    
    /// 获取连接列表
    pub async fn get_connections(&self) -> Vec<ConnectionStatus> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }
    
    /// 获取活跃连接数
    pub async fn get_active_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.values().filter(|c| c.is_active).count()
    }
    
    /// 心跳检查
    pub async fn heartbeat_check(&self) -> Result<usize> {
        let mut connections = self.connections.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut inactive_count = 0;
        
        for connection in connections.values_mut() {
            let inactive_time = now - connection.last_activity;
            
            if inactive_time > self.config.heartbeat_interval_secs * 3 {
                connection.is_active = false;
                inactive_count += 1;
                
                warn!("💔 连接超时: {} ({}秒未活动)", 
                    connection.peer_id, inactive_time);
            }
        }
        
        // 更新网络状态
        let mut status = self.status.write().await;
        status.active_connections = connections.values().filter(|c| c.is_active).count();
        
        if inactive_count > 0 {
            info!("💓 心跳检查: {} 个连接不活跃", inactive_count);
        }
        
        Ok(inactive_count)
    }
    
    /// 清理不活跃连接
    pub async fn cleanup_inactive_connections(&self, max_inactive_secs: u64) -> Result<usize> {
        let mut connections = self.connections.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let before_count = connections.len();
        
        connections.retain(|_, connection| {
            let inactive_time = now - connection.last_activity;
            inactive_time <= max_inactive_secs
        });
        
        let after_count = connections.len();
        let removed_count = before_count - after_count;
        
        // 更新网络状态
        let mut status = self.status.write().await;
        status.active_connections = connections.values().filter(|c| c.is_active).count();
        status.total_connections = connections.len();
        
        if removed_count > 0 {
            info!("🧹 清理不活跃连接: {} 个", removed_count);
        }
        
        Ok(removed_count)
    }
}
