// 模拟Solana智能体注册API
// 用于在不依赖实际区块链的情况下测试

class MockSolanaAgentRegistry {
    constructor(programId) {
        this.programId = programId;
        this.agents = new Map();
        this.transactions = [];
        this.blockHeight = 1000;
    }

    // 模拟注册智能体
    async registerAgent(did, publicKey, metadataUri) {
        const agentId = \gent_\_\\;
        
        const agent = {
            id: agentId,
            did: did,
            publicKey: publicKey,
            metadataUri: metadataUri,
            registeredAt: new Date().toISOString(),
            isActive: true,
            isVerified: false,
            reputation: 100,
            tier: this._determineTier(publicKey)
        };

        this.agents.set(agentId, agent);

        const tx = {
            type: 'register_agent',
            agentId: agentId,
            did: did,
            timestamp: new Date().toISOString(),
            status: 'success',
            txHash: \MOCK_\\
        };

        this.transactions.push(tx);
        this.blockHeight++;

        return {
            success: true,
            agentId: agentId,
            transaction: tx,
            blockHeight: this.blockHeight
        };
    }

    // 根据公钥确定层级（模拟逻辑）
    _determineTier(publicKey) {
        const tiers = ['gateway', 'data', 'validator', 'core'];
        const hash = this._simpleHash(publicKey);
        return tiers[hash % tiers.length];
    }

    // 简单哈希函数
    _simpleHash(str) {
        let hash = 0;
        for (let i = 0; i < str.length; i++) {
            hash = ((hash << 5) - hash) + str.charCodeAt(i);
            hash |= 0;
        }
        return Math.abs(hash);
    }

    // 获取所有智能体
    async getAllAgents() {
        return Array.from(this.agents.values());
    }

    // 获取交易历史
    async getTransactionHistory(limit = 10) {
        return this.transactions.slice(-limit).reverse();
    }

    // 获取区块链状态
    async getBlockchainState() {
        return {
            network: 'mock_localnet',
            programId: this.programId,
            blockHeight: this.blockHeight,
            agentCount: this.agents.size,
            transactionCount: this.transactions.length,
            timestamp: new Date().toISOString()
        };
    }
}

// 导出模拟API
if (typeof module !== 'undefined' && module.exports) {
    module.exports = MockSolanaAgentRegistry;
}

console.log('✅ 模拟Solana智能体注册API已加载');
console.log('程序ID: DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b');
console.log('智能体数量: 4');
