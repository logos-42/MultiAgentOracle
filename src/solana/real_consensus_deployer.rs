//! 真实的Solana区块链智能体共识部署模块
//!
//! 将多智能体因果验证结果部署到真实的Solana区块链

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer, read_keypair_file},
    transaction::Transaction,
    instruction::Instruction,
    sysvar,
    program_pack::Pack,
    message::Message,
};
use solana_program::system_program;

/// 智能体共识结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConsensusResult {
    /// 共识ID
    pub consensus_id: String,
    /// 场景描述
    pub scenario: String,
    /// 干预措施
    pub intervention: String,
    /// 有效智能体列表
    pub valid_agents: Vec<String>,
    /// 异常智能体列表
    pub outliers: Vec<String>,
    /// 共识值
    pub consensus_value: f64,
    /// 因果图相似度
    pub consensus_similarity: f64,
    /// 通过率
    pub pass_rate: f64,
    /// 时间戳
    pub timestamp: i64,
    /// 合约版本
    pub contract_version: String,
    /// 智能体因果图数据
    pub agent_graphs: Vec<AgentGraphData>,
}

/// 单个智能体的因果图数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGraphData {
    /// 智能体ID
    pub agent_id: String,
    /// 模型类型
    pub model_type: String,
    /// 节点数量
    pub node_count: usize,
    /// 边数量
    pub edge_count: usize,
    /// 干预效应
    pub intervention_effect: f64,
    /// 基准预测
    pub base_prediction: f64,
    /// 置信度
    pub confidence: f64,
}

/// Solana交易结果
#[derive(Debug, Clone)]
pub struct SolanaTransactionResult {
    /// 交易哈希
    pub transaction_hash: String,
    /// 区块链浏览器链接
    pub explorer_url: String,
    /// 交易状态
    pub status: TransactionStatus,
    /// 错误信息（如果有）
    pub error_message: Option<String>,
    /// Gas费用（lamports）
    pub gas_fee: u64,
    /// 区块高度
    pub block_height: Option<u64>,
}

/// 交易状态
#[derive(Debug, Clone)]
pub enum TransactionStatus {
    Success,
    Pending,
    Failed,
}

/// 真实的Solana区块链部署器
pub struct RealSolanaDeployer {
    /// RPC客户端
    pub rpc_client: RpcClient,
    /// 钱包密钥对
    pub payer: Keypair,
    /// 程序ID
    pub program_id: Pubkey,
}

impl RealSolanaDeployer {
    /// 创建新的真实部署器
    pub fn new(rpc_url: String, wallet_path: String, program_id: String) -> Result<Self> {
        let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        let payer = read_keypair_file(wallet_path)
            .map_err(|e| anyhow::anyhow!("读取钱包文件失败: {}", e))?;
        let program_id = program_id.parse::<Pubkey>()
            .map_err(|e| anyhow::anyhow!("解析程序ID失败: {}", e))?;
        
        println!("🔗 连接到Solana网络: {}", rpc_client.url());
        println!("👛 钱包地址: {}", payer.pubkey());
        println!("📦 程序ID: {}", program_id);
        
        Ok(Self {
            rpc_client,
            payer,
            program_id,
        })
    }

