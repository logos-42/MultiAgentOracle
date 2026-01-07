use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};

/// 信誉分数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    /// 智能体DID
    pub agent_did: String,
    /// 当前信誉分（0-1000）
    pub score: f64,
    /// 质押金额
    pub staked_amount: u64,
    /// 总服务次数
    pub total_services: u64,
    /// 成功服务次数
    pub successful_services: u64,
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
    pub fn new(agent_did: String, initial_score: f64, staked_amount: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let tier = Self::calculate_tier(initial_score);
        
        Self {
            agent_did,
            score: initial_score.clamp(0.0, 1000.0),
            staked_amount,
            total_services: 0,
            successful_services: 0,
            last_updated: now,
            created_at: now,
            is_active: true,
            tier,
            history: Vec::new(),
        }
    }
    
    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_services == 0 {
            return 0.0;
        }
        self.successful_services as f64 / self.total_services as f64
    }
    
    /// 更新信誉分
    pub fn update(&mut self, update: ReputationUpdate) {
        let old_score = self.score;
        
        // 应用更新
        self.score += update.delta;
        self.score = self.score.clamp(0.0, 1000.0);
        
        // 更新统计
        self.total_services += update.total_services;
        self.successful_services += update.successful_services;
        
        // 更新时间和状态
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 更新等级
        self.tier = Self::calculate_tier(self.score);
        
        // 检查是否应该停用
        if self.score < 10.0 {
            self.is_active = false;
        }
        
        // 记录历史
        let mut history_update = update.clone();
        history_update.old_score = old_score;
        history_update.new_score = self.score;
        self.history.push(history_update);
        
        // 限制历史记录大小
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }
    
    /// 增加质押
    pub fn stake(&mut self, amount: u64) {
        self.staked_amount += amount;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// 减少质押
    pub fn unstake(&mut self, amount: u64) -> Result<(), String> {
        if amount > self.staked_amount {
            return Err("质押金额不足".to_string());
        }
        self.staked_amount -= amount;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(())
    }
    
    /// 计算投票权重
    pub fn voting_weight(&self) -> f64 {
        // 权重 = 信誉分 × sqrt(质押金额)
        // 这样既考虑信誉又考虑经济承诺，但质押的影响是次线性的
        self.score * (self.staked_amount as f64).sqrt()
    }
    
    /// 计算信誉等级
    fn calculate_tier(score: f64) -> ReputationTier {
        match score {
            s if s >= 900.0 => ReputationTier::Platinum,
            s if s >= 800.0 => ReputationTier::Diamond,
            s if s >= 700.0 => ReputationTier::Gold,
            s if s >= 600.0 => ReputationTier::Silver,
            s if s >= 500.0 => ReputationTier::Bronze,
            s if s >= 300.0 => ReputationTier::Iron,
            s if s >= 100.0 => ReputationTier::Copper,
            _ => ReputationTier::Newbie,
        }
    }
    
    /// 获取格式化时间
    pub fn last_updated_formatted(&self) -> String {
        let dt = DateTime::<Utc>::from_timestamp(self.last_updated as i64, 0)
            .unwrap_or(Utc::now());
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    /// 获取创建时间格式化
    pub fn created_at_formatted(&self) -> String {
        let dt = DateTime::<Utc>::from_timestamp(self.created_at as i64, 0)
            .unwrap_or(Utc::now());
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    /// 获取简要信息
    pub fn get_summary(&self) -> ReputationSummary {
        ReputationSummary {
            agent_did: self.agent_did.clone(),
            score: self.score,
            tier: self.tier.clone(),
            success_rate: self.success_rate(),
            total_services: self.total_services,
            staked_amount: self.staked_amount,
            is_active: self.is_active,
            last_updated: self.last_updated_formatted(),
        }
    }
}

/// 信誉更新记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationUpdate {
    /// 更新原因
    pub reason: UpdateReason,
    /// 变化量
    pub delta: f64,
    /// 总服务次数
    pub total_services: u64,
    /// 成功服务次数
    pub successful_services: u64,
    /// 旧分数（更新后填充）
    pub old_score: f64,
    /// 新分数（更新后填充）
    pub new_score: f64,
    /// 时间戳
    pub timestamp: u64,
    /// 相关数据ID（可选）
    pub data_id: Option<String>,
    /// 备注
    pub note: Option<String>,
}

impl ReputationUpdate {
    /// 创建新的更新记录
    pub fn new(
        reason: UpdateReason,
        delta: f64,
        total_services: u64,
        successful_services: u64,
        data_id: Option<String>,
        note: Option<String>,
    ) -> Self {
        Self {
            reason,
            delta,
            total_services,
            successful_services,
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

/// 更新原因
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateReason {
    /// 数据准确性
    DataAccuracy {
        expected: f64,
        actual: f64,
        tolerance: f64,
    },
    /// 响应时间
    ResponseTime {
        expected_ms: u64,
        actual_ms: u64,
    },
    /// 服务可用性
    ServiceAvailability {
        expected_uptime: f64,
        actual_uptime: f64,
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
    /// 轻微违规
    Minor,
    /// 一般违规
    Moderate,
    /// 严重违规
    Severe,
    /// 恶意行为
    Malicious,
}

/// 信誉等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReputationTier {
    /// 新手 (0-99)
    Newbie,
    /// 铜牌 (100-299)
    Copper,
    /// 铁牌 (300-499)
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
    
    /// 获取等级颜色（用于UI）
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
    
    /// 获取最小分数
    pub fn min_score(&self) -> f64 {
        match self {
            Self::Newbie => 0.0,
            Self::Copper => 100.0,
            Self::Iron => 300.0,
            Self::Bronze => 500.0,
            Self::Silver => 600.0,
            Self::Gold => 700.0,
            Self::Diamond => 800.0,
            Self::Platinum => 900.0,
        }
    }
    
    /// 获取最大分数
    pub fn max_score(&self) -> f64 {
        match self {
            Self::Newbie => 99.9,
            Self::Copper => 299.9,
            Self::Iron => 499.9,
            Self::Bronze => 599.9,
            Self::Silver => 699.9,
            Self::Gold => 799.9,
            Self::Diamond => 899.9,
            Self::Platinum => 1000.0,
        }
    }
}

/// 信誉历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationHistory {
    /// 智能体DID
    pub agent_did: String,
    /// 历史记录
    pub updates: Vec<ReputationUpdate>,
    /// 开始时间
    pub start_time: u64,
    /// 结束时间
    pub end_time: u64,
}

/// 信誉摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSummary {
    /// 智能体DID
    pub agent_did: String,
    /// 当前分数
    pub score: f64,
    /// 信誉等级
    pub tier: ReputationTier,
    /// 成功率
    pub success_rate: f64,
    /// 总服务次数
    pub total_services: u64,
    /// 质押金额
    pub staked_amount: u64,
    /// 是否活跃
    pub is_active: bool,
    /// 最后更新时间
    pub last_updated: String,
}
