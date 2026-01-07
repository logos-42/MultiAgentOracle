//! Causal Fingerprinting Oracle Program for Multi-Agent System
//!
//! A Solana program that implements causal fingerprint verification for AI agents.
//! Key concepts:
//! - TaskAccount: Manages the causal fingerprint lifecycle
//! - FingerprintSubmission: Stores agent's delta response (”y)
//! - GlobalFingerprint: Tracks long-term logical identity of each agent
//!
//! Protocol flow:
//! 1. Agent submits base prediction y€ (commitment)
//! 2. Chain generates random perturbation ´ (challenge)
//! 3. Agent submits fingerprint ”y = f(x+´) - f(x)
//! 4. Aggregator performs spectral analysis + cosine clustering
//! 5. Contract distributes rewards/penalties based on logical consistency

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

declare_id!("CAUSAL111111111111111111111111111111111");

#[program]
pub mod solana_oracle {
    use super::*;

    // ==================== TASK LIFECYCLE ====================

    /// Initialize a new task for causal fingerprint verification
    pub fn initialize_task(
        ctx: Context<InitializeTask>,
        task_id: [u8; 32],
        context_features: Vec<i64>,
    ) -> Result<()> {
        let task = &mut ctx.accounts.task;
        
        require!(ctx.accounts.requester.key() != Pubkey::default(), ErrorCode::InvalidRequester);
        
        task.task_id = task_id;
        task.requester = ctx.accounts.requester.key();
        task.context_features = context_features;
        task.perturbation_vector = Vec::new();
        task.status = TaskStatus::Pending;
        task.submission_count = 0;
        task.created_at = Clock::get()?.unix_timestamp;
        task.challenge_issued_at = 0;
        task.aggregated_at = 0;
        
        emit!(TaskInitialized {
            task_id,
            requester: ctx.accounts.requester.key(),
            created_at: task.created_at,
        });
        
        Ok(())
    }

    /// Agent submits base prediction (y€) - commitment phase
    pub fn submit_base_prediction(
        ctx: Context<SubmitBasePrediction>,
        task_id: [u8; 32],
        base_prediction: i64,
    ) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let submission = &mut ctx.accounts.submission;
        
        require!(task.status == TaskStatus::Pending, ErrorCode::InvalidTaskStatus);
        require!(task.task_id == task_id, ErrorCode::TaskIdMismatch);
        
        // Verify agent is registered
        require!(ctx.accounts.agent_identity.is_active, ErrorCode::AgentNotActive);
        
        // Generate commitment hash from base prediction
        let base_hash = anchor_lang::solana_program::hash::hash(
            &base_prediction.to_le_bytes()
        ).to_bytes();
        
        // Initialize submission
        submission.task_id = task_id;
        submission.agent = ctx.accounts.agent.key();
        submission.base_prediction = base_prediction;
        submission.base_hash = base_hash;
        submission.delta_response = Vec::new();
        submission.spectral_features = [0i64; 8];
        submission.submitted_at = Clock::get()?.unix_timestamp;
        submission.is_valid = false;
        
        task.submission_count += 1;
        
        emit!(BasePredictionSubmitted {
            task_id,
            agent: ctx.accounts.agent.key(),
            base_prediction,
            base_hash,
        });
        
