//! 信誉管理系统
//!
//! 基于数据准确性的动态信誉评分系统，用于评估和激励预言机智能体。

mod reputation_score;
mod reputation_manager;
mod reputation_metrics;

// 重新导出
pub use reputation_score::{ReputationScore, ReputationUpdate, ReputationHistory};
pub use reputation_manager::{ReputationManager, ReputationConfig};
pub use reputation_metrics::{ReputationMetrics, PerformanceMetrics};

// 内部模块
pub(crate) mod algorithms;
pub(crate) mod storage;
