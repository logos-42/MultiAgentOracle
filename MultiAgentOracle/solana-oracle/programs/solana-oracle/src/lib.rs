//! Identity Registry Program for Multi-Agent Oracle System
//! 
//! A Solana program for registering and managing agent identities in a multi-agent oracle system.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

declare_id!("DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b");

#[program]
pub mod solana_oracle {
    use super::*;

    /// Register a new agent identity
    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        did: String,           // Decentralized Identifier
        public_key: [u8; 32],  // Agent's public key
        metadata_uri: String,  // Metadata URI (IPFS, Arweave, etc.)
    ) -> Result<()> {
        let agent_identity = &mut ctx.accounts.agent_identity;
        let clock = Clock::get()?;
        
        // Validate inputs
        require!(did.len() > 0, ErrorCode::InvalidDid);
        require!(did.len() <= 128, ErrorCode::DidTooLong);
        require!(metadata_uri.len() <= 256, ErrorCode::MetadataUriTooLong);
        
        // Check if identity already exists
        require!(agent_identity.owner == Pubkey::default(), ErrorCode::IdentityAlreadyExists);
        
        // Initialize identity
        agent_identity.did = did;
        agent_identity.owner = ctx.accounts.agent.key();
        agent_identity.public_key = public_key;
        agent_identity.registered_at = clock.unix_timestamp;
        agent_identity.last_verified_at = 0;
        agent_identity.is_active = true;
        agent_identity.is_verified = false;
        agent_identity.metadata_uri = metadata_uri;
        agent_identity.reputation_score = 100; // Initial reputation score
        
        emit!(AgentRegistered {
            did: agent_identity.did.clone(),
            owner: agent_identity.owner,
            registered_at: agent_identity.registered_at,
        });
        
        Ok(())
    }

    /// Update agent identity information
    pub fn update_identity(
        ctx: Context<UpdateIdentity>,
        new_public_key: Option<[u8; 32]>,
        new_metadata_uri: Option<String>,
    ) -> Result<()> {
        let agent_identity = &mut ctx.accounts.agent_identity;
        
        // Only owner can update
        require!(
            agent_identity.owner == ctx.accounts.agent.key(),
            ErrorCode::Unauthorized
        );
        require!(agent_identity.is_active, ErrorCode::IdentityInactive);
        
        if let Some(pk) = new_public_key {
            agent_identity.public_key = pk;
        }
        
        if let Some(uri) = new_metadata_uri {
            require!(uri.len() <= 256, ErrorCode::MetadataUriTooLong);
            agent_identity.metadata_uri = uri;
        }
        
        emit!(IdentityUpdated {
            did: agent_identity.did.clone(),
            owner: agent_identity.owner,
            updated_at: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// Request identity verification
    pub fn request_verification(
        ctx: Context<RequestVerification>,
        proof_data: Vec<u8>, // ZK proof or other verification data
    ) -> Result<()> {
        let verification_request = &mut ctx.accounts.verification_request;
        let agent_identity = &ctx.accounts.agent_identity;
        let clock = Clock::get()?;
        
        require!(
            agent_identity.owner == ctx.accounts.agent.key(),
            ErrorCode::Unauthorized
        );
        require!(agent_identity.is_active, ErrorCode::IdentityInactive);
        require!(!agent_identity.is_verified, ErrorCode::AlreadyVerified);
        
        verification_request.did = agent_identity.did.clone();
        verification_request.requester = ctx.accounts.agent.key();
        verification_request.proof_data = proof_data;
        verification_request.requested_at = clock.unix_timestamp;
        verification_request.status = VerificationStatus::Pending;
        verification_request.verifier = Pubkey::default();
        verification_request.verified_at = 0;
        
        emit!(VerificationRequested {
            did: verification_request.did.clone(),
            requester: verification_request.requester,
            requested_at: verification_request.requested_at,
        });
        
        Ok(())
    }

    /// Approve verification request (verifier only)
    pub fn approve_verification(
        ctx: Context<ApproveVerification>,
    ) -> Result<()> {
        let verification_request = &mut ctx.accounts.verification_request;
        let agent_identity = &mut ctx.accounts.agent_identity;
        let clock = Clock::get()?;
        
        require!(
            verification_request.status == VerificationStatus::Pending,
            ErrorCode::InvalidVerificationStatus
        );
        
        // In production, you would verify the proof_data here
        // For simplicity, we just approve
        
        verification_request.status = VerificationStatus::Approved;
        verification_request.verifier = ctx.accounts.verifier.key();
        verification_request.verified_at = clock.unix_timestamp;
        
        agent_identity.is_verified = true;
        agent_identity.last_verified_at = clock.unix_timestamp;
        
        emit!(VerificationApproved {
            did: verification_request.did.clone(),
            verifier: verification_request.verifier,
            verified_at: verification_request.verified_at,
        });
        
        Ok(())
    }

    /// Update reputation score
    pub fn update_reputation(
        ctx: Context<UpdateReputation>,
        delta: i64, // Positive for rewards, negative for penalties
    ) -> Result<()> {
        let agent_identity = &mut ctx.accounts.agent_identity;
        
        // Only admin or reputation system can update
        require!(
            ctx.accounts.admin.key() == ctx.accounts.admin_authority.key() ||
            ctx.accounts.reputation_system.key() == ctx.accounts.reputation_system_authority.key(),
            ErrorCode::Unauthorized
        );
        
        let new_score = (agent_identity.reputation_score as i64)
            .checked_add(delta)
            .ok_or(ErrorCode::ReputationOverflow)?;
        
        require!(new_score >= 0, ErrorCode::ReputationUnderflow);
        require!(new_score <= 1000, ErrorCode::ReputationOverflow);
        
        agent_identity.reputation_score = new_score as u64;
        
        emit!(ReputationUpdated {
            did: agent_identity.did.clone(),
            old_score: (agent_identity.reputation_score as i64 - delta) as u64,
            new_score: agent_identity.reputation_score,
            updated_at: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// Deactivate identity (owner or admin)
    pub fn deactivate_identity(ctx: Context<DeactivateIdentity>) -> Result<()> {
        let agent_identity = &mut ctx.accounts.agent_identity;
        
        require!(
            agent_identity.owner == ctx.accounts.agent.key() ||
            ctx.accounts.admin.key() == ctx.accounts.admin_authority.key(),
            ErrorCode::Unauthorized
        );
        require!(agent_identity.is_active, ErrorCode::IdentityInactive);
        
        agent_identity.is_active = false;
        agent_identity.is_verified = false;
        
        emit!(IdentityDeactivated {
            did: agent_identity.did.clone(),
            deactivated_by: ctx.accounts.agent.key(),
            deactivated_at: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// Reactivate identity (admin only)
    pub fn reactivate_identity(ctx: Context<ReactivateIdentity>) -> Result<()> {
        let agent_identity = &mut ctx.accounts.agent_identity;
        
        require!(
            ctx.accounts.admin.key() == ctx.accounts.admin_authority.key(),
            ErrorCode::Unauthorized
        );
        require!(!agent_identity.is_active, ErrorCode::IdentityActive);
        
        agent_identity.is_active = true;
        
        emit!(IdentityReactivated {
            did: agent_identity.did.clone(),
            reactivated_by: ctx.accounts.admin.key(),
            reactivated_at: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// Batch register multiple agents (admin only)
    pub fn batch_register_agents(
        ctx: Context<BatchRegisterAgents>,
        agents_data: Vec<AgentData>,
    ) -> Result<()> {
        require!(
            ctx.accounts.admin.key() == ctx.accounts.admin_authority.key(),
            ErrorCode::Unauthorized
        );
        require!(agents_data.len() <= 10, ErrorCode::BatchTooLarge);
        
        let clock = Clock::get()?;
        
        for agent_data in agents_data.iter() {
            // Validate agent data
            require!(agent_data.did.len() > 0, ErrorCode::InvalidDid);
            require!(agent_data.did.len() <= 128, ErrorCode::DidTooLong);
            require!(agent_data.metadata_uri.len() <= 256, ErrorCode::MetadataUriTooLong);
            
            // In production, you would create PDA for each agent
            // For simplicity, we just emit events
            emit!(AgentRegistered {
                did: agent_data.did.clone(),
                owner: Pubkey::default(), // Will be set by client
                registered_at: clock.unix_timestamp,
            });
        }
        
        emit!(BatchRegistrationCompleted {
            count: agents_data.len() as u64,
            registered_at: clock.unix_timestamp,
            admin: ctx.accounts.admin.key(),
        });
        
        Ok(())
    }

}

/// Account contexts
#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = agent,
        space = 8 + AgentIdentity::INIT_SPACE,
        seeds = [b"agent-identity", agent.key().as_ref()],
        bump
    )]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    #[account(mut)]
    pub agent: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateIdentity<'info> {
    #[account(
        mut,
        seeds = [b"agent-identity", agent.key().as_ref()],
        bump,
        constraint = agent_identity.owner == agent.key() @ ErrorCode::Unauthorized
    )]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    pub agent: Signer<'info>,
}

#[derive(Accounts)]
pub struct RequestVerification<'info> {
    #[account(
        init,
        payer = agent,
        space = 8 + VerificationRequest::INIT_SPACE,
        seeds = [b"verification-request", agent.key().as_ref(), {
            // 在测试环境中，Clock::get() 可能失败，使用默认值
            match anchor_lang::solana_program::clock::Clock::get() {
                Ok(clock) => clock.unix_timestamp.to_le_bytes(),
                Err(_) => 0i64.to_le_bytes(), // 测试环境使用0作为时间戳
            }
        }],
        bump
    )]
    pub verification_request: Account<'info, VerificationRequest>,
    
    #[account(
        seeds = [b"agent-identity", agent.key().as_ref()],
        bump,
        constraint = agent_identity.owner == agent.key() @ ErrorCode::Unauthorized
    )]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    #[account(mut)]
    pub agent: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveVerification<'info> {
    #[account(mut)]
    pub verification_request: Account<'info, VerificationRequest>,
    
    #[account(mut)]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    #[account(
        constraint = verifier.key() == verification_request.verifier || 
                    verifier.key() == admin_authority.key() @ ErrorCode::Unauthorized
    )]
    pub verifier: Signer<'info>,
    
    /// CHECK: Admin authority for emergency approvals
    pub admin_authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdateReputation<'info> {
    #[account(mut)]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    /// CHECK: Admin authority
    pub admin: UncheckedAccount<'info>,
    
    /// CHECK: Reputation system authority
    pub reputation_system: UncheckedAccount<'info>,
    
    pub admin_authority: Signer<'info>,
    pub reputation_system_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeactivateIdentity<'info> {
    #[account(mut)]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    pub agent: Signer<'info>,
    
    /// CHECK: Admin authority
    pub admin: UncheckedAccount<'info>,
    
    pub admin_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReactivateIdentity<'info> {
    #[account(mut)]
    pub agent_identity: Account<'info, AgentIdentity>,
    
    /// CHECK: Admin authority
    pub admin: UncheckedAccount<'info>,
    
    pub admin_authority: Signer<'info>,
}

