//! Solana智能合约增强版本
//! 支持智能体共识和因果验证结果上链

use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("CAUSAL111111111111111111111111111111111");

#[program]
pub mod solana_oracle {
    use super::*;

    /// 初始化智能体共识任务
    pub fn initialize_consensus_task(
        ctx: Context<InitializeConsensusTask>,
        task_id: [u8; 32],
        scenario: String,
        intervention: String,
        max_agents: u32,
    ) -> Result<()> {
        let task = &mut ctx.accounts.consensus_task;
        
        task.task_id = task_id;
        task.requester = ctx.accounts.requester.key();
        task.scenario = scenario;
        task.intervention = intervention;
        task.max_agents = max_agents;
        task.status = ConsensusStatus::Initialized;
        task.agent_count = 0;
        task.consensus_value = 0.0;
        task.consensus_similarity = 0.0;
        task.pass_rate = 0.0;
        task.created_at = Clock::get()?.unix_timestamp;
        task.finalized_at = 0;
        
        emit!(ConsensusTaskInitialized {
            task_id,
            requester: ctx.accounts.requester.key(),
            created_at: task.created_at,
        });
        
        Ok(())
    }

    /// 智能体提交因果图数据
    pub fn submit_agent_graph(
        ctx: Context<SubmitAgentGraph>,
        task_id: [u8; 32],
        agent_id: String,
        model_type: String,
        node_count: u32,
        edge_count: u32,
        intervention_effect: f64,
        base_prediction: f64,
        confidence: f64,
    ) -> Result<()> {
        let task = &mut ctx.accounts.consensus_task;
        let submission = &mut ctx.accounts.agent_submission;
        
        require!(task.status == ConsensusStatus::Initialized, ErrorCode::InvalidTaskStatus);
        require!(task.task_id == task_id, ErrorCode::TaskIdMismatch);
        require!(task.agent_count < task.max_agents, ErrorCode::MaxAgentsReached);
        
        // 存储智能体提交的数据
        submission.task_id = task_id;
        submission.agent_id = agent_id.clone();
        submission.model_type = model_type;
        submission.node_count = node_count;
        submission.edge_count = edge_count;
        submission.intervention_effect = intervention_effect;
        submission.base_prediction = base_prediction;
        submission.confidence = confidence;
        submission.submitted_at = Clock::get()?.unix_timestamp;
        submission.is_valid = true; // 默认有效，后续验证
        
        task.agent_count += 1;
        
        emit!(AgentGraphSubmitted {
            task_id,
            agent_id,
            intervention_effect,
            base_prediction,
        });
        
        Ok(())
    }

    /// 最终化共识结果
    pub fn finalize_consensus(
        ctx: Context<FinalizeConsensus>,
        task_id: [u8; 32],
        consensus_value: f64,
        consensus_similarity: f64,
        pass_rate: f64,
        valid_agents: Vec<String>,
        outlier_agents: Vec<String>,
    ) -> Result<()> {
        let task = &mut ctx.accounts.consensus_task;
        let result = &mut ctx.accounts.consensus_result;
        
        require!(task.status == ConsensusStatus::Initialized, ErrorCode::InvalidTaskStatus);
        require!(task.task_id == task_id, ErrorCode::TaskIdMismatch);
        require!(task.agent_count >= 2, ErrorCode::InsufficientAgents);
        
        // 更新任务状态
        task.status = ConsensusStatus::Finalized;
        task.consensus_value = consensus_value;
        task.consensus_similarity = consensus_similarity;
        task.pass_rate = pass_rate;
        task.finalized_at = Clock::get()?.unix_timestamp;
        
        // 存储共识结果
        result.task_id = task_id;
        result.consensus_value = consensus_value;
        result.consensus_similarity = consensus_similarity;
        result.pass_rate = pass_rate;
        result.valid_agents = valid_agents;
        result.outlier_agents = outlier_agents;
        result.finalized_at = task.finalized_at;
        
        emit!(ConsensusFinalized {
            task_id,
            consensus_value,
            consensus_similarity,
            pass_rate,
            finalized_at: task.finalized_at,
        });
        
        Ok(())
    }

    /// 验证智能体提交
    pub fn validate_agent_submission(
        ctx: Context<ValidateAgentSubmission>,
        task_id: [u8; 32],
        agent_id: String,
        is_valid: bool,
        reason: Option<String>,
    ) -> Result<()> {
        let submission = &mut ctx.accounts.agent_submission;
        
        require!(submission.task_id == task_id, ErrorCode::TaskIdMismatch);
        require!(submission.agent_id == agent_id, ErrorCode::AgentIdMismatch);
        
        submission.is_valid = is_valid;
        submission.validation_reason = reason;
        submission.validated_at = Clock::get()?.unix_timestamp;
        
        emit!(AgentValidated {
            task_id,
            agent_id,
            is_valid,
        });
        
        Ok(())
    }

