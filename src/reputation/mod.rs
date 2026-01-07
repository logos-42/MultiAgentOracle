//! 信誉管理系统 - 因果指纹版
//!
//! 基于逻辑一致性的信誉评分系统，用于评估和激励预言机智能体。
//! 核心指标：
//! - causal_credit: 基于逻辑一致性的信用分
//! - outlier_count: 离群次数（逻辑偏离惩罚）
//! - fingerprint_stability: 全局指纹稳定性

mod reputation_score;
mod reputation_manager;
mod reputation_metrics;

// 重新导出
pub use reputation_score::{ReputationScore, ReputationUpdate, ReputationHistory, ReputationTier};
pub use reputation_manager::{ReputationManager, ReputationConfig};
pub use reputation_metrics::{ReputationMetrics};

// 内部模块
pub(crate) mod algorithms;
pub(crate) mod storage;
