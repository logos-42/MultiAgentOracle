#!/bin/bash

# WSL Solana开发环境设置脚本
# 在Ubuntu WSL2中设置完整的Solana开发环境

set -e  # 遇到错误时退出

echo "🚀 WSL Solana开发环境设置"
echo "================================"

# 1. 更新系统
echo "📦 更新系统包..."
sudo apt update
sudo apt upgrade -y

# 2. 安装基础依赖
echo "📦 安装基础依赖..."
sudo apt install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    libclang-dev \
    cmake \
    protobuf-compiler

# 3. 安装Rust
echo "🦀 安装Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo "   Rust已安装: $(rustc --version)"
fi

# 4. 安装Solana工具链
echo "🔧 安装Solana工具链..."
if ! command -v solana &> /dev/null; then
    sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
    echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.bashrc
else
    echo "   Solana已安装: $(solana --version)"
fi

# 5. 安装Anchor
echo "⚓ 安装Anchor..."
if ! command -v anchor &> /dev/null; then
    cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
    avm install latest
    avm use latest
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
    source ~/.bashrc
else
    echo "   Anchor已安装: $(anchor --version)"
fi

# 6. 安装Node.js（用于测试）
echo "📦 安装Node.js..."
if ! command -v node &> /dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt install -y nodejs
else
    echo "   Node.js已安装: $(node --version)"
fi

# 7. 安装Yarn
echo "📦 安装Yarn..."
if ! command -v yarn &> /dev/null; then
    sudo npm install -g yarn
else
    echo "   Yarn已安装: $(yarn --version)"
fi

# 8. 创建项目目录
echo "📁 设置项目目录..."
PROJECT_DIR="/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"
if [ -d "$PROJECT_DIR" ]; then
    echo "   项目目录已存在: $PROJECT_DIR"
else
    echo "   ⚠️ 项目目录不存在: $PROJECT_DIR"
    echo "   请确保Windows文件系统已挂载到/mnt/d/"
fi

# 9. 测试安装
echo "🧪 测试安装..."
echo "   Rust: $(rustc --version 2>/dev/null || echo '未安装')"
echo "   Solana: $(solana --version 2>/dev/null || echo '未安装')"
echo "   Anchor: $(anchor --version 2>/dev/null || echo '未安装')"
echo "   Node.js: $(node --version 2>/dev/null || echo '未安装')"
echo "   Yarn: $(yarn --version 2>/dev/null || echo '未安装')"

# 10. 创建快速启动脚本
echo "🚀 创建快速启动脚本..."
cat > ~/start_solana_dev.sh << 'EOF'
#!/bin/bash
# Solana开发环境快速启动脚本

echo "🚀 启动Solana开发环境"
echo "========================"

# 设置环境变量
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"

# 检查工具
echo "🔧 检查工具..."
solana --version
anchor --version
rustc --version

# 启动本地测试网
echo "🌐 启动本地测试网..."
solana-test-validator --reset &
SOLANA_PID=$!
echo "   测试网进程ID: $SOLANA_PID"

# 等待启动
sleep 10

# 配置网络
echo "⚙️  配置网络..."
solana config set --url http://localhost:8899

# 检查状态
echo "📊 检查状态..."
solana cluster-version
solana balance

echo "✅ 开发环境已启动!"
echo "💡 按Ctrl+C停止测试网"
echo "💡 运行 'kill $SOLANA_PID' 停止测试网"

# 保持脚本运行
wait $SOLANA_PID
EOF

chmod +x ~/start_solana_dev.sh

# 11. 创建构建和测试脚本
echo "🔨 创建构建和测试脚本..."
cat > ~/build_and_test.sh << 'EOF'
#!/bin/bash
# Solana项目构建和测试脚本

echo "🔨 Solana项目构建和测试"
echo "=========================="

# 进入项目目录
cd /mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle

# 设置环境变量
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"

# 1. 构建项目
echo "1. 构建项目..."
anchor build

