//! 消息处理模块
//! 
//! 处理网络消息的接收、解析和分发

use crate::types::{NetworkMessage, NodeId, Timestamp, current_timestamp, SystemError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 消息类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageType {
    /// 心跳消息
    Heartbeat,
    /// 数据提交
    DataSubmission,
    /// 共识投票
    ConsensusVote,
    /// 层级变更
    TierChange,
    /// 错误消息
    Error,
    /// 自定义消息
    Custom(String),
}

/// 消息处理器
pub struct MessageHandler {
    /// 本地节点ID
    local_node_id: NodeId,
    /// 消息处理器映射
    handlers: Arc<RwLock<HashMap<MessageType, Box<dyn MessageHandlerFn + Send + Sync>>>>,
    /// 消息历史
    message_history: Arc<RwLock<Vec<ProcessedMessage>>>,
    /// 消息统计
    message_stats: Arc<RwLock<MessageStats>>,
}

/// 消息处理器函数trait
pub trait MessageHandlerFn: Send + Sync {
    /// 处理消息
    fn handle(&self, message: &NetworkMessage, sender: &NodeId) -> Result<(), String>;
}

// 为闭包实现 MessageHandlerFn trait
impl<F> MessageHandlerFn for F
where
    F: Fn(&NetworkMessage, &NodeId) -> Result<(), String> + Send + Sync,
{
    fn handle(&self, message: &NetworkMessage, sender: &NodeId) -> Result<(), String> {
        self(message, sender)
    }
}

/// 处理后的消息
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    /// 消息ID
    pub message_id: String,
    /// 消息类型
    pub message_type: MessageType,
    /// 发送者
    pub sender: NodeId,
    /// 接收时间
    pub received_at: Timestamp,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 处理结果
    pub result: MessageResult,
    /// 原始消息（摘要）
    pub message_summary: String,
}

/// 消息结果
#[derive(Debug, Clone)]
pub enum MessageResult {
    /// 处理成功
    Success,
    /// 处理失败
    Failure(String),
    /// 忽略的消息
    Ignored,
}

/// 消息统计
#[derive(Debug, Clone, Default)]
pub struct MessageStats {
    /// 总消息数
    pub total_messages: u64,
    /// 成功处理的消息数
    pub successful_messages: u64,
    /// 失败的消息数
    pub failed_messages: u64,
    /// 忽略的消息数
    pub ignored_messages: u64,
    /// 各类型消息统计
    pub type_stats: HashMap<MessageType, TypeStats>,
    /// 平均处理时间（毫秒）
    pub average_processing_time_ms: f64,
}

/// 类型统计
#[derive(Debug, Clone, Default)]
pub struct TypeStats {
    /// 消息数量
    pub count: u64,
    /// 成功数量
    pub success_count: u64,
    /// 失败数量
    pub failure_count: u64,
    /// 总处理时间（毫秒）
    pub total_processing_time_ms: u64,
}

