//! 网络路由模块
//! 
//! 负责消息的路由和转发，支持分层网络拓扑

use crate::types::{NodeId, NetworkMessage};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 路由配置
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    /// 是否启用路由
    pub enable_routing: bool,
    /// 最大跳数
    pub max_hops: u32,
    /// 路由表更新间隔（秒）
    pub routing_table_update_interval: u64,
    /// 是否启用分层路由
    pub enable_hierarchical_routing: bool,
    /// 路由算法
    pub routing_algorithm: RoutingAlgorithm,
}

/// 路由算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingAlgorithm {
    /// 距离向量路由
    DistanceVector,
    /// 链路状态路由
    LinkState,
    /// 分层路由
    Hierarchical,
    /// 洪泛路由
    Flooding,
}

/// 路由表项
#[derive(Debug, Clone)]
pub struct RoutingEntry {
    /// 目标节点ID
    pub destination: NodeId,
    /// 下一跳节点ID
    pub next_hop: NodeId,
    /// 跳数
    pub hops: u32,
    /// 路径成本
    pub cost: f64,
    /// 最后更新时间
    pub last_updated: std::time::SystemTime,
    /// 是否有效
    pub valid: bool,
}

/// 路由表
#[derive(Debug, Clone, Default)]
pub struct RoutingTable {
    /// 路由条目
    entries: HashMap<NodeId, RoutingEntry>,
    /// 本地节点ID
    local_node_id: NodeId,
    /// 邻居节点
    neighbors: HashSet<NodeId>,
}

/// 路由管理器
pub struct RoutingManager {
    /// 配置
    config: RoutingConfig,
    /// 本地节点ID
    local_node_id: NodeId,
    /// 路由表
    routing_table: Arc<RwLock<RoutingTable>>,
    /// 消息队列
    message_queue: Arc<RwLock<VecDeque<QueuedMessage>>>,
    /// 路由统计
    stats: Arc<RwLock<RoutingStats>>,
}

/// 排队消息
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    /// 消息
    pub message: NetworkMessage,
    /// 目标节点ID
    pub destination: NodeId,
    /// 源节点ID
    pub source: NodeId,
    /// 当前跳数
    pub current_hops: u32,
    /// 入队时间
    pub enqueued_at: std::time::SystemTime,
}

/// 路由统计
#[derive(Debug, Clone, Default)]
pub struct RoutingStats {
    /// 总路由消息数
    pub total_routed_messages: u64,
    /// 成功路由消息数
    pub successful_routes: u64,
    /// 失败路由消息数
    pub failed_routes: u64,
    /// 平均路由延迟（毫秒）
    pub average_routing_delay_ms: f64,
    /// 路由表大小
    pub routing_table_size: usize,
    /// 邻居节点数
    pub neighbor_count: usize,
}

impl RoutingTable {
    /// 创建新的路由表
    pub fn new(local_node_id: NodeId) -> Self {
        Self {
            entries: HashMap::new(),
            local_node_id,
            neighbors: HashSet::new(),
        }
    }
    
    /// 添加路由条目
    pub fn add_entry(&mut self, destination: NodeId, next_hop: NodeId, hops: u32, cost: f64) {
        let entry = RoutingEntry {
            destination: destination.clone(),
            next_hop,
            hops,
            cost,
            last_updated: std::time::SystemTime::now(),
            valid: true,
        };
        
        self.entries.insert(destination, entry);
    }
    
    /// 更新路由条目
    pub fn update_entry(&mut self, destination: &NodeId, next_hop: NodeId, hops: u32, cost: f64) -> bool {
        if let Some(entry) = self.entries.get_mut(destination) {
            entry.next_hop = next_hop;
            entry.hops = hops;
            entry.cost = cost;
            entry.last_updated = std::time::SystemTime::now();
            entry.valid = true;
            true
        } else {
            false
        }
    }
    
    /// 获取到目标节点的路由
    pub fn get_route(&self, destination: &NodeId) -> Option<&RoutingEntry> {
        self.entries.get(destination)
    }
    
    /// 添加邻居节点
    pub fn add_neighbor(&mut self, node_id: NodeId) {
        let node_id_clone = node_id.clone();
        self.neighbors.insert(node_id_clone);
        
        // 为邻居添加直接路由
        self.add_entry(node_id.clone(), node_id, 1, 1.0);
    }
    
    /// 移除邻居节点
    pub fn remove_neighbor(&mut self, node_id: &NodeId) {
        self.neighbors.remove(node_id);
        
        // 移除相关路由条目
        self.entries.retain(|dest, _| dest != node_id);
        
        // 移除通过该邻居的路由
        self.entries.retain(|_, entry| entry.next_hop != *node_id);
    }
    