if [ $? -eq 0 ]; then
    echo "   ✅ 构建成功!"
    
    # 获取程序ID
    PROGRAM_ID=$(solana address -k target/deploy/solana_oracle-keypair.json)
    echo "   程序ID: $PROGRAM_ID"
    
    # 2. 更新程序ID
    echo "2. 更新程序ID..."
    sed -i "s|declare_id(\".*\")|declare_id(\"$PROGRAM_ID\")|" programs/solana-oracle/src/lib.rs
    
    # 3. 重新构建
    echo "3. 重新构建..."
    anchor build
    
    # 4. 部署到本地测试网
    echo "4. 部署到本地测试网..."
    anchor deploy
    
    if [ $? -eq 0 ]; then
        echo "   ✅ 部署成功!"
        
        # 5. 运行测试
        echo "5. 运行测试..."
        anchor test
        
        # 6. 运行JavaScript测试
        echo "6. 运行JavaScript测试..."
        if [ -f "test_simple.js" ]; then
            node test_simple.js
        fi
    else
        echo "   ❌ 部署失败"
    fi
else
    echo "   ❌ 构建失败"
fi

echo "🎉 构建和测试完成!"
EOF

chmod +x ~/build_and_test.sh

# 12. 创建多智能体测试脚本
echo "🤖 创建多智能体测试脚本..."
cat > ~/test_multi_agent.sh << 'EOF'
#!/bin/bash
# 多智能体注册测试脚本

echo "🤖 多智能体注册测试"
echo "========================"

# 进入项目目录
cd /mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle

# 设置环境变量
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 程序ID
PROGRAM_ID="DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"

echo "🔍 检查程序状态..."
solana program show $PROGRAM_ID

echo "📊 测试智能体数据..."
cat > /tmp/test_agents.json << 'JSONDATA'
[
    {
        "name": "预言机核心节点",
        "did": "did:example:oracle-core-001",
        "publicKey": "0x1111111111111111111111111111111111111111111111111111111111111111",
        "metadataUri": "https://ipfs.io/ipfs/QmCoreAgent",
        "reputation": 850,
        "tier": "core"
    },
    {
        "name": "数据验证节点",
        "did": "did:example:validator-002",
        "publicKey": "0x2222222222222222222222222222222222222222222222222222222222222222",
        "metadataUri": "https://ipfs.io/ipfs/QmValidator",
        "reputation": 650,
        "tier": "validator"
    },
    {
        "name": "数据提供节点",
        "did": "did:example:data-provider-003",
        "publicKey": "0x3333333333333333333333333333333333333333333333333333333333333333",
        "metadataUri": "https://ipfs.io/ipfs/QmDataProvider",
        "reputation": 350,
        "tier": "data"
    },
    {
        "name": "轻量级网关",
        "did": "did:example:gateway-004",
        "publicKey": "0x4444444444444444444444444444444444444444444444444444444444444444",
        "metadataUri": "https://ipfs.io/ipfs/QmGateway",
        "reputation": 200,
        "tier": "gateway"
    }
]
JSONDATA

echo "   已创建4个测试智能体"
echo "   程序ID: $PROGRAM_ID"

# 创建测试脚本
cat > /tmp/simple_test.js << 'JSDATA'
console.log('🧪 WSL环境智能体测试');
console.log('程序ID: $PROGRAM_ID');
console.log('智能体数量: 4');
console.log('测试环境: Ubuntu WSL2');
console.log('✅ 测试环境准备完成');
JSDATA

node /tmp/simple_test.js

echo "🎉 多智能体测试准备完成!"
echo "💡 下一步: 部署智能合约并运行完整测试"
EOF

chmod +x ~/test_multi_agent.sh

echo ""
echo "🎉 WSL Solana开发环境设置完成!"
echo ""
echo "📋 可用脚本:"
echo "   ~/start_solana_dev.sh    - 启动开发环境"
echo "   ~/build_and_test.sh      - 构建和测试项目"
echo "   ~/test_multi_agent.sh    - 多智能体测试"
echo ""
echo "🚀 立即开始:"
echo "   1. 启动开发环境: ./start_solana_dev.sh"
echo "   2. 构建项目: ./build_and_test.sh"
echo "   3. 测试多智能体: ./test_multi_agent.sh"
echo ""
echo "💡 提示: 确保Windows文件系统已正确挂载到/mnt/d/"