impl MessageHandler {
    /// 创建新的消息处理器
    pub fn new(local_node_id: NodeId) -> Self {
        let mut handler = Self {
            local_node_id,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::new())),
            message_stats: Arc::new(RwLock::new(MessageStats::default())),
        };
        
        // 注册默认处理器
        handler.register_default_handlers();
        
        handler
    }
    
    /// 注册默认处理器
    fn register_default_handlers(&mut self) {
        // 心跳消息处理器
        self.register_handler(MessageType::Heartbeat, Box::new(|message, sender| {
            if let NetworkMessage::Heartbeat { node_id, timestamp } = message {
                println!("💓 收到来自 {} 的心跳消息，时间戳: {}", node_id, timestamp);
                Ok(())
            } else {
                Err("消息类型不匹配".to_string())
            }
        }));
        
        // 数据提交处理器
        self.register_handler(MessageType::DataSubmission, Box::new(|message, sender| {
            if let NetworkMessage::DataSubmission { node_id, data_type, data, signature } = message {
                println!("📊 收到来自 {} 的数据提交: {} (签名: {})", 
                    node_id, data_type, &signature[..10.min(signature.len())]);
                Ok(())
            } else {
                Err("消息类型不匹配".to_string())
            }
        }));
        
        // 共识投票处理器
        self.register_handler(MessageType::ConsensusVote, Box::new(|message, sender| {
            if let NetworkMessage::ConsensusVote { node_id, proposal_id, vote, weight } = message {
                println!("🗳️  收到来自 {} 的共识投票: 提案 {}，投票: {}，权重: {}", 
                    node_id, proposal_id, vote, weight);
                Ok(())
            } else {
                Err("消息类型不匹配".to_string())
            }
        }));
        
        // 层级变更处理器
        self.register_handler(MessageType::TierChange, Box::new(|message, sender| {
            if let NetworkMessage::TierChange { node_id, old_tier, new_tier, reason } = message {
                println!("📈 节点 {} 层级变更: {} -> {}，原因: {}", 
                    node_id, old_tier, new_tier, reason);
                Ok(())
            } else {
                Err("消息类型不匹配".to_string())
            }
        }));
        
        // 错误消息处理器
        self.register_handler(MessageType::Error, Box::new(|message, sender| {
            if let NetworkMessage::Error { code, message: error_msg, details } = message {
                println!("❌ 收到错误消息: 代码 {}，消息: {}", code, error_msg);
                Ok(())
            } else {
                Err("消息类型不匹配".to_string())
            }
        }));
    }
    
    /// 注册消息处理器
    pub fn register_handler(&mut self, message_type: MessageType, handler: Box<dyn MessageHandlerFn + Send + Sync>) {
        let mut handlers = self.handlers.blocking_write();
        handlers.insert(message_type, handler);
    }
    
    /// 处理消息
    pub async fn process_message(&self, message: NetworkMessage, sender: NodeId) -> MessageResult {
        let start_time = std::time::Instant::now();
        let message_id = format!("msg_{}_{}", sender, current_timestamp());
        
        // 确定消息类型
        let message_type = self.determine_message_type(&message);
        let message_summary = self.summarize_message(&message);
        
        println!("📨 处理消息 {} 来自 {}", message_type_to_string(&message_type), sender);
        
        let result = {
            let handlers = self.handlers.read().await;
            
            if let Some(handler) = handlers.get(&message_type) {
                match handler.handle(&message, &sender) {
                    Ok(_) => MessageResult::Success,
                    Err(e) => MessageResult::Failure(e),
                }
            } else {
                println!("⚠️  没有找到 {} 类型的处理器", message_type_to_string(&message_type));
                MessageResult::Ignored
            }
        };
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        // 记录消息历史
        let processed_message = ProcessedMessage {
            message_id,
            message_type: message_type.clone(),
            sender,
            received_at: current_timestamp(),
            processing_time_ms: processing_time,
            result: result.clone(),
            message_summary,
        };
        
        {
            let mut history = self.message_history.write().await;
            history.push(processed_message);
            
            // 限制历史记录大小
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        
        // 更新统计
        self.update_stats(&message_type, &result, processing_time).await;
        
        result
    }
    
    /// 确定消息类型
    fn determine_message_type(&self, message: &NetworkMessage) -> MessageType {
        match message {
            NetworkMessage::Heartbeat { .. } => MessageType::Heartbeat,
            NetworkMessage::DataSubmission { .. } => MessageType::DataSubmission,
            NetworkMessage::ConsensusVote { .. } => MessageType::ConsensusVote,
            NetworkMessage::TierChange { .. } => MessageType::TierChange,
            NetworkMessage::Error { .. } => MessageType::Error,
        }
    }
    
    /// 摘要消息内容
    fn summarize_message(&self, message: &NetworkMessage) -> String {
        match message {
            NetworkMessage::Heartbeat { node_id, timestamp } => {
                format!("心跳 from {} at {}", node_id, timestamp)
            }
            NetworkMessage::DataSubmission { node_id, data_type, data, signature } => {
                format!("数据提交 from {}: {} ({} bytes)", node_id, data_type, data.to_string().len())
            }
            NetworkMessage::ConsensusVote { node_id, proposal_id, vote, weight } => {
                format!("共识投票 from {}: 提案 {}，投票 {}", node_id, proposal_id, vote)
            }
            NetworkMessage::TierChange { node_id, old_tier, new_tier, reason } => {
                format!("层级变更 from {}: {} -> {}，原因: {}", node_id, old_tier, new_tier, reason)
            }
            NetworkMessage::Error { code, message: error_msg, details } => {
                format!("错误: 代码 {}，消息: {}", code, error_msg)
            }
        }
    }
    
    /// 更新统计信息
    async fn update_stats(&self, message_type: &MessageType, result: &MessageResult, processing_time: u64) {
        let mut stats = self.message_stats.write().await;
        
        stats.total_messages += 1;
        
        match result {
            MessageResult::Success => stats.successful_messages += 1,
            MessageResult::Failure(_) => stats.failed_messages += 1,
            MessageResult::Ignored => stats.ignored_messages += 1,
        }
        
        // 更新类型统计
        let type_stats = stats.type_stats.entry(message_type.clone()).or_default();
        type_stats.count += 1;
        
        match result {
            MessageResult::Success => type_stats.success_count += 1,
            MessageResult::Failure(_) => type_stats.failure_count += 1,
            _ => {}
        }
        
        type_stats.total_processing_time_ms += processing_time;
        
        // 更新平均处理时间
        if stats.total_messages > 0 {
            let total_time = stats.type_stats.values().map(|s| s.total_processing_time_ms).sum::<u64>();
            stats.average_processing_time_ms = total_time as f64 / stats.total_messages as f64;
        }
    }
    
    /// 获取消息历史
    pub async fn get_message_history(&self, limit: usize) -> Vec<ProcessedMessage> {
        let history = self.message_history.read().await;
        let start = if history.len() > limit {
            history.len() - limit
        } else {
            0
        };
        
        history[start..].to_vec()
    }
    
    /// 获取消息统计
    pub async fn get_message_stats(&self) -> MessageStats {
        self.message_stats.read().await.clone()
    }
    
    /// 获取处理成功率
    pub async fn get_success_rate(&self) -> f64 {
        let stats = self.message_stats.read().await;
        
        if stats.total_messages == 0 {
            return 0.0;
        }
        
        stats.successful_messages as f64 / stats.total_messages as f64
    }
    
    /// 清理旧的消息历史
    pub async fn cleanup_old_messages(&self, max_age_seconds: u64) -> usize {
        let current_time = current_timestamp();
        let max_age_ms = max_age_seconds * 1000;
        
        let mut history = self.message_history.write().await;
        let initial_count = history.len();
        
        history.retain(|msg| {
            let age = current_time.saturating_sub(msg.received_at);
            age <= max_age_ms
        });
        
        let removed_count = initial_count - history.len();
        if removed_count > 0 {
            println!("🧹 清理了 {} 条旧消息", removed_count);
        }
        
        removed_count
    }
}

