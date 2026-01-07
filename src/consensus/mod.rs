//! 共识模块 - 因果指纹版
//!
//! 基于因果指纹的共识算法，用于多智能体预言机网络的数据聚合和验证。
//! 核心概念：
//! - 因果指纹：Agent 对随机干预的响应向量 Δy
//! - 谱分析：提取 Agent 逻辑的"骨架"
//! - 余弦聚类：基于逻辑一致性达成共识

mod consensus_engine;
mod consensus_result;
mod voting;
mod aggregation;
mod causal_fingerprint;
mod spectral_analysis;

// 重新导出
pub use consensus_engine::{ConsensusEngine, ConsensusConfig, ConsensusState};
pub use consensus_result::{ConsensusResult, ConsensusStatus, ConsensusError};
pub use voting::{Vote, VotingResult, VotingWeight};
pub use aggregation::{AggregationAlgorithm, AggregationResult};
pub use causal_fingerprint::{
    CausalFingerprint,
    ConsensusResult as CausalConsensusResult,
    CausalFingerprintConfig,
    cosine_similarity,
    cluster_by_consensus,
    detect_model_homogeneity,
    logical_consistency_score,
    extract_spectral_features,
};
pub use spectral_analysis::{
    SpectralFeatures,
    SpectralConfig,
    spectral_distance,
    spectral_similarity,
    is_homogeneous,
    fingerprint_consistency_score,
    features_to_i64,
    i64_to_features,
};

// 内部模块
pub(crate) mod algorithms;
pub(crate) mod validation;
