//! 节点发现模块
//! 
//! 负责发现和管理网络中的其他节点

use crate::types::{NodeId, NodeInfo, Timestamp, current_timestamp};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 节点发现配置
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// 发现间隔（秒）
    pub discovery_interval: u64,
    /// 最大节点数
    pub max_nodes: u32,
    /// 是否启用主动发现
    pub enable_active_discovery: bool,
    /// 是否启用被动发现
    pub enable_passive_discovery: bool,
    /// 发现超时（秒）
    pub discovery_timeout: u64,
}

/// 节点信息
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// 节点ID
    pub node_id: NodeId,
    /// 节点地址
    pub address: String,
    /// 节点层级
    pub tier: String,
    /// 信誉分数
    pub reputation: f64,
    /// 最后发现时间
    pub last_discovered: Timestamp,
    /// 是否在线
    pub online: bool,
    /// 连接次数
    pub connection_count: u32,
}

/// 节点发现器
pub struct PeerDiscovery {
    /// 本地节点ID
    local_node_id: NodeId,
    /// 配置
    config: DiscoveryConfig,
    /// 发现的节点
    discovered_peers: Arc<RwLock<HashMap<NodeId, PeerInfo>>>,
    /// 已知的引导节点
    bootstrap_nodes: Vec<String>,
    /// 发现历史
    discovery_history: Vec<DiscoveryEvent>,
}

/// 发现事件
#[derive(Debug, Clone)]
pub struct DiscoveryEvent {
    /// 事件类型
    pub event_type: DiscoveryEventType,
    /// 节点ID
    pub node_id: NodeId,
    /// 时间戳
    pub timestamp: Timestamp,
    /// 详细信息
    pub details: String,
}

/// 发现事件类型
#[derive(Debug, Clone)]
pub enum DiscoveryEventType {
    /// 节点发现
    NodeDiscovered,
    /// 节点丢失
    NodeLost,
    /// 节点更新
    NodeUpdated,
    /// 发现错误
    DiscoveryError,
}

impl PeerDiscovery {
    /// 创建新的节点发现器
    pub fn new(local_node_id: NodeId, config: DiscoveryConfig) -> Self {
        Self {
            local_node_id,
            config,
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
            bootstrap_nodes: Vec::new(),
            discovery_history: Vec::new(),
        }
    }
    
    /// 添加引导节点
    pub fn add_bootstrap_node(&mut self, address: String) {
        self.bootstrap_nodes.push(address);
    }
    
    /// 开始节点发现
    pub async fn start_discovery(&self) -> Result<(), String> {
        println!("🔍 开始节点发现...");
        
        if self.config.enable_active_discovery {
            self.active_discovery().await?;
        }
        
        if self.config.enable_passive_discovery {
            self.passive_discovery().await?;
        }
        
        Ok(())
    }
    
    /// 主动发现节点
    async fn active_discovery(&self) -> Result<(), String> {
        println!("  主动发现节点...");
        
        // 模拟发现一些节点
        let mock_peers = vec![
            ("node1".to_string(), "127.0.0.1:8081".to_string(), "core".to_string(), 850.0),
            ("node2".to_string(), "127.0.0.1:8082".to_string(), "core".to_string(), 820.0),
            ("node3".to_string(), "127.0.0.1:8083".to_string(), "validator".to_string(), 650.0),
        ];
        
        let mut peers = self.discovered_peers.write().await;
        
        for (node_id, address, tier, reputation) in mock_peers {
            if node_id == self.local_node_id {
                continue; // 跳过本地节点
            }
            
            let peer_info = PeerInfo {
                node_id: node_id.clone(),
                address,
                tier,
                reputation,
                last_discovered: current_timestamp(),
                online: true,
                connection_count: 0,
            };
            
            peers.insert(node_id.clone(), peer_info);
            
            self.record_discovery_event(
                DiscoveryEventType::NodeDiscovered,
                node_id,
                "通过主动发现找到节点".to_string(),
            );
        }
        
        println!("  发现 {} 个节点", peers.len());
        Ok(())
    }
    
    /// 被动发现节点
    async fn passive_discovery(&self) -> Result<(), String> {
        println!("  被动发现节点...");
        // 在实际实现中，这里会监听网络广播和节点公告
        Ok(())
    }
    
