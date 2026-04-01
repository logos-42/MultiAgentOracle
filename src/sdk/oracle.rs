//! 核心 Oracle 类
//!
//! 提供高级 API 用于发起查询、获取共识结果和提交到链上。

use crate::oracle_agent::{OracleAgent, OracleAgentConfig};
use crate::consensus::{ConsensusEngine, ConsensusConfig};
use crate::reputation::ReputationManager;
use crate::sdk::builder::SdkConfig;
use crate::sdk::types::{
    AgentSubmission, ConsensusOutput, OracleQuery, OracleResponse, OracleResult, SdkError,
    SdkResult,
};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use log::{info, warn};

/// 预言机实例
pub struct Oracle {
    /// SDK 配置
    config: SdkConfig,
    /// Agent 列表
    agents: RwLock<Vec<OracleAgent>>,
    /// 共识引擎
    consensus_engine: RwLock<Option<ConsensusEngine>>,
    /// 信誉管理器
    reputation_manager: Arc<ReputationManager>,
}

impl Oracle {
    /// 创建新的预言机实例
    pub fn new(config: SdkConfig) -> SdkResult<Self> {
        info!("🚀 创建 Oracle 实例");

        let reputation_manager = Arc::new(
            ReputationManager::new(crate::reputation::ReputationConfig::default())
        );

        Ok(Self {
            config,
            agents: RwLock::new(Vec::new()),
            consensus_engine: RwLock::new(None),
            reputation_manager,
        })
    }

    /// 添加 Agent
    pub async fn add_agent(&self, agent: OracleAgent) {
        let mut agents = self.agents.write().await;
        agents.push(agent);
        info!("✅ 添加 Agent，当前数量: {}", agents.len());
    }

    /// 初始化默认 Agents
    pub async fn init_default_agents(&self) -> SdkResult<()> {
        let count = self.config.agent_count;
        info!("🔧 初始化 {} 个默认 Agents", count);

        for i in 0..count {
            let agent_config = OracleAgentConfig::default_with_name(&format!("agent_{}", i));
            let agent = OracleAgent::new(agent_config)
                .map_err(|e| SdkError::InternalError(e.to_string()))?;
            self.add_agent(agent).await;
        }

        Ok(())
    }

    /// 初始化共识引擎
    pub async fn init_consensus(&self) -> SdkResult<()> {
        let consensus_config = ConsensusConfig {
            min_quorum_ratio: self.config.consensus_threshold,
            min_reputation_threshold: 100.0,
            max_weight_variance: 3.0,
            timeout_secs: self.config.query_timeout.as_secs(),
            max_retries: self.config.max_retries,
            auto_dispute_resolution: true,
            dispute_resolution_threshold: 0.8,
            confirmation_rounds: 2,
        };

        let engine = ConsensusEngine::new(self.reputation_manager.clone(), consensus_config);
        *self.consensus_engine.write().await = Some(engine);

        info!("✅ 共识引擎初始化完成");
        Ok(())
    }

    /// 发起查询
    pub async fn query(&self, query: OracleQuery) -> SdkResult<OracleResult> {
        info!("🔍 发起查询: {}", query.query);

        let agents = self.agents.read().await;
        if agents.len() < self.config.min_agents {
            return Err(SdkError::QueryError(format!(
                "Agent 数量不足: {} < {}",
                agents.len(),
                self.config.min_agents
            )));
        }

        // 收集所有 Agent 的响应
        let mut responses = Vec::new();
        for agent in agents.iter() {
            let submission = self.collect_agent_submission(agent, &query).await?;
            responses.push(OracleResponse {
                query_id: query.query_id.clone(),
                submission,
                metadata: None,
            });
        }

        // 运行共识
        let consensus_output = self.run_consensus(&query, &responses).await?;

        let result = OracleResult {
            query_id: query.query_id.clone(),
            responses,
            consensus: Some(consensus_output),
            timestamp: current_timestamp(),
        };

        info!("✅ 查询完成: {}", query.query_id);
        Ok(result)
    }

    /// 收集单个 Agent 的提交
    async fn collect_agent_submission(
        &self,
        _agent: &OracleAgent,
        query: &OracleQuery,
    ) -> SdkResult<AgentSubmission> {
        // 简化实现：返回模拟数据
        // 实际应该调用 agent.collect_data() 并处理结果
        Ok(AgentSubmission {
            agent_id: format!("agent_{}", rand::random::<u64>() % 1000),
            value: rand::random::<f64>() * 100.0,
            confidence: 0.8 + rand::random::<f64>() * 0.2,
            causal_fingerprint: vec![rand::random::<f64>(); 10],
            timestamp: current_timestamp(),
            signature: None,
        })
    }

    /// 运行共识
    async fn run_consensus(
        &self,
        query: &OracleQuery,
        responses: &[OracleResponse],
    ) -> SdkResult<ConsensusOutput> {
        let consensus_engine = self.consensus_engine.read().await;
        let engine = consensus_engine
            .as_ref()
            .ok_or_else(|| SdkError::ConsensusError("共识引擎未初始化".to_string()))?;

        // 简化实现：计算加权平均
        let total_value: f64 = responses.iter().map(|r| r.submission.value).sum();
        let count = responses.len() as f64;
        let consensus_value = total_value / count;

        let avg_confidence: f64 =
            responses.iter().map(|r| r.submission.confidence).sum::<f64>() / count;

        let spectral_features = if self.config.enable_spectral_analysis {
            Some(
                responses
                    .iter()
                    .flat_map(|r| r.submission.causal_fingerprint.clone())
                    .collect(),
            )
        } else {
            None
        };

        // 启动共识（如果需要与链上交互）
        let participants: Vec<String> = responses
            .iter()
            .map(|r| r.submission.agent_id.clone())
            .collect();

        engine
            .start_consensus(query.query_id.clone(), crate::oracle_agent::OracleDataType::CryptoPrice { symbol: "BTC".to_string() }, participants)
            .await
            .map_err(|e| SdkError::ConsensusError(e.to_string()))?;

        Ok(ConsensusOutput {
            query_id: query.query_id.clone(),
            consensus_value,
            confidence: avg_confidence,
            participant_count: responses.len(),
            spectral_features,
            anomaly_detected: false,
            timestamp: current_timestamp(),
            chain_data: None,
        })
    }

    /// 获取 Agent 数量
    pub async fn agent_count(&self) -> usize {
        self.agents.read().await.len()
    }

    /// 获取配置
    pub fn config(&self) -> &SdkConfig {
        &self.config
    }
}

/// 获取当前时间戳（秒）
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