    /// 获取所有路由条目
    pub fn get_all_entries(&self) -> Vec<&RoutingEntry> {
        self.entries.values().collect()
    }
    
    /// 获取邻居节点
    pub fn get_neighbors(&self) -> &HashSet<NodeId> {
        &self.neighbors
    }
    
    /// 清理过期路由
    pub fn cleanup_expired_routes(&mut self, max_age_seconds: u64) -> usize {
        let initial_count = self.entries.len();
        let now = std::time::SystemTime::now();
        
        self.entries.retain(|_, entry| {
            if let Ok(duration) = now.duration_since(entry.last_updated) {
                duration.as_secs() <= max_age_seconds
            } else {
                true // 如果时间计算失败，保留路由
            }
        });
        
        initial_count - self.entries.len()
    }
}

impl RoutingManager {
    /// 创建新的路由管理器
    pub fn new(config: RoutingConfig, local_node_id: NodeId) -> Self {
        Self {
            config,
            local_node_id: local_node_id.clone(),
            routing_table: Arc::new(RwLock::new(RoutingTable::new(local_node_id))),
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(RoutingStats::default())),
        }
    }
    
    /// 添加邻居节点
    pub async fn add_neighbor(&self, node_id: NodeId) {
        let mut routing_table = self.routing_table.write().await;
        routing_table.add_neighbor(node_id.clone());
        
        let mut stats = self.stats.write().await;
        stats.neighbor_count = routing_table.neighbors.len();
        stats.routing_table_size = routing_table.entries.len();
        
        println!("👥 添加邻居节点: {}", node_id);
    }
    
    /// 路由消息
    pub async fn route_message(&self, message: NetworkMessage, destination: NodeId, source: NodeId) -> Result<NodeId, String> {
        if !self.config.enable_routing {
            return Err("路由功能未启用".to_string());
        }
        
        println!("🛣️  路由消息到节点 {}", destination);
        
        let start_time = std::time::Instant::now();
        
        // 检查是否是本地节点
        if destination == self.local_node_id {
            println!("📍 消息目标为本地节点");
            
            let mut stats = self.stats.write().await;
            stats.total_routed_messages += 1;
            stats.successful_routes += 1;
            
            return Ok(self.local_node_id.clone());
        }
        
        let routing_table = self.routing_table.read().await;
        
        // 查找路由
        if let Some(route) = routing_table.get_route(&destination) {
            if route.hops > self.config.max_hops {
                let mut stats = self.stats.write().await;
                stats.total_routed_messages += 1;
                stats.failed_routes += 1;
                
                return Err(format!("跳数超过限制: {} > {}", route.hops, self.config.max_hops));
            }
            
            let next_hop = route.next_hop.clone();
            
            // 更新统计
            let routing_delay = start_time.elapsed().as_millis() as f64;
            let mut stats = self.stats.write().await;
            stats.total_routed_messages += 1;
            stats.successful_routes += 1;
            
            // 更新平均路由延迟
            let total_delay = stats.average_routing_delay_ms * (stats.successful_routes - 1) as f64;
            stats.average_routing_delay_ms = (total_delay + routing_delay) / stats.successful_routes as f64;
            
            println!("✅ 找到路由: {} -> {} (跳数: {}, 成本: {:.2})", 
                self.local_node_id, next_hop, route.hops, route.cost);
            
            Ok(next_hop)
        } else {
            // 没有找到路由
            let mut stats = self.stats.write().await;
            stats.total_routed_messages += 1;
            stats.failed_routes += 1;
            
            println!("❌ 未找到到节点 {} 的路由", destination);
            
            // 根据配置决定是否排队消息
            if self.config.routing_algorithm == RoutingAlgorithm::Flooding {
                self.queue_message_for_flooding(message, destination, source).await;
                Ok(self.local_node_id.clone()) // 返回本地节点表示已处理
            } else {
                Err("未找到路由".to_string())
            }
        }
    }
    
    /// 为洪泛路由排队消息
    async fn queue_message_for_flooding(&self, message: NetworkMessage, destination: NodeId, source: NodeId) {
        let queued_message = QueuedMessage {
            message,
            destination,
            source,
            current_hops: 0,
            enqueued_at: std::time::SystemTime::now(),
        };
        
        let mut message_queue = self.message_queue.write().await;
        message_queue.push_back(queued_message);
        
        println!("📥 消息已排队等待洪泛路由");
    }
    
    /// 处理排队消息
    pub async fn process_queued_messages(&self) -> usize {
        let mut message_queue = self.message_queue.write().await;
        let initial_count = message_queue.len();
        
        if initial_count == 0 {
            return 0;
        }
        
        println!("🔄 处理 {} 个排队消息", initial_count);
        
        let mut processed_count = 0;
        let mut new_queue = VecDeque::new();
        
        while let Some(mut queued_message) = message_queue.pop_front() {
            // 检查跳数限制
            if queued_message.current_hops >= self.config.max_hops {
                println!("⚠️  消息跳数超过限制，丢弃");
                continue;
            }
            
            // 尝试重新路由
            let result = self.route_message(
                queued_message.message.clone(),
                queued_message.destination.clone(),
                queued_message.source.clone(),
            ).await;
            
            match result {
                Ok(next_hop) => {
                    if next_hop == self.local_node_id {
                        // 消息已到达目标或无法路由
                        processed_count += 1;
                    } else {
                        // 需要继续路由，增加跳数并重新排队
                        queued_message.current_hops += 1;
                        new_queue.push_back(queued_message);
                    }
                }
                Err(_) => {
                    // 路由失败，保留在队列中
                    new_queue.push_back(queued_message);
                }
            }
        }
        
        // 更新队列
        *message_queue = new_queue;
        
        println!("✅ 处理了 {} 个排队消息，剩余 {} 个", processed_count, message_queue.len());
        processed_count
    }
    
    /// 更新路由表（距离向量算法）
    pub async fn update_routing_table_distance_vector(&self, updates: Vec<(NodeId, NodeId, u32, f64)>) {
        let mut routing_table = self.routing_table.write().await;
        
        for (destination, next_hop, hops, cost) in updates {
            if let Some(existing_entry) = routing_table.entries.get(&destination) {
                // 如果新路由更好，则更新
                if hops < existing_entry.hops || cost < existing_entry.cost {
                    routing_table.update_entry(&destination, next_hop, hops, cost);
                }
            } else {
                // 添加新路由
                routing_table.add_entry(destination, next_hop, hops, cost);
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.routing_table_size = routing_table.entries.len();
        
        println!("📊 路由表已更新，当前大小: {}", routing_table.entries.len());
    }
    
    /// 获取路由表
    pub async fn get_routing_table(&self) -> RoutingTable {
        self.routing_table.read().await.clone()
    }
    
    /// 获取路由统计
    pub async fn get_stats(&self) -> RoutingStats {
        self.stats.read().await.clone()
    }
    
    /// 清理过期路由
    pub async fn cleanup_expired_routes(&self, max_age_seconds: u64) -> usize {
        let mut routing_table = self.routing_table.write().await;
        let removed_count = routing_table.cleanup_expired_routes(max_age_seconds);
        
        if removed_count > 0 {
            println!("🧹 清理了 {} 个过期路由", removed_count);
            
            let mut stats = self.stats.write().await;
            stats.routing_table_size = routing_table.entries.len();
        }
        
        removed_count
    }
    
    /// 获取下一跳节点（用于分层路由）
    pub async fn get_next_hop_for_tier(&self, destination_tier: &str) -> Option<NodeId> {
        let routing_table = self.routing_table.read().await;
        
        // 在实际实现中，这里会根据层级选择下一跳
        // 目前返回第一个邻居节点
        routing_table.neighbors.iter().next().cloned()
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            enable_routing: true,
            max_hops: 10,
            routing_table_update_interval: 30,
            enable_hierarchical_routing: true,
            routing_algorithm: RoutingAlgorithm::Hierarchical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_routing_manager() {
        let config = RoutingConfig::default();
        let manager = RoutingManager::new(config, "local_node".to_string());
        
        // 测试添加邻居
        manager.add_neighbor("node1".to_string()).await;
        manager.add_neighbor("node2".to_string()).await;
        
        // 测试获取路由表
        let routing_table = manager.get_routing_table().await;
        assert_eq!(routing_table.neighbors.len(), 2);
        
        // 测试路由消息（应该失败，因为没有到目标的路由）
        let message = NetworkMessage::Heartbeat {
            node_id: "local_node".to_string(),
            timestamp: 1234567890,
        };
        
        let result = manager.route_message(message, "target_node".to_string(), "local_node".to_string()).await;
        assert!(result.is_err());
        
        // 测试更新路由表
        let updates = vec![
            ("target_node".to_string(), "node1".to_string(), 2, 1.5),
        ];
        
        manager.update_routing_table_distance_vector(updates).await;
        
        // 现在应该能路由到目标节点
        let message2 = NetworkMessage::Heartbeat {
            node_id: "local_node".to_string(),
            timestamp: 1234567891,
        };
        
        let result = manager.route_message(message2, "target_node".to_string(), "local_node".to_string()).await;
        assert!(result.is_ok());
        
        // 测试获取统计
        let stats = manager.get_stats().await;
        assert!(stats.total_routed_messages >= 2);
    }
}