    /// 获取共识结果
    pub fn get_consensus_result(
        ctx: Context<GetConsensusResult>,
        task_id: [u8; 32],
    ) -> Result<()> {
        // 这是一个只读函数，实际查询由客户端处理
        Ok(())
    }
}

// ==================== 账户结构 ====================

#[account]
pub struct ConsensusTask {
    /// 任务ID
    pub task_id: [u8; 32],
    /// 请求者
    pub requester: Pubkey,
    /// 场景描述
    pub scenario: String,
    /// 干预措施
    pub intervention: String,
    /// 最大智能体数量
    pub max_agents: u32,
    /// 当前智能体数量
    pub agent_count: u32,
    /// 任务状态
    pub status: ConsensusStatus,
    /// 共识值
    pub consensus_value: f64,
    /// 共识相似度
    pub consensus_similarity: f64,
    /// 通过率
    pub pass_rate: f64,
    /// 创建时间
    pub created_at: i64,
    /// 最终化时间
    pub finalized_at: i64,
}

#[account]
pub struct AgentSubmission {
    /// 任务ID
    pub task_id: [u8; 32],
    /// 智能体ID
    pub agent_id: String,
    /// 模型类型
    pub model_type: String,
    /// 节点数量
    pub node_count: u32,
    /// 边数量
    pub edge_count: u32,
    /// 干预效应
    pub intervention_effect: f64,
    /// 基准预测
    pub base_prediction: f64,
    /// 置信度
    pub confidence: f64,
    /// 是否有效
    pub is_valid: bool,
    /// 提交时间
    pub submitted_at: i64,
    /// 验证时间
    pub validated_at: i64,
    /// 验证原因
    pub validation_reason: Option<String>,
}

#[account]
pub struct ConsensusResult {
    /// 任务ID
    pub task_id: [u8; 32],
    /// 共识值
    pub consensus_value: f64,
    /// 共识相似度
    pub consensus_similarity: f64,
    /// 通过率
    pub pass_rate: f64,
    /// 有效智能体列表
    pub valid_agents: Vec<String>,
    /// 异常智能体列表
    pub outlier_agents: Vec<String>,
    /// 最终化时间
    pub finalized_at: i64,
}

// ==================== 枚举 ====================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ConsensusStatus {
    Initialized,
    InProgress,
    Finalized,
    Failed,
}

// ==================== 指令结构 ====================

#[derive(Accounts)]
pub struct InitializeConsensusTask<'info> {
    #[account(
        init,
        payer = requester,
        space = 8 + 32 + 32 + 4 + 200 + 4 + 200 + 4 + 4 + 1 + 8 + 8 + 8 + 8 + 8
    )]
    pub consensus_task: Account<'info, ConsensusTask>,
    #[account(mut)]
    pub requester: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitAgentGraph<'info> {
    #[account(mut)]
    pub consensus_task: Account<'info, ConsensusTask>,
    #[account(
        init,
        payer = agent,
        space = 8 + 32 + 4 + 50 + 4 + 50 + 4 + 4 + 8 + 8 + 8 + 1 + 8 + 8 + 4 + 100
    )]
    pub agent_submission: Account<'info, AgentSubmission>,
    #[account(mut)]
    pub agent: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeConsensus<'info> {
    #[account(mut)]
    pub consensus_task: Account<'info, ConsensusTask>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 8 + 8 + 4 + 100 + 4 + 100 + 8
    )]
    pub consensus_result: Account<'info, ConsensusResult>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ValidateAgentSubmission<'info> {
    #[account(mut)]
    pub agent_submission: Account<'info, AgentSubmission>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetConsensusResult<'info> {
    pub consensus_result: Account<'info, ConsensusResult>,
}

// ==================== 事件 ====================

#[event]
pub struct ConsensusTaskInitialized {
    pub task_id: [u8; 32],
    pub requester: Pubkey,
    pub created_at: i64,
}

#[event]
pub struct AgentGraphSubmitted {
    pub task_id: [u8; 32],
    pub agent_id: String,
    pub intervention_effect: f64,
    pub base_prediction: f64,
}

#[event]
pub struct ConsensusFinalized {
    pub task_id: [u8; 32],
    pub consensus_value: f64,
    pub consensus_similarity: f64,
    pub pass_rate: f64,
    pub finalized_at: i64,
}

#[event]
pub struct AgentValidated {
    pub task_id: [u8; 32],
    pub agent_id: String,
    pub is_valid: bool,
}

// ==================== 错误代码 ====================

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid task status")]
    InvalidTaskStatus,
    #[msg("Task ID mismatch")]
    TaskIdMismatch,
    #[msg("Agent ID mismatch")]
    AgentIdMismatch,
    #[msg("Maximum agents reached")]
    MaxAgentsReached,
    #[msg("Insufficient agents for consensus")]
    InsufficientAgents,
    #[msg("Invalid requester")]
    InvalidRequester,
    #[msg("Agent not active")]
    AgentNotActive,
}