    /// 记录发现事件
    fn record_discovery_event(&self, event_type: DiscoveryEventType, node_id: NodeId, details: String) {
        let event = DiscoveryEvent {
            event_type,
            node_id,
            timestamp: current_timestamp(),
            details,
        };
        
        // 在实际实现中，这里会存储到持久化存储
        // 这里只是记录到内存中
        let mut history = self.discovery_history.clone();
        history.push(event);
        
        // 限制历史记录大小
        if history.len() > 1000 {
            history.remove(0);
        }
    }
    
    /// 获取发现的节点
    pub async fn get_discovered_peers(&self) -> Vec<PeerInfo> {
        let peers = self.discovered_peers.read().await;
        peers.values().cloned().collect()
    }
    
    /// 获取指定层级的节点
    pub async fn get_peers_by_tier(&self, tier: &str) -> Vec<PeerInfo> {
        let peers = self.discovered_peers.read().await;
        peers
            .values()
            .filter(|peer| peer.tier == tier)
            .cloned()
            .collect()
    }
    
    /// 更新节点状态
    pub async fn update_peer_status(&self, node_id: &NodeId, online: bool) -> Result<(), String> {
        let mut peers = self.discovered_peers.write().await;
        
        if let Some(peer) = peers.get_mut(node_id) {
            peer.online = online;
            peer.last_discovered = current_timestamp();
            
            let event_type = if online {
                DiscoveryEventType::NodeUpdated
            } else {
                DiscoveryEventType::NodeLost
            };
            
            self.record_discovery_event(
                event_type,
                node_id.clone(),
                format!("节点状态更新为: {}", if online { "在线" } else { "离线" }),
            );
            
            Ok(())
        } else {
            Err(format!("节点 {} 未发现", node_id))
        }
    }
    
    /// 获取在线节点数
    pub async fn get_online_peer_count(&self) -> usize {
        let peers = self.discovered_peers.read().await;
        peers.values().filter(|peer| peer.online).count()
    }
    
    /// 获取发现统计
    pub async fn get_discovery_stats(&self) -> DiscoveryStats {
        let peers = self.discovered_peers.read().await;
        
        let mut tier_distribution = HashMap::new();
        for peer in peers.values() {
            *tier_distribution.entry(peer.tier.clone()).or_insert(0) += 1;
        }
        
        DiscoveryStats {
            total_peers: peers.len(),
            online_peers: peers.values().filter(|p| p.online).count(),
            tier_distribution,
            discovery_events: self.discovery_history.len(),
            last_discovery_time: self.discovery_history.last().map(|e| e.timestamp),
        }
    }
    
    /// 清理过期的节点
    pub async fn cleanup_expired_peers(&self, max_age_seconds: u64) -> usize {
        let current_time = current_timestamp();
        let max_age_ms = max_age_seconds * 1000;
        
        let mut peers = self.discovered_peers.write().await;
        let initial_count = peers.len();
        
        peers.retain(|_, peer| {
            let age = current_time.saturating_sub(peer.last_discovered);
            age <= max_age_ms
        });
        
        let removed_count = initial_count - peers.len();
        if removed_count > 0 {
            println!("  清理了 {} 个过期节点", removed_count);
        }
        
        removed_count
    }
}

/// 发现统计
#[derive(Debug, Clone)]
pub struct DiscoveryStats {
    /// 总节点数
    pub total_peers: usize,
    /// 在线节点数
    pub online_peers: usize,
    /// 层级分布
    pub tier_distribution: HashMap<String, usize>,
    /// 发现事件数
    pub discovery_events: usize,
    /// 最后发现时间
    pub last_discovery_time: Option<Timestamp>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_interval: 30,
            max_nodes: 100,
            enable_active_discovery: true,
            enable_passive_discovery: true,
            discovery_timeout: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_peer_discovery() {
        let config = DiscoveryConfig::default();
        let discovery = PeerDiscovery::new("local_node".to_string(), config);
        
        // 测试开始发现
        let result = discovery.start_discovery().await;
        assert!(result.is_ok());
        
        // 测试获取发现的节点
        let peers = discovery.get_discovered_peers().await;
        assert!(!peers.is_empty());
        
        // 测试获取统计
        let stats = discovery.get_discovery_stats().await;
        assert!(stats.total_peers > 0);
    }
}
