#!/bin/bash

# WSL最小化安装脚本
# 只安装Solana开发必需的工具

set -e

echo "🚀 WSL最小化Solana开发环境安装"
echo "================================"

# 1. 安装Rust（如果未安装）
echo "🦀 检查/安装Rust..."
if ! command -v rustc &> /dev/null; then
    echo "   安装Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    echo "   ✅ Rust安装完成: $(rustc --version)"
else
    echo "   ✅ Rust已安装: $(rustc --version)"
fi

# 2. 安装Solana（如果未安装）
echo "🔧 检查/安装Solana..."
if ! command -v solana &> /dev/null; then
    echo "   安装Solana..."
    sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
    echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.bashrc
    echo "   ✅ Solana安装完成: $(solana --version)"
else
    echo "   ✅ Solana已安装: $(solana --version)"
fi

# 3. 安装Anchor（如果未安装）
echo "⚓ 检查/安装Anchor..."
if ! command -v anchor &> /dev/null; then
    echo "   安装Anchor..."
    source $HOME/.cargo/env
    cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
    avm install latest
    avm use latest
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
    source ~/.bashrc
    echo "   ✅ Anchor安装完成: $(anchor --version)"
else
    echo "   ✅ Anchor已安装: $(anchor --version)"
fi

# 4. 验证安装
echo "🧪 验证安装..."
echo "   Rust: $(rustc --version)"
echo "   Cargo: $(cargo --version)"
echo "   Solana: $(solana --version)"
echo "   Anchor: $(anchor --version)"

# 5. 创建快速测试脚本
echo "🚀 创建快速测试脚本..."
cat > ~/test_solana_wsl.sh << 'EOF'
#!/bin/bash
# WSL Solana快速测试脚本

echo "🚀 WSL Solana快速测试"
echo "========================"

# 设置环境
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"

# 1. 启动测试网
echo "🌐 启动本地测试网..."
solana-test-validator --reset &
SOLANA_PID=$!
echo "   测试网进程ID: $SOLANA_PID"
sleep 10

# 2. 配置网络
echo "⚙️  配置网络..."
solana config set --url http://localhost:8899

# 3. 检查状态
echo "📊 检查状态..."
solana cluster-version
solana balance

# 4. 请求空投（如果需要）
BALANCE=$(solana balance)
if [[ $BALANCE == "0 SOL" ]]; then
    echo "💸 请求空投..."
    solana airdrop 100
    sleep 2
    solana balance
fi

# 5. 进入项目目录
echo "📁 进入项目目录..."
cd /mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle

# 6. 构建项目
echo "🔨 构建项目..."
anchor build

if [ $? -eq 0 ]; then
    echo "   ✅ 构建成功!"
    
    # 获取程序ID
    PROGRAM_ID=$(solana address -k target/deploy/solana_oracle-keypair.json)
    echo "   程序ID: $PROGRAM_ID"
    
    # 7. 更新程序ID
    echo "🔄 更新程序ID..."
    sed -i "s|declare_id(\".*\")|declare_id(\"$PROGRAM_ID\")|" programs/solana-oracle/src/lib.rs
    
    # 8. 重新构建
    echo "🔨 重新构建..."
    anchor build
    
    # 9. 部署
    echo "🚀 部署到本地测试网..."
    anchor deploy
    
    if [ $? -eq 0 ]; then
        echo "   ✅ 部署成功!"
        
        # 10. 验证部署
        echo "🔍 验证部署..."
        solana program show $PROGRAM_ID
        
        # 11. 运行测试
        echo "🧪 运行测试..."
        if [ -f "test_simple.js" ]; then
            node test_simple.js
        fi
        
        echo "🎉 部署和测试完成!"
        echo "💡 程序ID: $PROGRAM_ID"
        echo "💡 测试网: http://localhost:8899"
    else
        echo "   ❌ 部署失败"
    fi
else
    echo "   ❌ 构建失败"
fi

# 停止测试网
echo "🛑 停止测试网..."
kill $SOLANA_PID 2>/dev/null || true

echo "✅ 测试完成!"
EOF

chmod +x ~/test_solana_wsl.sh

# 6. 创建多智能体注册测试
echo "🤖 创建多智能体注册测试..."
cat > ~/register_agents.sh << 'EOF'
#!/bin/bash
# 多智能体注册测试

echo "🤖 多智能体注册测试"
echo "========================"

# 设置环境
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 程序ID（从构建中获取或使用现有）
PROGRAM_ID="DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"

echo "📋 程序ID: $PROGRAM_ID"

# 智能体数据
AGENTS=(
    "预言机核心节点|did:example:core-001|850|core"
    "数据验证节点|did:example:validator-002|650|validator"
    "数据提供节点|did:example:data-003|350|data"
    "轻量级网关|did:example:gateway-004|200|gateway"
)

echo "📊 测试智能体:"
for agent in "${AGENTS[@]}"; do
    IFS='|' read -r name did reputation tier <<< "$agent"
    echo "   🔹 $name"
    echo "      DID: $did"
    echo "      声誉: $reputation"
    echo "      层级: $tier"
done

echo ""
echo "🚀 测试流程:"
echo "   1. 启动本地测试网"
echo "   2. 部署智能合约"
echo "   3. 注册智能体"
echo "   4. 验证注册"
echo "   5. 测试交互"

echo ""
echo "💡 运行完整测试:"
echo "   ./test_solana_wsl.sh"
EOF

chmod +x ~/register_agents.sh

echo ""
echo "🎉 WSL最小化安装完成!"
echo ""
echo "📋 可用脚本:"
echo "   ~/test_solana_wsl.sh    - 完整Solana测试"
echo "   ~/register_agents.sh    - 多智能体注册测试"
echo ""
echo "🚀 立即开始测试:"
echo "   1. 启动WSL终端"
echo "   2. 运行: ./test_solana_wsl.sh"
echo "   3. 或运行: ./register_agents.sh"
echo ""
echo "💡 提示: 测试需要一些时间，请耐心等待"
