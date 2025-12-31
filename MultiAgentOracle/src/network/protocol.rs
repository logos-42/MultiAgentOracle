//! 网络协议模块
//! 
//! 定义网络通信协议和消息格式

use crate::types::{NetworkMessage, NodeId, Timestamp};
use serde::{Deserialize, Serialize};

/// 协议配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// 协议版本
    pub version: String,
    /// 消息编码格式
    pub encoding: EncodingFormat,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 最大消息大小（字节）
    pub max_message_size: usize,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
}

/// 编码格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncodingFormat {
    /// JSON编码
    Json,
    /// MessagePack编码
    MessagePack,
    /// Bincode编码
    Bincode,
    /// Protobuf编码
    Protobuf,
}

/// 协议消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// 消息ID
    pub message_id: String,
    /// 发送者节点ID
    pub sender_id: NodeId,
    /// 接收者节点ID（可选，广播消息为空）
    pub receiver_id: Option<NodeId>,
    /// 消息类型
    pub message_type: MessageType,
    /// 消息体
    pub payload: Vec<u8>,
    /// 时间戳
    pub timestamp: Timestamp,
    /// 消息签名
    pub signature: Option<String>,
    /// TTL（生存时间，秒）
    pub ttl: Option<u64>,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 心跳消息
    Heartbeat,
    /// 数据消息
    Data,
    /// 控制消息
    Control,
    /// 错误消息
    Error,
    /// 自定义消息
    Custom(String),
}

/// 协议处理器
pub struct Protocol {
    /// 配置
    config: ProtocolConfig,
    /// 本地节点ID
    local_node_id: NodeId,
}

impl Protocol {
    /// 创建新的协议处理器
    pub fn new(config: ProtocolConfig, local_node_id: NodeId) -> Self {
        Self {
            config,
            local_node_id,
        }
    }
    
    /// 获取协议名称
    pub fn name(&self) -> &str {
        "multi-agent-oracle-protocol"
    }
    
