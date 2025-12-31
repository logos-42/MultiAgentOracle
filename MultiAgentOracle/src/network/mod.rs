//! 网络模块
//!
//! P2P通信和节点发现系统，基于libp2p和Iroh实现。

mod network_manager;
mod peer_discovery;
mod message_handler;
mod protocol;

// 重新导出
pub use network_manager::{NetworkManager, NetworkConfig, NetworkStatus};
pub use peer_discovery::{PeerDiscovery, PeerInfo, DiscoveryConfig};
pub use message_handler::{MessageHandler, MessageType};
pub use crate::types::NetworkMessage;
pub use protocol::{Protocol, ProtocolConfig, ProtocolMessage};

// 内部模块
pub(crate) mod transport;
pub(crate) mod security;
pub(crate) mod routing;