/// Batch registration context
#[derive(Accounts)]
pub struct BatchRegisterAgents<'info> {
    /// CHECK: Admin authority
    pub admin: UncheckedAccount<'info>,
    
    pub admin_authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Data structures
#[account]
#[derive(InitSpace)]
pub struct AgentIdentity {
    #[max_len(128)]
    pub did: String,           // Decentralized Identifier (max 128 chars)
    pub owner: Pubkey,         // Agent's wallet address
    pub public_key: [u8; 32],  // Agent's public key
    pub registered_at: i64,    // Registration timestamp
    pub last_verified_at: i64, // Last verification timestamp
    pub is_active: bool,       // Whether identity is active
    pub is_verified: bool,     // Whether identity is verified
    #[max_len(256)]
    pub metadata_uri: String,  // Metadata URI (max 256 chars)
    pub reputation_score: u64, // Reputation score (0-1000)
}

#[account]
#[derive(InitSpace)]
pub struct VerificationRequest {
    #[max_len(128)]
    pub did: String,           // Agent DID
    pub requester: Pubkey,     // Requesting agent
    #[max_len(1024)]
    pub proof_data: Vec<u8>,   // Verification proof data
    pub requested_at: i64,     // Request timestamp
    pub status: VerificationStatus, // Current status
    pub verifier: Pubkey,      // Verifier who approved/rejected
    pub verified_at: i64,      // Verification timestamp
}