    /// 编码消息
    pub fn encode_message(&self, message: &NetworkMessage, receiver_id: Option<&NodeId>) -> Result<ProtocolMessage, String> {
        let message_id = format!("msg_{}_{}", self.local_node_id, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        
        // 确定消息类型
        let message_type = match message {
            NetworkMessage::Heartbeat { .. } => MessageType::Heartbeat,
            NetworkMessage::DataSubmission { .. } => MessageType::Data,
            NetworkMessage::ConsensusVote { .. } => MessageType::Control,
            NetworkMessage::TierChange { .. } => MessageType::Control,
            NetworkMessage::Error { .. } => MessageType::Error,
        };
        
        // 编码消息体
        let payload = match self.config.encoding {
            EncodingFormat::Json => {
                serde_json::to_vec(message).map_err(|e| format!("JSON编码失败: {}", e))?
            }
            EncodingFormat::MessagePack => {
                rmp_serde::to_vec(message).map_err(|e| format!("MessagePack编码失败: {}", e))?
            }
            EncodingFormat::Bincode => {
                bincode::serialize(message).map_err(|e| format!("Bincode编码失败: {}", e))?
            }
            EncodingFormat::Protobuf => {
                // 在实际实现中，这里会使用protobuf编码
                serde_json::to_vec(message).map_err(|e| format!("编码失败: {}", e))?
            }
        };
        
        // 检查消息大小
        if payload.len() > self.config.max_message_size {
            return Err(format!("消息大小 {} 超过限制 {}", payload.len(), self.config.max_message_size));
        }
        
        let protocol_message = ProtocolMessage {
            message_id,
            sender_id: self.local_node_id.clone(),
            receiver_id: receiver_id.cloned(),
            message_type,
            payload,
            timestamp: chrono::Utc::now().timestamp() as u64,
            signature: None, // 在实际实现中，这里会添加签名
            ttl: Some(300), // 默认5分钟TTL
        };
        
        Ok(protocol_message)
    }
    
    /// 解码消息
    pub fn decode_message(&self, protocol_message: &ProtocolMessage) -> Result<NetworkMessage, String> {
        // 检查消息TTL
        if let Some(ttl) = protocol_message.ttl {
            let current_time = chrono::Utc::now().timestamp() as u64;
            let message_age = current_time.saturating_sub(protocol_message.timestamp);
            
            if message_age > ttl {
                return Err(format!("消息已过期: 年龄 {} 秒, TTL {} 秒", message_age, ttl));
            }
        }
        
        // 解码消息体
        let message = match self.config.encoding {
            EncodingFormat::Json => {
                serde_json::from_slice(&protocol_message.payload)
                    .map_err(|e| format!("JSON解码失败: {}", e))?
            }
            EncodingFormat::MessagePack => {
                rmp_serde::from_slice(&protocol_message.payload)
                    .map_err(|e| format!("MessagePack解码失败: {}", e))?
            }
            EncodingFormat::Bincode => {
                bincode::deserialize(&protocol_message.payload)
                    .map_err(|e| format!("Bincode解码失败: {}", e))?
            }
            EncodingFormat::Protobuf => {
                // 在实际实现中，这里会使用protobuf解码
                serde_json::from_slice(&protocol_message.payload)
                    .map_err(|e| format!("解码失败: {}", e))?
            }
        };
        
        Ok(message)
    }
    
    /// 创建心跳消息
    pub fn create_heartbeat_message(&self) -> ProtocolMessage {
        let heartbeat = NetworkMessage::Heartbeat {
            node_id: self.local_node_id.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        self.encode_message(&heartbeat, None).unwrap_or_else(|e| {
            // 如果编码失败，返回一个简单的心跳消息
            ProtocolMessage {
                message_id: format!("hb_{}_{}", self.local_node_id, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                sender_id: self.local_node_id.clone(),
                receiver_id: None,
                message_type: MessageType::Heartbeat,
                payload: b"heartbeat".to_vec(),
                timestamp: chrono::Utc::now().timestamp() as u64,
                signature: None,
                ttl: Some(60), // 心跳消息TTL较短
            }
        })
    }
    
    /// 验证消息签名
    pub fn verify_signature(&self, message: &ProtocolMessage) -> bool {
        // 在实际实现中，这里会验证消息签名
        // 目前返回true表示验证通过
        true
    }
    
    /// 获取协议版本
    pub fn version(&self) -> &str {
        &self.config.version
    }
    
    /// 检查消息是否来自可信来源
    pub fn is_trusted_source(&self, sender_id: &NodeId) -> bool {
        // 在实际实现中，这里会检查发送者是否在可信列表中
        // 目前返回true表示可信
        true
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            encoding: EncodingFormat::Json,
            enable_compression: true,
            max_message_size: 1024 * 1024, // 1MB
            heartbeat_interval: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_encoding() {
        let config = ProtocolConfig::default();
        let protocol = Protocol::new(config, "test_node".to_string());
        
        // 测试心跳消息编码
        let heartbeat = NetworkMessage::Heartbeat {
            node_id: "test_node".to_string(),
            timestamp: 1234567890,
        };
        
        let encoded = protocol.encode_message(&heartbeat, None);
        assert!(encoded.is_ok());
        
        let protocol_message = encoded.unwrap();
        assert_eq!(protocol_message.sender_id, "test_node");
        assert_eq!(protocol_message.message_type, MessageType::Heartbeat);
        
        // 测试消息解码
        let decoded = protocol.decode_message(&protocol_message);
        assert!(decoded.is_ok());
        
        if let Ok(NetworkMessage::Heartbeat { node_id, timestamp }) = decoded {
            assert_eq!(node_id, "test_node");
            assert_eq!(timestamp, 1234567890);
        } else {
            panic!("解码消息类型不正确");
        }
    }
    
    #[test]
    fn test_heartbeat_message() {
        let config = ProtocolConfig::default();
        let protocol = Protocol::new(config, "node1".to_string());
        
        let heartbeat_msg = protocol.create_heartbeat_message();
        
        assert_eq!(heartbeat_msg.sender_id, "node1");
        assert_eq!(heartbeat_msg.message_type, MessageType::Heartbeat);
        assert!(heartbeat_msg.ttl.is_some());
    }
}