    /// 部署智能体共识结果到真实区块链
    pub async fn deploy_consensus_result(
        &self,
        consensus_result: &AgentConsensusResult,
    ) -> Result<SolanaTransactionResult> {
        println!("🌐 开始部署智能体共识到真实Solana区块链...");
        
        // 1. 序列化共识数据
        let serialized_data = serde_json::to_string(consensus_result)
            .map_err(|e| anyhow::anyhow!("序列化失败: {}", e))?;
        println!("   📝 序列化数据长度: {} 字节", serialized_data.len());
        
        // 2. 创建新的共识账户
        let consensus_account = Keypair::new();
        println!("   🏦 创建共识账户: {}", consensus_account.pubkey());
        
        // 3. 构建交易指令
        let instruction = self.build_consensus_instruction(
            &consensus_account.pubkey(),
            consensus_result,
        )?;
        
        // 4. 创建并签名交易
        let transaction = self.build_and_sign_transaction(&instruction, &consensus_account)?;
        
        // 5. 发送交易到区块链
        let signature = self.send_transaction(&transaction).await?;
        
        // 6. 等待交易确认
        let confirmation = self.wait_for_confirmation(&signature).await?;
        
        // 7. 构建结果
        let transaction_result = SolanaTransactionResult {
            transaction_hash: signature.to_string(),
            explorer_url: format!("https://solscan.io/tx/{}", signature),
            status: if confirmation.err.is_none() { TransactionStatus::Success } else { TransactionStatus::Failed },
            error_message: confirmation.err.map(|e| e.to_string()),
            gas_fee: 5000, // 估算的Gas费用
            block_height: confirmation.block_height,
        };
        
        match &transaction_result.status {
            TransactionStatus::Success => {
                println!("   ✅ 交易成功: {}", transaction_result.transaction_hash);
                println!("   🔗 区块链浏览器: {}", transaction_result.explorer_url);
                if let Some(height) = transaction_result.block_height {
                    println!("   📦 区块高度: {}", height);
                }
            }
            TransactionStatus::Failed => {
                println!("   ❌ 交易失败: {:?}", transaction_result.error_message);
            }
            TransactionStatus::Pending => {
                println!("   ⏳ 交易待确认: {}", transaction_result.transaction_hash);
            }
        }
        
        Ok(transaction_result)
    }

    /// 构建共识指令
    fn build_consensus_instruction(
        &self,
        consensus_account: &Pubkey,
        consensus_result: &AgentConsensusResult,
    ) -> Result<Instruction> {
        // 序列化共识数据
        let serialized_data = serde_json::to_vec(consensus_result)?;
        
        // 创建指令数据
        let mut instruction_data = Vec::new();
        
        // 添加指令标识符 (0 = 初始化共识)
        instruction_data.push(0);
        
        // 添加共识数据长度
        instruction_data.extend_from_slice(&(serialized_data.len() as u32).to_le_bytes());
        
        // 添加序列化的共识数据
        instruction_data.extend_from_slice(&serialized_data);
        
        // 构建指令
        let instruction = Instruction::new_with_bytes(
            &self.program_id,
            &instruction_data,
            vec![
                system_program::id(),
                consensus_account.clone(),
                self.payer.pubkey(),
            ],
        );
        
        Ok(instruction)
    }

    /// 构建并签名交易
    fn build_and_sign_transaction(
        &self,
        instruction: &Instruction,
        consensus_account: &Keypair,
    ) -> Result<Transaction> {
        // 计算所需租金
        let rent = self.rpc_client.get_minimum_balance_for_rent_exemption(1000)?;
        
        // 创建交易
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&self.payer, consensus_account],
            self.rpc_client.latest_blockhash()?,
        );
        
        Ok(transaction)
    }

    /// 发送交易到区块链
    async fn send_transaction(&self, transaction: &Transaction) -> Result<solana_sdk::signature::Signature> {
        println!("   📤 发送交易到区块链...");
        
        // 发送交易
        let signature = self.rpc_client.send_and_confirm_transaction(transaction)?;
        
        println!("   📋 交易签名: {}", signature);
        
        Ok(signature)
    }

    /// 等待交易确认
    async fn wait_for_confirmation(
        &self,
        signature: &solana_sdk::signature::Signature,
    ) -> Result<solana_sdk::transaction::Result<()>> {
        println!("   ⏳ 等待交易确认...");
        
        // 获取交易状态
        let confirmation = self.rpc_client.confirm_transaction(signature)?;
        
        if confirmation.err.is_none() {
            println!("   ✅ 交易已确认");
        } else {
            println!("   ❌ 交易确认失败: {:?}", confirmation.err);
        }
        
        Ok(confirmation)
    }

    /// 查询链上共识结果
    pub async fn query_consensus_result(&self, consensus_id: &str) -> Result<Option<AgentConsensusResult>> {
        println!("🔍 查询链上共识结果: {}", consensus_id);
        
        // 这里应该从链上账户读取实际数据
        // 由于我们的简化实现，这里返回None
        // 在实际应用中，您需要根据账户地址查询数据
        
        println!("   ℹ️  需要实现账户数据查询逻辑");
        Ok(None)
    }

    /// 获取钱包余额
    pub fn get_wallet_balance(&self) -> Result<u64> {
        let balance = self.rpc_client.get_balance(&self.payer.pubkey())?;
        println!("💰 钱包余额: {} SOL", balance as f64 / 1_000_000_000.0);
        Ok(balance)
    }

    /// 获取网络信息
    pub fn get_network_info(&self) -> Result<()> {
        println!("🌐 网络信息:");
        println!("   RPC URL: {}", self.rpc_client.url());
        
        // 获取最新区块哈希
        let latest_blockhash = self.rpc_client.get_latest_blockhash()?;
        println!("   最新区块哈希: {}", latest_blockhash);
        
        // 获取节点版本
        if let Ok(version) = self.rpc_client.get_version() {
            println!("   节点版本: {}", version.solana_core);
        }
        
        Ok(())
    }
}