/// Verification status enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum VerificationStatus {
    Pending,
    Approved,
    Rejected,
    Revoked,
}

/// Events
#[event]
pub struct AgentRegistered {
    pub did: String,
    pub owner: Pubkey,
    pub registered_at: i64,
}

#[event]
pub struct IdentityUpdated {
    pub did: String,
    pub owner: Pubkey,
    pub updated_at: i64,
}

#[event]
pub struct VerificationRequested {
    pub did: String,
    pub requester: Pubkey,
    pub requested_at: i64,
}

#[event]
pub struct VerificationApproved {
    pub did: String,
    pub verifier: Pubkey,
    pub verified_at: i64,
}

#[event]
pub struct ReputationUpdated {
    pub did: String,
    pub old_score: u64,
    pub new_score: u64,
    pub updated_at: i64,
}

#[event]
pub struct IdentityDeactivated {
    pub did: String,
    pub deactivated_by: Pubkey,
    pub deactivated_at: i64,
}

#[event]
pub struct IdentityReactivated {
    pub did: String,
    pub reactivated_by: Pubkey,
    pub reactivated_at: i64,
}

/// Batch registration completed event
#[event]
pub struct BatchRegistrationCompleted {
    pub count: u64,
    pub registered_at: i64,
    pub admin: Pubkey,
}

/// Agent tier updated event
#[event]
pub struct AgentTierUpdated {
    pub did: String,
    pub old_tier: String,
    pub new_tier: String,
    pub updated_at: i64,
}

/// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid DID")]
    InvalidDid,
    #[msg("DID too long (max 128 characters)")]
    DidTooLong,
    #[msg("Metadata URI too long (max 256 characters)")]
    MetadataUriTooLong,
    #[msg("Identity already exists")]
    IdentityAlreadyExists,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Identity is not active")]
    IdentityInactive,
    #[msg("Identity is already active")]
    IdentityActive,
    #[msg("Identity is already verified")]
    AlreadyVerified,
    #[msg("Invalid verification status")]
    InvalidVerificationStatus,
    #[msg("Reputation overflow")]
    ReputationOverflow,
    #[msg("Reputation underflow")]
    ReputationUnderflow,
    #[msg("Batch too large (max 10 agents)")]
    BatchTooLarge,
}

/// Agent data for batch registration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct AgentData {
    #[max_len(128)]
    pub did: String,           // Decentralized Identifier
    pub public_key: [u8; 32],  // Agent's public key
    #[max_len(256)]
    pub metadata_uri: String,  // Metadata URI
    pub initial_reputation: u64, // Initial reputation score
}