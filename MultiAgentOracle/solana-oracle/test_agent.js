// 智能体注册测试脚本
// 使用现有程序ID: DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b

const anchor = require('@project-serum/anchor');
const { PublicKey, Keypair, SystemProgram } = require('@solana/web3.js');
const { BN } = require('bn.js');

// 连接到本地测试网
const provider = anchor.AnchorProvider.local();
anchor.setProvider(provider);

// 程序ID
const programId = new PublicKey('DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b');

// 加载IDL（接口定义语言）
// 注意：需要先构建项目生成IDL
async function testAgentRegistration() {
    try {
        console.log('🧪 开始智能体注册测试...');
        console.log('程序ID:', programId.toString());
        
        // 创建测试智能体
        const agent = Keypair.generate();
        console.log('智能体公钥:', agent.publicKey.toString());
        
        // 模拟DID
        const did = 'did:example:agent123';
        const publicKey = new Uint8Array(32).fill(1); // 模拟公钥
        const metadataUri = 'https://ipfs.io/ipfs/QmExampleMetadata';
        
        console.log('✅ 测试准备完成');
        console.log('DID:', did);
        console.log('元数据URI:', metadataUri);
        
        // 在实际部署后，这里会调用智能合约
        console.log('💡 部署后，将调用:');
        console.log('   register_agent(did, publicKey, metadataUri)');
        
    } catch (error) {
        console.error('❌ 测试错误:', error);
    }
}

// 运行测试
testAgentRegistration();