        Ok(())
    }

    /// Issue causal challenge (generate perturbation vector ´)
    /// This can only be called after enough agents have submitted base predictions
    pub fn issue_challenge(
        ctx: Context<IssueChallenge>,
        task_id: [u8; 32],
        perturbation_dimensions: u8,
    ) -> Result<()> {
        let task = &mut ctx.accounts.task;
        
        require!(task.status == TaskStatus::Pending, ErrorCode::InvalidTaskStatus);
        require!(task.task_id == task_id, ErrorCode::TaskIdMismatch);
        require!(task.submission_count >= 3, ErrorCode::InsufficientSubmissions);
        require!(perturbation_dimensions >= 3 && perturbation_dimensions <= 20, ErrorCode::InvalidDimensions);
        
        // Generate pseudo-random perturbation using block hash
        let clock = Clock::get()?;
        let block_hash = ctx.accounts.recent_blockhash.key().to_bytes();
        
        let mut perturbation = Vec::with_capacity(perturbation_dimensions as usize);
        for i in 0..perturbation_dimensions {
            // Simple pseudo-random generation for demo
            // In production, use a proper VRF
            let seed = (clock.unix_timestamp as u64)
                .wrapping_add(block_hash[i as usize] as u64)
                .wrapping_add(task.submission_count as u64 * 17 + i as u64);
            let perturbation_value = (seed % 1000) as i64 - 500; // Range: -500 to 499
            perturbation.push(perturbation_value);
        }
        
        task.perturbation_vector = perturbation;
        task.status = TaskStatus::Challenge;
        task.challenge_issued_at = clock.unix_timestamp;
        
        emit!(ChallengeIssued {
            task_id,
            perturbation_vector: task.perturbation_vector.clone(),
            issued_at: clock.unix_timestamp,
        });
        
        Ok(())
    }

    /// Agent submits causal fingerprint (”y) - response phase
    pub fn submit_fingerprint(
        ctx: Context<SubmitFingerprint>,
        task_id: [u8; 32],
        delta_response: Vec<i64>,
        spectral_features: [i64; 8],
    ) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let submission = &mut ctx.accounts.submission;
        let agent_identity = &mut ctx.accounts.agent_identity;
        
        require!(task.status == TaskStatus::Challenge, ErrorCode::InvalidTaskStatus);
        require!(task.task_id == task_id, ErrorCode::TaskIdMismatch);
        require!(
            submission.agent == ctx.accounts.agent.key(),
            ErrorCode::Unauthorized
        );
        require!(!submission.is_valid, ErrorCode::AlreadySubmitted);
        
        // Verify base hash matches
        let expected_hash = anchor_lang::solana_program::hash::hash(
            &submission.base_prediction.to_le_bytes()
        ).to_bytes();
        require!(submission.base_hash == expected_hash, ErrorCode::BaseHashMismatch);
        
        // Update submission with fingerprint
        submission.delta_response = delta_response;
        submission.spectral_features = spectral_features;
        submission.submitted_at = Clock::get()?.unix_timestamp;
        submission.is_valid = true;
        
        // Update agent's task count
        agent_identity.total_tasks = agent_identity.total_tasks.wrapping_add(1);
        
        emit!(FingerprintSubmitted {
            task_id,
            agent: ctx.accounts.agent.key(),
            delta_response_len: submission.delta_response.len(),
            spectral_features,
        });
        
        Ok(())
    }

    /// Aggregate consensus using spectral analysis and cosine clustering
    /// Distribute rewards/penalties based on logical consistency
    pub fn aggregate_consensus(
        ctx: Context<AggregateConsensus>,
        task_id: [u8; 32],
        cosine_threshold: f64,
        outlier_threshold: f64,
    ) -> Result<()> {
        let task = &mut ctx.accounts.task;
        
        require!(task.status == TaskStatus::Challenge, ErrorCode::InvalidTaskStatus);
        require!(task.task_id == task_id, ErrorCode::TaskIdMismatch);
        require!(task.submission_count >= 3, ErrorCode::InsufficientSubmissions);
        require!(cosine_threshold >= 0.0 && cosine_threshold <= 1.0, ErrorCode::InvalidThreshold);
        require!(outlier_threshold > 0.0, ErrorCode::InvalidThreshold);
        
        let clock = Clock::get()?;
        let mut valid_submissions = Vec::new();
        let mut outliers = Vec::new();
        
        // Collect all valid submissions
        for account in &ctx.remaining_accounts {
            if account.key == ctx.accounts.submission1.key() {
                continue;
            }
            // In real implementation, we'd iterate through all submission accounts
        }
        
        // Simplified: process the primary submission
        if ctx.accounts.submission1.is_valid {
            let similarity = calculate_cosine_similarity(&task.perturbation_vector, &ctx.accounts.submission1.delta_response);
            
            if similarity >= cosine_threshold {
                valid_submissions.push((ctx.accounts.submission1.agent, similarity));
                
                // Reward the agent
                let reward = calculate_reward(similarity, task.submission_count);
                ctx.accounts.agent_identity.causal_credit += reward as u64;
            } else {
                outliers.push(ctx.accounts.submission1.agent);
                
                // Penalize the agent
                let penalty = calculate_penalty(similarity, outlier_threshold);
                ctx.accounts.agent_identity.causal_credit = ctx.accounts.agent_identity.causal_credit.saturating_sub(penalty as u64);
                ctx.accounts.agent_identity.outlier_count += 1;
            }
            
            // Update global fingerprint with spectral features
            update_global_fingerprint(&mut ctx.accounts.agent_identity, &ctx.accounts.submission1.spectral_features);
        }
        
        task.status = TaskStatus::Aggregated;
        task.aggregated_at = clock.unix_timestamp;
        task.consensus_similarity = valid_submissions.iter()
            .map(|(_, s)| *s)
            .sum::<f64>() / valid_submissions.len() as f64;
        
        emit!(ConsensusAggregated {
            task_id,
            valid_count: valid_submissions.len(),
            outlier_count: outliers.len(),
            consensus_similarity: task.consensus_similarity,
            aggregated_at: clock.unix_timestamp,
        });
        
        Ok(())
    }

    /// Update agent's global fingerprint (called by aggregator)
    pub fn update_global_fingerprint(
        ctx: Context<UpdateGlobalFingerprint>,
        new_features: [i64; 16],
    ) -> Result<()> {
        let fingerprint = &mut ctx.accounts.global_fingerprint;
        let agent_identity = &ctx.accounts.agent_identity;
        
        require!(
            fingerprint.agent == ctx.accounts.agent.key() || 
            fingerprint.agent == Pubkey::default(),
            ErrorCode::Unauthorized
        );
        
        // Exponential moving average update
        let alpha = 0.1; // EMA factor
        let task_count = fingerprint.task_count;
        
        for i in 0..16 {
            let old_value = fingerprint.spectral_ema[i] as f64;
            let new_value = new_features[i] as f64;
            fingerprint.spectral_ema[i] = ((old_value * (1.0 - alpha)) + (new_value * alpha)) as i64;
        }
        
        fingerprint.task_count = task_count.wrapping_add(1);
        fingerprint.last_updated = Clock::get()?.unix_timestamp;
        fingerprint.agent = agent_identity.owner;
        
        emit!(GlobalFingerprintUpdated {
            agent: agent_identity.owner,
            task_count: fingerprint.task_count,
        });
        
        Ok(())
    }

    // ==================== AGENT REGISTRATION ====================

    /// Register a new agent
    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        did: String,
    ) -> Result<()> {
        let agent_identity = &mut ctx.accounts.agent_identity;
        let clock = Clock::get()?;
        
        require!(did.len() > 0 && did.len() <= 128, ErrorCode::InvalidDid);
        require!(agent_identity.owner == Pubkey::default(), ErrorCode::AgentAlreadyExists);
        
        agent_identity.did = did;
        agent_identity.owner = ctx.accounts.agent.key();
        agent_identity.registered_at = clock.unix_timestamp;
        agent_identity.is_active = true;
        agent_identity.total_tasks = 0;
        agent_identity.causal_credit = 0;
        agent_identity.outlier_count = 0;
        agent_identity.spectral_ema = [0i64; 16];
        
        emit!(AgentRegistered {
            did: agent_identity.did.clone(),
            owner: ctx.accounts.agent.key(),
            registered_at: clock.unix_timestamp,
        });
        
        Ok(())
    }

    /// Deactivate agent
    pub fn deactivate_agent(ctx: Context<DeactivateAgent>) -> Result<()> {
        let agent_identity = &mut ctx.accounts.agent_identity;
        
        require!(
            agent_identity.owner == ctx.accounts.agent.key(),
            ErrorCode::Unauthorized
        );
        
        agent_identity.is_active = false;
        
        emit!(AgentDeactivated {
            did: agent_identity.did.clone(),
            owner: ctx.accounts.agent.key(),
            deactivated_at: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }
}

// ==================== HELPER FUNCTIONS ====================

fn calculate_cosine_similarity(a: &[i64], b: &[i64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let mut dot_product = 0i128;
    let mut norm_a = 0i128;
    let mut norm_b = 0i128;
    
    for i in 0..a.len() {
        dot_product += (a[i] as i128) * (b[i] as i128);
        norm_a += (a[i] as i128) * (a[i] as i128);
        norm_b += (b[i] as i128) * (b[i] as i128);
    }
    
    let norm_a_sqrt = (norm_a as f64).sqrt();
    let norm_b_sqrt = (norm_b as f64).sqrt();
    
    if norm_a_sqrt == 0.0 || norm_b_sqrt == 0.0 {
        return 0.0;
    }
    
    dot_product as f64 / (norm_a_sqrt * norm_b_sqrt)
}

fn calculate_reward(similarity: f64, submission_count: u32) -> f64 {
    // Higher similarity = higher reward
    // Base reward: 10 * similarity
    // Bonus for consensus participation
    (10.0 * similarity).max(0.0)
}

fn calculate_penalty(similarity: f64, threshold: f64) -> f64 {
    // Penalty proportional to deviation from consensus
    let deviation = (threshold - similarity).max(0.0);
    (20.0 * deviation).min(100.0) // Max penalty: 100
}

fn update_global_fingerprint(agent_identity: &mut Account<AgentIdentity>, new_features: &[i64; 8]) {
    let alpha = 0.1; // EMA factor
    
    for i in 0..8 {
        let old = agent_identity.spectral_ema[i] as f64;
        let new = new_features[i] as f64;
        agent_identity.spectral_ema[i] = ((old * (1.0 - alpha)) + (new * alpha)) as i64;
    }
}

// ==================== ACCOUNT CONTEXTS ====================

#[derive(Accounts)]
pub struct InitializeTask<'info> {
    #[account(
        init,
        payer = requester,
        space = 8 + TaskAccount::INIT_SPACE,
        seeds = [b"task", task_id.as_ref()],
        bump
    )]
    pub task: Account<'info, TaskAccount>,
    
    #[account(mut)]
    pub requester: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitBasePrediction<'info> {
    #[account(
        mut,
        seeds = [b"task", task_id.as_ref()],
        bump,
    )]
    pub task: Account<'info, TaskAccount>,
    
    #[account(
        init,
        payer = agent,
        space = 8 + FingerprintSubmission::INIT_SPACE,
        seeds = [b"submission", task_id.as_ref(), agent.key().as_ref()],
        bump
    )]
    pub submission: Account<'info, FingerprintSubmission>,
    
    #[account(
        seeds = [b"agent", agent.key().as_ref()],
        bump,
    )]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    #[account(mut)]
    pub agent: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IssueChallenge<'info> {
    #[account(
        mut,
        seeds = [b"task", task_id.as_ref()],
        bump,
    )]
    pub task: Account<'info, TaskAccount>,
    
    /// CHECK: Recent blockhash for randomness
    pub recent_blockhash: UncheckedAccount<'info>,
    
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitFingerprint<'info> {
    #[account(
        mut,
        seeds = [b"task", task_id.as_ref()],
        bump,
    )]
    pub task: Account<'info, TaskAccount>,
    
    #[account(
        mut,
        seeds = [b"submission", task_id.as_ref(), agent.key().as_ref()],
        bump,
        constraint = submission.agent == agent.key() @ ErrorCode::Unauthorized
    )]
    pub submission: Account<'info, FingerprintSubmission>,
    
    #[account(
        mut,
        seeds = [b"agent", agent.key().as_ref()],
        bump,
    )]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    pub agent: Signer<'info>,
}

#[derive(Accounts)]
pub struct AggregateConsensus<'info> {
    #[account(
        mut,
        seeds = [b"task", task_id.as_ref()],
        bump,
    )]
    pub task: Account<'info, TaskAccount>,
    
    #[account(
        mut,
        seeds = [b"submission", task_id.as_ref(), submission1.agent.as_ref()],
        bump,
    )]
    pub submission1: Account<'info, FingerprintSubmission>,
    
    #[account(
        mut,
        seeds = [b"agent", submission1.agent.as_ref()],
        bump,
    )]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    pub aggregator: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateGlobalFingerprint<'info> {
    #[account(
        init,
        payer = agent,
        space = 8 + GlobalFingerprint::INIT_SPACE,
        seeds = [b"fingerprint", agent.key().as_ref()],
        bump
    )]
    pub global_fingerprint: Account<'info, GlobalFingerprint>,
    
    #[account(
        seeds = [b"agent", agent.key().as_ref()],
        bump,
    )]
    pub agent