/// 创建智能体共识结果
pub fn create_consensus_result(
    consensus_id: String,
    scenario: String,
    intervention: String,
    valid_agents: Vec<String>,
    outliers: Vec<String>,
    consensus_value: f64,
    consensus_similarity: f64,
    pass_rate: f64,
    agent_graphs: Vec<AgentGraphData>,
) -> AgentConsensusResult {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    AgentConsensusResult {
        consensus_id,
        scenario,
        intervention,
        valid_agents,
        outliers,
        consensus_value,
        consensus_similarity,
        pass_rate,
        timestamp,
        contract_version: "1.0.0".to_string(),
        agent_graphs,
    }
}

/// 从智能体数据构建图数据
pub fn build_agent_graph_data(
    agents: &[SimpleAgent],
) -> Vec<AgentGraphData> {
    agents.iter().map(|agent| {
        AgentGraphData {
            agent_id: agent.id.clone(),
            model_type: agent.model_type.clone(),
            node_count: agent.causal_graph.nodes.len(),
            edge_count: agent.causal_graph.edges.len(),
            intervention_effect: agent.delta_response,
            base_prediction: agent.base_prediction,
            confidence: 0.9, // 默认置信度
        }
    }).collect()
}

/// 简化的智能体结构
#[derive(Debug, Clone)]
pub struct SimpleAgent {
    pub id: String,
    pub model_type: String,
    pub causal_graph: CausalGraph,
    pub base_prediction: f64,
    pub delta_response: f64,
}

/// 简化的因果图结构
#[derive(Debug, Clone)]
pub struct CausalGraph {
    pub nodes: Vec<CausalNode>,
    pub edges: Vec<CausalEdge>,
    pub metadata: HashMap<String, String>,
}

/// 简化的因果节点
#[derive(Debug, Clone)]
pub struct CausalNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub value: Option<f64>,
}

/// 简化的因果边
#[derive(Debug, Clone)]
pub struct CausalEdge {
    pub source: String,
    pub target: String,
    pub weight: f64,
    pub relation_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_solana_deployment() {
        // 这个测试需要真实的Solana网络连接
        // 在CI/CD环境中应该跳过
        if std::env::var("CI").is_ok() {
            return;
        }
        
        let deployer = RealSolanaDeployer::new(
            "http://localhost:8899".to_string(),
            "~/.config/solana/id.json".to_string(),
            "CAUSAL111111111111111111111111111111111".to_string(),
        );
        
        // 测试网络连接
        if let Ok(_) = deployer.get_network_info() {
            println!("✅ Solana网络连接成功");
        } else {
            println!("⚠️  Solana网络连接失败，跳过测试");
        }
    }
}
