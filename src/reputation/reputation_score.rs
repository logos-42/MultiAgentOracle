//! 信誉分数 - 因果指纹版
//!
//! 基于逻辑一致性的信誉评分，用于评估预言机智能体。
//! 核心指标：
//! - causal_credit: 基于余弦相似度和逻辑一致性的信用分
//! - outlier_count: 离群次数（被判定为逻辑不一致的次数）
//! - fingerprint_stability: 全局指纹稳定性（EMA一致性）
//! - spectral_diversity: 谱多样性（模型异构性）

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};

/// 信誉分数 - 基于因果指纹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    /// 智能体 DID
    pub agent_did: String,
    /// 因果信用分 (0-1000) - 基于逻辑一致性
    pub causal_credit: f64,
    /// 离群次数 (被惩罚次数)
    pub outlier_count: u64,
    /// 总任务数
    pub total_tasks: u64,
    /// 成功任务数 (在共识中)
    pub successful_tasks: u64,
    /// 指纹稳定性分数 (0-1)
    pub fingerprint_stability: f64,
    /// 谱多样性分数 (0-1)
    pub spectral_diversity: f64,
    /// 全局指纹的 EMA (16维)
    pub global_fingerprint_ema: [f64; 16],
    /// 最后更新时间
    pub last_updated: u64,
    /// 创建时间
    pub created_at: u64,
    /// 是否活跃
    pub is_active: bool,
    /// 信誉等级
    pub tier: ReputationTier,
    /// 历史记录
    pub history: Vec<ReputationUpdate>,
}

impl ReputationScore {
    /// 创建新的信誉分数
    pub fn new(agent_did: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let tier = Self::calculate_tier(500.0); // 默认从500开始
        
        Self {
            agent_did,
            causal_credit: 500.0,
            outlier_count: 0,
            total_tasks: 0,
            successful_tasks: 0,
            fingerprint_stability: 1.0,
            spectral_diversity: 1.0,
            global_fingerprint_ema: [0.0; 16],
            last_updated: now,
            created_at: now,
            is_active: true,
            tier,
            history: Vec::new(),
        }
    }
    
