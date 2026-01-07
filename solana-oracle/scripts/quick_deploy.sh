#!/bin/bash

# 快速部署脚本 - 基于已安装的工具
echo "🚀 快速部署智能合约到本地测试网"

# 设置环境变量（根据安装文档）
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 检查工具
echo "🔧 检查工具..."
rustc --version || { echo "❌ Rust未安装"; exit 1; }
cargo --version || { echo "❌ Cargo未安装"; exit 1; }
solana --version || { echo "❌ Solana未安装"; exit 1; }
anchor --version || { echo "❌ Anchor未安装"; exit 1; }

echo "✅ 所有工具就绪"

# 进入项目目录
PROJECT_DIR="/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"
cd "$PROJECT_DIR" || { echo "❌ 无法进入项目目录"; exit 1; }
echo "📁 项目目录: $(pwd)"

# 1. 停止现有验证器
echo "🛑 停止现有验证器..."
pkill -f solana-test-validator 2>/dev/null || true
sleep 2

# 2. 启动本地测试网络
echo "🌐 启动本地测试网络..."
solana-test-validator --reset --quiet &
VALIDATOR_PID=$!
sleep 5

if ! ps -p $VALIDATOR_PID > /dev/null; then
    echo "❌ 无法启动测试验证器"
    exit 1
fi

echo "✅ 测试网络已启动 (PID: $VALIDATOR_PID)"

# 3. 配置网络
echo "🔧 配置网络..."
solana config set --url http://localhost:8899

# 4. 使用现有测试钱包或创建新钱包
echo "💰 设置测试钱包..."
if [ -f "test-wallet.json" ]; then
    echo "使用现有测试钱包"
    solana config set --keypair test-wallet.json
else
    echo "创建新测试钱包"
    solana-keygen new --outfile test-wallet.json --no-passphrase --silent
    solana config set --keypair test-wallet.json
fi

# 5. 获取测试SOL
echo "🪙 获取测试SOL..."
solana airdrop 10
BALANCE=$(solana balance)
echo "✅ 余额: $BALANCE"

# 6. 构建智能合约
echo "🔨 构建智能合约..."
if anchor build; then
    echo "✅ 构建成功"
else
    echo "❌ 构建失败，尝试清理后重新构建..."
    cargo clean
    rm -rf target/
    if anchor build; then
        echo "✅ 重新构建成功"
    else
        echo "❌ 构建仍然失败"
        kill $VALIDATOR_PID
        exit 1
    fi
fi

# 7. 部署智能合约
echo "🚀 部署智能合约..."
PROGRAM_ID=$(solana-keygen pubkey target/deploy/solana_oracle-keypair.json 2>/dev/null || echo "未知")

if anchor deploy; then
    echo "✅ Anchor部署成功"
else
    echo "尝试手动部署..."
    if solana program deploy target/deploy/solana_oracle.so; then
        echo "✅ 手动部署成功"
    else
        echo "❌ 部署失败"
        kill $VALIDATOR_PID
        exit 1
    fi
fi

# 8. 更新配置文件
echo "📝 更新配置文件..."
if [ -f "Anchor.toml" ]; then
    # 备份原文件
    cp Anchor.toml Anchor.toml.backup
    
    # 更新程序ID
    sed -i "s|solana_oracle = \".*\"|solana_oracle = \"$PROGRAM_ID\"|g" Anchor.toml
    echo "✅ Anchor.toml已更新"
fi

# 9. 验证部署
echo "🔍 验证部署..."
if solana program show $PROGRAM_ID 2>/dev/null | grep -q "Program Id:"; then
    echo "✅ 部署验证成功"
    
    # 显示程序详情
    echo "📋 程序详情:"
    solana program show $PROGRAM_ID
else
    echo "⚠️  部署验证警告"
fi

# 显示部署结果
echo ""
echo "🎉 部署完成！"
echo "========================"
echo "程序ID: $PROGRAM_ID"
echo "网络: http://localhost:8899"
echo "钱包: test-wallet.json"
echo "余额: $BALANCE"
echo "验证器PID: $VALIDATOR_PID"
echo ""
echo "💡 下一步操作:"
echo "1. 运行测试: anchor test --skip-local-validator"
echo "2. 测试客户端: node test_client.js"
echo "3. 停止网络: kill $VALIDATOR_PID"
echo ""
echo "📋 保持终端运行测试网络"
echo "按 Ctrl+C 停止"

# 保存部署信息
cat > deploy-success.txt << EOF
部署成功！
时间: $(date)
程序ID: $PROGRAM_ID
网络: http://localhost:8899
钱包公钥: $(solana-keygen pubkey test-wallet.json)
余额: $BALANCE
EOF

echo "✅ 部署信息保存到: deploy-success.txt"

# 保持运行
trap "echo '停止验证器...'; kill $VALIDATOR_PID; echo '部署完成！'; exit 0" INT
while true; do sleep 10; done
