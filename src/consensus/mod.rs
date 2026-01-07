//! 共识模块
//!
//! 信誉加权共识算法，用于多智能体预言机网络的数据聚合和验证。

mod consensus_engine;
mod consensus_result;
mod voting;
mod aggregation;

// 重新导出
pub use consensus_engine::{ConsensusEngine, ConsensusConfig, ConsensusState};
pub use consensus_result::{ConsensusResult, ConsensusStatus, ConsensusError};
pub use voting::{Vote, VotingResult, VotingWeight};
pub use aggregation::{AggregationAlgorithm, AggregationResult};

// 内部模块
pub(crate) mod algorithms;
pub(crate) mod validation;