    /// 计算成功率（基于共识参与）
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            return 0.0;
        }
        self.successful_tasks as f64 / self.total_tasks as f64
    }
    
    /// 基于逻辑一致性更新信誉
    pub fn update_for_logical_consistency(&mut self, update: ReputationUpdate) {
        let old_credit = self.causal_credit;
        
        // 应用更新
        self.causal_credit += update.delta;
        self.causal_credit = self.causal_credit.clamp(0.0, 1000.0);
        
        // 更新统计
        self.total_tasks += update.total_tasks;
        self.successful_tasks += update.successful_tasks;
        
        // 更新时间和状态
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 更新等级
        self.tier = Self::calculate_tier(self.causal_credit);
        
        // 检查是否应该停用（过多的离群）
        if self.outlier_count > 10 && self.success_rate() < 0.5 {
            self.is_active = false;
        }
        
        // 记录历史
        let mut history_update = update.clone();
        history_update.old_score = old_credit;
        history_update.new_score = self.causal_credit;
        self.history.push(history_update);
        
        // 限制历史记录大小
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }
    
    /// 计算投票权重（基于因果信用和指纹稳定性）
    pub fn voting_weight(&self) -> f64 {
        // 权重 = 因果信用 × 指纹稳定性 × log(任务数 + 1)
        let task_bonus = ((self.total_tasks + 1) as f64).ln();
        self.causal_credit * self.fingerprint_stability * task_bonus / 10.0
    }
    
    /// 计算信誉等级
    fn calculate_tier(score: f64) -> ReputationTier {
        match score {
            s if s >= 900.0 => ReputationTier::Platinum,
            s if s >= 800.0 => ReputationTier::Diamond,
            s if s >= 700.0 => ReputationTier::Gold,
            s if s >= 600.0 => ReputationTier::Silver,
            s if s >= 500.0 => ReputationTier::Bronze,
            s if s >= 400.0 => ReputationTier::Iron,
            s if s >= 300.0 => ReputationTier::Copper,
            _ => ReputationTier::Newbie,
        }
    }
    
    /// 获取格式化时间
    pub fn last_updated_formatted(&self) -> String {
        let dt = DateTime::<Utc>::from_timestamp(self.last_updated as i64, 0)
            .unwrap_or(Utc::now());
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    /// 获取简要信息
    pub fn get_summary(&self) -> ReputationSummary {
        ReputationSummary {
            agent_did: self.agent_did.clone(),
            causal_credit: self.causal_credit,
            tier: self.tier.clone(),
            success_rate: self.success_rate(),
            total_tasks: self.total_tasks,
            fingerprint_stability: self.fingerprint_stability,
            is_active: self.is_active,
            last_updated: self.last_updated_formatted(),
        }
    }
    
    /// 更新全局指纹 (EMA)
    pub fn update_global_fingerprint(&mut self, new_features: &[f64; 16], alpha: f64) {
        for i in 0..16 {
            self.global_fingerprint_ema[i] = 
                self.global_fingerprint_ema[i] * (1.0 - alpha) + new_features[i] * alpha;
        }
        
        // 重新计算稳定性
        self.fingerprint_stability = self.calculate_stability();
    }
    
    /// 计算指纹稳定性
    fn calculate_stability(&self) -> f64 {
        // 稳定性基于 EMA 的方差（低方差 = 高稳定性）
        let mean: f64 = self.global_fingerprint_ema.iter().sum::<f64>() / 16.0;
        let variance: f64 = self.global_fingerprint_ema.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / 16.0;
        
        // 将方差转换为稳定性分数 (0-1)
        let std_dev = variance.sqrt();
        (1.0 / (1.0 + std_dev * 10.0)).clamp(0.0, 1.0)
    }
    
    /// 是否活跃 (添加这个方法以解决 oracle_agent.rs 中的编译错误)
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

/// 信誉更新记录 - 因果指纹版
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationUpdate {
    /// 更新原因
    pub reason: UpdateReason,
    /// 变化量
    pub delta: f64,
    /// 总任务数
    pub total_tasks: u64,
    /// 成功任务数
    pub successful_tasks: u64,
    /// 旧分数
    pub old_score: f64,
    /// 新分数
    pub new_score: f64,
    /// 时间戳
    pub timestamp: u64,
    /// 相关数据ID
    pub data_id: Option<String>,
    /// 备注
    pub note: Option<String>,
}

impl ReputationUpdate {
    /// 创建新的更新记录
    pub fn new(
        reason: UpdateReason,
        delta: f64,
        total_tasks: u64,
        successful_tasks: u64,
        data_id: Option<String>,
        note: Option<String>,
    ) -> Self {
        Self {
            reason,
            delta,
            total_tasks,
            successful_tasks,
            old_score: 0.0,
            new_score: 0.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data_id,
            note,
        }
    }
}

/// 更新原因 - 因果指纹版
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateReason {
    /// 逻辑一致性（正）
    LogicalConsistency {
        cosine_similarity: f64,
        cluster_position: usize,
    },
    /// 逻辑一致性（负 - 离群）
    LogicalInconsistency {
        cosine_similarity: f64,
        is_outlier: bool,
    },
    /// 谱一致性（正）
    SpectralConsistency {
        consistency_score: f64,
    },
    /// 谱发散（负）
    SpectralDivergence {
        global_distance: f64,
    },
    /// 逻辑同质性（检测到供应商一致）
    LogicHomogeneity {
        cluster_size: usize,
        penalty_applied: bool,
    },
    /// 指纹稳定性更新
    FingerprintStability {
        old_stability: f64,
        new_stability: f64,
    },
    /// 质押变化
    StakeChange {
        old_amount: u64,
        new_amount: u64,
    },
    /// 手动调整
    ManualAdjustment {
        admin: String,
        reason: String,
    },
    /// 惩罚
    Penalty {
        reason: String,
        severity: PenaltySeverity,
    },
    /// 奖励
    Reward {
        reason: String,
        amount: f64,
    },
}

/// 惩罚严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PenaltySeverity {
    Minor,      // 轻微离群
    Moderate,   // 频繁离群
    Severe,     // 恶意攻击
    Malicious,  // 供应商一致攻击
}

/// 信誉等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReputationTier {
    /// 新手 (0-299)
    Newbie,
    /// 铜牌 (300-399)
    Copper,
    /// 铁牌 (400-499)
    Iron,
    /// 铜牌 (500-599)
    Bronze,
    /// 银牌 (600-699)
    Silver,
    /// 金牌 (700-799)
    Gold,
    /// 钻石 (800-899)
    Diamond,
    /// 白金 (900-1000)
    Platinum,
}

impl ReputationTier {
    /// 获取等级名称
    pub fn name(&self) -> &str {
        match self {
            Self::Newbie => "新手",
            Self::Copper => "铜牌",
            Self::Iron => "铁牌",
            Self::Bronze => "铜牌",
            Self::Silver => "银牌",
            Self::Gold => "金牌",
            Self::Diamond => "钻石",
            Self::Platinum => "白金",
        }
    }
    
    /// 获取等级颜色
    pub fn color(&self) -> &str {
        match self {
            Self::Newbie => "#808080",
            Self::Copper => "#B87333",
            Self::Iron => "#A19D94",
            Self::Bronze => "#CD7F32",
            Self::Silver => "#C0C0C0",
            Self::Gold => "#FFD700",
            Self::Diamond => "#B9F2FF",
            Self::Platinum => "#E5E4E2",
        }
    }
}

/// 信誉历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationHistory {
    pub agent_did: String,
    pub updates: Vec<ReputationUpdate>,
    pub start_time: u64,
    pub end_time: u64,
}

/// 信誉摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSummary {
    pub agent_did: String,
    pub causal_credit: f64,
    pub tier: ReputationTier,
    pub success_rate: f64,
    pub total_tasks: u64,
    pub fingerprint_stability: f64,
    pub is_active: bool,
    pub last_updated: String,
}

// 为 ReputationTier 实现 Display trait
impl std::fmt::Display for ReputationTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}