/// 将消息类型转换为字符串
fn message_type_to_string(message_type: &MessageType) -> &str {
    match message_type {
        MessageType::Heartbeat => "心跳",
        MessageType::DataSubmission => "数据提交",
        MessageType::ConsensusVote => "共识投票",
        MessageType::TierChange => "层级变更",
        MessageType::Error => "错误",
        MessageType::Custom(name) => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_message_processing() {
        let handler = MessageHandler::new("local_node".to_string());
        
        // 测试心跳消息
        let heartbeat = NetworkMessage::Heartbeat {
            node_id: "node1".to_string(),
            timestamp: current_timestamp(),
        };
        
        let result = handler.process_message(heartbeat, "node1".to_string()).await;
        assert!(matches!(result, MessageResult::Success));
        
        // 测试数据提交消息
        let data_submission = NetworkMessage::DataSubmission {
            node_id: "node2".to_string(),
            data_type: "crypto".to_string(),
            data: serde_json::json!({"price": 45000}),
            signature: "sig123".to_string(),
        };
        
        let result = handler.process_message(data_submission, "node2".to_string()).await;
        assert!(matches!(result, MessageResult::Success));
        
        // 测试获取统计
        let stats = handler.get_message_stats().await;
        assert!(stats.total_messages >= 2);
    }
}
